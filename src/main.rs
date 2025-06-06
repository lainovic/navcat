use clap::Parser;
use std::error::Error;

mod application;
mod domain;
mod shared;

use application::adb::{check_adb_available, check_device_connected, start_logcat};
use application::cli::Args;
use application::file_processor::process_file;
use application::terminal::{TerminalControl, TerminalController};
use domain::filter::LogFilter;
use domain::filter_config::FilterConfig;
use shared::logger::{Logger, LogLevel};

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let config = FilterConfig::parse(&args);
    let filter = LogFilter::new(config);

    // Set log level
    let level = match args.log_level.to_lowercase().as_str() {
        "error" => LogLevel::Error,
        "info" => LogLevel::Info,
        "debug" => LogLevel::Debug,
        _ => LogLevel::None,
    };
    Logger::set_level(level);

    Logger::info("Starting with:");
    Logger::info_fmt("-> Levels: {:?}", &[&filter.levels]);
    Logger::info_fmt("-> Tags: {:?}", &[&filter.tags.all_tags]);
    Logger::info_fmt("-> Blacklisted items: {:?}", &[&filter.blacklisted_items]);


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
