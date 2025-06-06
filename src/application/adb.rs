use crate::domain::filter::LogFilter;
use std::error::Error;
use std::io::{BufRead, BufReader};
use std::process::Command;

use super::terminal::TerminalControl;

pub fn check_adb_available() -> Result<(), Box<dyn Error>> {
    // Try to run 'adb version' to check if adb is available
    match Command::new("adb").arg("version").output() {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                Err("ADB not found. Please make sure Android Debug Bridge (adb) is installed and in your PATH.".into())
            } else {
                Err(format!("Error checking ADB: {}", e).into())
            }
        }
    }
}

pub fn check_device_connected() -> Result<(), Box<dyn Error>> {
    // Check if any device is connected
    let output = Command::new("adb").arg("devices").output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);

    // Check if there's at least one device listed (excluding the header line)
    if output_str.lines().count() <= 1 {
        Err("No Android devices found. Please connect a device or start an emulator.".into())
    } else {
        Ok(())
    }
}

pub fn start_logcat(
    filter: LogFilter,
    terminal: &dyn TerminalControl,
) -> Result<(), Box<dyn Error>> {
    // Start adb logcat process
    let mut child = Command::new("adb")
        .arg("logcat")
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    // Get the stdout handle
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let reader = BufReader::new(stdout);

    // Read and print each line while running
    let mut lines = reader.lines();
    while terminal.is_running() {
        match lines.next() {
            Some(Ok(line)) => {
                if let Some(colored_line) = filter.matches(&line) {
                    println!("{}", colored_line);
                }
            }
            Some(Err(e)) => eprintln!("Error reading line: {}", e),
            None => break, // End of stream
        }
    }

    // Clean up
    child.kill()?;
    Ok(())
}
