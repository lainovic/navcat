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

# Hide guidance messages
navcat --no-guidance

# Hide routing messages
navcat --no-routing

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

All categories are shown by default. Use opt-out flags to reduce noise.

### Filtering Options

The tool provides opt-out flags to hide specific message categories:

- `--no-guidance`: Hide guidance and warning messages
  - Filters out tags containing "Guidance" or "Warning"
  - Also blacklists messages containing "guidance", "instruction", or "warning"

- `--no-routing`: Hide route planning and calculation messages
  - Filters out tags containing "Planner"

- `--no-mapmatching`: Hide map-matching and location projection messages
  - Filters out tags containing "Match" or "Project"

These flags can be combined to focus on specific areas:
```bash
# Show only core navigation (hide everything else)
navcat --no-guidance --no-routing --no-mapmatching

# Show everything except guidance noise
navcat --no-guidance
```

### Color Highlighting

The tool uses different colors to highlight various types of messages. When multiple highlights could apply to the same text, the following priority order is used:

> **Note on Highlight Priority**
> 
> 1. Red (Bold) - Highest priority
> 2. Yellow - Second priority
> 3. Green (Bold) - Third priority
> 4. Custom highlights - Lowest priority
> 
> When multiple matches have the same priority, the last match in the sequence is used. This means that the order in which words are added to highlight rules can affect which highlight is applied when multiple words could match the same text.

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
