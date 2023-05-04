use std::collections::HashMap;
use std::ffi::{c_void, CString};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::ptr::null;

use anyhow::{Ok, Result};
use highs_sys::*;
use rayon::prelude::*;
use regex::Regex;
use serde::Serialize;
use serde_json::to_string_pretty;

use crate::cli_args::Cli;
use crate::houseinfo::*;
use crate::node_manipulation::{count_subtrees, count_subtrees_multistate};
use crate::region_nodes::RegionNodes;

type ChainVec = Vec<Chain>;

fn retain_dominating(chains: &mut ChainVec) {
    let mut j = 0;
    for i in 0..chains.len() {
        if (0..j)
            .chain(i + 1..chains.len())
            .all(|a| !chains[a].dominates(&chains[i]))
        {
            chains.swap(i, j);
            j += 1;
        }
    }
    chains.truncate(j);
}

#[derive(Clone, Debug, Serialize)]
struct Chain {
    #[serde(rename = "lodging")]
    worker_count: usize,
    #[serde(rename = "storage")]
    warehouse_count: usize,
    cost: usize,
    indices: Vec<usize>,
    states: Vec<usize>,
}

impl Chain {
    fn from_highs(highs: &SubsetModel, region: &RegionNodes) -> Self {
        let cost = unsafe { Highs_getObjectiveValue(highs.highs_ptr).round() as usize };
        let col_values = unsafe { highs.solution_col_values() };

        let num_nodes = region.num_nodes;
        let mut warehouse_count = 0;
        let mut worker_count = 0;
        let mut indices = vec![0];
        let mut states = vec![0];

        for (i, col) in col_values.chunks_exact(3).take(num_nodes).enumerate() {
            if col[0] != 1 {
                // item_flag
                continue;
            } else if col[1] == 1 {
                // state_1_flag
                warehouse_count += region.warehouse_counts[i];
                indices.push(i);
                states.push(1);
            } else if col[2] == 1 {
                // state_2_flag
                worker_count += region.worker_counts[i];
                indices.push(i);
                states.push(2);
            }
        }

        let chain = Self {
            worker_count,
            warehouse_count,
            cost,
            indices,
            states,
        };
        trace!("{:?}", chain);
        chain
    }

    #[inline(always)]
    fn dominates(&self, other: &Chain) -> bool {
        // Dominate when we can get the same or more for less or the same.
        // No domination when equal. While the equality test is not needed if the caller
        // guarantees to not test self the testing for that would have to be done on all
        // calls rather than the check here being done only on the dominant chains.
        self.cost <= other.cost
            && self.warehouse_count >= other.warehouse_count
            && self.worker_count >= other.worker_count
    }
}

struct SubsetModel {
    highs_ptr: *mut c_void,
}

impl SubsetModel {
    pub fn new(region: &RegionNodes, worker_id: usize) -> Self {
        let mut model = SubsetModel {
            highs_ptr: unsafe { Highs_create() },
        };
        unsafe {
            Self::set_options(&mut model, worker_id);
            Self::initialize(&mut model, region);
        }
        model
    }

    unsafe fn initialize(&mut self, region: &RegionNodes) {
        let items: Vec<_> = region.children.iter().map(|x| *x as u32).collect();
        let item_reqs: Vec<_> = region.parents.iter().map(|x| *x as u32).collect();
        let state_1_values: Vec<_> = region.warehouse_counts.iter().map(|x| *x as f64).collect();
        let state_2_values: Vec<_> = region.worker_counts.iter().map(|x| *x as f64).collect();

        // Presolve -> MIP fails on a few known instances.
        // See https://github.com/ERGO-Code/HiGHS/issues/1273
        // This is a workaround.
        let no_presolve_regions = ["Port Epheria", "Altinova", "Heidel"];
        if no_presolve_regions.contains(&region.region_name.as_str()) {
            warn!("Setting tolerance for {}.", region.region_name);
            let option = CString::new("mip_feasibility_tolerance").unwrap();
            unsafe {
                Highs_setDoubleOptionValue(self.highs_ptr, option.as_ptr(), 0.0027);
            };
        }

        let costs: Vec<_> = region.costs.iter().map(|x| *x as f64).collect();
        let mut item_flags: HashMap<u32, i32> = HashMap::new();
        let mut state_1_flags: HashMap<u32, i32> = HashMap::new();
        let mut state_2_flags: HashMap<u32, i32> = HashMap::new();

        self.initialize_flag_variables(
            &items,
            &costs,
            &mut item_flags,
            &mut state_1_flags,
            &mut state_2_flags,
        );
        self.initialize_item_requirement_constraints(&items, &item_flags, &item_reqs);
        self.initialize_item_selection_constraints(
            &items,
            &item_flags,
            &state_1_flags,
            &state_2_flags,
        );
        self.initialize_state_value_sum_constraints(&items, &state_1_flags, &state_1_values);
        self.initialize_state_value_sum_constraints(&items, &state_2_flags, &state_2_values);
    }

    unsafe fn initialize_flag_variables(
        &mut self,
        items: &[u32],
        costs: &[f64],
        item_flags: &mut HashMap<u32, i32>,
        state_1_flags: &mut HashMap<u32, i32>,
        state_2_flags: &mut HashMap<u32, i32>,
    ) {
        // Map {item: column_id} since HiGHS doesn't have assignment/retrieval by name yet.

        let highs = self.highs_ptr;
        let mut column_id = 0;
        for (i, item) in items.iter().enumerate() {
            // item_flags (with objective item selection costs)
            if Highs_addCol(highs, costs[i], 0.0, 1.0, 0, null(), null()) == kHighsStatusOk {
                Highs_changeColIntegrality(highs, column_id, kHighsVarTypeInteger);
                item_flags.insert(*item, column_id);
                column_id += 1;
            }
            // state_1_flags
            if Highs_addCol(highs, 0.0, 0.0, 1.0, 0, null(), null()) == kHighsStatusOk {
                Highs_changeColIntegrality(highs, column_id, kHighsVarTypeInteger);
                state_1_flags.insert(*item, column_id);
                column_id += 1;
            }
            // state_2_flags
            if Highs_addCol(highs, 0.0, 0.0, 1.0, 0, null(), null()) == kHighsStatusOk {
                Highs_changeColIntegrality(highs, column_id, kHighsVarTypeInteger);
                state_2_flags.insert(*item, column_id);
                column_id += 1;
            }
        }
    }

    fn initialize_item_req_tree(items: &[u32], item_reqs: &[u32]) -> HashMap<u32, Vec<u32>> {
        // parent -> child relation tree for item selection requirements.
        let mut item_req_tree: HashMap<u32, Vec<u32>> = HashMap::new();
        for (i, &item_req) in item_reqs.iter().enumerate() {
            if item_req == 0 {
                item_req_tree.entry(items[0]).or_default().push(items[i]);
            } else {
                item_req_tree.entry(item_req).or_default().push(items[i]);
            }
        }
        item_req_tree
    }

    unsafe fn initialize_item_requirement_constraints(
        &mut self,
        items: &[u32],
        item_flags: &HashMap<u32, i32>,
        item_reqs: &[u32],
    ) {
        // The item parent <- child requirements constraints.
        // Transitive; ensures children must have all ancestors back to root.

        let highs_inf = Highs_getInfinity(self.highs_ptr);
        let item_req_tree = Self::initialize_item_req_tree(items, item_reqs);

        for (parent, children) in item_req_tree.iter() {
            for child in children.iter() {
                if *parent == items[0] || *parent == 0 {
                    continue;
                }
                // item_flags[child] - item_flags[parent] <= 0
                let aindex: [i32; 2] = [item_flags[child], item_flags[parent]];
                let avalue: [f64; 2] = [1.0, -1.0];
                Highs_addRow(
                    self.highs_ptr,
                    -highs_inf,
                    0.0,
                    aindex.len() as i32,
                    aindex.as_ptr(),
                    avalue.as_ptr(),
                );
            }
        }
    }

    unsafe fn initialize_item_selection_constraints(
        &mut self,
        items: &[u32],
        item_flags: &HashMap<u32, i32>,
        state_1_flags: &HashMap<u32, i32>,
        state_2_flags: &HashMap<u32, i32>,
    ) {
        // Item selection constraint: one state on flagged items, no state otherwise.
        for item in items.iter() {
            if *item == items[0] {
                continue;
            }
            // state_1_flags[child] + state_2_flags[child] - items_flag[child] == 0
            let aindex: [i32; 3] = [state_1_flags[item], state_2_flags[item], item_flags[item]];
            let avalue: [f64; 3] = [1.0, 1.0, -1.0];
            Highs_addRow(
                self.highs_ptr,
                0.0,
                0.0,
                aindex.len() as i32,
                aindex.as_ptr(),
                avalue.as_ptr(),
            );
        }
    }

    unsafe fn initialize_state_value_sum_constraints(
        &mut self,
        items: &[u32],
        state_flags: &HashMap<u32, i32>,
        state_values: &[f64],
    ) {
        // Sum state values for selected items in the given state.
        let mut aindex: Vec<i32> = Vec::new();
        let mut avalue: Vec<f64> = Vec::new();
        for (i, item) in items.iter().enumerate() {
            if state_values[i] > 0.0 {
                aindex.push(state_flags[item]);
                avalue.push(state_values[i]);
            }
        }
        // lb is set to an arbitrary value for now and is assigned prior to solving the model.
        Highs_addRow(
            self.highs_ptr,
            9999.0,
            Highs_getInfinity(self.highs_ptr),
            aindex.len() as i32,
            aindex.as_ptr(),
            avalue.as_ptr(),
        );
    }

    fn mut_ptr(&mut self) -> *mut c_void {
        self.highs_ptr
    }

    unsafe fn set_options(&mut self, worker_id: usize) {
        let do_logging = log_enabled!(log::Level::Trace) as i32;

        let option = CString::new("output_flag").unwrap();
        Highs_setBoolOptionValue(self.highs_ptr, option.as_ptr(), do_logging);

        let option = CString::new("log_to_console").unwrap();
        Highs_setBoolOptionValue(self.highs_ptr, option.as_ptr(), do_logging);

        if do_logging == 1 {
            let option = CString::new("log_file").unwrap();
            let option_value =
                CString::new(format!("subset_select_{worker_id}_highs.log")).unwrap();
            Highs_setStringOptionValue(self.highs_ptr, option.as_ptr(), option_value.as_ptr());
        }

        let option = CString::new("threads").unwrap();
        Highs_setIntOptionValue(self.highs_ptr, option.as_ptr(), 1);
    }

    unsafe fn solution_col_values(&self) -> Vec<u32> {
        let num_cols = unsafe { Highs_getNumCols(self.highs_ptr) };
        let num_rows = unsafe { Highs_getNumRows(self.highs_ptr) };
        let mut col_value: Vec<f64> = vec![0.; num_cols as usize];
        let mut col_dual: Vec<f64> = vec![0.; num_cols as usize];
        let mut row_value: Vec<f64> = vec![0.; num_rows as usize];
        let mut row_dual: Vec<f64> = vec![0.; num_rows as usize];
        unsafe {
            Highs_getSolution(
                self.highs_ptr,
                col_value.as_mut_ptr(),
                col_dual.as_mut_ptr(),
                row_value.as_mut_ptr(),
                row_dual.as_mut_ptr(),
            );
        };
        let col_value: Vec<_> = col_value.iter().map(|v| v.round() as u32).collect();
        trace!("Solution values:\n\t{:?}", col_value);
        col_value
    }
}

impl Drop for SubsetModel {
    fn drop(&mut self) {
        unsafe {
            Highs_destroy(self.highs_ptr);
        }
    }
}

pub(crate) fn optimize(cli: &mut Cli) -> Result<()> {
    info!("preparing...");
    let region_name = cli.region.clone().unwrap();
    let all_region_buildings: RegionBuildingMap = if region_name == *"ALL" {
        parse_houseinfo_data()?
    } else {
        get_region_buildings(Some(region_name))?
    };

    for (region_name, region_buildings) in all_region_buildings.iter() {
        cli.region = Some(region_name.to_owned());
        let region = RegionNodes::new(region_buildings)?;

        if !cli.verbose.is_silent() {
            trace!("Buildings");
            region_buildings.iter().for_each(|b| trace!("{:#?}", b));
            print_region_specs(&region);
            print_starting_status(&region);
        }

        info!("optimizing...");
        let mut chains: ChainVec = (0..=region.max_worker_count)
            .into_par_iter()
            .map(|state_2_lb| optimize_worker(cli.clone(), region.clone(), state_2_lb))
            .flatten()
            .collect();
        info!("Captured chain count: {:?}", chains.len());
        info!("retaining...");
        retain_dominating(&mut chains);
        info!("Retained chain count: {:?}", chains.len());
        info!("writing...");
        write_chains(cli, &region, &mut chains)?;
    }
    Ok(())
}

fn optimize_worker(cli: Cli, region: RegionNodes, state_2_sum_lb: usize) -> ChainVec {
    let mut highs = SubsetModel::new(&region, state_2_sum_lb);
    let highs_inf = unsafe { Highs_getInfinity(highs.mut_ptr()) };
    let state_1_sum_row: HighsInt = unsafe { Highs_getNumRows(highs.mut_ptr()) - 2 };
    let state_2_sum_row: HighsInt = state_1_sum_row + 1;

    info!("START: Job {state_2_sum_lb}");
    debug!("state lb rows: [{state_1_sum_row}, {state_2_sum_row}]");

    unsafe {
        Highs_changeRowBounds(
            highs.mut_ptr(),
            state_2_sum_row,
            state_2_sum_lb as f64,
            highs_inf,
        );
    };

    let state_1_sum_ub = if cli.limit_warehouse {
        std::cmp::min(region.max_warehouse_count, 172)
    } else {
        region.max_warehouse_count
    };
    let mut chains = ChainVec::with_capacity(state_1_sum_ub);

    let mut state_1_sum_lb = 0;
    while state_1_sum_lb <= state_1_sum_ub {
        let status = unsafe {
            Highs_changeRowBounds(
                highs.mut_ptr(),
                state_1_sum_row,
                state_1_sum_lb as f64,
                highs_inf,
            );
            Highs_run(highs.mut_ptr());
            Highs_getModelStatus(highs.mut_ptr())
        };
        if status == MODEL_STATUS_OPTIMAL {
            let chain = Chain::from_highs(&highs, &region);
            state_1_sum_lb = chain.warehouse_count + 1;
            chains.push(chain);
        } else {
            break;
        };
    }

    info!(
        "COMPLETE: Job {state_2_sum_lb} yielding {} chains.",
        chains.len()
    );
    chains
}

fn print_region_specs(region: &RegionNodes) {
    debug!(
        "Region node parameters...\
        \n         parents: {:?}\
        \n        children: {:?}\
        \n           costs: {:?}\
        \n          states: {:?}\
        \nwarehouse_counts: {:?}\
        \n   worker_counts: {:?}\
        \n    jump_indices: {:?}\n",
        region.parents,
        region.children,
        region.costs,
        region.states,
        region.warehouse_counts,
        region.worker_counts,
        region.jump_indices,
    );
}

fn print_starting_status(region: &RegionNodes) {
    let building_chain_count = count_subtrees(region.root, &region.parents, &region.children);
    let multistate_count = count_subtrees_multistate(
        region.root,
        &region.parents,
        &region.children,
        &region.states,
    );
    println!(
        "Optimizing values for {} consisting of {} buildings in {} chains with {} storage/lodging combinations",
        region.region_name, region.buildings.len(), building_chain_count, multistate_count
    );
    println!(
        "With a maximum cost of {} with {} lodging and {} storage (out of {:?} possible).",
        region.usage_counts.cost,
        region.usage_counts.worker_count,
        region.usage_counts.warehouse_count,
        region.max_warehouse_count
    );
}

fn write_chains(cli: &Cli, region: &RegionNodes, chains: &mut Vec<Chain>) -> Result<()> {
    if cli.for_validation {
        for chain in chains.iter_mut() {
            chain.indices.clear();
            chain.states.clear();
        }
    } else {
        for chain in chains.iter_mut() {
            chain.indices = chain.indices.iter().map(|j| region.children[*j]).collect();
        }
    }

    let region_name = cli.region.clone().unwrap();
    let file_name = region_name.replace(' ', "_");
    let path = if cli.for_validation {
        format!("./data/housecraft/validation/HiGHS/{}.json", file_name)
    } else {
        format!("./data/housecraft/{}.json", file_name)
    };
    let path = PathBuf::from(path);
    fs::create_dir_all(path.parent().unwrap())?;
    let mut output = File::create(path.clone())?;

    let re = Regex::new(r"\{[^}]*?\}").unwrap();
    let json = to_string_pretty(chains)?;
    let json = re
        .replace_all(&json, |caps: &regex::Captures<'_>| {
            caps[0].replace(['\n', ' '], "")
        })
        .to_string();
    output.write_all(json.as_bytes())?;

    if !cli.verbose.is_silent() {
        println!(
            "Result: {} 'best of best' scored storage/lodging chains written to {}.",
            chains.len(),
            path.to_str().unwrap()
        );
    }

    Ok(())
}
