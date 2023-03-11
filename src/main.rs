#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use clap::Parser;

pub mod cli_args;
use cli_args::Cli;

fn main() {
    let cli = Cli::parse();
    println!("{cli:#?}");
}
