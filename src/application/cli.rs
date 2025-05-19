use clap::Parser;
use std::fmt;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to logcat file (if not provided, runs in live mode)
    #[arg(short, long)]
    pub file: Option<String>,

    /// Log levels to filter (e.g., "I,D,E")
    #[arg(short, long, default_value = "I,D,E")]
    pub levels: String,

    
    /// Parse guidance-related information.
    #[arg(short, long, default_value_t = false)]
    pub guidance: bool,

    /// Parse routing-related information.
    #[arg(short, long, default_value_t = false)]
    pub routing: bool,

    /// Tags to filter (comma-separated)
    #[arg(short, long, default_value =
        "\
        DefaultTomTomNavigation,\
        NavigationProcess,\
        LocationContextProvidingStep,\
        NavigationHistoryStep,\
        RouteProjectionStep,\
        LocationMatchingStep,\
        SoftDrAttemptStep,\
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
        "
    )]
    pub tags: String,
}

impl fmt::Debug for Args {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Args")
            .field("file", &self.file)
            .field("levels", &self.levels)
            .field("guidance", &self.guidance)
            .field("tags", &self.tags)
            .finish()
    }
} 