use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
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

    /// Show guidance messages
    #[arg(short, long)]
    pub guidance: bool,

    /// Show routing messages
    #[arg(short, long)]
    pub routing: bool,

    /// Show map-matching messages
    #[arg(short, long)]
    pub mapmatching: bool,

    /// Set verbosity level (error, info, debug)
    #[arg(short = 'v', long, default_value = "none")]
    pub verbosity_level: String,

    /// Comma-separated list of items to highlight in the output
    #[arg(short = 'i', long, default_value = "", allow_hyphen_values = true)]
    pub highlighted_items: String,

    /// Comma-separated list of items to show in the output. Only entries containing these items will be displayed
    #[arg(short = 's', long, default_value = "", allow_hyphen_values = true)]
    pub show_items: String,
}
