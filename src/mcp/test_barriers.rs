use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

#[cfg(debug_assertions)]
const MCP_STARTUP_TEST_BARRIER_ENV: &str = "SPECSYNC_TEST_MCP_STARTUP_IDENTITY_BARRIER";
#[cfg(debug_assertions)]
const MCP_STARTUP_TEST_BARRIER_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
#[cfg(debug_assertions)]
const MCP_SNAPSHOT_FILE_TEST_BARRIER_ENV: &str = "SPECSYNC_TEST_MCP_SNAPSHOT_FILE_IDENTITY_BARRIER";
#[cfg(debug_assertions)]
const MCP_SNAPSHOT_FILE_TEST_PATH_ENV: &str = "SPECSYNC_TEST_MCP_SNAPSHOT_FILE_PATH";
#[cfg(debug_assertions)]
const MCP_SNAPSHOT_DIRECTORY_TEST_BARRIER_ENV: &str =
    "SPECSYNC_TEST_MCP_SNAPSHOT_DIRECTORY_IDENTITY_BARRIER";
#[cfg(debug_assertions)]
const MCP_SNAPSHOT_DIRECTORY_TEST_PATH_ENV: &str = "SPECSYNC_TEST_MCP_SNAPSHOT_DIRECTORY_PATH";
#[cfg(debug_assertions)]
const MCP_READ_ROOT_REVALIDATION_TEST_BARRIER_ENV: &str =
    "SPECSYNC_TEST_MCP_READ_ROOT_REVALIDATION_BARRIER";

#[cfg(debug_assertions)]
pub(super) fn wait_for_mcp_startup_test_barrier() -> Result<(), String> {
    let Some(barrier_directory) = std::env::var_os(MCP_STARTUP_TEST_BARRIER_ENV) else {
        return Ok(());
    };
    let barrier_directory = PathBuf::from(barrier_directory);
    let ready_path = barrier_directory.join("identity-bound");
    let resume_path = barrier_directory.join("resume");
    let mut ready_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&ready_path)
        .map_err(|error| format!("Cannot create MCP startup test barrier marker: {error}"))?;
    ready_file
        .write_all(b"identity-bound\n")
        .map_err(|error| format!("Cannot write MCP startup test barrier marker: {error}"))?;
    drop(ready_file);

    let started = std::time::Instant::now();
    loop {
        match fs::metadata(&resume_path) {
            Ok(metadata) if metadata.is_file() => return Ok(()),
            Ok(_) => {
                return Err("MCP startup test barrier resume marker is not a file".to_string());
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP startup test barrier resume marker: {error}"
                ));
            }
        }
        if started.elapsed() >= MCP_STARTUP_TEST_BARRIER_TIMEOUT {
            return Err("Timed out waiting for MCP startup test barrier".to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[cfg(debug_assertions)]
pub(super) fn wait_for_mcp_snapshot_file_test_barrier(relative: &Path) -> Result<(), String> {
    let Some(target) = std::env::var_os(MCP_SNAPSHOT_FILE_TEST_PATH_ENV) else {
        return Ok(());
    };
    if Path::new(&target) != relative {
        return Ok(());
    }
    let Some(barrier_directory) = std::env::var_os(MCP_SNAPSHOT_FILE_TEST_BARRIER_ENV) else {
        return Err("MCP snapshot file test barrier path is not configured".to_string());
    };
    let barrier_directory = PathBuf::from(barrier_directory);
    let ready_path = barrier_directory.join("retained-open");
    let resume_path = barrier_directory.join("resume");
    let mut ready_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&ready_path)
        .map_err(|error| format!("Cannot create MCP snapshot file test barrier: {error}"))?;
    ready_file
        .write_all(b"retained-open\n")
        .and_then(|_| ready_file.sync_all())
        .map_err(|error| format!("Cannot publish MCP snapshot file test barrier: {error}"))?;
    drop(ready_file);

    let started = std::time::Instant::now();
    loop {
        match fs::metadata(&resume_path) {
            Ok(metadata) if metadata.is_file() => return Ok(()),
            Ok(_) => {
                return Err(
                    "MCP snapshot file test barrier resume marker is not a file".to_string()
                );
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP snapshot file test barrier resume marker: {error}"
                ));
            }
        }
        if started.elapsed() >= MCP_STARTUP_TEST_BARRIER_TIMEOUT {
            return Err("Timed out waiting for MCP snapshot file test barrier".to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[cfg(debug_assertions)]
pub(super) fn wait_for_mcp_snapshot_directory_test_barrier(relative: &Path) -> Result<(), String> {
    let Some(target) = std::env::var_os(MCP_SNAPSHOT_DIRECTORY_TEST_PATH_ENV) else {
        return Ok(());
    };
    if Path::new(&target) != relative {
        return Ok(());
    }
    let Some(barrier_directory) = std::env::var_os(MCP_SNAPSHOT_DIRECTORY_TEST_BARRIER_ENV) else {
        return Err("MCP snapshot directory test barrier path is not configured".to_string());
    };
    let barrier_directory = PathBuf::from(barrier_directory);
    let ready_path = barrier_directory.join("enumerated-open");
    let resume_path = barrier_directory.join("resume");
    let mut ready_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&ready_path)
        .map_err(|error| format!("Cannot create MCP snapshot directory test barrier: {error}"))?;
    ready_file
        .write_all(b"enumerated-open\n")
        .and_then(|_| ready_file.sync_all())
        .map_err(|error| format!("Cannot publish MCP snapshot directory test barrier: {error}"))?;
    drop(ready_file);

    let started = std::time::Instant::now();
    loop {
        match fs::metadata(&resume_path) {
            Ok(metadata) if metadata.is_file() => return Ok(()),
            Ok(_) => {
                return Err(
                    "MCP snapshot directory test barrier resume marker is not a file".to_string(),
                );
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP snapshot directory test barrier resume marker: {error}"
                ));
            }
        }
        if started.elapsed() >= MCP_STARTUP_TEST_BARRIER_TIMEOUT {
            return Err("Timed out waiting for MCP snapshot directory test barrier".to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[cfg(debug_assertions)]
pub(super) fn wait_for_mcp_read_root_revalidation_test_barrier() -> Result<(), String> {
    let Some(barrier_directory) = std::env::var_os(MCP_READ_ROOT_REVALIDATION_TEST_BARRIER_ENV)
    else {
        return Ok(());
    };
    let barrier_directory = PathBuf::from(barrier_directory);
    let ready_path = barrier_directory.join("operation-complete");
    let resume_path = barrier_directory.join("resume");
    let mut ready_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&ready_path)
        .map_err(|error| format!("Cannot create MCP read-root test barrier: {error}"))?;
    ready_file
        .write_all(b"operation-complete\n")
        .and_then(|_| ready_file.sync_all())
        .map_err(|error| format!("Cannot publish MCP read-root test barrier: {error}"))?;
    drop(ready_file);

    let started = std::time::Instant::now();
    loop {
        match fs::metadata(&resume_path) {
            Ok(metadata) if metadata.is_file() => return Ok(()),
            Ok(_) => {
                return Err("MCP read-root test barrier resume marker is not a file".to_string());
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP read-root test barrier resume marker: {error}"
                ));
            }
        }
        if started.elapsed() >= MCP_STARTUP_TEST_BARRIER_TIMEOUT {
            return Err("Timed out waiting for MCP read-root test barrier".to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
