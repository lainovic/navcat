use clap::Parser;

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum VerbosityLevel {
    #[value(alias = "n")]
    None,
    #[value(alias = "e")]
    Error,
    #[value(alias = "i")]
    Info,
    #[value(alias = "d")]
    Debug,
}

#[derive(Parser, Debug)]
#[command(author, version, about,
    long_about = r#"A tool for processing Android logcat output that highlights navigation-related log entries and provides filtering and highlighting capabilities.

EXAMPLES:
  # Live mode with default settings
  navcat

  # Process a log file
  navcat -f logcat.txt

  # Show only error and warning levels
  navcat -l "E,W"

  # Add custom tags
  navcat -a RouteDispatcher,LocationService

  # Show only entries containing 'error'
  navcat -s error

  # Highlight specific terms
  navcat -i "deviation,warning"

  # Disable tag filtering to see all tags
  navcat --no-tag-filter

  # In live mode, use g/r/m keys to toggle guidance/routing/mapmatching at runtime"#)]
pub struct Args {
    /// Path to logcat file (if not provided, runs in live mode)
    #[arg(short, long)]
    pub file: Option<String>,

    /// Log levels to show, comma-separated (I/INFO, D/DEBUG, E/ERROR, W/WARN, T/TRACE)
    #[arg(short, long, default_value = "I,D,E,W")]
    pub logcat_levels: String,

    /// Tags to show (comma-separated)
    #[arg(
        short,
        long,
        default_value = "\
        DefaultTomTomNavigation,\
        DistanceAlongRouteCalculator,\
        ProgressCalculationStep,\
        RouteTrackingStateStep,\
        WaypointStatusCheckStep,\
        DestinationArrivalCheckStep,\
        DefaultRouteTrackingEngine,\
        DefaultRouteProgressEngine,\
        Replan,\
        Warning,\
        Guidance,\
        Planner,\
        Match,\
        Project\
    "
    )]
    pub tags: String,

    /// Additional tags to include beyond the default tag list
    #[arg(short = 'a', long, value_delimiter = ',')]
    pub add_tag: Vec<String>,

    /// Disable tag filtering to show all tags
    #[arg(short, long)]
    pub no_tag_filter: bool,

    /// Set verbosity level
    #[arg(short = 'v', long, default_value = "none")]
    pub verbosity_level: VerbosityLevel,

    /// Items to highlight in the output (comma-separated)
    #[arg(short = 'i', long, value_delimiter = ',', allow_hyphen_values = true)]
    pub highlighted_items: Vec<String>,

    /// Items to show in the output (comma-separated); only entries containing these items will be displayed
    #[arg(short = 's', long, value_delimiter = ',', allow_hyphen_values = true)]
    pub show_items: Vec<String>,
}
