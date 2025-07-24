use clap::Parser;

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

  # Enable additional message types
  navcat --guidance --routing --mapmatching"#)]
pub struct Args {
    /// Path to logcat file (if not provided, runs in live mode)
    #[arg(short, long)]
    pub file: Option<String>,

    /// Log levels to show (comma-separated, e.g. "I,D,E")
    #[arg(short, long, default_value = "I,D,E")]
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

    /// Display guidance-related log messages as well
    #[arg(short, long)]
    pub guidance: bool,

    /// Display route planning and calculation messages as well
    #[arg(short, long)]
    pub routing: bool,

    /// Display map-matching and location projection messages as well
    #[arg(short, long)]
    pub mapmatching: bool,

    /// Set verbosity level (error/e, info/i, debug/d)
    #[arg(short = 'v', long, default_value = "none")]
    pub verbosity_level: String,

    /// Comma-separated list of items to highlight in the output
    #[arg(short = 'i', long, default_value = "", allow_hyphen_values = true)]
    pub highlighted_items: String,

    /// Comma-separated list of items to show in the output. Only entries containing these items will be displayed
    #[arg(short = 's', long, default_value = "", allow_hyphen_values = true)]
    pub show_items: String,
}
