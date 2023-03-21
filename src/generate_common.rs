use crate::cli_args::Cli;
use crate::generate::generate_chains;
use crate::generate_par::generate_chains_par;
use crate::generate_par::JobControl;
use crate::houseinfo::UsageCounters;
use crate::houseinfo::*;
use crate::node_manipulation::{count_subtrees, count_subtrees_multistate};
use crate::region_nodes::RegionNodes;
use crate::retain_traits::SplitLastRetain;
use anyhow::{Ok, Result};
use std::fs::File;
use std::io::Write;

use chrono::Utc;

// pub(crate) type ChainMap = Vec<(usize, Chain)>;

// This is a small sample: 12_500_000_000 'entry' calls on 5964 unique keys.
// 15 threads processing ~ 833,333,333

// 168477.9889
// Using:
use stable_vec::ExternStableVec;

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
}

// Using:
//    pub(crate) fn visit(chain: &Chain, chains: &mut ChainMap) {
//        chains
//            .entry(chain.elegant_pair())
//            .and_modify(|entry| {
//                if chain.usage_counts.cost < entry.usage_counts.cost {
//                    chain.clone_into(entry);
//                }
//            })
//            .or_insert_with(|| chain.to_owned());
//    }

// 210657.1583ms
// use ahash::RandomState;
// use std::collections::HashMap;
// pub(crate) type ChainMap = HashMap<usize, Chain, RandomState>;

// 313227.4553ms
// use std::collections::HashMap;
// pub(crate) type ChainMap = HashMap<usize, Chain>;

// 510342.9126ms
// use nohash_hasher::NoHashHasher;
// use std::{collections::HashMap, hash::BuildHasherDefault};
// pub(crate) type ChainMap = HashMap<usize, Chain, BuildHasherDefault<NoHashHasher<usize>>>;

// 539879.451ms
// use std::collections::BTreeMap;
// pub(crate) type ChainMap = BTreeMap<usize, Chain>;

////////////////////////////////////////////////////////////////////////////////////////////////////

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

    pub fn new_par(cli: &Cli, region: &RegionNodes, job: &JobControl) -> Self {
        let indices = job.start_indices.clone();
        let states = job.start_states.clone();
        let mut usage_counts = UsageCounters::new();

        for (i, &state) in states.iter().enumerate().skip(1) {
            let building = region.buildings.get(&region.children[indices[i]]).unwrap();
            usage_counts.cost += building.cost;
            if state == 1 {
                usage_counts.warehouse_count += building.warehouse_count;
            } else {
                usage_counts.worker_count += building.worker_count;
            }
        }
        let chain = Self {
            indices,
            states,
            usage_counts,
        };
        if cli.progress {
            println!("j-{} start: {:?}", job.job_id, chain)
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

#[inline(always)]
pub(crate) fn visit(chain: &Chain, chains: &mut ChainMap) {
    let key = chain.elegant_pair();
    unsafe {
        if *chains.seen.get_unchecked(key) {
            let index = chains.keys.get_unchecked(key);
            let entry = chains.chains.get_unchecked_mut(*index);
            if chain.usage_counts.cost < entry.usage_counts.cost {
                chain.clone_into(entry);
            }
        } else {
            chains.seen[key] = true;
            let index = chains.chains.push(chain.to_owned());
            chains.keys.insert(key, index);
        }
    }
}

#[inline(always)]
fn dominates(chain: &Chain, other_chain: &Chain) -> bool {
    !(other_chain.usage_counts.cost == chain.usage_counts.cost
        && other_chain.usage_counts.worker_count >= chain.usage_counts.worker_count
        && other_chain.usage_counts.warehouse_count > chain.usage_counts.warehouse_count)
}

#[inline(always)]
fn dominates_all(chain: &Chain, chains: &[Chain]) -> bool {
    chains.iter().all(|other| dominates(chain, other))
}

fn retain_dominating_chains(chains: &ChainMap) -> Vec<Chain> {
    let mut chains: Vec<Chain> = chains.chains.values().map(|c| c.to_owned()).collect();
    println!("Captured chain count: {:?}", chains.len());

    chains.sort_unstable_by_key(|chain| {
        (
            chain.usage_counts.worker_count,
            chain.usage_counts.warehouse_count,
        )
    });
    chains.retain_split_last(|chain, remaining_chains| dominates_all(chain, remaining_chains));
    chains
}

pub(crate) fn generate_main(cli: Cli) -> Result<()> {
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
    let chains = retain_dominating_chains(&chains);
    println!("[{:?}] writing...", Utc::now());
    write_chains(&cli, &chains)?;

    Ok(())
}
