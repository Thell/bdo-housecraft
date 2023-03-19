use crate::cli_args::Cli;
use crate::generate::generate_chains;
use crate::generate_par::generate_chains_par;
use crate::generate_par::JobControl;
use crate::houseinfo::UsageCounters;
use crate::houseinfo::*;
use crate::node_manipulation::{count_subtrees, count_subtrees_multistate};
use crate::region_nodes::RegionNodes;
use ahash::RandomState;
use anyhow::{Ok, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

pub(crate) type BestChains = HashMap<(usize, usize), Chain, RandomState>;

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
        "With a maximum cost of {} with {} storage and {} lodging.",
        region.usage_counts.cost,
        region.usage_counts.warehouse_count,
        region.usage_counts.worker_count
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
pub(crate) fn visit(chain: &Chain, best_chains: &mut BestChains) {
    let key = (
        chain.usage_counts.worker_count,
        chain.usage_counts.warehouse_count,
    );
    let current_best = best_chains.entry(key).or_insert_with(|| chain.clone());
    if chain.usage_counts.cost < current_best.usage_counts.cost {
        *current_best = chain.clone();
    }
}

#[inline(always)]
fn dominates(other_chain: &Chain, chain: &Chain) -> bool {
    other_chain.usage_counts.cost == chain.usage_counts.cost
        && other_chain.usage_counts.worker_count >= chain.usage_counts.worker_count
        && other_chain.usage_counts.warehouse_count > chain.usage_counts.warehouse_count
}

#[inline(always)]
fn is_dominated_by_any(chain: &Chain, chains: &[Chain], start_pos: usize) -> bool {
    chains[start_pos..].iter().any(|c| dominates(c, chain))
}

fn retain_best_chains(chains: &BestChains) -> Vec<Chain> {
    let mut chains: Vec<Chain> = chains.values().cloned().collect();
    chains.sort_unstable_by_key(|chain| {
        (
            chain.usage_counts.worker_count,
            chain.usage_counts.warehouse_count,
        )
    });

    let mut best_chains = vec![];
    for (i, chain) in chains.iter().enumerate() {
        if !is_dominated_by_any(chain, &chains, i + 1) {
            best_chains.push(chain.clone());
        }
    }
    best_chains
}

pub(crate) fn generate_main(cli: Cli) -> Result<()> {
    let region_name = cli.region.clone().unwrap();
    let region_buildings = get_region_buildings(Some(region_name.clone()))?;
    let region = RegionNodes::new(region_buildings.get(&region_name).unwrap())?;
    print_starting_status(&region);

    let chains = match cli.jobs.unwrap_or(1) {
        1 => generate_chains(&cli, &region)?,
        _ => generate_chains_par(&cli, &region)?,
    };

    let chains = retain_best_chains(&chains);
    write_chains(&cli, &chains)?;

    Ok(())
}
