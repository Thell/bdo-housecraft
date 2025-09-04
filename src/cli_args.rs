use clap::{ArgGroup, Parser};

#[derive(clap::ValueEnum, Clone, Debug)]
pub(crate) enum ContextType {
    S,
    Storage,
    L,
    Lodging,
}

/// A BDO buildings chain tool.
/// (Calpheon City, Valencia City and Heidel are not available for exact score listings.)
#[derive(Clone, Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[clap(group(
    ArgGroup::new("listing")
        .required(false)
        .multiple(true)
        .args(&["storage", "lodging"]),
))]

pub(crate) struct Cli {
    /// list warehouse regions
    #[arg(short = 'l', long)]
    pub(crate) list_regions: bool,

    /// list craft types
    #[arg(short = 'c', long)]
    pub(crate) list_crafts: bool,

    /// list all regions storage sorted by global efficiency
    #[arg(short = 's', long)]
    pub(crate) list_storage: bool,

    /// find buildings for craft
    #[arg(short = 'f', long)]
    pub(crate) find_craft: Option<String>,

    #[clap(flatten)]
    pub(crate) verbose: clap_verbosity_flag::Verbosity,

    /// specific region or ALL
    #[arg(short = 'R', long, help_heading = Some("Listing"))]
    pub(crate) region: Option<String>,

    /// warehouse storage slots count minimum
    #[arg(short = 'S', long, group = "listing", help_heading = Some("Listing"))]
    pub(crate) storage: Option<u16>,

    /// worker lodging slots count minimum
    #[arg(short = 'L', long, group = "listing", help_heading = Some("Listing"))]
    pub(crate) lodging: Option<u16>,

    /// generate exact scored houseinfo building chains
    #[arg(long, group = "generation", requires = "region", conflicts_with = "listing", help_heading = Some("Generation"))]
    pub(crate) generate: bool,

    /// find optimal houseinfo building chain for region, storage and lodging
    /// (Always parallel; uses all processors as needed.)
    #[arg(long, group = "generation", requires = "region", conflicts_with = "listing", help_heading = Some("Generation"))]
    pub(crate) optimize: bool,

    /// limit warehouse slots during optimize (192 maximum - 8 minimum given for free)
    #[arg(long, requires = "generation", conflicts_with = "generate", help_heading = Some("Generation"))]
    pub(crate) limit_warehouse: Option<Option<usize>>,

    /// output only the lodging, storage and cost to /data/housecraft/validation
    #[arg(long, requires = "generation", help_heading = Some("Generation"))]
    pub(crate) for_validation: bool,

    /// use NUM parallel jobs
    #[arg(short, long, requires = "generation", conflicts_with = "optimize", help_heading = Some("Generation"))]
    pub(crate) jobs: Option<u8>,
}
