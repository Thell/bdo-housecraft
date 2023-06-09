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
mod optimize;
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
use optimize::optimize;

fn main() -> Result<()> {
    let mut cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .format_timestamp_nanos()
        .init();

    info!("Start up");
    if log_enabled!(Debug) {
        debug!("Parsed cli arguments...");
        debug!("{:#?}", cli);
    }

    if cli.list_regions {
        list_regions()?
    } else if cli.list_crafts {
        list_crafts(cli.region)?
    } else if let Some(craft) = cli.find_craft {
        find_craft_buildings(cli.region, craft)?
    } else if cli.generate {
        generate(&mut cli)?
    } else if cli.optimize {
        optimize(&mut cli)?
    } else if cli.region.is_some() {
        list_buildings(cli)?
    } else {
        let mut cmd = cli_args::Cli::command();
        let _ = cmd.print_help();
    }

    info!("Complete");
    Ok(())
}
