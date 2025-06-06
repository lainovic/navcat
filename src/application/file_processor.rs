use crate::domain::filter::LogFilter;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::Command;
use tempfile::NamedTempFile;

pub fn process_file(file_path: &str, filter: LogFilter) -> Result<(), Box<dyn Error>> {
    println!("Debug - Filter levels: {:?}", filter.levels);
    println!("Debug - Filter tags: {:?}", filter.tags);
    println!("Debug - Processing file: {}", file_path);

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    // Create a temporary file for the filtered output
    let mut temp_file = NamedTempFile::new()?;

    // Process and write each line to the temp file
    for line in reader.lines() {
        if let Ok(line) = line {
            if let Some(colored_line) = filter.matches(&line) {
                writeln!(temp_file, "{}", colored_line)?;
            }
        }
    }

    // Flush the temp file to ensure all data is written
    temp_file.flush()?;

    let mut temp_file_for_read = File::open(temp_file.path())?;
    let mut contents = String::new();
    temp_file_for_read.read_to_string(&mut contents)?;
    println!("Temp file contents:\n{}", contents);

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
