use std::error::Error;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use crate::shared::logger::Logger;

pub fn check_adb_available() -> Result<(), Box<dyn Error>> {
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
    let output = Command::new("adb").arg("devices").output()?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    if output_str.lines().count() <= 1 {
        Err("No Android devices found. Please connect a device or start an emulator.".into())
    } else {
        Ok(())
    }
}

fn logcat_args() -> Vec<&'static str> {
    vec!["logcat", "-v", "threadtime", "-T", "0"]
}

/// Spawns `adb logcat -T 0` and returns a channel receiver that emits raw log lines.
/// `-T 0` skips the historical ring-buffer and streams only live events.
/// The reading thread automatically restarts on exit so live logs keep flowing.
pub fn spawn_logcat() -> Result<(Child, Receiver<String>), Box<dyn Error>> {
    let mut child = Command::new("adb")
        .args(logcat_args())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().ok_or("Failed to capture adb stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture adb stderr")?;
    let (sender, receiver) = mpsc::channel();

    // Reading thread: streams the initial spawn, then restarts live-only on exit.
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            if sender.send(line).is_err() {
                return;
            }
        }
        Logger::info_fmt("adb logcat exited after initial buffer; restarting live-only", &[]);

        loop {
            thread::sleep(Duration::from_secs(1));

            let mut c = match Command::new("adb")
                .args(logcat_args())
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    Logger::info_fmt("adb restart failed:", &[&e.to_string()]);
                    return;
                }
            };

            let stdout = match c.stdout.take() {
                Some(s) => s,
                None => return,
            };

            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                if sender.send(line).is_err() {
                    return;
                }
            }
            Logger::info_fmt("adb logcat live instance exited, retrying", &[]);
        }
    });

    // Stderr logging thread.
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            Logger::info_fmt("adb stderr:", &[&line]);
        }
    });

    Ok((child, receiver))
}
