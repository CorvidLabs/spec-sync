//! #574 (SPEED-1.b): `specsync check` time grows with the number of undocumented exports, not
//! with its square. Each undocumented export used to rescan every export to name its file.
//!
//! Wall-clock, so it is a ratio and not a budget: it compares two sizes on the same machine in
//! the same run. Four times the exports may cost about four times as long, never sixteen. A
//! fixed number of seconds would fail on a slow or busy machine for no fault of the code (#707);
//! a ratio moves only when the growth does. The two sizes are run in turn, small then large,
//! three times, and each keeps its fastest run, so a change in load during the test reaches both.
//!
//! The sizes are 32,000 and 128,000 exports because the old cost has to show in a release build
//! too. At 8,000 against 32,000 a release build of the old code took only about five times as
//! long, under the bound, since start-up and parsing still outweighed the rescans.

use crate::helpers::{specsync_std, valid_spec, write_config};
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use tempfile::TempDir;

fn project_with_undocumented_exports(count: usize) -> TempDir {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/wide")).unwrap();
    fs::create_dir_all(root.join("specs/wide")).unwrap();
    let source: String = (0..count)
        .map(|index| format!("pub fn undocumented_export_{index}() {{}}\n"))
        .collect();
    fs::write(root.join("src/wide/lib.rs"), source).unwrap();
    fs::write(
        root.join("specs/wide/wide.spec.md"),
        valid_spec("wide", &["src/wide/lib.rs"]),
    )
    .unwrap();
    temp
}

/// One timed `check` run.
fn check_time(root: &Path) -> Duration {
    let started = Instant::now();
    let output = specsync_std()
        .args(["check", "--force", "--root"])
        .arg(root)
        .output()
        .unwrap();
    let elapsed = started.elapsed();
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("undocumented_export_")
            || String::from_utf8_lossy(&output.stderr).contains("undocumented_export_"),
        "check did not report the undocumented exports"
    );
    elapsed
}

#[test]
fn check_time_grows_linearly_with_undocumented_exports() {
    let small = project_with_undocumented_exports(32_000);
    let large = project_with_undocumented_exports(128_000);
    let mut small_time = Duration::MAX;
    let mut large_time = Duration::MAX;
    for _ in 0..3 {
        small_time = small_time.min(check_time(small.path()));
        large_time = large_time.min(check_time(large.path()));
    }
    let ratio = large_time.as_secs_f64() / small_time.as_secs_f64().max(0.001);
    eprintln!(
        "check: 32,000 exports {small_time:?}, 128,000 exports {large_time:?}, ratio {ratio:.1}"
    );
    assert!(
        ratio < 7.0,
        "four times the undocumented exports took {ratio:.1} times as long \
         ({small_time:?} vs {large_time:?}); linear would be about 4, quadratic about 16"
    );
}
