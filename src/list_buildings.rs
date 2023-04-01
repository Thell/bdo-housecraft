use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::{Context, Ok, Result};
use serde::Deserialize;
use serde_json::from_reader;

use crate::cli_args::Cli;
use crate::houseinfo::*;
use crate::region_nodes::RegionNodes;

type ChainVec = Vec<Chain>;

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct Chain {
    lodging: u16,
    storage: u16,
    cost: u16,
    indices: Vec<usize>,
    states: Vec<usize>,
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "lodging: {}, storage: {}, cost: {}, indices: {:?}, states: {:?}",
            self.lodging, self.storage, self.cost, self.indices, self.states
        )
    }
}

fn get_json_file(region_name: &str) -> Result<File> {
    let region_name = region_name.replace(' ', "_");
    let path = Path::new("./data/housecraft/filename.json");
    let path = path.with_file_name(format!("{}.json", region_name));
    let path_string = path.clone();
    let path_string = path_string.as_path().display();
    File::open(path).context(format!("Can't find {path_string})"))
}

fn read_chains_json_data(region_name: &str) -> Result<ChainVec> {
    let json_file = get_json_file(region_name)?;
    let reader = BufReader::new(json_file);
    let chains: ChainVec = from_reader(reader)?;
    Ok(chains)
}

fn list_all(_cli: &Cli, _region: &RegionNodes, chains: &ChainVec) {
    chains.iter().for_each(|chain| println!("{}", chain));
    todo!("Implementing - get building and state info... possibly colorize too.")
}

fn list_by_worker_count(_cli: &Cli, _region: &RegionNodes, _chains: &ChainVec, _worker_count: u8) {
    todo!("Implement")
}

fn list_by_warehouse_count(
    _cli: &Cli,
    _region: &RegionNodes,
    _chains: &ChainVec,
    _warehouse_count: u8,
) {
    todo!("Implement")
}

fn list_by_worker_and_warehouse_count(
    _cli: &Cli,
    _region: &RegionNodes,
    _chains: &ChainVec,
    _worker_count: u8,
    _warehouse_count: u8,
) {
    todo!("Implement")
}

pub(crate) fn list_buildings(cli: Cli) -> Result<()> {
    let region_name = cli.region.clone().unwrap();
    if ["Calpheon City", "Valencia City", "Heidel"]
        .iter()
        .any(|&s| s == region_name)
    {
        println!(
            "*** Generating exact results for {} will take years. ***",
            region_name
        );
        println!("*** Once the optimizer is implemented listing will work. ***");
    }

    let region_buildings = get_region_buildings(Some(region_name.clone()))?;
    let region = RegionNodes::new(region_buildings.get(&region_name).unwrap())?;
    let chains = read_chains_json_data(&region_name)?;

    let desired_counts = (cli.lodging_count, cli.storage_count);
    match desired_counts {
        (None, None) => list_all(&cli, &region, &chains),
        (Some(worker_count), None) => list_by_worker_count(&cli, &region, &chains, worker_count),
        (None, Some(warehouse_count)) => {
            list_by_warehouse_count(&cli, &region, &chains, warehouse_count)
        }
        (Some(worker_count), Some(warehouse_count)) => list_by_worker_and_warehouse_count(
            &cli,
            &region,
            &chains,
            worker_count,
            warehouse_count,
        ),
    }

    Ok(())
}
