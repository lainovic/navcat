# navcat

<div align="center">
  <img src="navcat.png" width="300">
  <br><br>
</div>

A command-line tool for filtering and highlighting Android logcat output, specifically designed for TomTom navigation-related logs.

## Features

- Filter logs by level (I, D, E, W, T)
- Filter by specific tags
- Highlight important messages
- Live mode (using adb) or file mode
- Customizable verbosity levels
- Focus on navigation, map-matching, guidance, and routing messages

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

The tool is pre-configured to filter for navigation-related tags:
- DefaultTomTomNavigation
- DistanceAlongRouteCalculator
- ProgressCalculationStep
- RouteTrackingStateStep
- WaypointStatusCheckStep
- DestinationArrivalCheckStep
- DefaultRouteTrackingEngine
- DefaultRouteProgressEngine

or anything that contains:
- Replan
- Warning
- Guidance
- Planner
- Match
- Project

depending on the input options, e.g. if flags for including map-matching, guidance, or routing messages are supplied.

### Filtering Options

The tool provides several flags to control which messages are shown:

- `-g` or `--guidance`: Show guidance and warning messages
  - When disabled, filters out tags containing "Guidance" or "Warning"
  - Also blacklists messages containing "guidance", "instruction", or "warning"

- `-r` or `--routing`: Show routing messages
  - When disabled, filters out tags containing "Planner"

- `-m` or `--mapmatching`: Show map-matching messages
  - When disabled, filters out tags containing "Match" or "Project"

These flags can be combined to show only the messages you're interested in. For example:
```bash
# Show only guidance messages
navcat -g

# Show guidance and routing messages
navcat -gr

# Show guidance and routing and map-matching messages
navcat -grm
```

### Color Highlighting

The tool uses different colors to highlight various types of messages:

#### Log Levels
- ERROR (E): Red
- WARN (W): Yellow
- INFO (I): Green
- DEBUG (D): Cyan
- TRACE (T): Magenta

#### Tags
- Top-level classes: Blue
- Steps: Magenta
- Engines: Cyan
- Other tags: Bold Red

#### Message Content
- Red (Bold): Warnings, errors, and deviations
- Green (Bold): Success and positive messages
- Yellow: Navigation and map matching events
- Custom highlights: Yellow background (can be added via command line)

## Requirements

- Rust 1.70 or higher
- Android Debug Bridge (adb) for live mode
- Android device or emulator for live mode

## License

MIT
