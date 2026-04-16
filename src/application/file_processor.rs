use crate::domain::filter::LogFilter;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::Command;
use tempfile::NamedTempFile;

pub fn process_file(file_path: &str, filter: LogFilter) -> Result<(), Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    // Create a temporary file for the filtered output
    let mut temp_file = NamedTempFile::new()?;

    // Process and write each line to the temp file
    for line in reader.lines().map_while(Result::ok) {
        if let Some(colored_line) = filter.matches(&line) {
            writeln!(temp_file, "{}", colored_line)?;
        }
    }

    temp_file.flush()?;

    // Get the path of the temp file
    let temp_path = temp_file.into_temp_path();

    // Run less on the temp file
    let status = Command::new("less")
        .arg("-R") // Enable raw control characters
        .arg("--no-init") // Don't clear screen on exit
        .arg(temp_path.to_str().unwrap())
        .status()?;

    if !status.success() {
        eprintln!("less exited with status: {}", status);
    }

    Ok(())
}
