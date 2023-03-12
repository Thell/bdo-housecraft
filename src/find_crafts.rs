use super::houseinfo::*;
use anyhow::{Ok, Result};
use comfy_table::{Attribute, Cell, Table};
use console::style;
use std::collections::BTreeMap;

fn filter_craft_buildings(
    regions_buildings: BTreeMap<String, BTreeMap<u32, Building>>,
    craft_match: String,
) -> Result<BTreeMap<String, BTreeMap<u32, Building>>> {
    let craft_usage = read_csv_data("HouseInfoReceipe.csv")?;
    let mut filtered_regions = BTreeMap::<String, BTreeMap<u32, Building>>::new();

    for (region, buildings) in regions_buildings.iter() {
        let mut filtered_buildings = BTreeMap::<u32, Building>::new();
        for (key, building) in buildings.into_iter() {
            let filtered_craft_list: Vec<CraftList> = building
                .craft_list
                .clone()
                .into_iter()
                .filter(|craft| {
                    craft_usage
                        .get(&craft.item_craft_index)
                        .map(|usage| format!("{} {}", usage, craft.house_level))
                        .unwrap()
                        .contains(&craft_match)
                })
                .collect();
            if !filtered_craft_list.is_empty() {
                let mut new_building = building.clone();
                new_building.craft_list = filtered_craft_list;
                filtered_buildings.insert(*key, new_building);
            }
        }
        if !filtered_buildings.is_empty() {
            filtered_regions.insert(region.to_string(), filtered_buildings);
        }
    }
    Ok(filtered_regions)
}

fn summarize_craft_buildings(
    regions_buildings: BTreeMap<String, BTreeMap<u32, Building>>,
) -> Result<BTreeMap<String, BTreeMap<String, (u32, Vec<Building>)>>> {
    let mut crafts_summary = BTreeMap::<String, BTreeMap<String, (u32, Vec<Building>)>>::new();
    let craft_usage = read_csv_data("HouseInfoReceipe.csv")?;
    for (region, buildings) in regions_buildings.iter() {
        let mut region_crafts = BTreeMap::<String, (u32, Vec<Building>)>::new();
        for (_, building) in buildings.iter() {
            for craft in building.craft_list.iter() {
                if let Some(usage) = craft_usage.get(&craft.item_craft_index) {
                    let craft_name = format!("{} {}", usage, craft.house_level);
                    if let Some(craft) = region_crafts.get_mut(&craft_name) {
                        craft.0 += 1;
                        craft.1.push(building.clone())
                    } else {
                        region_crafts.insert(craft_name, (1, vec![building.clone()]));
                    }
                } else {
                    eprint!(
                        "Invalid HouseInfoReceipe index {} encountered and skipped.",
                        craft.item_craft_index
                    );
                }
            }
        }
        crafts_summary.insert(region.to_string(), region_crafts);
    }
    Ok(crafts_summary)
}

// pub const UTF8_COMFY_TABLE_STYLE: &str = "0123456789abcdefghi";
pub const UTF8_COMFY_TABLE_STYLE: &str = "   ═────      ═  ══";

fn print_listing(
    crafts_summary: BTreeMap<String, BTreeMap<String, (u32, Vec<Building>)>>,
) -> Result<()> {
    for (region, usages) in crafts_summary.iter() {
        let mut table = Table::new();
        table.load_preset(UTF8_COMFY_TABLE_STYLE);
        table.set_header(vec![
            Cell::new("Crafting Usage").add_attribute(Attribute::Dim),
            Cell::new("Key").add_attribute(Attribute::Dim),
            Cell::new("Building").add_attribute(Attribute::Dim),
            Cell::new("Cost").add_attribute(Attribute::Dim),
        ]);
        for (usage, buildings) in usages.iter() {
            for building in buildings.1.iter() {
                table.add_row(vec![
                    &format!("{usage}"),
                    &format!("{}", building.key),
                    &format!("{}", building.building_name),
                    &format!("{}", building.cost),
                ]);
            }
        }
        let cost_column = table.column_mut(3).unwrap();
        cost_column.set_cell_alignment(comfy_table::CellAlignment::Left);
        println!("\n{}", style(region).bold());
        println!("{table}");
    }
    Ok(())
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
