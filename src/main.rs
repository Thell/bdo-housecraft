#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod houseinfo;
pub mod list_crafts;
pub mod list_regions;
use anyhow::{Ok, Result};
use clap::Parser;

pub mod cli_args;
use cli_args::Cli;
use list_crafts::list_crafts;
use list_regions::list_regions;

fn main() -> Result<()> {
    let cli = Cli::parse();
    println!("{cli:#?}");

    // Options dispatch
    if cli.list_regions {
        list_regions()?
    } else if cli.list_crafts {
        list_crafts(cli.region)?
    }

    Ok(())
}
