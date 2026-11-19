//! #574 (SPEED-1.b): `specsync check` time grows with the number of undocumented exports, not
//! with its square. Each undocumented export used to rescan every export to name its file.
//!
//! Wall-clock, so it compares two sizes on the same machine in the same run rather than against
//! a fixed budget: four times the exports may cost about four times as long, never sixteen.

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

/// The faster of two `check` runs, so one scheduling hiccup does not decide the ratio.
fn check_time(root: &Path) -> Duration {
    (0..2)
        .map(|_| {
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
        })
        .min()
        .unwrap()
}

#[test]
fn check_time_grows_linearly_with_undocumented_exports() {
    let small = project_with_undocumented_exports(8_000);
    let large = project_with_undocumented_exports(32_000);
    let small_time = check_time(small.path());
    let large_time = check_time(large.path());
    let ratio = large_time.as_secs_f64() / small_time.as_secs_f64().max(0.001);
    eprintln!(
        "check: 8,000 exports {small_time:?}, 32,000 exports {large_time:?}, ratio {ratio:.1}"
    );
    assert!(
        ratio < 7.0,
        "four times the undocumented exports took {ratio:.1} times as long \
         ({small_time:?} vs {large_time:?}); linear would be about 4, quadratic about 16"
    );
}
