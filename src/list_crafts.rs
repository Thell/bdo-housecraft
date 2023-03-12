use super::houseinfo::*;
use anyhow::{Ok, Result};
use comfy_table::{Attribute, Cell, Table};
use console::style;
use std::collections::BTreeMap;

fn summarize_crafts(
    regions_buildings: BTreeMap<String, BTreeMap<u32, Building>>,
) -> Result<BTreeMap<String, BTreeMap<String, u32>>> {
    let mut crafts_summary = BTreeMap::<String, BTreeMap<String, u32>>::new();
    let craft_usage = read_csv_data("HouseInfoReceipe.csv")?;
    for (region, buildings) in regions_buildings.iter() {
        let mut region_crafts = BTreeMap::<String, u32>::new();
        for (_, building) in buildings.iter() {
            for craft in building.craft_list.iter() {
                if let Some(usage) = craft_usage.get(&craft.item_craft_index) {
                    let craft_name = format!("{} {}", usage, craft.house_level);
                    if let Some(craft) = region_crafts.get_mut(&craft_name) {
                        *craft += 1;
                    } else {
                        region_crafts.insert(craft_name, 1);
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

fn print_listing(crafts_summary: BTreeMap<String, BTreeMap<String, u32>>) -> Result<()> {
    for (region, usages) in crafts_summary.iter() {
        let mut table = Table::new();
        table.load_preset(UTF8_COMFY_TABLE_STYLE);
        table.set_header(vec![
            Cell::new("Crafting Usage").add_attribute(Attribute::Dim),
            Cell::new("Count").add_attribute(Attribute::Dim),
        ]);
        for (usage, count) in usages.iter() {
            table.add_row(vec![&format!("{usage}"), &format!("{count}")]);
        }
        let count_column = table.column_mut(1).unwrap();
        count_column.set_cell_alignment(comfy_table::CellAlignment::Right);
        println!("\n{}", style(region).bold());
        println!("{table}");
    }
    Ok(())
}

pub(crate) fn list_crafts(town_name: Option<String>) -> Result<()> {
    let mut regions_buildings = merge_houseinfo_data()?;
    if let Some(town_name) = town_name {
        regions_buildings.retain(|k, _| *k == town_name);
    }
    let crafts_summary = summarize_crafts(regions_buildings)?;
    print_listing(crafts_summary)
}
