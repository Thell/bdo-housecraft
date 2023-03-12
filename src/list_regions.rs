use super::houseinfo::*;
use anyhow::{Ok, Result};
use comfy_table::{Attribute, Cell, Table};
use std::collections::BTreeMap;

fn summarize_regions(
    regions_buildings: BTreeMap<String, BTreeMap<u32, Building>>,
) -> Result<BTreeMap<String, (u32, u32, u32)>> {
    let mut regions_summary = BTreeMap::<String, (u32, u32, u32)>::new();
    for (region, buildings) in regions_buildings.iter() {
        regions_summary.insert(
            region.to_string(),
            buildings.iter().fold((0, 0, 0), |acc, (_, building)| {
                (
                    acc.0 + building.cost,
                    acc.1 + building.warehouse_count,
                    acc.2 + building.worker_count,
                )
            }),
        );
    }
    Ok(regions_summary)
}

// pub const UTF8_COMFY_TABLE_STYLE: &str = "0123456789abcdefghi";
pub const UTF8_COMFY_TABLE_STYLE: &str = "   ═────      ═  ══";

fn print_listing(
    regions_summary: BTreeMap<String, (u32, u32, u32)>,
    totals: (u32, u32, u32),
) -> Result<()> {
    let mut table = Table::new();
    table.load_preset(UTF8_COMFY_TABLE_STYLE);
    table.set_header(vec![
        Cell::new("Region").add_attribute(Attribute::Dim),
        Cell::new("CP").add_attribute(Attribute::Dim),
        Cell::new("Storage").add_attribute(Attribute::Dim),
        Cell::new("Lodging").add_attribute(Attribute::Dim),
    ]);
    for (region, (cost, storage, lodging)) in regions_summary.iter() {
        table.add_row(vec![
            region,
            &format!("{cost}"),
            &format!("{storage}"),
            &format!("{lodging}"),
        ]);
    }
    table.add_row(vec![
        Cell::new("Totals").add_attribute(Attribute::Bold),
        Cell::new(totals.0).add_attribute(Attribute::Bold),
        Cell::new(totals.1).add_attribute(Attribute::Bold),
        Cell::new(totals.2).add_attribute(Attribute::Bold),
    ]);
    println!("\n{table}");
    Ok(())
}

pub(crate) fn list_regions() -> Result<()> {
    let regions_buildings = merge_houseinfo_data()?;
    let regions_summary = summarize_regions(regions_buildings)?;
    let totals = regions_summary.iter().fold(
        (0, 0, 0),
        |acc, (_, (cost, warehouse_count, worker_count))| {
            (acc.0 + cost, acc.1 + warehouse_count, acc.2 + worker_count)
        },
    );
    print_listing(regions_summary, totals)
}
