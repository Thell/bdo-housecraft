use std::collections::HashMap;
use std::ffi::{c_void, CString};
use std::fs::File;
use std::io::Write;
use std::ptr::null;

use anyhow::{Ok, Result};
use highs_sys::*;
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::*;
use regex::Regex;
use serde::Serialize;
use serde_json::to_string_pretty;
use stable_vec::ExternStableVec;

use crate::cli_args::Cli;
use crate::houseinfo::*;
use crate::node_manipulation::{count_subtrees, count_subtrees_multistate};
use crate::region_nodes::RegionNodes;

type ChainVec = Vec<Chain>;
type ChainMapVec = Vec<ChainMap>;
type JobControlVec = Vec<JobControl>;

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
    fn from_highs(
        highs: &SubsetModel,
        warehouse_count: u32,
        worker_count: u32,
        region: &RegionNodes,
    ) -> Self {
        let highs = highs.highs_ptr;
        let cost = unsafe { Highs_getObjectiveValue(highs).round() as usize };
        let num_cols = unsafe { Highs_getNumCols(highs) };
        let num_rows = unsafe { Highs_getNumRows(highs) };
        let num_nodes = region.num_nodes;

        let mut col_value: Vec<f64> = vec![0.; num_cols as usize];
        let mut col_dual: Vec<f64> = vec![0.; num_cols as usize];
        let mut row_value: Vec<f64> = vec![0.; num_rows as usize];
        let mut row_dual: Vec<f64> = vec![0.; num_rows as usize];
        unsafe {
            Highs_getSolution(
                highs,
                col_value.as_mut_ptr(),
                col_dual.as_mut_ptr(),
                row_value.as_mut_ptr(),
                row_dual.as_mut_ptr(),
            );
        };
        trace!("Solution values:\n\t{:?}", col_value);
        let indices: Vec<_> = col_value
            .chunks_exact(3)
            .take(num_nodes)
            .enumerate()
            .filter_map(|(i, col)| {
                let item_flag: u32 = col[0].round() as u32;
                let state_1_flag: u32 = col[1].round() as u32;
                let state_2_flag: u32 = col[2].round() as u32;
                if i == 0 {
                    Some((i, 0))
                } else if item_flag == 0 {
                    None
                } else if state_1_flag == 1 {
                    Some((i, 1))
                } else if state_2_flag == 1 {
                    Some((i, 2))
                } else {
                    None
                }
            })
            .collect();
        let (indices, states): (Vec<_>, Vec<_>) = indices.iter().cloned().unzip();

        let chain = Self {
            worker_count: worker_count as usize,
            warehouse_count: warehouse_count as usize,
            cost,
            indices,
            states,
        };
        trace!("{:?}", chain);
        chain
    }

    #[inline(always)]
    fn elegant_pair(&self) -> usize {
        let x = self.worker_count;
        let y = self.warehouse_count;

        if x != std::cmp::max(x, y) {
            y.pow(2) + x
        } else {
            x.pow(2) + x + y
        }
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
            && !(self.cost == other.cost
                && self.warehouse_count == other.warehouse_count
                && self.worker_count == other.worker_count)
    }
}

/// An Elegantly Indexed Arena.
///
/// The main store for dominant chains is within a StableVec to reduce re-allocations and moves as
/// dominant chains are added. It is similar to an arena allocator and guarantees stable indices.
/// The indices are stored in another StableVec with a fixed capacity so that they can be accessed
/// directly by using an identity index.
/// Finally a third vector indicates which identities have been seen by storing the cost to unseat
/// the incumbent.
/// The identity is calculated using Elegant Pair matrix indexing which is a `mul` and `add` instead
/// of going through the division, mod, shift and mask of the stablevec's `has_element_at` at the
/// cost of some extra memory since a `n×n` matrix must be used.
/// I thought using an `n×m` matrix and `x+y×stride` would be more efficient in terms of memory
/// and time but in testing it was always a few percentage points slower.
#[derive(Clone, Debug)]
struct ChainMap {
    cost: Vec<u16>,
    keys: ExternStableVec<usize>,
    chains: ExternStableVec<Chain>,
}

impl ChainMap {
    fn new(region: &RegionNodes) -> Self {
        let len = (std::cmp::max(region.max_worker_count, region.max_warehouse_count) + 1).pow(2);
        let cost = vec![u16::MAX; len];
        let mut keys = ExternStableVec::<usize>::new();
        keys.reserve(len);
        let chains = ExternStableVec::<Chain>::new();
        Self { cost, keys, chains }
    }

    #[inline(always)]
    fn insert_or_update(&mut self, chain: &Chain) {
        let key = chain.elegant_pair();
        if self.cost[key] == u16::MAX {
            self.cost[key] = chain.cost as u16;
            let index = self.chains.push(chain.to_owned());
            self.keys.insert(key, index);
        } else if self.cost[key] > chain.cost as u16 {
            self.cost[key] = chain.cost as u16;
            unsafe {
                let index = self.keys.get_unchecked(key);
                let entry = self.chains.get_unchecked_mut(*index);
                chain.clone_into(entry);
            }
        }
    }

    #[inline(always)]
    fn flatten_many_by_insert_update(chain_maps: &mut ChainMapVec) -> ChainMap {
        let mut chains = chain_maps[0].clone();
        chain_maps
            .iter()
            .skip(1)
            .for_each(|cm| cm.chains.values().for_each(|c| chains.insert_or_update(c)));
        chains
    }

    #[inline(always)]
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

    #[inline(always)]
    fn retain_dominating_to_vec(&self) -> ChainVec {
        let mut chains: ChainVec = self.chains.values().map(|c| c.to_owned()).collect();
        Self::retain_dominating(&mut chains);
        info!("Retained chain count: {:?}", chains.len());
        chains.sort_unstable_by_key(|chain| (chain.worker_count, chain.warehouse_count));
        chains
    }
}

struct SubsetModel {
    highs_ptr: *mut c_void,
}

impl SubsetModel {
    pub fn new(job: usize) -> Self {
        SubsetModel {
            highs_ptr: unsafe {
                let highs = Highs_create();
                let do_logging = log_enabled!(log::Level::Debug) as i32;

                let option = CString::new("output_flag").unwrap();
                Highs_setBoolOptionValue(highs, option.as_ptr(), do_logging);

                let option = CString::new("log_to_console").unwrap();
                Highs_setBoolOptionValue(highs, option.as_ptr(), do_logging);

                if do_logging == 1 {
                    let option = CString::new("log_file").unwrap();
                    let option_value =
                        CString::new(format!("subset_select_job{job}_highs.log")).unwrap();
                    Highs_setStringOptionValue(highs, option.as_ptr(), option_value.as_ptr());
                }

                let option = CString::new("threads").unwrap();
                Highs_setIntOptionValue(highs, option.as_ptr(), 1);

                // Most of the errors encountered are presolve assertions in the `latest` branch.
                // let option = CString::new("presolve").unwrap();
                // let option_value = CString::new("off").unwrap();
                // Highs_setStringOptionValue(highs, option.as_ptr(), option_value.as_ptr());

                highs
            },
        }
    }

    fn populate_problem(&mut self, region: &RegionNodes) {
        let items: Vec<_> = region.children.iter().map(|x| *x as u32).collect();
        let item_reqs: Vec<_> = region.parents.iter().map(|x| *x as u32).collect();
        let state_1_values: Vec<_> = region.warehouse_counts.iter().map(|x| *x as f64).collect();
        let state_2_values: Vec<_> = region.worker_counts.iter().map(|x| *x as f64).collect();

        unsafe {
            // parent -> child relation tree for item selection requirements.
            let mut item_req_tree: HashMap<u32, Vec<u32>> = HashMap::new();
            for (i, &item_req) in item_reqs.iter().enumerate() {
                if item_req == 0 {
                    item_req_tree.entry(items[0]).or_default().push(items[i]);
                } else {
                    item_req_tree.entry(item_req).or_default().push(items[i]);
                }
            }

            // Use the HiGHS optimizer.
            let highs = self.highs_ptr;

            // Variables to flag selected items and indicate the state of the selected item.
            // Map {item: column_id} since HiGHS doesn't have assignment/retrieval by name yet.
            let costs: Vec<_> = region.costs.iter().map(|x| *x as f64).collect();
            let mut item_flags: HashMap<u32, i32> = HashMap::new();
            let mut state_1_flags: HashMap<u32, i32> = HashMap::new();
            let mut state_2_flags: HashMap<u32, i32> = HashMap::new();

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

            // The item requirements constraints.
            let highs_inf = Highs_getInfinity(highs);

            // The item parent <- child requirements constraints.
            // Transitive; ensures children must have all ancestors back to root.
            for (parent, children) in item_req_tree.iter() {
                for child in children.iter() {
                    if *parent == items[0] || *parent == 0 {
                        continue;
                    }
                    // item_flags[child] - item_flags[parent] <= 0
                    let aindex: [i32; 2] = [item_flags[child], item_flags[parent]];
                    let avalue: [f64; 2] = [1.0, -1.0];
                    Highs_addRow(
                        highs,
                        -highs_inf,
                        0.0,
                        aindex.len() as i32,
                        aindex.as_ptr(),
                        avalue.as_ptr(),
                    );
                }
            }

            // Item selection constraint: one state on flagged items, no state otherwise.
            for item in items.iter() {
                if *item == items[0] {
                    continue;
                }
                // state_1_flags[child] + state_2_flags[child] - items_flag[child] == 0
                let aindex: [i32; 3] = [state_1_flags[item], state_2_flags[item], item_flags[item]];
                let avalue: [f64; 3] = [1.0, 1.0, -1.0];
                Highs_addRow(
                    highs,
                    0.0,
                    0.0,
                    aindex.len() as i32,
                    aindex.as_ptr(),
                    avalue.as_ptr(),
                );
            }

            // State values sum constraints.
            // Sum items selected as state 1 values
            let mut aindex: Vec<i32> = Vec::new();
            let mut avalue: Vec<f64> = Vec::new();
            for (i, item) in items.iter().enumerate() {
                if state_1_values[i] > 0.0 {
                    aindex.push(state_1_flags[item]);
                    avalue.push(state_1_values[i]);
                }
            }
            Highs_addRow(
                highs,
                9999.0,
                highs_inf,
                aindex.len() as i32,
                aindex.as_ptr(),
                avalue.as_ptr(),
            );

            // Sum items selected as state 2 values
            aindex.clear();
            avalue.clear();
            for (i, item) in items.iter().enumerate() {
                if state_2_values[i] > 0.0 {
                    aindex.push(state_2_flags[item]);
                    avalue.push(state_2_values[i]);
                }
            }
            Highs_addRow(
                highs,
                9999.0,
                highs_inf,
                aindex.len() as i32,
                aindex.as_ptr(),
                avalue.as_ptr(),
            );
        }
    }

    fn mut_ptr(&mut self) -> *mut c_void {
        self.highs_ptr
    }
}

impl Drop for SubsetModel {
    fn drop(&mut self) {
        unsafe {
            Highs_destroy(self.highs_ptr);
        }
    }
}

#[derive(Clone, Debug)]
struct JobControl {
    job_id: usize,
    state_value_sum_lb_pairs: Vec<(u32, u32)>,
}

impl JobControl {
    /// Returns job controls for parallel dominating chains optimization.
    fn new(cli: &Cli, region: &RegionNodes) -> Result<JobControlVec> {
        Ok(Self::chunk_state_value_pairs(cli, region)
            .iter()
            .enumerate()
            .map(|(job_id, pairs)| JobControl {
                job_id,
                state_value_sum_lb_pairs: pairs.to_vec(),
            })
            .collect::<Vec<_>>())
    }

    fn chunk_state_value_pairs(cli: &Cli, region: &RegionNodes) -> Vec<Vec<(u32, u32)>> {
        let warehouse_count_ub = if cli.limit_warehouse {
            std::cmp::min(172, region.max_warehouse_count + 1) as u32
        } else {
            (region.max_warehouse_count + 1) as u32
        };
        let mut state_value_sum_pairs = (0..warehouse_count_ub)
            .cartesian_product(0..region.max_worker_count as u32 + 1)
            .collect::<Vec<_>>();
        let mut rng = thread_rng();
        state_value_sum_pairs.shuffle(&mut rng);

        let jobs = cli.jobs.unwrap_or(1) as usize;
        let chunk_size = (state_value_sum_pairs.len() + jobs) / jobs;
        state_value_sum_pairs
            .chunks(chunk_size)
            .map(|chunk| chunk.into())
            .collect()
    }
}

pub(crate) fn optimize(cli: Cli) -> Result<()> {
    info!("preparing...");
    let region_name = cli.region.clone().unwrap();
    let region_buildings = get_region_buildings(Some(region_name.clone()))?;
    let region = RegionNodes::new(region_buildings.get(&region_name).unwrap())?;
    let jobs = JobControl::new(&cli, &region)?;

    if !cli.verbose.is_silent() {
        trace!("Buildings");
        region_buildings.iter().for_each(|b| trace!("{:#?}", b));
        debug!("Job Controls");
        jobs.iter().for_each(|j| debug!("  {:?}", j));
        print_region_specs(&region);
        print_starting_status(&region);
    }

    info!("optimizing...");
    let chains = optimize_par(&region, jobs)?;
    info!("retaining...");
    let mut chains = chains.retain_dominating_to_vec();
    info!("writing...");
    write_chains(&cli, &region, &mut chains)?;

    Ok(())
}

fn optimize_par(region: &RegionNodes, jobs: JobControlVec) -> Result<ChainMap> {
    let mut results = jobs
        .into_par_iter()
        .map(|job| optimize_par_worker(region.clone(), job).unwrap())
        .collect::<ChainMapVec>();
    info!("merging...");
    let results = ChainMap::flatten_many_by_insert_update(&mut results);
    info!("Captured chain count: {:?}", results.chains.num_elements());
    Ok(results)
}

fn optimize_par_worker(region: RegionNodes, job: JobControl) -> Result<ChainMap> {
    let mut chains = ChainMap::new(&region);
    let mut counter = 0;
    let mut highs = SubsetModel::new(job.job_id);
    highs.populate_problem(&region);
    let highs_inf = unsafe { Highs_getInfinity(highs.mut_ptr()) };

    // Change these to use Highs_getRowByName when it's available.
    let state_1_sum_row: HighsInt = unsafe { Highs_getNumRows(highs.mut_ptr()) - 2 };
    let state_2_sum_row: HighsInt = state_1_sum_row + 1;
    debug!("state lb rows: [{}, {}]", state_1_sum_row, state_2_sum_row);

    info!(
        "START: Job {} with {} combinations on {} nodes using {} cols and {} rows.",
        job.job_id,
        job.state_value_sum_lb_pairs.len(),
        region.num_nodes,
        unsafe { Highs_getNumCols(highs.mut_ptr()) },
        unsafe { Highs_getNumRows(highs.mut_ptr()) },
    );

    for (state_1_sum_lb, state_2_sum_lb) in job.state_value_sum_lb_pairs.iter() {
        let status = unsafe {
            Highs_changeRowBounds(
                highs.mut_ptr(),
                state_1_sum_row,
                *state_1_sum_lb as f64,
                highs_inf,
            );
            Highs_changeRowBounds(
                highs.mut_ptr(),
                state_2_sum_row,
                *state_2_sum_lb as f64,
                highs_inf,
            );
            debug!("running for ({},{})...", state_1_sum_lb, state_2_sum_lb);
            let result = Highs_run(highs.mut_ptr());
            debug!(
                "run result for ({},{}): {}",
                state_1_sum_lb, state_2_sum_lb, result
            );
            Highs_getModelStatus(highs.mut_ptr())
        };
        debug!(
            "model status for ({},{}): {}",
            state_1_sum_lb, state_2_sum_lb, status
        );
        if status == MODEL_STATUS_OPTIMAL {
            counter += 1;
            let chain = Chain::from_highs(&highs, *state_1_sum_lb, *state_2_sum_lb, &region);
            chains.insert_or_update(&chain);
        };
    }

    info!(
        "COMPLETE: Job {} with {} combinations with {} feasible yielding {:?} chains.",
        job.job_id,
        job.state_value_sum_lb_pairs.len(),
        counter,
        chains.chains.num_elements(),
    );
    Ok(chains)
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
    for chain in chains.iter_mut() {
        chain.indices = chain.indices.iter().map(|j| region.children[*j]).collect();
    }
    let region_name = cli.region.clone().unwrap();
    let file_name = region_name.replace(' ', "_");
    let path = format!("./data/housecraft/{}.json", file_name);
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
            path
        );
    }

    Ok(())
}
