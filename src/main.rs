#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[macro_use]
extern crate lazy_static;

extern crate num_cpus;

mod cli_args;
mod find_crafts;
mod generate;
mod generate_common;
mod generate_par;
mod houseinfo;
mod list_crafts;
mod list_regions;
mod node_manipulation;
mod region_nodes;
use anyhow::{Ok, Result};
use clap::Parser;
use cli_args::Cli;
use find_crafts::find_craft_buildings;
use generate_common::generate_main;
use list_crafts::list_crafts;
use list_regions::list_regions;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Options dispatch
    if cli.list_regions {
        list_regions()?
    } else if cli.list_crafts {
        list_crafts(cli.region)?
    } else if let Some(craft) = cli.find_craft {
        find_craft_buildings(cli.region, craft)?
    // Generation dispatch
    } else if cli.generate {
        // println!("{cli:#?}");
        println!("WIP... implementing");
        generate_main(cli)?
    // Optimizer dispatch
    } else if cli.optimize {
        // This is going to be the last section implemented.
        let region = cli.region.unwrap();
        let desired_warehouse_count = cli.storage_count.unwrap();
        let desired_worker_count = cli.lodging_count.unwrap();
        let progress = cli.progress;
        let workers = cli.jobs.unwrap_or(1);
        println!("running an optimizer for {region} with {desired_warehouse_count} storage and {desired_worker_count} lodging");
        println!("using [progress: {progress}, workers: {workers}]");
        println!("is not implemented yet.")
    // Main listing dispatch
    } else if let Some(region) = cli.region {
        let mut desired_warehouse_count = 0;
        let mut desired_worker_count = 0;
        if cli.storage_count.is_none() && cli.lodging_count.is_none() {
            println!("listings for {region} without storage or lodging is not implemented yet.");
        } else if cli.storage_count.is_none() {
            desired_worker_count = cli.lodging_count.unwrap();
            println!("listings for {region} without storage but with {desired_worker_count} lodging is not implemented yet.");
        } else if cli.lodging_count.is_none() {
            desired_warehouse_count = cli.storage_count.unwrap();
            println!("listings for {region} with {desired_warehouse_count} storage but without lodging is not implemented yet.");
        } else {
            desired_warehouse_count = cli.storage_count.unwrap();
            desired_worker_count = cli.lodging_count.unwrap();
        }
        println!("if this was implemented...");
        println!("{region} would be listed showing results with {desired_warehouse_count} storage and {desired_worker_count} lodging.");
        println!("using the given output and context flags which haven't been implmented yet.");
    }

    Ok(())
}
