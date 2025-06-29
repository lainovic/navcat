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
use shared::logger::Logger;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    Logger::set_log_level(args.verbosity_level.clone());

    let config = FilterConfig::parse(&args);

    Logger::info("Starting with:");
    Logger::info_fmt("Levels:", &[&config.levels]);
    Logger::info_fmt("Tags:", &[&config.tags.all_tags]);
    Logger::info_fmt("Blacklisted items:", &[&config.blacklisted_items]);
    Logger::info_fmt("Highlighted items:", &[&config.highlighted_items]);
    Logger::info_fmt("Show items:", &[&config.show_items]);

    let filter = LogFilter::new(config);

    match args.file {
        Some(file_path) => {
            Logger::info_fmt("Reading from file:", &[&file_path]);
            process_file(&file_path, filter)
        }
        None => {
            Logger::info("Running in live mode.");
            check_adb_available()?;
            check_device_connected()?;

            let terminal = TerminalController::new();
            terminal.start();
            start_logcat(filter, &terminal)
        }
    }
}
