use anyhow::{Ok, Result};

use crate::cli_args::Cli;

pub(crate) fn list_buildings(cli: Cli) -> Result<()> {
    let region = cli.region.unwrap();
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
    println!("using the given output and context flags which haven't been implemented yet.");
    Ok(())
}
