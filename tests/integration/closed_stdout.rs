//! #1036: a reader that closes SpecSync's output early (`specsync … | head`) never crashes it or
//! hides its verdict. The process ends with the command's own exit status (REQ-cli: exit 0 on
//! success, 1 on errors), and nothing panics.

use super::helpers::*;
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// Runs SpecSync with its stdout reader dropped before it writes anything; `close_stderr` drops
/// the stderr reader too. Returns the exit code and whatever stderr was read.
fn run_with_closed_output(args: &[&str], close_stderr: bool) -> (Option<i32>, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_specsync"))
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let mut stderr = String::new();
    if close_stderr {
        drop(child.stderr.take());
    } else {
        child
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut stderr)
            .unwrap();
    }
    let status = child.wait().unwrap();
    (status.code(), stderr)
}

fn failing_project(tmp: &TempDir) -> std::path::PathBuf {
    let root = setup_minimal_project(tmp);
    std::fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts", "src/auth/missing.ts"]),
    )
    .unwrap();
    root
}

#[test]
fn a_closed_stdout_ends_a_passing_check_with_exit_zero_and_no_panic() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    for args in [
        vec!["--root", root.to_str().unwrap(), "check"],
        vec!["--root", root.to_str().unwrap(), "check", "--json"],
    ] {
        let (code, stderr) = run_with_closed_output(&args, false);
        assert_eq!(code, Some(0), "{args:?}: {stderr}");
        assert!(!stderr.contains("panicked"), "{args:?}: {stderr}");
    }
}

#[test]
fn a_closed_stdout_keeps_a_failing_checks_exit_one() {
    let tmp = TempDir::new().unwrap();
    let root = failing_project(&tmp);
    let args = ["--root", root.to_str().unwrap(), "check"];
    let (code, stderr) = run_with_closed_output(&args, false);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
    // With stderr gone as well, the verdict still stands.
    let (code, _) = run_with_closed_output(&args, true);
    assert_eq!(code, Some(1));
}

#[test]
fn watch_stops_once_nobody_reads_its_output() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let mut child = Command::new(env!("CARGO_BIN_EXE_specsync"))
        .args(["--root", root.to_str().unwrap(), "watch"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let deadline = Instant::now() + Duration::from_secs(60);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            panic!("watch kept running with nobody reading its output");
        }
        std::thread::sleep(Duration::from_millis(100));
    };
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    assert_eq!(status.code(), Some(0), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
}

/// The closed-pipe handling covers only a reader that has gone (#1048 review): any other write
/// error still panics and is reported as a bug (exit 1), never dropped silently. Linux only:
/// `/dev/full` fails every write with ENOSPC.
#[cfg(target_os = "linux")]
#[test]
fn a_write_error_other_than_a_closed_pipe_still_panics() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_specsync"))
        .args(["--root", root.to_str().unwrap(), "check"])
        .stdout(full)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("failed printing to stdout") && stderr.contains("please report it"),
        "{stderr}"
    );
}
