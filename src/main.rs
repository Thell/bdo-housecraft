#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod houseinfo;
use anyhow::{Ok, Result};
use clap::Parser;

pub mod cli_args;
use cli_args::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    println!("{cli:#?}");
    Ok(())
}
