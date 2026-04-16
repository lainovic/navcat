use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

const ZSH_COMPLETION: &str = r#"#compdef navcat

_navcat() {
  _arguments \
    '(-f --file)'{-f,--file}'[Load a logcat file into the TUI]:file:_files -g "*.txt(-.)"' \
    '(-l --logcat-levels)'{-l,--logcat-levels}'[Log levels to show, comma-separated (I/D/E/W/T)]:levels' \
    '(-t --tags)'{-t,--tags}'[Override the default tag filter list]:tags' \
    '(-a --add-tag)'{-a,--add-tag}'[Add tags on top of the default list]:tag' \
    '(-n --no-tag-filter)'{-n,--no-tag-filter}'[Disable tag filtering, show all tags]' \
    '(-v --verbosity-level)'{-v,--verbosity-level}'[Logging verbosity]:level:(none error info debug)' \
    '(-i --highlighted-items)'{-i,--highlighted-items}'[Terms to highlight, comma-separated]:items' \
    '(-s --show-items)'{-s,--show-items}'[Only show lines containing these terms, comma-separated]:items' \
    '--help[Show help]' \
    '--version[Show version]'
}

_navcat "$@"
"#;

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

    if let Some(shell) = &args.completions {
        match shell.as_str() {
            "zsh" => {
                print!("{}", ZSH_COMPLETION);
                return Ok(());
            }
            other => {
                eprintln!("unsupported shell: {}. Supported: zsh", other);
                std::process::exit(1);
            }
        }
    }

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
