use clap::Parser;
use std::error::Error;

mod application;
mod domain;
mod shared;

use application::adb::{check_adb_available, check_device_connected, spawn_logcat};
use application::cli::{Args, VerbosityLevel};
use application::file_processor::process_file;
use application::tui::run_tui;
use domain::filter::LogFilter;
use domain::filter_config::{FilterConfig, FilterState};
use shared::logger::Logger;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    Logger::set_level(match args.verbosity_level {
        VerbosityLevel::None => shared::logger::LogLevel::None,
        VerbosityLevel::Error => shared::logger::LogLevel::Error,
        VerbosityLevel::Info => shared::logger::LogLevel::Info,
        VerbosityLevel::Debug => shared::logger::LogLevel::Debug,
    });

    match &args.file {
        Some(file_path) => {
            let config = FilterConfig::parse(&args);
            Logger::info_fmt("Reading from file:", &[&file_path]);
            process_file(file_path, LogFilter::new(config))
        }
        None => {
            check_adb_available()?;
            check_device_connected()?;

            Logger::set_log_file("/tmp/navcat.log")
                .unwrap_or_else(|e| eprintln!("Warning: could not open log file: {}", e));

            let filter_state = FilterState::from_args(&args);
            let (child, receiver) = spawn_logcat()?;
            run_tui(child, receiver, filter_state)
        }
    }
}
