use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::{c_void, CString};
use std::fs::File;
use std::io::Write;

use anyhow::{Ok, Result};
use highs_sys::*;
use itertools::Itertools;
use log::Level::Debug;
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

pub const VAR_TYPE_INTEGER: *const i32 = &kHighsVarTypeInteger as *const i32;
pub const LB: *const f64 = &0.0 as *const f64;
pub const UB: *const f64 = &1.0 as *const f64;

#[derive(Debug, Clone)]
struct HighsPtr(*mut c_void);

// fn generate_model(
//     items: &[u32],
//     item_reqs: &[u32],
//     weights: &[u32],
//     state_1_values: &[u32],
//     state_2_values: &[u32],
//     state_1_sum_lb: u32,
//     state_2_sum_lb: u32,
// ) {
//     // parent -> child relation tree for item selection requirements.
//     let mut item_req_tree: HashMap<u32, Vec<u32>> = HashMap::new();
//     for (i, &item_req) in item_reqs.iter().enumerate() {
//         if item_req == 0 {
//             item_req_tree.entry(items[0]).or_default().push(items[i]);
//         } else {
//             item_req_tree.entry(item_req).or_default().push(items[i]);
//         }
//     }

//     let mut problem = ProblemVariables::new();

//     let mut item_flags: HashMap<u32, Variable> = HashMap::new();
//     let mut state_1_flags: HashMap<u32, Variable> = HashMap::new();
//     let mut state_2_flags: HashMap<u32, Variable> = HashMap::new();
//     for item in items.iter() {
//         item_flags.insert(
//             *item,
//             problem.add(variable().name(format!("key_{item}")).binary()),
//         );
//         state_1_flags.insert(
//             *item,
//             problem.add(variable().name(format!("state_1_flag_{item}")).binary()),
//         );
//         state_2_flags.insert(
//             *item,
//             problem.add(variable().name(format!("state_2_flag_{item}")).binary()),
//         );
//     }

//     // The objective: select "items" that minimize the weight subject to value sum lb constraints.
//     let mut objective_expr: Expression = 0.into();
//     for (i, item) in items.iter().enumerate() {
//         objective_expr += item_flags[item] * weights[i];
//     }
//     let mut problem = problem.minimise(objective_expr).using(good_lp::highs);

//     // The item requirements constraints.
//     // This constraint is transitive to all ancestors of item such that every item selected requires
//     // all ancestor items to be selected where item_req => item is a parent => child relation.
//     for (parent, children) in item_req_tree.iter() {
//         for child in children.iter() {
//             problem = problem.with(constraint!(item_flags[child] <= item_flags[parent]));
//         }
//     }

//     // The state flag constraints.
//     for (item, item_flag) in item_flags.iter() {
//         // The no state on non-flagged items constraints.
//         problem = problem.with(constraint!(state_1_flags[item] <= item_flag));
//         problem = problem.with(constraint!(state_2_flags[item] <= item_flag));
//         // The mandatory state flag constraint on flagged items.
//         problem = problem.with(constraint!(
//             state_1_flags[item] + state_2_flags[item] <= item_flag
//         ));
//         // The state flag "mutual exclusivity" constraint.
//         problem = problem.with(constraint!(state_1_flags[item] + state_2_flags[item] <= 1));
//     }

//     // The state sum LB expressions (the coefficients).
//     let mut state_1_sum_expr: Expression = 0.into();
//     let mut state_2_sum_expr: Expression = 0.into();
//     for (i, item) in items.iter().enumerate() {
//         state_1_sum_expr += state_1_flags[item] * state_1_values[i];
//         state_2_sum_expr += state_2_flags[item] * state_2_values[i];
//     }
//     let state_1_sum_lb_expr: Expression = (state_1_sum_lb as f64).into();
//     let state_2_sum_lb_expr: Expression = (state_2_sum_lb as f64).into();
//     let problem = problem
//         .with(constraint!(state_1_sum_lb_expr <= state_1_sum_expr))
//         .with(constraint!(state_2_sum_lb_expr <= state_2_sum_expr));

//     let problem = problem.into_inner();
//     let result = problem.solve();
//     println!("{:?}", result.get_solution());

//     // Is this is what we want to use?
//     // Highs_passMip(
//     //     highs.mut_ptr(),
//     //     c(problem.num_cols()),
//     //     c(problem.num_rows()),
//     //     c(problem.matrix.avalue.len()),
//     //     MATRIX_FORMAT_COLUMN_WISE,
//     //     OBJECTIVE_SENSE_MINIMIZE,
//     //     offset,
//     //     problem.colcost.as_ptr(),
//     //     problem.collower.as_ptr(),
//     //     problem.colupper.as_ptr(),
//     //     problem.rowlower.as_ptr(),
//     //     problem.rowupper.as_ptr(),
//     //     problem.matrix.astart.as_ptr(),
//     //     problem.matrix.aindex.as_ptr(),
//     //     problem.matrix.avalue.as_ptr(),
//     //     integrality.as_ptr()
//     // )
// }

// fn subset_model(region: &RegionNodes) {
//     let items = region
//         .children
//         .iter()
//         .map(|e| *e as u32)
//         .collect::<Vec<_>>();
//     let item_reqs = region.parents.iter().map(|e| *e as u32).collect::<Vec<_>>();
//     let weights = region.costs.iter().map(|e| *e as u32).collect::<Vec<_>>();
//     let state_1_values = region
//         .warehouse_counts
//         .iter()
//         .map(|e| *e as u32)
//         .collect::<Vec<_>>();
//     let state_2_values = region
//         .worker_counts
//         .iter()
//         .map(|e| *e as u32)
//         .collect::<Vec<_>>();
//     println!("Generating model...");
//     generate_model(
//         &items,
//         &item_reqs,
//         &weights,
//         &state_1_values,
//         &state_2_values,
//         0,
//         0,
//     );
// }

fn subset_solver(region: &RegionNodes) -> HighsPtr {
    // subset_model(region);

    println!("Getting solver...");
    unsafe {
        let highs = Highs_create();

        let option = CString::new("output_flag").unwrap();
        Highs_setBoolOptionValue(highs, option.as_ptr(), 0);

        let option = CString::new("log_to_console").unwrap();
        Highs_setBoolOptionValue(highs, option.as_ptr(), 0);

        let option = CString::new("threads").unwrap();
        Highs_setIntOptionValue(highs, option.as_ptr(), 1);

        let mps_file = CString::new("subset_selection.mps").unwrap();
        Highs_readModel(highs, mps_file.as_ptr());

        HighsPtr(highs)
    }
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
    fn from_highs(
        highs: HighsPtr,
        warehouse_count: u32,
        worker_count: u32,
        region: &RegionNodes,
    ) -> Self {
        let highs = highs.0;
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
            cost: cost as usize,
            indices,
            states,
        };
        debug!("{:?}", chain);
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
    fn extend(&mut self, index: usize, region: &RegionNodes) {
        for i in index..region.num_nodes {
            self.indices.push(i);
            let state = region.states[i];
            self.states.push(state);
            match state {
                1 => self.warehouse_count += region.warehouse_counts[i],
                _ => self.worker_count += region.worker_counts[i],
            }
            self.cost += region.costs[i];
        }
    }

    #[inline(always)]
    fn next_state(&mut self, region: &RegionNodes) {
        let index = match self.states.last() {
            Some(n) if n > &1 => self.reduce_last_state(region),
            _ => self.reduce(region),
        };
        if index < region.num_nodes {
            self.extend(index, region);
        }
    }

    #[inline(always)]
    fn reduce(&mut self, region: &RegionNodes) -> usize {
        self.states.pop();
        let index = self.indices.pop().unwrap();
        self.cost -= region.costs[index];
        self.warehouse_count -= region.warehouse_counts[index];
        region.jump_indices[index]
    }

    #[inline(always)]
    fn reduce_last_state(&mut self, region: &RegionNodes) -> usize {
        *self.states.last_mut().unwrap() -= 1;
        let index = *self.indices.last().unwrap();
        self.warehouse_count += region.warehouse_counts[index];
        self.worker_count -= region.worker_counts[index];
        index + 1
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

    #[inline(always)]
    fn indices_difference_from_set(&self, set: &HashSet<usize>) -> Vec<usize> {
        set.difference(&self.indices.iter().copied().collect())
            .cloned()
            .collect()
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

#[derive(Clone, Debug)]
struct JobControl {
    job_id: usize,
    state_value_sum_lb_pairs: Vec<(u32, u32)>,
}

impl JobControl {
    /// Returns job controls for parallel dominating chains generation.
    ///
    /// While the problem space for all combinations of nodes can be equally chunked for a number of
    /// workers the chains can not be as easily divided since the connected chains aren't contiguous
    /// within the domain. However, jobs can be assigned a starting chain and a stopping point. This
    /// is done by creating length limited chains until there would be more than the max number of
    /// workers and determining when the job should stop generating chain states so the jobs don't
    /// duplicate work. While this can under utilize hardware it works well for chunking the chain
    /// generation without needing to consider the full domain space.
    fn new(cli: &Cli, region: &RegionNodes) -> Result<JobControlVec> {
        let mut job_controls = vec![];
        let state_lb_pairs = Self::chunk_state_value_pairs(cli, region);
        for job_id in 0..cli.jobs.unwrap() as usize {
            job_controls.push(JobControl {
                job_id,
                state_value_sum_lb_pairs: state_lb_pairs[job_id].clone(),
            });
        }
        if log_enabled!(Debug) {
            debug!("Job Controls");
            job_controls.iter().for_each(|j| debug!("  {:?}", j));
        }
        Ok(job_controls)
    }

    fn chunk_state_value_pairs(cli: &Cli, region: &RegionNodes) -> Vec<Vec<(u32, u32)>> {
        let mut state_value_sum_pairs = (0..region.max_warehouse_count as u32 + 1)
            .cartesian_product(0..region.max_worker_count as u32 + 1)
            .collect::<Vec<_>>();
        let mut rng = thread_rng();
        state_value_sum_pairs.shuffle(&mut rng);

        let jobs = cli.jobs.unwrap() as usize;
        let chunk_size = (state_value_sum_pairs.len() + jobs) / jobs;
        state_value_sum_pairs
            .chunks(chunk_size)
            .map(|chunk| chunk.into())
            .collect()
    }
}

pub(crate) fn optimize(cli: Cli) -> Result<()> {
    let region_name = cli.region.clone().unwrap();
    let region_buildings = get_region_buildings(Some(region_name.clone()))?;
    region_buildings.iter().for_each(|b| trace!("{:#?}", b));
    let region = RegionNodes::new(region_buildings.get(&region_name).unwrap())?;
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

    if !cli.verbose.is_silent() {
        print_starting_status(&region);
    }

    info!("optimizing...");
    let chains = match cli.jobs.unwrap_or(1) {
        1 => optimize_dominating(&region)?,
        _ => optimize_dominating_par(&cli, &region)?,
    };
    info!("retaining...");
    let mut chains = chains.retain_dominating_to_vec();
    info!("writing...");
    write_chains(&cli, &region, &mut chains)?;

    Ok(())
}

fn optimize_dominating(region: &RegionNodes) -> Result<ChainMap> {
    let mut chains = ChainMap::new(region);
    let highs = subset_solver(region).0;

    let num_cols: HighsInt = unsafe { Highs_getNumCols(highs) };
    let num_rows: HighsInt = unsafe { Highs_getNumRows(highs) };
    let state_1_sum_row: HighsInt = unsafe { Highs_getNumRows(highs) - 2 };
    let state_2_sum_row: HighsInt = state_1_sum_row + 1;
    let highs_inf = unsafe { Highs_getInfinity(highs) };
    info!(
        "Optimizing {} combinations using {num_cols} cols and {num_rows} num_rows.",
        (region.max_warehouse_count + 1) * (region.max_worker_count + 1),
    );
    let mut counter = 0;

    for state_2_sum_lb in 0..region.max_worker_count as u32 + 1 {
        for state_1_sum_lb in 0..region.max_warehouse_count as u32 + 1 {
            let status = unsafe {
                Highs_changeRowBounds(highs, state_1_sum_row, state_1_sum_lb as f64, highs_inf);
                Highs_changeRowBounds(highs, state_2_sum_row, state_2_sum_lb as f64, highs_inf);
                Highs_run(highs);
                Highs_getModelStatus(highs)
            };
            if status == MODEL_STATUS_OPTIMAL {
                counter += 1;
                let chain =
                    Chain::from_highs(HighsPtr(highs), state_1_sum_lb, state_2_sum_lb, region);
                chains.insert_or_update(&chain);
            };
        }
    }
    let _ = unsafe { Highs_destroy(highs) };
    info!(
        "Optimizer processed {} combinations with {} feasible yielding {:?} chains.",
        (region.max_warehouse_count + 1) * (region.max_worker_count + 1),
        counter,
        chains.chains.num_elements(),
    );
    Ok(chains)
}

fn optimize_dominating_par(cli: &Cli, region: &RegionNodes) -> Result<ChainMap> {
    !todo!()
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
    for i in 0..chains.len() {
        chains[i].indices = chains[i]
            .indices
            .iter()
            .map(|j| region.children[*j])
            .collect();
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
