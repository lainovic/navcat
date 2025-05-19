use clap::Parser;
use std::error::Error;

mod application;
mod domain;

use application::adb::{check_adb_available, check_device_connected, start_logcat};
use application::cli::Args;
use application::file_processor::process_file;
use application::terminal::{TerminalControl, TerminalController};
use domain::filter::LogFilter;
use domain::filter_config::FilterConfig;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let config = FilterConfig::parse(&args.levels, &args.tags, args.guidance, args.routing);
    let filter = LogFilter::new(config.levels, config.tags, config.blacklisted_items);

    println!("Starting with filters:");
    println!("Levels: {:?}", filter.levels);
    println!("Tags: {:?}", filter.tags);

    match args.file {
        Some(file_path) => {
            println!("Reading from file: {}", file_path);
            process_file(&file_path, filter)
        }
        None => {
            println!("Running in live mode.");
            check_adb_available()?;
            check_device_connected()?;

            let terminal = TerminalController::new();
            terminal.start();
            start_logcat(filter, &terminal)
        }
    }
}
