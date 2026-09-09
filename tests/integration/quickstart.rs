//! The committed quickstart example (`examples/quickstart/`) is the first thing the README
//! tells a new user to run. It has to hold three promises standalone, from its own root:
//! `specsync check` passes with full coverage, the README's "make it fail" step produces the
//! undocumented-export warning, and `--strict` turns that warning into a failing exit.
//!
//! Before this test existed the spec listed `examples/quickstart/src/lib.rs` — a path relative
//! to the outer repository — and carried `status: draft`, so from the example's own root the
//! source "did not exist yet", coverage was 0/1, and export validation was skipped entirely.
//! Nothing checked the example, so the README promised a loop the example could not run.

use crate::helpers::*;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

/// A disposable copy of `examples/quickstart/`, so the tests can mutate it.
fn quickstart_copy() -> TempDir {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/quickstart");
    let tmp = TempDir::new().unwrap();
    copy_dir(&source, tmp.path());
    tmp
}

#[test]
fn quickstart_example_passes_strict_check_with_full_coverage_from_its_own_root() {
    let tmp = quickstart_copy();

    specsync()
        .args(["check", "--strict", "--require-coverage", "100"])
        .arg("--root")
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("All source files exist"))
        .stdout(predicate::str::contains("1/1 exports documented"))
        .stdout(predicate::str::contains("1 passed, 0 warning(s), 0 failed"))
        .stdout(predicate::str::contains("File coverage: 1/1 (100%)"));
}

#[test]
fn quickstart_example_reports_the_readme_undocumented_export() {
    let tmp = quickstart_copy();
    let lib = tmp.path().join("src/lib.rs");
    let mut source = fs::read_to_string(&lib).unwrap();
    source
        .push_str("\npub fn farewell(name: &str) -> String {\n    format!(\"bye, {name}!\")\n}\n");
    fs::write(&lib, source).unwrap();

    // Default enforcement: the drift is a warning and the exit stays zero.
    specsync()
        .arg("check")
        .arg("--root")
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Undocumented export 'farewell'"));

    // `--strict` is the CI form the README recommends: the same drift fails.
    specsync()
        .args(["check", "--strict"])
        .arg("--root")
        .arg(tmp.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("Undocumented export 'farewell'"));
}

#[test]
fn readme_quick_start_init_add_spec_and_check_succeed() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    specsync()
        .args(["init", "--root"])
        .arg(root)
        .assert()
        .success();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/auth.ts"),
        "export function login(): boolean {\n  return true;\n}\n",
    )
    .unwrap();

    specsync()
        .args(["add-spec", "auth", "--root"])
        .arg(root)
        .assert()
        .success();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(root)
        .assert()
        .success();

    specsync()
        .arg("coverage")
        .arg("--root")
        .arg(root)
        .assert()
        .success();

    specsync()
        .args(["score", "--all", "--root"])
        .arg(root)
        .assert()
        .success();
}
