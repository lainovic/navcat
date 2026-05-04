use std::error::Error;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

use crate::shared::logger::Logger;

pub enum LogcatEvent {
    Line(String),
    Connected,
    Disconnected,
}

pub struct LogcatHandle {
    receiver: Receiver<LogcatEvent>,
    stop: Arc<AtomicBool>,
    current_child: Arc<Mutex<Option<Child>>>,
}

impl LogcatHandle {
    pub fn receiver(&self) -> &Receiver<LogcatEvent> {
        &self.receiver
    }

    pub fn shutdown(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        kill_current_child(&self.current_child);
        wait_current_child(&self.current_child);
    }
}

impl Drop for LogcatHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeviceCheck {
    Ready,
    Unready,
    Missing,
    MultipleReady,
    RequestedSerialMissing,
    RequestedSerialUnready,
}

pub fn check_adb_available() -> Result<(), Box<dyn Error>> {
    match Command::new(adb_program()).arg("version").output() {
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

pub fn check_device_connected(serial: Option<&str>) -> Result<(), Box<dyn Error>> {
    let output = Command::new(adb_program()).arg("devices").output()?;
    match parse_adb_devices_output(&String::from_utf8_lossy(&output.stdout), serial) {
        DeviceCheck::Ready => Ok(()),
        DeviceCheck::Unready => Err(
            "Android device detected, but it is not ready. Check adb authorization / device state."
                .into(),
        ),
        DeviceCheck::MultipleReady => {
            Err("Multiple adb devices are ready. Re-run with --serial <device-serial>.".into())
        }
        DeviceCheck::RequestedSerialMissing => Err(
            "Requested adb serial was not found. Re-run with --serial <device-serial> using an attached device."
                .into(),
        ),
        DeviceCheck::RequestedSerialUnready => Err(
            "Requested adb serial is attached, but it is not ready. Check adb authorization / device state."
                .into(),
        ),
        DeviceCheck::Missing => {
            Err("No Android devices found. Please connect a device or start an emulator.".into())
        }
    }
}

fn parse_adb_devices_output(output: &str, serial: Option<&str>) -> DeviceCheck {
    let mut saw_any_device = false;
    let mut ready_devices = Vec::new();
    let mut requested_serial_seen = false;

    for line in output.lines().skip(1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        saw_any_device = true;
        if let Some((device_serial, state)) = trimmed.split_once('\t') {
            if serial == Some(device_serial) {
                requested_serial_seen = true;
            }
            if state == "device" {
                ready_devices.push(device_serial);
            }
        }
    }

    if let Some(serial) = serial {
        if ready_devices.iter().any(|device| *device == serial) {
            return DeviceCheck::Ready;
        }
        return if requested_serial_seen {
            DeviceCheck::RequestedSerialUnready
        } else if saw_any_device {
            DeviceCheck::RequestedSerialMissing
        } else {
            DeviceCheck::Missing
        };
    }

    if ready_devices.len() == 1 {
        return DeviceCheck::Ready;
    }
    if ready_devices.len() > 1 {
        return DeviceCheck::MultipleReady;
    }

    if saw_any_device {
        DeviceCheck::Unready
    } else {
        DeviceCheck::Missing
    }
}

fn logcat_args(serial: Option<&str>) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(serial) = serial {
        args.push("-s".to_string());
        args.push(serial.to_string());
    }
    args.extend(
        ["logcat", "-v", "threadtime", "-T", "0"]
            .into_iter()
            .map(str::to_string),
    );
    args
}

fn adb_program() -> String {
    std::env::var("NAVCAT_ADB").unwrap_or_else(|_| "adb".to_string())
}

fn spawn_adb_logcat(serial: Option<&str>, stderr: Stdio) -> Result<Child, Box<dyn Error>> {
    Ok(Command::new(adb_program())
        .args(logcat_args(serial))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(stderr)
        .spawn()?)
}

fn spawn_stderr_logger(stderr: impl std::io::Read + Send + 'static) {
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            Logger::info_fmt("adb stderr:", &[&line]);
        }
    });
}

fn kill_current_child(current_child: &Arc<Mutex<Option<Child>>>) {
    if let Ok(mut guard) = current_child.lock() {
        if let Some(child) = guard.as_mut() {
            let _ = child.kill();
        }
    }
}

fn wait_current_child(current_child: &Arc<Mutex<Option<Child>>>) {
    if let Ok(mut guard) = current_child.lock() {
        if let Some(mut child) = guard.take() {
            let _ = child.wait();
        }
    }
}

fn stream_stdout(
    stdout: impl std::io::Read,
    sender: &mpsc::Sender<LogcatEvent>,
    stop: &AtomicBool,
) -> bool {
    let reader = BufReader::new(stdout);
    for line in reader.lines().map_while(Result::ok) {
        if stop.load(Ordering::Relaxed) {
            return false;
        }
        if sender.send(LogcatEvent::Line(line)).is_err() {
            return false;
        }
    }
    true
}

/// Spawns `adb logcat -T 0` and returns a channel receiver that emits raw log lines.
/// `-T 0` skips the historical ring-buffer and streams only live events.
/// The reading thread automatically restarts on exit so live logs keep flowing.
pub fn spawn_logcat(serial: Option<&str>) -> Result<LogcatHandle, Box<dyn Error>> {
    let mut child = spawn_adb_logcat(serial, Stdio::piped())?;

    let stdout = child.stdout.take().ok_or("Failed to capture adb stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture adb stderr")?;
    let (sender, receiver) = mpsc::channel();
    let stop = Arc::new(AtomicBool::new(false));
    let current_child = Arc::new(Mutex::new(Some(child)));
    let reader_stop = Arc::clone(&stop);
    let reader_child = Arc::clone(&current_child);
    let serial = serial.map(str::to_owned);

    spawn_stderr_logger(stderr);

    // Reading thread: streams the initial spawn, then restarts live-only on exit.
    thread::spawn(move || {
        if !stream_stdout(stdout, &sender, &reader_stop) {
            kill_current_child(&reader_child);
            wait_current_child(&reader_child);
            return;
        }
        wait_current_child(&reader_child);

        if reader_stop.load(Ordering::Relaxed) {
            return;
        }
        if sender.send(LogcatEvent::Disconnected).is_err() {
            return;
        }
        Logger::info_fmt(
            "adb logcat exited after initial buffer; restarting live-only",
            &[],
        );

        loop {
            if reader_stop.load(Ordering::Relaxed) {
                return;
            }
            thread::sleep(Duration::from_secs(1));
            if reader_stop.load(Ordering::Relaxed) {
                return;
            }

            let mut child = match spawn_adb_logcat(serial.as_deref(), Stdio::piped()) {
                Ok(c) => c,
                Err(e) => {
                    Logger::info_fmt("adb restart failed:", &[&e.to_string()]);
                    continue;
                }
            };

            let stdout = match child.stdout.take() {
                Some(s) => s,
                None => return,
            };
            let stderr = match child.stderr.take() {
                Some(s) => s,
                None => return,
            };
            if let Ok(mut guard) = reader_child.lock() {
                *guard = Some(child);
            } else {
                return;
            }
            spawn_stderr_logger(stderr);

            if sender.send(LogcatEvent::Connected).is_err() {
                kill_current_child(&reader_child);
                wait_current_child(&reader_child);
                return;
            }

            if !stream_stdout(stdout, &sender, &reader_stop) {
                kill_current_child(&reader_child);
                wait_current_child(&reader_child);
                return;
            }
            wait_current_child(&reader_child);

            if reader_stop.load(Ordering::Relaxed) {
                return;
            }
            if sender.send(LogcatEvent::Disconnected).is_err() {
                return;
            }
            Logger::info_fmt("adb logcat live instance exited, retrying", &[]);
        }
    });

    Ok(LogcatHandle {
        receiver,
        stop,
        current_child,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adb_devices_ready_when_at_least_one_device_is_authorized() {
        let output = "List of devices attached\nemulator-5554\tdevice\n";
        assert_eq!(parse_adb_devices_output(output, None), DeviceCheck::Ready);
    }

    #[test]
    fn adb_devices_unready_for_unauthorized_or_offline_entries() {
        let output =
            "List of devices attached\nemulator-5554\tunauthorized\nemulator-5556\toffline\n";
        assert_eq!(parse_adb_devices_output(output, None), DeviceCheck::Unready);
    }

    #[test]
    fn adb_devices_missing_when_no_entries_exist() {
        let output = "List of devices attached\n\n";
        assert_eq!(parse_adb_devices_output(output, None), DeviceCheck::Missing);
    }

    #[test]
    fn adb_devices_require_serial_when_multiple_ready_devices_exist() {
        let output = "List of devices attached\nemulator-5554\tdevice\nemulator-5556\tdevice\n";
        assert_eq!(
            parse_adb_devices_output(output, None),
            DeviceCheck::MultipleReady
        );
    }

    #[test]
    fn adb_devices_accept_matching_serial() {
        let output = "List of devices attached\nemulator-5554\tdevice\nemulator-5556\tdevice\n";
        assert_eq!(
            parse_adb_devices_output(output, Some("emulator-5556")),
            DeviceCheck::Ready
        );
    }

    #[test]
    fn adb_devices_report_missing_requested_serial() {
        let output = "List of devices attached\nemulator-5554\tdevice\n";
        assert_eq!(
            parse_adb_devices_output(output, Some("emulator-5556")),
            DeviceCheck::RequestedSerialMissing
        );
    }

    #[test]
    fn adb_devices_report_requested_serial_as_unready() {
        let output = "List of devices attached\nemulator-5554\tunauthorized\n";
        assert_eq!(
            parse_adb_devices_output(output, Some("emulator-5554")),
            DeviceCheck::RequestedSerialUnready
        );
    }
}
