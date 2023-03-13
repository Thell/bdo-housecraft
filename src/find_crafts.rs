use super::houseinfo::*;
use anyhow::{Ok, Result};
use comfy_table::{Attribute, Cell, Row, Table};
use console::style;
use std::collections::BTreeMap;
use std::collections::HashMap;

type RegionCraftMap = BTreeMap<String, CraftBuildingMap>;

lazy_static! {
    static ref CRAFT_USAGE: HashMap<u32, String> = {
        let map =
            read_csv_data("HouseInfoReceipe.csv").expect("Error reading HouseInfoReceipe.csv");
        map
    };
}

fn print_listing(crafts_summary: RegionCraftMap) -> Result<()> {
    let header_row = Row::from(vec![
        Cell::new("Crafting Usage").add_attribute(Attribute::Dim),
        Cell::new("Key").add_attribute(Attribute::Dim),
        Cell::new("Building").add_attribute(Attribute::Dim),
        Cell::new("Cost").add_attribute(Attribute::Dim),
    ]);

    for (region, usages) in crafts_summary.iter() {
        let mut table = Table::new();
        table.load_preset(HOUSECRAFT_TABLE_STYLE);
        table.set_header(header_row.clone());

        for (usage, buildings) in usages.iter() {
            for building in buildings.1.iter() {
                table.add_row(vec![
                    usage,
                    &building.key.to_string(),
                    &building.building_name.to_string(),
                    &building.cost.to_string(),
                ]);
            }
        }
        println!("\n{}", style(region).bold());
        println!("{table}");
    }

    Ok(())
}

fn insert_craft_building(
    region_crafts: &mut CraftBuildingMap,
    craft: &CraftList,
    building: &Building,
) {
    if let Some(usage) = CRAFT_USAGE.get(&craft.item_craft_index) {
        let craft_name = format!("{} {}", usage, craft.house_level);
        if let Some(craft) = region_crafts.get_mut(&craft_name) {
            craft.0 += 1;
            craft.1.push(building.clone())
        } else {
            region_crafts.insert(craft_name, (1, vec![building.clone()]));
        }
    }
}

fn summarize_craft_buildings(regions_buildings: RegionBuildingMap) -> Result<RegionCraftMap> {
    let mut crafts_summary = RegionCraftMap::new();
    for (region, buildings) in regions_buildings.iter() {
        let mut region_crafts = CraftBuildingMap::new();
        for (_key, building) in buildings.iter() {
            for craft in building.craft_list.iter() {
                insert_craft_building(&mut region_crafts, craft, building);
            }
        }
        crafts_summary.insert(region.to_string(), region_crafts);
    }
    Ok(crafts_summary)
}

fn filter_craft_list(craft_match: &str, building: &Building) -> Vec<CraftList> {
    building
        .craft_list
        .clone()
        .into_iter()
        .filter(|craft| {
            CRAFT_USAGE
                .get(&craft.item_craft_index)
                .map(|usage| format!("{} {}", usage, craft.house_level))
                .unwrap()
                .contains(craft_match)
        })
        .collect()
}

fn filter_craft_buildings(
    regions_buildings: RegionBuildingMap,
    craft_match: String,
) -> Result<RegionBuildingMap> {
    let mut filtered_regions = BTreeMap::<String, BuildingMap>::new();

    for (region, buildings) in regions_buildings.iter() {
        let mut filtered_buildings = BuildingMap::new();
        for (key, building) in buildings.iter() {
            let craft_list = filter_craft_list(&craft_match, building);
            if !craft_list.is_empty() {
                let mut new_building = building.clone();
                new_building.craft_list = craft_list;
                filtered_buildings.insert(*key, new_building);
            }
        }
        if !filtered_buildings.is_empty() {
            filtered_regions.insert(region.to_string(), filtered_buildings);
        }
    }
    Ok(filtered_regions)
}

pub(crate) fn find_craft_buildings(town_name: Option<String>, craft: String) -> Result<()> {
    let mut regions_buildings = merge_houseinfo_data()?;
    if let Some(town_name) = town_name {
        regions_buildings.retain(|k, _| *k == town_name);
    }
    let craft_buildings = filter_craft_buildings(regions_buildings, craft)?;
    let crafts_summary = summarize_craft_buildings(craft_buildings)?;
    print_listing(crafts_summary)
}
