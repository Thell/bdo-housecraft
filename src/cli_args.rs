use clap::{ArgGroup, Parser};

#[derive(clap::ValueEnum, Clone, Debug)]
enum ContextType {
    S,
    Storage,
    L,
    Lodging,
}

/// A BDO buildings chain tool.
/// (Calpheon City and Valencia City are not available for exact score listings.)
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[clap(group(
    ArgGroup::new("listing")
        .required(false)
        .multiple(true)
        .args(&["storage_count", "lodging_count"]),
))]
#[clap(group(
            ArgGroup::new("contexts")
        .required(false)
        .multiple(true)
        .args(&["before_context", "after_context", "context"]),
))]
pub(crate) struct Cli {
    /// list warehouse regions
    #[arg(short = 'l', long)]
    list_regions: bool,

    /// list craft types
    #[arg(short = 'c', long)]
    list_crafts: bool,

    /// find buildings for craft
    #[arg(short = 'f', long)]
    find_craft: bool,

    /// warehouse region
    #[arg(short = 'R', long, help_heading = Some("Listing"))]
    region: String,

    /// warehouse storage slots count minimum
    #[arg(short = 'S', long, group = "listing", help_heading = Some("Listing"))]
    storage_count: Option<u8>,

    /// worker lodging slots count minimum
    #[arg(short = 'L', long, group = "listing", help_heading = Some("Listing"))]
    lodging_count: Option<u8>,

    /// list only chains containing buildings (excludes buildings from storage and lodging count)
    #[arg(long, group = "listing", help_heading = Some("Listing"))]
    require_buildings: Option<Vec<String>>,

    /// short listing
    #[arg(short = 's', long, requires = "listing", group = "output", help_heading = Some("Output control"))]
    short: bool,

    /// detailed listing
    #[arg(short = 'd', long, requires = "listing", group = "output", help_heading = Some("Output control"))]
    detailed: bool,

    /// format as json
    #[arg(short = 'j', long, requires = "listing", group = "output", help_heading = Some("Output control"))]
    json: bool,

    /// print NUM entries of leading context
    #[arg(short = 'B', long, requires = "listing", group = "contexts", help_heading = Some("Context control"))]
    before_context: Option<Option<u8>>,

    /// print NUM entries of trailing context
    #[arg(short = 'A', long, requires = "listing", group = "contexts", help_heading = Some("Context control"))]
    after_context: Option<Option<u8>>,

    /// print NUM entries of context
    #[arg(
        short = 'C',
        long,
        requires = "listing",
        group = "contexts",
        conflicts_with = "before_context",
        conflicts_with = "after_context",
        help_heading = Some("Context control")
    )]
    context: Option<Option<u8>>,

    /// lock context
    #[arg(short = 'T', long, requires = "listing", requires = "contexts", help_heading = Some("Context control"))]
    lock_context: Option<ContextType>,

    /// generate exact scored houseinfo building chains
    #[arg(long, group = "generation", conflicts_with = "listing", help_heading = Some("Generation"))]
    generate: bool,

    /// find optimal houseinfo building chain for region, storage and lodging
    #[arg(long, group = "generation", requires = "listing", help_heading = Some("Generation"))]
    optimize: bool,

    /// print periodic generation progress report
    #[arg(long, requires = "generation", help_heading = Some("Generation"))]
    progress: bool,

    /// use NUM parallel jobs
    #[arg(long, requires = "generation", help_heading = Some("Generation"))]
    workers: Option<u8>,
}
