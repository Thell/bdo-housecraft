use crate::cli_args::Cli;
use crate::generate_common::*;
use crate::region_nodes::*;
use anyhow::{Ok, Result};

#[inline(always)]
pub(crate) fn extend_chain(index: usize, chain: &mut Chain, region: &RegionNodes) {
    let mut index = index;
    while index < region.num_nodes {
        chain.indices.push(index);
        chain.states.push(region.states[index]);
        match region.states[index] {
            1 => chain.usage_counts.warehouse_count += region.state_values[&1][index],
            _ => chain.usage_counts.worker_count += region.state_values[&2][index],
        }
        chain.usage_counts.cost += region.costs[index];
        index += 1;
    }
}

#[inline(always)]
pub(crate) fn reduce_chain(chain: &mut Chain, region: &RegionNodes) -> usize {
    chain.states.pop();
    let index = chain.indices.pop().unwrap();
    chain.usage_counts.cost -= region.costs[index];
    chain.usage_counts.warehouse_count -= region.state_values[&1][index];
    region.jump_indices[index]
}

#[inline(always)]
pub(crate) fn reduce_last_state(chain: &mut Chain, region: &RegionNodes) -> usize {
    *chain.states.last_mut().unwrap() = 1;
    let index = *chain.indices.last().unwrap();
    chain.usage_counts.warehouse_count += region.state_values[&1][index];
    chain.usage_counts.worker_count -= region.state_values[&2][index];
    index + 1
}

pub(crate) fn generate_chains(cli: &Cli, region: &RegionNodes) -> Result<BestChains> {
    let mut chain = Chain::new(cli, region);
    let mut chains = BestChains::new();
    let mut counter = 0;

    while !chain.indices.is_empty() {
        counter += 1;
        visit(&chain, &mut chains, cli);
        let index = match chain.states.last() {
            Some(&2) => reduce_last_state(&mut chain, region),
            _ => reduce_chain(&mut chain, region),
        };
        if index < region.num_nodes {
            extend_chain(index, &mut chain, region);
        }
    }

    if cli.progress {
        println!("\tVisited {counter} combinations.");
    }
    Ok(chains)
}
