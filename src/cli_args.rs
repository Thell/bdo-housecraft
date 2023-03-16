use clap::{ArgGroup, Parser};

#[derive(clap::ValueEnum, Clone, Debug)]
pub(crate) enum ContextType {
    S,
    Storage,
    L,
    Lodging,
}

/// A BDO buildings chain tool.
/// (Calpheon City and Valencia City are not available for exact score listings.)
#[derive(Clone, Debug, Parser)]
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
    pub(crate) list_regions: bool,

    /// list craft types
    #[arg(short = 'c', long)]
    pub(crate) list_crafts: bool,

    /// find buildings for craft
    #[arg(short = 'f', long)]
    pub(crate) find_craft: Option<String>,

    /// warehouse region
    #[arg(short = 'R', long, help_heading = Some("Listing"))]
    pub(crate) region: Option<String>,

    /// warehouse storage slots count minimum
    #[arg(short = 'S', long, group = "listing", help_heading = Some("Listing"))]
    pub(crate) storage_count: Option<u8>,

    /// worker lodging slots count minimum
    #[arg(short = 'L', long, group = "listing", help_heading = Some("Listing"))]
    pub(crate) lodging_count: Option<u8>,

    /// short listing
    #[arg(short = 's', long, requires = "listing", group = "output", help_heading = Some("Output control"))]
    pub(crate) short: bool,

    /// detailed listing
    #[arg(short = 'd', long, requires = "listing", group = "output", help_heading = Some("Output control"))]
    pub(crate) detailed: bool,

    /// format as json
    #[arg(long, requires = "listing", group = "output", help_heading = Some("Output control"))]
    pub(crate) json: bool,

    /// print NUM entries of leading context
    #[arg(short = 'B', long, requires = "listing", group = "contexts", help_heading = Some("Context control"))]
    pub(crate) before_context: Option<Option<u8>>,

    /// print NUM entries of trailing context
    #[arg(short = 'A', long, requires = "listing", group = "contexts", help_heading = Some("Context control"))]
    pub(crate) after_context: Option<Option<u8>>,

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
    pub(crate) context: Option<Option<u8>>,

    /// lock context
    #[arg(short = 'T', long, requires = "listing", requires = "contexts", help_heading = Some("Context control"))]
    pub(crate) lock_context: Option<ContextType>,

    /// generate exact scored houseinfo building chains
    #[arg(long, group = "generation", requires = "region", conflicts_with = "listing", help_heading = Some("Generation"))]
    pub(crate) generate: bool,

    /// find optimal houseinfo building chain for region, storage and lodging
    #[arg(long, group = "generation", requires = "listing", help_heading = Some("Generation"))]
    pub(crate) optimize: bool,

    /// optimizer will only consider chains containing buildings (excluded storage/lodging counts)
    #[arg(long, requires = "optimize", group = "listing", help_heading = Some("Listing"))]
    pub(crate) require_buildings: Option<Vec<String>>,

    /// print periodic generation progress report
    #[arg(long, requires = "generation", help_heading = Some("Generation"))]
    pub(crate) progress: bool,

    /// use NUM parallel jobs
    #[arg(short, long, requires = "generation", help_heading = Some("Generation"))]
    pub(crate) jobs: Option<u8>,
}
