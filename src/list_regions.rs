use super::houseinfo::*;
use anyhow::{Ok, Result};
use comfy_table::{Attribute, Cell, Table};
use std::collections::BTreeMap;

type RegionStatsMap = BTreeMap<String, Stats>;

struct Stats {
    cost: u32,
    warehouse_count: u32,
    worker_count: u32,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            cost: 0,
            warehouse_count: 0,
            worker_count: 0,
        }
    }
}

fn print_listing(regions_summary: RegionStatsMap, totals: Stats) -> Result<()> {
    let mut table = Table::new();
    table.load_preset(HOUSECRAFT_TABLE_STYLE);
    table.set_header(vec![
        Cell::new("Region").add_attribute(Attribute::Dim),
        Cell::new("CP").add_attribute(Attribute::Dim),
        Cell::new("Storage").add_attribute(Attribute::Dim),
        Cell::new("Lodging").add_attribute(Attribute::Dim),
    ]);

    for (region, stats) in regions_summary.iter() {
        table.add_row(vec![
            region,
            &stats.cost.to_string(),
            &stats.warehouse_count.to_string(),
            &stats.worker_count.to_string(),
        ]);
    }
    table.add_row(vec![
        Cell::new("Totals").add_attribute(Attribute::Bold),
        Cell::new(totals.cost).add_attribute(Attribute::Bold),
        Cell::new(totals.warehouse_count).add_attribute(Attribute::Bold),
        Cell::new(totals.worker_count).add_attribute(Attribute::Bold),
    ]);
    println!("\n{table}");

    Ok(())
}

fn summarize_regions(regions_buildings: RegionBuildingMap) -> Result<RegionStatsMap> {
    let mut regions_summary = RegionStatsMap::new();

    for (region, buildings) in regions_buildings.iter() {
        regions_summary.insert(
            region.to_string(),
            buildings
                .iter()
                .fold(Stats::new(), |mut stats, (_, building)| {
                    stats.cost += building.cost;
                    stats.warehouse_count += building.warehouse_count;
                    stats.worker_count += building.worker_count;
                    stats
                }),
        );
    }

    Ok(regions_summary)
}

pub(crate) fn list_regions() -> Result<()> {
    let regions_buildings = parse_houseinfo_data()?;
    let regions_summary = summarize_regions(regions_buildings)?;
    let totals = regions_summary
        .iter()
        .fold(Stats::new(), |mut stats, (_, region_stats)| {
            stats.cost += region_stats.cost;
            stats.warehouse_count += region_stats.warehouse_count;
            stats.worker_count += region_stats.worker_count;
            stats
        });
    print_listing(regions_summary, totals)
}
