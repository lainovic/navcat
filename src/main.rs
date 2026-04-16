use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

mod application;
mod domain;
mod shared;

use application::adb::{check_adb_available, check_device_connected, spawn_logcat};
use application::cli::{Args, VerbosityLevel};
use application::tui::run_tui;
use domain::filter_config::FilterState;
use shared::logger::Logger;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    Logger::set_level(match args.verbosity_level {
        VerbosityLevel::None => shared::logger::LogLevel::None,
        VerbosityLevel::Error => shared::logger::LogLevel::Error,
        VerbosityLevel::Info => shared::logger::LogLevel::Info,
        VerbosityLevel::Debug => shared::logger::LogLevel::Debug,
    });

    let filter_state = FilterState::from_args(&args);

    match &args.file {
        Some(file_path) => {
            Logger::info_fmt("Reading from file:", &[&file_path]);
            let lines: Vec<String> = BufReader::new(File::open(file_path)?)
                .lines()
                .map_while(Result::ok)
                .collect();
            run_tui(None, None, filter_state, lines)
        }
        None => {
            check_adb_available()?;
            check_device_connected()?;

            Logger::set_log_file("/tmp/navcat.log")
                .unwrap_or_else(|e| eprintln!("Warning: could not open log file: {}", e));

            let (child, receiver) = spawn_logcat()?;
            run_tui(Some(child), Some(receiver), filter_state, vec![])
        }
    }
}
