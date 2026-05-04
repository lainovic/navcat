use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use navcat::application::adb::{LogcatEvent, check_device_connected, spawn_logcat};
use navcat::application::cli::{Args, VerbosityLevel};
use navcat::domain::filter::LogFilter;
use navcat::domain::filter_config::FilterState;
use tempfile::TempDir;

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn fake_adb_script(dir: &Path) -> PathBuf {
    let path = dir.join("adb");
    fs::write(
        &path,
        r#"#!/bin/sh
set -eu

STATE_DIR="${NAVCAT_TEST_STATE_DIR:?}"

case "${1:-}" in
  version)
    echo "Android Debug Bridge version fake"
    ;;
  devices)
    echo "List of devices attached"
    if [ -n "${NAVCAT_TEST_DEVICES_LINE:-}" ]; then
      printf '%s\n' "$NAVCAT_TEST_DEVICES_LINE"
    fi
    ;;
  logcat)
    count_file="$STATE_DIR/logcat_count"
    count=0
    if [ -f "$count_file" ]; then
      count=$(cat "$count_file")
    fi
    count=$((count + 1))
    echo "$count" > "$count_file"

    if [ "$count" -eq 1 ]; then
      printf '%s\n' '01-15 10:30:45.123 1234 5678 I FirstTag: first'
      exit 0
    fi

    echo $$ > "$STATE_DIR/restart_pid"
    trap 'exit 0' TERM INT
    printf '%s\n' '01-15 10:30:46.123 1234 5678 I SecondTag: second'
    while :; do
      sleep 1
    done
    ;;
  *)
    echo "unexpected adb command: $*" >&2
    exit 1
    ;;
esac
"#,
    )
    .unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    path
}

fn base_args(tags: &str) -> Args {
    Args {
        file: None,
        logcat_levels: "I".to_string(),
        tags: tags.to_string(),
        add_tag: vec![],
        no_tag_filter: false,
        serial: None,
        debug_level: VerbosityLevel::None,
        highlighted_items: vec![],
        show_items: vec![],
        completions: None,
        version: false,
    }
}

#[test]
fn lowercase_tags_work_through_cli_to_filter_pipeline() {
    let args = base_args("guidance,replan");
    let state = FilterState::from_args(&args);
    let filter = LogFilter::new(state.to_filter_config());

    let guidance = "2024-01-15 10:30:45 1234 5678 I LaneGuidance: hello";
    let replan = "2024-01-15 10:30:46 1234 5678 I ReplanEngine: hello";

    assert!(filter.matches(guidance).is_some());
    assert!(filter.matches(replan).is_some());
}

#[test]
fn logcat_reconnects_and_shutdown_kills_restarted_child() {
    let _guard = env_lock().lock().unwrap();
    let dir = TempDir::new().unwrap();
    let adb = fake_adb_script(dir.path());

    unsafe {
        std::env::set_var("NAVCAT_ADB", &adb);
        std::env::set_var("NAVCAT_TEST_STATE_DIR", dir.path());
        std::env::set_var("NAVCAT_TEST_DEVICES_LINE", "emulator-5554\tdevice");
    }

    check_device_connected(None).unwrap();
    let mut handle = spawn_logcat(None).unwrap();

    assert!(matches!(
        handle.receiver().recv_timeout(Duration::from_secs(1)).unwrap(),
        LogcatEvent::Line(line) if line.contains("FirstTag")
    ));
    assert!(matches!(
        handle
            .receiver()
            .recv_timeout(Duration::from_secs(2))
            .unwrap(),
        LogcatEvent::Disconnected
    ));
    assert!(matches!(
        handle
            .receiver()
            .recv_timeout(Duration::from_secs(2))
            .unwrap(),
        LogcatEvent::Connected
    ));
    assert!(matches!(
        handle.receiver().recv_timeout(Duration::from_secs(1)).unwrap(),
        LogcatEvent::Line(line) if line.contains("SecondTag")
    ));

    let restart_pid: u32 = fs::read_to_string(dir.path().join("restart_pid"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    handle.shutdown();

    for _ in 0..20 {
        if !Command::new("kill")
            .arg("-0")
            .arg(restart_pid.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
        {
            unsafe {
                std::env::remove_var("NAVCAT_ADB");
                std::env::remove_var("NAVCAT_TEST_STATE_DIR");
                std::env::remove_var("NAVCAT_TEST_DEVICES_LINE");
            }
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    unsafe {
        std::env::remove_var("NAVCAT_ADB");
        std::env::remove_var("NAVCAT_TEST_STATE_DIR");
        std::env::remove_var("NAVCAT_TEST_DEVICES_LINE");
    }
    panic!("restarted adb child was not terminated by shutdown");
}

#[test]
fn multiple_ready_devices_require_serial_and_matching_serial_is_accepted() {
    let _guard = env_lock().lock().unwrap();
    let dir = TempDir::new().unwrap();
    let adb = fake_adb_script(dir.path());

    unsafe {
        std::env::set_var("NAVCAT_ADB", &adb);
        std::env::set_var("NAVCAT_TEST_STATE_DIR", dir.path());
        std::env::set_var(
            "NAVCAT_TEST_DEVICES_LINE",
            "emulator-5554\tdevice\nemulator-5556\tdevice",
        );
    }

    let err = check_device_connected(None).unwrap_err().to_string();
    assert!(err.contains("Multiple adb devices are ready"));
    check_device_connected(Some("emulator-5556")).unwrap();

    unsafe {
        std::env::remove_var("NAVCAT_ADB");
        std::env::remove_var("NAVCAT_TEST_STATE_DIR");
        std::env::remove_var("NAVCAT_TEST_DEVICES_LINE");
    }
}
