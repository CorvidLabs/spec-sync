#[cfg(debug_assertions)]
use std::fs;
#[cfg(debug_assertions)]
use std::io::{self, Write};
#[cfg(debug_assertions)]
use std::path::PathBuf;

#[cfg(debug_assertions)]
const COVERAGE_SNAPSHOT_TEST_BARRIER_ENV: &str = "SPECSYNC_TEST_COVERAGE_SNAPSHOT_IDENTITY_BARRIER";
#[cfg(debug_assertions)]
const COVERAGE_SNAPSHOT_TEST_BARRIER_PHASE_ENV: &str =
    "SPECSYNC_TEST_COVERAGE_SNAPSHOT_IDENTITY_BARRIER_PHASE";
#[cfg(debug_assertions)]
const COVERAGE_SNAPSHOT_TEST_CONTEXT_ENV: &str = "SPECSYNC_TEST_CONTEXT";
#[cfg(debug_assertions)]
const COVERAGE_SNAPSHOT_TEST_CONTEXT: &str = "coverage-snapshot-identity";
#[cfg(debug_assertions)]
const COVERAGE_SNAPSHOT_TEST_BARRIER_TIMEOUT: std::time::Duration =
    std::time::Duration::from_secs(30);

#[derive(Clone, Copy)]
pub(super) enum CoverageSnapshotCheckpoint {
    RootRetained,
    ManifestDiscovered,
}

#[cfg(debug_assertions)]
impl CoverageSnapshotCheckpoint {
    fn marker(self) -> &'static str {
        match self {
            Self::RootRetained => "root-retained",
            Self::ManifestDiscovered => "manifest-discovered",
        }
    }
}

#[cfg(debug_assertions)]
pub(super) fn coverage_snapshot_test_barrier(
    checkpoint: CoverageSnapshotCheckpoint,
) -> Result<(), String> {
    let Some(directory) = std::env::var_os(COVERAGE_SNAPSHOT_TEST_BARRIER_ENV) else {
        return Ok(());
    };
    if std::env::var(COVERAGE_SNAPSHOT_TEST_CONTEXT_ENV).as_deref()
        != Ok(COVERAGE_SNAPSHOT_TEST_CONTEXT)
    {
        return Err(format!(
            "Coverage snapshot test barrier requires {COVERAGE_SNAPSHOT_TEST_CONTEXT_ENV}={COVERAGE_SNAPSHOT_TEST_CONTEXT}"
        ));
    }
    let selected_checkpoint = std::env::var(COVERAGE_SNAPSHOT_TEST_BARRIER_PHASE_ENV)
        .unwrap_or_else(|_| {
            CoverageSnapshotCheckpoint::RootRetained
                .marker()
                .to_string()
        });
    if selected_checkpoint != checkpoint.marker() {
        return Ok(());
    }
    let directory = PathBuf::from(directory);
    let marker = checkpoint.marker();
    let mut ready = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join(marker))
        .map_err(|error| format!("Cannot create coverage snapshot test barrier: {error}"))?;
    ready
        .write_all(format!("{marker}\n").as_bytes())
        .and_then(|_| ready.sync_all())
        .map_err(|error| format!("Cannot publish coverage snapshot test barrier: {error}"))?;
    drop(ready);
    let resume = directory.join("resume");
    let started = std::time::Instant::now();
    loop {
        match fs::symlink_metadata(&resume) {
            Ok(metadata) if metadata.is_file() => return Ok(()),
            Ok(_) => {
                return Err(
                    "Coverage snapshot test barrier resume marker is not a file".to_string()
                );
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot inspect coverage snapshot test barrier resume marker: {error}"
                ));
            }
        }
        if started.elapsed() >= COVERAGE_SNAPSHOT_TEST_BARRIER_TIMEOUT {
            return Err("Timed out waiting for coverage snapshot test barrier".to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[cfg(not(debug_assertions))]
pub(super) fn coverage_snapshot_test_barrier(
    _checkpoint: CoverageSnapshotCheckpoint,
) -> Result<(), String> {
    Ok(())
}
