use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to logcat file (if not provided, runs in live mode)
    #[arg(short, long)]
    pub file: Option<String>,

    /// Log levels to show (comma-separated, e.g. "I,D,E")
    #[arg(short, long, default_value = "I,D,E")]
    pub levels: String,

    /// Tags to show (comma-separated)
    #[arg(short, long, default_value = "\
        DefaultTomTomNavigation,\
        NavigationProcess,\
        LocationContextProvidingStep,\
        NavigationHistoryStep,\
        RouteProjectionStep,\
        LocationMatchingStep,\
        ProgressCalculationStep,\
        RouteTrackingStateStep,\
        WaypointStatusCheckStep,\
        DestinationArrivalCheckStep,\
        GuidanceGenerationStep,\
        LaneGuidanceGenerationStep,\
        WarningGenerationStep,\
        RouteReplanningStep,\
        DistanceAlongRouteCalculator,\
        DefaultRouteTrackingEngine,\
        DefaultRouteProgressEngine,\
        RoutePlanner\
    ")]
    pub tags: String,

    /// Show guidance messages
    #[arg(short, long)]
    pub guidance: bool,

    /// Show routing messages
    #[arg(short, long)]
    pub routing: bool,

    /// Enable debug/verbose logging
    #[arg(short = 'd', long)]
    pub debug: bool,
}

