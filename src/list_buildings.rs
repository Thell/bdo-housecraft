use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::{bail, Context, Ok, Result};
use comfy_table::{Attribute, Cell, Table};
use once_cell::sync::OnceCell;
use serde::Deserialize;

use crate::cli_args::Cli;
use crate::houseinfo::*;
use crate::region_nodes::RegionNodes;

static REGION: OnceCell<RegionNodes> = OnceCell::new();

type ChainVec = Vec<Chain>;

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct Chain {
    lodging: u16,
    storage: u16,
    cost: u16,
    indices: Vec<usize>,
    states: Vec<usize>,
}

impl Chain {
    fn many_from_region_json(region_name: &str) -> Result<ChainVec> {
        let filename = format!("{}.json", region_name.replace(' ', "_"));
        let path = PathBuf::from("./data/housecraft").join(filename);
        let file = File::open(&path).with_context(|| format!("Can't find {}", path.display()))?;
        let reader = BufReader::new(file);
        let chains = serde_json::from_reader(reader)?;
        Ok(chains)
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let region = REGION.get().unwrap();

        let mut table = Table::new();
        table.load_preset(HOUSECRAFT_TABLE_STYLE);
        table.set_header(vec![
            Cell::new("Building").add_attribute(Attribute::Dim),
            Cell::new("🪙"),
            Cell::new("📦"),
            Cell::new("👷"),
        ]);

        for (index, state) in self.indices.iter().zip(self.states.iter()).skip(1) {
            let building = &region.buildings[&region.children[*index]];
            let (warehouse_count, worker_count) = if *state == 1 {
                (building.warehouse_count.to_string(), "".to_string())
            } else {
                ("".to_string(), building.worker_count.to_string())
            };
            table.add_row(vec![
                &building.building_name,
                &building.cost.to_string(),
                &warehouse_count,
                &worker_count,
            ]);
        }

        table.add_row(vec![
            Cell::new("Totals").add_attribute(Attribute::Bold),
            Cell::new(self.cost).add_attribute(Attribute::Bold),
            Cell::new(self.storage).add_attribute(Attribute::Bold),
            Cell::new(self.lodging).add_attribute(Attribute::Bold),
        ]);

        writeln!(f, "{table}")?;
        std::result::Result::Ok(())
    }
}

fn filter_by_storage_and_lodging(chains: &mut ChainVec, cli: Cli) {
    if cli.storage.is_none() && cli.lodging.is_none() {
        return;
    }
    let cost_anchor = OnceCell::new();
    let lodging = cli.lodging.unwrap_or(0);
    let storage = cli.storage.unwrap_or(0);
    chains.retain(|chain| chain.storage >= storage && chain.lodging >= lodging);
    chains.retain(|chain| chain.cost == *cost_anchor.get_or_init(|| chain.cost));
}

fn initialize_region(cli: &Cli) -> Result<String> {
    let region_name = cli.region.as_ref().unwrap().clone();
    let forbidden_regions = ["Calpheon City", "Valencia City", "Heidel"];
    if forbidden_regions.contains(&region_name.as_str()) {
        let msg = format!(
            "*** Generating exact results for {} will take years. ***\n \
             *** Once the optimizer is implemented listing will work. ***",
            region_name
        );
        bail!("{}", msg);
    }
    let region = get_region_buildings(Some(region_name.clone()))?;
    let region = RegionNodes::new(region.get(&region_name).unwrap())?;
    let _region = REGION.get_or_init(|| region);
    Ok(region_name)
}

pub(crate) fn list_buildings(cli: Cli) -> Result<()> {
    let region_name = initialize_region(&cli)?;
    let mut chains = Chain::many_from_region_json(&region_name)?;
    filter_by_storage_and_lodging(&mut chains, cli);

    if chains.is_empty() {
        let region = REGION.get().unwrap();
        bail!(
            "The maximum storage and lodging counts for {} are {} and {}.",
            region.region_name,
            region.max_warehouse_count,
            region.max_worker_count
        );
    } else {
        println!();
        chains.iter().for_each(|chain| println!("{chain}"));
    }
    Ok(())
}