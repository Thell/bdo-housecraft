use std::cmp::min;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

use anyhow::{Ok, Result};
use chrono::Utc;
use rayon::prelude::*;
use stable_vec::ExternStableVec;

use crate::cli_args::Cli;
use crate::houseinfo::*;
use crate::node_manipulation::{count_subtrees, count_subtrees_multistate};
use crate::region_nodes::RegionNodes;
use crate::retain_traits::SplitLastRetain;

type ChainVec = Vec<Chain>;

#[derive(Clone, Debug)]
pub(crate) struct ChainMap {
    pub seen: Vec<bool>,
    pub keys: ExternStableVec<usize>,
    pub chains: ExternStableVec<Chain>,
}

impl ChainMap {
    pub fn new(region: &RegionNodes) -> Self {
        let len = (std::cmp::max(region.max_worker_count, region.max_warehouse_count) + 1).pow(2);
        let seen = vec![false; len];
        let mut keys = ExternStableVec::<usize>::new();
        keys.reserve(len);
        let chains = ExternStableVec::<Chain>::new();
        Self { seen, keys, chains }
    }

    #[inline(always)]
    pub fn insert_or_update(&mut self, chain: &Chain) {
        let key = chain.elegant_pair();
        unsafe {
            if *self.seen.get_unchecked(key) {
                let index = self.keys.get_unchecked(key);
                let entry = self.chains.get_unchecked_mut(*index);
                if chain.usage_counts.cost < entry.usage_counts.cost {
                    chain.clone_into(entry);
                }
            } else {
                self.seen[key] = true;
                let index = self.chains.push(chain.to_owned());
                self.keys.insert(key, index);
            }
        }
    }

    fn retain_dominating_to_vec(&self) -> ChainVec {
        let mut chains = self.values_to_vec();
        println!("Captured chain count: {:?}", chains.len());

        chains.sort_unstable_by_key(|chain| {
            (
                chain.usage_counts.worker_count,
                chain.usage_counts.warehouse_count,
            )
        });

        chains.retain_split_last(|chain, remaining_chains| chain.dominates_all(remaining_chains));
        chains
    }

    pub fn values_to_vec(&self) -> ChainVec {
        self.chains.values().map(|c| c.to_owned()).collect()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Chain {
    pub indices: Vec<usize>,
    pub states: Vec<usize>,
    pub usage_counts: UsageCounters,
}

impl Chain {
    pub fn new(cli: &Cli, region: &RegionNodes) -> Self {
        let chain = Self {
            usage_counts: region.usage_counts.clone(),
            indices: (0..region.num_nodes).collect::<Vec<_>>(),
            states: region.states.clone(),
        };
        if cli.progress {
            println!("{:?}", chain);
        }
        chain
    }

    pub fn elegant_pair(&self) -> usize {
        let x = self.usage_counts.worker_count;
        let y = self.usage_counts.warehouse_count;

        if x != std::cmp::max(x, y) {
            y.pow(2) + x
        } else {
            x.pow(2) + x + y
        }
    }

    #[inline(always)]
    pub fn extend(&mut self, index: usize, region: &RegionNodes) {
        (index..region.num_nodes).for_each(|i| {
            self.indices.push(i);
            self.states.push(region.states[i]);
            match region.states[i] {
                1 => self.usage_counts.warehouse_count += region.warehouse_counts[i],
                _ => self.usage_counts.worker_count += region.worker_counts[i],
            }
            self.usage_counts.cost += region.costs[i];
        });
    }

    #[inline(always)]
    pub fn reduce(&mut self, region: &RegionNodes) -> usize {
        self.states.pop();
        let index = self.indices.pop().unwrap();
        self.usage_counts.cost -= region.costs[index];
        self.usage_counts.warehouse_count -= region.warehouse_counts[index];
        region.jump_indices[index]
    }

    #[inline(always)]
    pub fn reduce_last_state(&mut self, region: &RegionNodes) -> usize {
        *self.states.last_mut().unwrap() -= 1;
        let index = *self.indices.last().unwrap();
        self.usage_counts.warehouse_count += region.warehouse_counts[index];
        self.usage_counts.worker_count -= region.worker_counts[index];
        index + 1
    }

    #[inline(always)]
    pub fn dominates(&self, other_chain: &Chain) -> bool {
        !(other_chain.usage_counts.cost == self.usage_counts.cost
            && other_chain.usage_counts.worker_count >= self.usage_counts.worker_count
            && other_chain.usage_counts.warehouse_count > self.usage_counts.warehouse_count)
    }

    #[inline(always)]
    pub fn dominates_all(&self, others: &[Chain]) -> bool {
        others.iter().all(|other| self.dominates(other))
    }

    #[inline(always)]
    fn indices_difference_from_set(&self, set: &HashSet<usize>) -> Vec<usize> {
        set.difference(&self.indices.iter().copied().collect())
            .cloned()
            .collect()
    }
}

pub(crate) fn generate(cli: Cli) -> Result<()> {
    let region_name = cli.region.clone().unwrap();
    let region_buildings = get_region_buildings(Some(region_name.clone()))?;
    let region = RegionNodes::new(region_buildings.get(&region_name).unwrap())?;
    print_starting_status(&region);

    println!("[{:?}] generating...", Utc::now());
    let chains = match cli.jobs.unwrap_or(1) {
        1 => generate_chains(&cli, &region)?,
        _ => generate_chains_par(&cli, &region)?,
    };
    println!("[{:?}] retaining...", Utc::now());
    let chains = chains.retain_dominating_to_vec();
    println!("[{:?}] writing...", Utc::now());
    write_chains(&cli, &chains)?;

    Ok(())
}

pub(crate) fn generate_all_chains(cli: &Cli, region: &RegionNodes) -> Result<Vec<Chain>> {
    let mut chain = Chain::new(cli, region);
    let mut chains = Vec::<Chain>::new();
    let mut counter = 0;

    while !chain.indices.is_empty() {
        counter += 1;
        chains.push(chain.clone());
        let index = match chain.states.last() {
            Some(&2) => chain.reduce_last_state(region),
            _ => chain.reduce(region),
        };
        if index < region.num_nodes {
            chain.extend(index, region);
        }
    }

    if cli.progress {
        println!("\tVisited {counter} combinations.");
    }
    Ok(chains)
}

pub(crate) fn generate_chains(cli: &Cli, region: &RegionNodes) -> Result<ChainMap> {
    generate_dominating_chains(cli, region)
}

pub(crate) fn generate_chains_par(cli: &Cli, region: &RegionNodes) -> Result<ChainMap> {
    let job_controls = job_controls_from_region(cli, region)?;

    let results = job_controls
        .into_par_iter()
        .map(|job| generate_dominating_chains_par(cli.clone(), region.clone(), job).unwrap())
        .collect::<Vec<ChainMap>>();

    println!("[{:?}] merging...", Utc::now());
    let mut chains = results[0].clone();
    results.iter().skip(1).for_each(|cm| {
        cm.chains
            .values()
            .for_each(|chain| chains.insert_or_update(chain))
    });

    Ok(chains)
}

pub(crate) fn generate_dominating_chains(cli: &Cli, region: &RegionNodes) -> Result<ChainMap> {
    let mut chain = Chain::new(cli, region);
    let mut chains = ChainMap::new(region);
    let mut counter: usize = 0;

    while !chain.indices.is_empty() {
        counter += 1;
        chains.insert_or_update(&chain);
        let index = match chain.states.last() {
            Some(n) if n > &1 => chain.reduce_last_state(region),
            _ => chain.reduce(region),
        };
        if index < region.num_nodes {
            chain.extend(index, region);
        }
    }

    println!("  [{:?}] Visited {} combinations.", Utc::now(), counter);
    Ok(chains)
}

fn generate_dominating_chains_par(
    _cli: Cli,
    region: RegionNodes,
    job: JobControl,
) -> Result<ChainMap> {
    let mut chains = ChainMap::new(&region);
    let mut chain = job.chain;
    let mut counter: usize = 0;

    while chain.indices.len() > job.stop_index && chain.indices[job.stop_index] >= job.stop_value {
        counter += 1;
        chains.insert_or_update(&chain);
        let index = match chain.states.last() {
            Some(n) if n > &1 => chain.reduce_last_state(&region),
            _ => chain.reduce(&region),
        };
        if index < region.num_nodes {
            chain.extend(index, &region);
        }
        if counter == 100_000_000_000 {
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

type JobControlVec = Vec<JobControl>;

#[derive(Clone, Debug)]
pub(crate) struct JobControl {
    pub job_id: usize,
    pub chain: Chain,
    pub stop_index: usize,
    pub stop_value: usize,
    #[allow(unused)]
    pub stop_state: usize,
}

fn job_controls_from_region(cli: &Cli, region: &RegionNodes) -> Result<JobControlVec> {
    let mut prefix_chains = job_prefixes(cli, region)?;
    let min_index = prefix_chains[0].indices.last().unwrap() + 1;
    let base_indices = (0..min_index).collect::<HashSet<_>>();
    let indices = (0..region.num_nodes).collect::<Vec<_>>();

    let mut job_controls = vec![];
    for (job_id, chain) in prefix_chains.iter_mut().enumerate() {
        let deactivated_nodes = chain.indices_difference_from_set(&base_indices);
        let stop_value = job_stop_value(region, &deactivated_nodes, min_index);
        let stop_index = chain.indices.len();
        let stop_state = region.states[stop_value];
        let tmp = &indices[stop_value..];
        chain.indices.extend_from_slice(tmp);
        let tmp = &region.states[stop_value..];
        chain.states.extend_from_slice(tmp);

        chain.usage_counts = UsageCounters::new();
        for (i, &state) in chain.states.iter().enumerate().skip(1) {
            let building = region
                .buildings
                .get(&region.children[chain.indices[i]])
                .unwrap();
            chain.usage_counts.cost += building.cost;
            if state == 1 {
                chain.usage_counts.warehouse_count += building.warehouse_count;
            } else if state == 2 {
                chain.usage_counts.worker_count += building.worker_count;
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

fn job_prefixes(cli: &Cli, region: &RegionNodes) -> Result<ChainVec> {
    let num_jobs = min(
        cli.jobs.unwrap_or(num_cpus::get() as u8) as usize,
        num_cpus::get(),
    );

    let mut prefixes = Vec::<Chain>::new();
    for num_nodes in 1..=num_jobs {
        let tmp_prefixes = job_prefix_chains(cli, region, num_nodes)?;
        if tmp_prefixes.len() > num_jobs {
            break;
        }
        prefixes = tmp_prefixes;
    }

    if cli.progress {
        println!("--- Prefixes");
        prefixes.iter().for_each(|j| println!("{:?}", j));
    }
    Ok(prefixes)
}

fn job_prefix_chains(cli: &Cli, region: &RegionNodes, num_nodes: usize) -> Result<ChainVec> {
    let mut prefix_region = region.clone();
    prefix_region.num_nodes = num_nodes;
    prefix_region.jump_indices = region.jump_indices[0..num_nodes].to_vec();
    prefix_region.states = region.states[0..num_nodes].to_vec();
    generate_all_chains(cli, &prefix_region)
}

fn job_stop_value(region: &RegionNodes, deactivated_nodes: &[usize], min_index: usize) -> usize {
    std::cmp::max(
        deactivated_nodes
            .iter()
            .map(|i| region.jump_indices[*i])
            .max()
            .unwrap_or(min_index),
        min_index,
    )
}

pub(crate) fn print_starting_status(region: &RegionNodes) {
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
    let path = format!("./data/housecraft/{}.csv", file_name);
    let mut output = File::create(path.clone())?;
    writeln!(&mut output, "lodging,storage,cost,indices,states")?;
    chains.iter().for_each(|chain| {
        _ = writeln!(
            &mut output,
            "{:?},{:?},{:?},{:?},{:?}",
            chain.usage_counts.worker_count,
            chain.usage_counts.warehouse_count,
            chain.usage_counts.cost,
            chain.indices,
            chain.states,
        );
    });
    println!(
        "Result: {} 'best of best' scored storage/lodging chains written to {}.",
        chains.len(),
        path
    );

    Ok(())
}