use super::houseinfo::*;
use anyhow::{Ok, Result};
use comfy_table::{Attribute, Cell, Table};
use console::style;
use std::collections::BTreeMap;

type CraftCounts = BTreeMap<String, u32>;
type RegionCraftingCounts = BTreeMap<String, CraftCounts>;

lazy_static! {
    static ref CRAFT_USAGE: IndexedStringMap =
        read_csv_data("HouseInfoReceipe.csv").expect("Error reading HouseInfoReceipe.csv");
}

fn print_listing(crafts_summary: RegionCraftingCounts) -> Result<()> {
    for (region, usages) in crafts_summary.iter() {
        let mut table = Table::new();
        table.load_preset(HOUSECRAFT_TABLE_STYLE);
        table.set_header(vec![
            Cell::new("Crafting Usage").add_attribute(Attribute::Dim),
            Cell::new("Count").add_attribute(Attribute::Dim),
        ]);

        for (usage, count) in usages.iter() {
            table.add_row(vec![usage, &count.to_string()]);
        }
        println!("\n{}", style(region).bold());
        println!("{table}");
    }

    Ok(())
}

fn increment_region_craft(region_crafts: &mut CraftCounts, craft: &CraftList) {
    if let Some(usage) = CRAFT_USAGE.get(&craft.item_craft_index) {
        let craft_name = format!("{} {}", usage, craft.house_level);
        if let Some(craft) = region_crafts.get_mut(&craft_name) {
            *craft += 1;
        } else {
            region_crafts.insert(craft_name, 1);
        }
    }
}

fn summarize_crafts(regions_buildings: RegionBuildingMap) -> Result<RegionCraftingCounts> {
    let mut crafts_summary = RegionCraftingCounts::new();
    for (region, buildings) in regions_buildings.iter() {
        let mut region_crafts = CraftCounts::new();
        for (_key, building) in buildings.iter() {
            for craft in building.craft_list.iter() {
                increment_region_craft(&mut region_crafts, craft);
            }
        }
        crafts_summary.insert(region.to_string(), region_crafts);
    }
    Ok(crafts_summary)
}

pub(crate) fn list_crafts(town_name: Option<String>) -> Result<()> {
    let mut regions_buildings = parse_houseinfo_data()?;
    if let Some(town_name) = town_name {
        regions_buildings.retain(|k, _| *k == town_name);
    }
    let crafts_summary = summarize_crafts(regions_buildings)?;
    print_listing(crafts_summary)
}
