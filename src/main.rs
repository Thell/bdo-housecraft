#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

extern crate num_cpus;

mod cli_args;
mod find_crafts;
mod generate;
mod houseinfo;
mod list_buildings;
mod list_crafts;
mod list_regions;
mod node_manipulation;
mod region_nodes;

use anyhow::{Ok, Result};
use clap::{CommandFactory, Parser};
use log::Level::Debug;

use cli_args::Cli;
use find_crafts::find_craft_buildings;
use generate::generate;
use list_buildings::list_buildings;
use list_crafts::list_crafts;
use list_regions::list_regions;

fn main() -> Result<()> {
    let cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .format_timestamp_nanos()
        .init();

    info!("Start up");
    if log_enabled!(Debug) {
        debug!("Parsed cli arguments...");
        debug!("{:#?}", cli);
    }

    // Options dispatch
    if cli.list_regions {
        list_regions()?
    } else if cli.list_crafts {
        list_crafts(cli.region)?
    } else if let Some(craft) = cli.find_craft {
        find_craft_buildings(cli.region, craft)?
    // Generation dispatch
    } else if cli.generate {
        generate(cli)?
    // Optimizer dispatch
    } else if cli.optimize {
        // This is going to be the last section implemented.
        let region = cli.region.unwrap();
        let desired_warehouse_count = cli.storage.unwrap();
        let desired_worker_count = cli.lodging.unwrap();
        let workers = cli.jobs.unwrap_or(1);
        println!("running an optimizer for {region} with {desired_warehouse_count} storage and {desired_worker_count} lodging");
        println!("using [workers: {workers}]");
        println!("is not implemented yet.")
    // Main listing dispatch
    } else if cli.region.is_some() {
        list_buildings(cli)?
    } else {
        let mut cmd = cli_args::Cli::command();
        let _ = cmd.print_help();
    }

    info!("Complete");
    Ok(())
}
