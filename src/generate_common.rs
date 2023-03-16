use crate::cli_args::Cli;
use crate::generate::generate_chains;
use crate::houseinfo::UsageCounters;
use crate::houseinfo::*;
use crate::node_manipulation::{count_subtrees, count_subtrees_multistate};
use crate::region_nodes::RegionNodes;
use anyhow::{Ok, Result};
use console::style;
use std::collections::BTreeMap;

pub(crate) type BestChains = BTreeMap<(usize, usize), Chain>;

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
}

pub(crate) fn print_chain(chain: &Chain, marker: char) {
    match marker {
        '-' => {
            println!(
                "{} {:?}, {:?}, {:?}, {:?}, {:?}",
                style(marker).red(),
                chain.usage_counts.worker_count,
                chain.usage_counts.warehouse_count,
                chain.usage_counts.cost,
                style(chain.indices.clone()).dim(),
                style(chain.states.clone()).dim(),
            )
        }
        '+' => {
            println!(
                "{} {:?}, {:?}, {:?}, {:?}, {:?}",
                style(marker).green(),
                style(chain.usage_counts.worker_count).bold(),
                style(chain.usage_counts.warehouse_count).bold(),
                style(chain.usage_counts.cost).bold(),
                style(chain.indices.clone()).dim(),
                style(chain.states.clone()).dim(),
            )
        }
        _ => {
            println!(
                "{} {:?}, {:?}, {:?}, {:?}, {:?}",
                style(marker).bold(),
                style(chain.usage_counts.worker_count).dim(),
                style(chain.usage_counts.warehouse_count).dim(),
                style(chain.usage_counts.cost).dim(),
                style(chain.indices.clone()).bold(),
                style(chain.states.clone()).bold(),
            )
        }
    };
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

#[inline]
pub(crate) fn visit(chain: &Chain, best_chains: &mut BestChains, cli: &Cli) {
    let key = (
        chain.usage_counts.worker_count,
        chain.usage_counts.warehouse_count,
    );
    if let Some(current_best) = best_chains.get_mut(&key) {
        if chain.usage_counts.cost < current_best.usage_counts.cost {
            if cli.progress {
                print_chain(current_best, '-');
                print_chain(chain, '+')
            }
            *current_best = chain.clone();
        }
    } else {
        best_chains.insert(key, chain.clone());
        if cli.progress {
            print_chain(chain, '+');
        }
    }
}

use std::fs::File;
use std::io::Write;

pub(crate) fn generate_main(cli: Cli) -> Result<()> {
    let region_name = cli.region.clone().unwrap();
    let region_buildings = get_region_buildings(Some(region_name.clone()))?;
    let region = RegionNodes::new(region_buildings.get(&region_name).unwrap())?;
    print_starting_status(&region);

    let best_chains = generate_chains(&cli, &region)?;

    let mut best_of_best_chains = best_chains.clone();
    best_of_best_chains.retain(|_k, v| {
        !best_chains.values().any(|c| {
            (c.usage_counts.cost == v.usage_counts.cost)
                && (c.usage_counts.worker_count >= v.usage_counts.worker_count)
                && (c.usage_counts.warehouse_count > v.usage_counts.warehouse_count)
        })
    });

    let file_name = region_name.replace(" ", "_");
    let path = format!("./data/housecraft/{}.csv", file_name);
    let mut output = File::create(path.clone())?;
    for chain in best_of_best_chains.iter() {
        writeln!(&mut output, "{:?}", chain)?;
    }
    println!(
        "Result: {} 'best of best' scored storage/lodging chains written to {}.",
        best_of_best_chains.len(),
        path.to_string()
    );
    Ok(())
}
