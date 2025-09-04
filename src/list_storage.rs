use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::{Context, Ok, Result};
use comfy_table::{Attribute, Cell, Table};
use serde::Deserialize;

use crate::houseinfo::*;

type ChainVec = Vec<Chain>;

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct Chain {
    total_storage: u16,
    total_cost: u16,
    storage: u16,
    cost: u16,
    indices: Vec<usize>,
    states: Vec<usize>,
}

impl Chain {
    fn all_from_regions_storage_json() -> Result<ChainVec> {
        let filename = "all_regions_storage.json";
        let path = PathBuf::from("./data/housecraft").join(filename);
        let file = File::open(&path).with_context(|| format!("Can't find {}", path.display()))?;
        let reader = BufReader::new(file);
        let chains = serde_json::from_reader(reader)?;
        Ok(chains)
    }
}

fn region_name_from_chain(chain: &Chain, regions_buildings: &RegionBuildingMap) -> Result<String> {
    let region_id = chain.indices[0];
    for (name, building_map) in regions_buildings.iter() {
        if building_map
            .iter()
            .any(|(_, building)| building.region_key == region_id)
        {
            return Ok(name.replace('_', " "));
        }
    }
    Err(anyhow::anyhow!("Region not found for id {}", region_id))
}

/// Processes and emits storage chain entries in table format from `all_regions_storage.json`
pub(crate) fn list_storage() -> Result<()> {
    let regions_buildings = parse_houseinfo_data()?;
    let chains = Chain::all_from_regions_storage_json()?;

    let mut seen_buildings: HashMap<usize, HashSet<usize>> = HashMap::new();

    let mut table = Table::new();
    table.load_preset(HOUSECRAFT_TABLE_STYLE);
    table.set_header(vec![
        Cell::new("Region").add_attribute(Attribute::Dim),
        Cell::new("Building").add_attribute(Attribute::Dim),
        Cell::new("C").add_attribute(Attribute::Dim),
        Cell::new("S").add_attribute(Attribute::Dim),
        Cell::new("S/C").add_attribute(Attribute::Dim),
        Cell::new("ttlC").add_attribute(Attribute::Dim),
        Cell::new("ttlS").add_attribute(Attribute::Dim),
        Cell::new("ttlS/ttlC").add_attribute(Attribute::Dim),
    ]);

    let mut previous_region_name = "".to_string();
    let mut total_cost = 0;
    let mut total_storage = 0;

    for (i, chain) in chains.iter().enumerate() {
        let region_id = chain.indices[0];
        let next_region = chains.get(i + 1).map(|c| c.indices[0]);
        if next_region == Some(region_id) {
            continue;
        }

        let region_name = region_name_from_chain(chain, &regions_buildings)?;
        let seen = seen_buildings.entry(region_id).or_default();

        for (index, _state) in chain.indices.iter().zip(chain.states.iter()).skip(1) {
            if seen.contains(index) {
                continue;
            }
            seen.insert(*index);

            let building = regions_buildings
                .values()
                .flat_map(|map| map.values())
                .find(|b| b.key == *index)
                .unwrap();

            let region_name_value = if region_name != previous_region_name {
                previous_region_name = region_name.clone();
                region_name.clone()
            } else {
                "".to_string()
            };

            total_cost += building.cost;
            total_storage += building.warehouse_count;
            let sp_ratio = building.warehouse_count as f32 / building.cost as f32;
            let ttlsp_ratio = total_storage as f32 / total_cost as f32;

            table.add_row(vec![
                &region_name_value,
                &building.building_name,
                &building.cost.to_string(),
                &building.warehouse_count.to_string(),
                &format!("{:.2}", sp_ratio),
                &total_cost.to_string(),
                &total_storage.to_string(),
                &format!("{:.2}", ttlsp_ratio),
            ]);
        }
    }

    println!("\n{table}");
    Ok(())
}
