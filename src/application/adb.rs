use std::error::Error;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;

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

/// Spawns `adb logcat` and returns a channel receiver that emits raw log lines.
/// The caller is responsible for killing the returned `Child` on exit.
pub fn spawn_logcat() -> Result<(Child, Receiver<String>), Box<dyn Error>> {
    let mut child = Command::new("adb")
        .arg("logcat")
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().ok_or("Failed to capture adb stdout")?;
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            if sender.send(line).is_err() {
                break; // TUI dropped the receiver — exit thread
            }
        }
    });

    Ok((child, receiver))
}
