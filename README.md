# NavCat

<div align="center">
  <img src="navcat.png" width="300">
</div>

A command-line tool for filtering and highlighting Android logcat output, specifically designed for TomTom navigation-related logs.

## Features

- Filter logs by level (I, D, E, W, T)
- Filter by specific tags
- Highlight important messages
- Live mode (using adb) or file mode
- Customizable verbosity levels
- Focus on navigation, guidance, and routing messages

## Installation

```bash
cargo install --path .
```

## Usage

### Basic Usage

```bash
# Live mode (requires adb)
navcat

# Read from file
navcat -f logcat.txt
```

### Options

```bash
# Filter specific log levels
navcat -l I,D,E

# Show only guidance messages
navcat -g

# Show only routing messages
navcat -r

# Set verbosity level
navcat -v debug  # Options: none, error, info, debug

# Show help
navcat --help
```

### Default Tags

The tool is pre-configured to filter for tags:
- DefaultTomTomNavigation
- DistanceAlongRouteCalculator
- LocationContextProvidingStep
- NavigationHistoryStep
- RouteProjectionStep
- LocationMatchingStep
- ProgressCalculationStep
- RouteTrackingStateStep
- WaypointStatusCheckStep
- DestinationArrivalCheckStep
- GuidanceGenerationStep
- LaneGuidanceGenerationStep
- WarningGenerationStep
- RouteReplanningStep
- DefaultRouteTrackingEngine
- DefaultRouteProgressEngine

or to contain:
- Replan
- Planner
- Matcher

## Requirements

- Rust 1.70 or higher
- Android Debug Bridge (adb) for live mode
- Android device or emulator for live mode

## License

MIT
