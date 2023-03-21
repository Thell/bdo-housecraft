use crate::cli_args::Cli;
use crate::generate::extend_chain;
use crate::generate::generate_all_chains;
use crate::generate::reduce_chain;
use crate::generate::reduce_last_state;
use crate::generate_common::*;
use crate::region_nodes::*;
use anyhow::{Ok, Result};
use rayon::prelude::*;
use std::cmp::min;
use std::collections::HashSet;

use chrono::Utc;

#[derive(Clone, Debug)]
pub(crate) struct JobControl {
    pub job_id: usize,
    pub start_indices: Vec<usize>,
    pub start_states: Vec<usize>,
    pub stop_index: usize,
    pub stop_value: usize,
    #[allow(unused)]
    pub stop_state: usize,
}

fn get_prefix_chains(cli: &Cli, num_nodes: usize, region: &RegionNodes) -> Result<Vec<Chain>> {
    let mut prefix_region = region.clone();
    prefix_region.num_nodes = num_nodes;
    prefix_region.jump_indices = region.jump_indices[0..num_nodes].to_vec();
    prefix_region.states = region.states[0..num_nodes].to_vec();
    generate_all_chains(cli, &prefix_region)
}

fn generate_job_prefixes(cli: &Cli, region: &RegionNodes) -> Result<Vec<Chain>> {
    let num_jobs = min(
        cli.jobs.unwrap_or(num_cpus::get() as u8) as usize,
        num_cpus::get(),
    );

    let mut prefixes = Vec::<Chain>::new();
    for num_nodes in 1..=num_jobs {
        let tmp_prefixes = get_prefix_chains(cli, num_nodes, region)?;
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

#[inline(always)]
fn find_deactived_nodes(base_indices: &HashSet<usize>, prefix_indices: &[usize]) -> Vec<usize> {
    base_indices
        .difference(&prefix_indices.iter().copied().collect())
        .cloned()
        .collect()
}

#[inline(always)]
fn get_stop_value(region: &RegionNodes, deactivated_nodes: &[usize], min_index: usize) -> usize {
    std::cmp::max(
        deactivated_nodes
            .iter()
            .map(|i| region.jump_indices[*i])
            .max()
            .unwrap_or(min_index),
        min_index,
    )
}

#[inline(always)]
fn extend_prefix_from(indices: &[usize], prefix: &Chain, from: usize) -> Vec<usize> {
    prefix
        .indices
        .iter()
        .chain(indices[from..].iter())
        .cloned()
        .collect()
}

#[inline(always)]
fn extend_states_from(indices: &[usize], prefix: &Chain, from: usize) -> Vec<usize> {
    prefix
        .states
        .iter()
        .chain(indices[from..].iter())
        .cloned()
        .collect()
}

fn get_job_controls(cli: &Cli, region: &RegionNodes) -> Result<Vec<JobControl>> {
    let prefixes = generate_job_prefixes(cli, region)?;
    let min_index = prefixes[0].indices.last().unwrap() + 1;
    let base_indices = (0..min_index).collect::<HashSet<_>>();
    let indices = (0..region.num_nodes).collect::<Vec<_>>();

    let mut job_controls = vec![];
    for (job_id, prefix) in prefixes.iter().enumerate() {
        let deactivated_nodes = find_deactived_nodes(&base_indices, &prefix.indices);
        let stop_value = get_stop_value(region, &deactivated_nodes, min_index);
        let stop_index = prefix.indices.len();
        let stop_state = region.states[stop_value];
        let start_indices = extend_prefix_from(&indices, prefix, stop_value);
        let start_states = extend_states_from(&region.states, prefix, stop_value);
        job_controls.push(JobControl {
            job_id,
            start_indices,
            start_states,
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

fn generate_dominating_chains_par(
    cli: Cli,
    region: RegionNodes,
    job: &JobControl,
) -> Result<ChainMap> {
    let mut chains = ChainMap::new(&region);
    let mut chain = Chain::new_par(&cli, &region, job);
    let mut counter: usize = 0;

    while chain.indices.len() > job.stop_index && chain.indices[job.stop_index] >= job.stop_value {
        counter += 1;
        visit(&chain, &mut chains);
        let index = match chain.states.last() {
            Some(&2) => reduce_last_state(&mut chain, &region),
            _ => reduce_chain(&mut chain, &region),
        };
        if index < region.num_nodes {
            extend_chain(index, &mut chain, &region);
        }
    }
    counter += 1;
    visit(&chain, &mut chains);

    println!(
        "\tJob {} visited {} combinations yielding {:?} chains.",
        job.job_id,
        counter,
        chains.chains.num_elements()
    );

    Ok(chains)
}

pub(crate) fn generate_chains_par(cli: &Cli, region: &RegionNodes) -> Result<ChainMap> {
    let job_controls = get_job_controls(cli, region)?;

    let results = job_controls
        .par_iter()
        .map(|job| generate_dominating_chains_par(cli.clone(), region.clone(), job).unwrap())
        .collect::<Vec<ChainMap>>();

    println!("[{:?}] merging...", Utc::now());
    let mut chains = results[0].clone();
    results.iter().skip(1).for_each(|cm| {
        cm.chains
            .values()
            .for_each(|chain| visit(chain, &mut chains))
    });

    Ok(chains)
}
