//! Generate dominating building chain combinations.
//!
//! Implementation of the pop_jump_push algorithm for iterating over all combinations of nodes in
//! an arborescence. The algorithm has been modified to handle multiple states for each node, namely
//! warehouse and worker usage states of buildings from houseinfo and to track the resulting counts
//! of those values.
//!
//! Each combination chain has its building node indices, states, cost, worker and warehouse counts
//! tracked. After each is generated its worker and warehouse counts are used to create an identity
//! key (using elegant_pair) and its cost is entered into a lookup table to indicate that it has
//! been seen and the chain is stored into an arena (stable-vec). Future chains with the same worker
//! and warehouse counts and lower cost replace existing chains.
//!
//! After all chains are generated the chains stored in the arena are then sifted to retain only
//! dominant chains. A dominant chain strictly dominates a different chain when it provides the same
//! or more workers and or warehouse counts for less or the same cost. The resulting chains are the
//! exact best-of-the-best chains for any combination of cost, worker and warehouse counts.
//!
//! The single and parallel versions of this modified version of the pop_jump_push algorithm perform
//! at about 35% of the reference pop_jump_push implementation with visits consisting of a blackbox
//! function call. This is expected since the reference implementation pops and extends a single
//! vector using a single lookup and is now popping and extending two vectors (indices and states)
//! as well as the extra lookups and cycles for the count tracking along with storing the dominant
//! chains. The `insert_or_update` time is about 17% of the overall time in the generate function
//! so greater than 50% more popping/extending plus the 17% for accounting puts us right about 35%
//! of the reference performance. In short, about 35% of pop_jump_push's blackbox throughput with
//! ~200M/s single and ~1.425B/s for 15x threads.

use std::cmp::min;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

use anyhow::{Ok, Result};
use chrono::Utc;
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
    fn new(cli: &Cli, region: &RegionNodes) -> Self {
        let chain = Self {
            worker_count: region.usage_counts.worker_count,
            warehouse_count: region.usage_counts.warehouse_count,
            cost: region.usage_counts.cost,
            indices: (0..region.num_nodes).collect::<Vec<_>>(),
            states: region.states.clone(),
        };
        if cli.progress {
            println!("{:?}", chain);
        }
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
        println!("Captured chain count: {:?}", chains.len());
        Self::retain_dominating(&mut chains);
        chains.sort_unstable_by_key(|chain| (chain.worker_count, chain.warehouse_count));
        chains
    }
}

#[derive(Clone, Debug)]
struct JobControl {
    job_id: usize,
    chain: Chain,
    stop_index: usize,
    stop_value: usize,
    #[allow(unused)]
    stop_state: usize,
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
    fn many_from_region(cli: &Cli, region: &RegionNodes) -> Result<JobControlVec> {
        let mut prefix_chains = Self::prefixes(cli, region)?;
        let min_index = prefix_chains[0].indices.last().unwrap() + 1;
        let base_indices = (0..min_index).collect::<HashSet<_>>();
        let indices = (0..region.num_nodes).collect::<Vec<_>>();

        let mut job_controls = vec![];
        for (job_id, chain) in prefix_chains.iter_mut().enumerate() {
            let deactivated_nodes = chain.indices_difference_from_set(&base_indices);
            let stop_value = Self::stop_value(region, &deactivated_nodes, min_index);
            let stop_index = chain.indices.len();
            let stop_state = region.states[stop_value];
            let tmp = &indices[stop_value..];
            chain.indices.extend_from_slice(tmp);
            let tmp = &region.states[stop_value..];
            chain.states.extend_from_slice(tmp);
            chain.worker_count = 0;
            chain.warehouse_count = 0;
            chain.cost = 0;
            for (i, &state) in chain.states.iter().enumerate().skip(1) {
                let building = region
                    .buildings
                    .get(&region.children[chain.indices[i]])
                    .unwrap();
                chain.cost += building.cost;
                if state == 1 {
                    chain.warehouse_count += building.warehouse_count;
                } else if state == 2 {
                    chain.worker_count += building.worker_count;
                }
            }

            job_controls.push(JobControl {
                job_id,
                chain: chain.clone(),
                stop_index,
                stop_value,
                stop_state,
            });
        }

        println!(
            "Using {} jobs of {:?} requested.",
            job_controls.len(),
            cli.jobs.unwrap()
        );
        if cli.progress {
            println!("--- Job Controls");
            job_controls.iter().for_each(|j| println!("{:?}", j));
            println!();
        }
        Ok(job_controls)
    }

    fn prefixes(cli: &Cli, region: &RegionNodes) -> Result<ChainVec> {
        let num_jobs = min(
            cli.jobs.unwrap_or(num_cpus::get() as u8) as usize,
            num_cpus::get(),
        );

        let mut chains = ChainVec::new();
        for num_nodes in 1..=num_jobs {
            let tmp_chains = Self::prefix_chains(cli, region, num_nodes)?;
            if tmp_chains.len() > num_jobs {
                break;
            }
            chains = tmp_chains;
        }

        if cli.progress {
            println!("--- Job Prefix Chains");
            chains.iter().for_each(|j| println!("{:?}", j));
        }
        Ok(chains)
    }

    fn prefix_chains(cli: &Cli, region: &RegionNodes, num_nodes: usize) -> Result<ChainVec> {
        let mut prefix_region = region.clone();
        prefix_region.num_nodes = num_nodes;
        prefix_region.jump_indices = region.jump_indices[0..num_nodes].to_vec();
        prefix_region.states = region.states[0..num_nodes].to_vec();
        generate_all(cli, &prefix_region)
    }

    fn stop_value(region: &RegionNodes, deactivated_nodes: &[usize], min_index: usize) -> usize {
        std::cmp::max(
            deactivated_nodes
                .iter()
                .map(|i| region.jump_indices[*i])
                .max()
                .unwrap_or(min_index),
            min_index,
        )
    }
}

pub(crate) fn generate(cli: Cli) -> Result<()> {
    let region_name = cli.region.clone().unwrap();
    if ["Calpheon City", "Valencia City", "Heidel"]
        .iter()
        .any(|&s| s == region_name)
    {
        println!(
            "*** Generating exact results for {} will take years. ***",
            region_name
        );
        println!("***    It is suggested you cancel this operation.    ***");
    }
    let region_buildings = get_region_buildings(Some(region_name.clone()))?;
    let region = RegionNodes::new(region_buildings.get(&region_name).unwrap())?;
    print_starting_status(&region);

    println!("[{:?}] generating...", Utc::now());
    let chains = match cli.jobs.unwrap_or(1) {
        1 => generate_dominating(&cli, &region)?,
        _ => generate_dominating_par(&cli, &region)?,
    };
    println!("[{:?}] retaining...", Utc::now());
    let chains = chains.retain_dominating_to_vec();
    println!("[{:?}] writing...", Utc::now());
    write_chains(&cli, &chains)?;

    Ok(())
}

fn generate_all(cli: &Cli, region: &RegionNodes) -> Result<ChainVec> {
    let mut chain = Chain::new(cli, region);
    let mut chains = Vec::<Chain>::new();
    let mut counter = 0;

    while !chain.indices.is_empty() {
        chains.push(chain.clone());
        chain.next_state(region);

        counter += 1;
    }

    if cli.progress {
        println!("\tVisited {counter} combinations.");
    }
    Ok(chains)
}

#[inline(always)]
fn generate_dominating(cli: &Cli, region: &RegionNodes) -> Result<ChainMap> {
    let mut chain = Chain::new(cli, region);
    let mut chains = ChainMap::new(region);
    let mut counter: usize = 0;

    while !chain.indices.is_empty() {
        chains.insert_or_update(&chain);
        chain.next_state(region);

        counter += 1;
        // if counter == 2_500_000_000 {
        //     break;
        // }
    }

    println!("  [{:?}] Visited {} combinations.", Utc::now(), counter);
    Ok(chains)
}

fn generate_dominating_par(cli: &Cli, region: &RegionNodes) -> Result<ChainMap> {
    let job_controls = JobControl::many_from_region(cli, region)?;
    let mut results = job_controls
        .into_par_iter()
        .map(|job| generate_dominating_par_worker(cli.clone(), region.clone(), job).unwrap())
        .collect::<ChainMapVec>();
    let results = ChainMap::flatten_many_by_insert_update(&mut results);
    Ok(results)
}

#[inline(always)]
fn generate_dominating_par_worker(
    _cli: Cli,
    region: RegionNodes,
    job: JobControl,
) -> Result<ChainMap> {
    let mut chains = ChainMap::new(&region);
    let mut chain = job.chain;
    let mut counter: usize = 0;

    while chain.indices.len() > job.stop_index && chain.indices[job.stop_index] >= job.stop_value {
        chains.insert_or_update(&chain);
        chain.next_state(&region);

        counter += 1;
        if counter == 12_500_000_000 {
            break;
        }
    }
    counter += 1;
    chains.insert_or_update(&chain);

    println!(
        "  [{:?}] Job {} visited {} combinations yielding {:?} chains.",
        Utc::now(),
        job.job_id,
        counter,
        chains.chains.num_elements()
    );

    Ok(chains)
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
        "Generating values for {} consisting of {} buildings in {} chains with {} storage/lodging combinations",
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

fn write_chains(cli: &Cli, chains: &Vec<Chain>) -> Result<()> {
    let region_name = cli.region.clone().unwrap();
    let file_name = region_name.replace(' ', "_");
    let path = format!("./data/housecraft/{}.json", file_name);
    let mut output = File::create(path.clone())?;

    let re = Regex::new(r"\{[^}]*?\}").unwrap();
    let json = to_string_pretty(chains)?;
    let json = re
        .replace_all(&json, |caps: &regex::Captures<'_>| {
            caps[0].replace("\n", "").replace(" ", "")
        })
        .to_string();
    output.write_all(json.as_bytes())?;

    println!(
        "Result: {} 'best of best' scored storage/lodging chains written to {}.",
        chains.len(),
        path
    );

    Ok(())
}
