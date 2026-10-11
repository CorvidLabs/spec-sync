use crate::config::{detect_source_dirs, load_config, parse_config_content_checked};
use crate::deps::build_dep_graph;
use crate::generator::generate_specs_for_unspecced_modules_paths;
use crate::manifest::{MAX_GRADLE_MANIFEST_BYTES, parse_gradle_settings};
use crate::output::{CoverageFindings, coverage_json};
use crate::scoring;
use crate::types::SpecSyncConfig;
use crate::validator::{
    compute_coverage_checked, find_spec_files, get_schema_table_names, validate_spec,
};
use cap_primitives::fs::FollowSymlinks;
use cap_std::ambient_authority;
use cap_std::fs::{Dir, Metadata, OpenOptions};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs;
use std::io::{self, BufRead, Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use walkdir::WalkDir;

const SERVER_NAME: &str = "specsync";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
const PROTOCOL_VERSION: &str = "2024-11-05";
const MAX_JSON_RPC_LINE_BYTES: usize = 1024 * 1024;
const MAX_JSON_RPC_RESPONSE_BYTES: usize = 1024 * 1024;
const MAX_PROJECT_FILE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_PROJECT_INPUT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_CONFINEMENT_ENTRIES: usize = 100_000;
const MAX_MANIFEST_PREFLIGHTS: usize = 1_000;
const MAX_CONFIGURED_PATHS: usize = 1_000;
const MAX_JSON_RPC_ID_BYTES: usize = 4 * 1024;
const MAX_GENERATED_SPECS: usize = 1_000;
const MAX_GENERATED_OUTPUT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_TOOL_CONTENT_RESPONSE_BYTES: usize = MAX_JSON_RPC_RESPONSE_BYTES - 16 * 1024;
const MAX_ISSUE_DIAGNOSTIC_PATH_CHARS: usize = 240;
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
static STAGED_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const SNAPSHOT_IGNORED_DIRS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    "node_modules",
    "target",
    "vendor",
    ".next",
    ".nuxt",
    ".output",
    ".cache",
    ".turbo",
    "__pycache__",
    ".mypy_cache",
    ".pytest_cache",
    ".tox",
    ".venv",
    "venv",
    ".dart_tool",
    ".gradle",
    "Pods",
    "obj",
];
const AUTODETECT_IGNORED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    ".hg",
    ".svn",
    "dist",
    "build",
    "out",
    "target",
    "vendor",
    ".next",
    ".nuxt",
    ".output",
    ".cache",
    ".turbo",
    "coverage",
    "__pycache__",
    ".mypy_cache",
    ".pytest_cache",
    ".tox",
    ".venv",
    "venv",
    "env",
    ".env",
    ".idea",
    ".vscode",
    ".DS_Store",
    "specs",
    "docs",
    "doc",
    ".github",
    ".gitlab",
    "migrations",
    "Pods",
    ".dart_tool",
    ".gradle",
    "bin",
    "obj",
];

fn read_only_nofollow_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options
        .read(true)
        ._cap_fs_ext_nonblock(true)
        ._cap_fs_ext_follow(FollowSymlinks::No);
    options
}

/// Run the MCP server on stdio.
pub fn run_mcp_server(root: &Path, allow_write: bool) -> Result<(), String> {
    let (server_root, server_directory) = open_server_root_capability(root, || {
        #[cfg(debug_assertions)]
        wait_for_mcp_startup_test_barrier()?;
        Ok(())
    })?;
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    loop {
        let line = match read_mcp_line(&mut stdin) {
            Ok(Some(Ok(line))) => line,
            Ok(Some(Err(message))) => {
                let error = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": { "code": -32700, "message": message }
                });
                write_mcp_response(&mut stdout, &error)?;
                continue;
            }
            Ok(None) => break,
            Err(error) => return Err(format!("Failed to read MCP stdin: {error}")),
        };

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let request: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => {
                let err = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": { "code": -32700, "message": "Parse error" }
                });
                write_mcp_response(&mut stdout, &err)?;
                continue;
            }
        };

        let id = match validate_request_envelope(&request) {
            Ok(id) => id,
            Err(response) => {
                write_mcp_response(&mut stdout, &response)?;
                continue;
            }
        };

        // JSON-RPC notifications never receive a response. Suppress them before
        // dispatch so even mutating tools cannot execute without an acknowledged ID.
        if id.is_none() {
            continue;
        }

        let method = request
            .get("method")
            .and_then(Value::as_str)
            .ok_or_else(|| "Validated MCP request lost its method".to_string())?;

        let response = match method {
            "initialize" => Some(handle_initialize_request(id, request.get("params"))),
            "tools/list" => Some(handle_tools_list(id, allow_write)),
            "tools/call" => {
                let params = request.get("params").cloned().unwrap_or(json!({}));
                Some(handle_tools_call_with_directory(
                    id,
                    &params,
                    &server_root,
                    root,
                    &server_directory,
                    allow_write,
                ))
            }
            "resources/list" => Some(handle_resources_list(id)),
            "resources/read" => {
                let params = request.get("params").cloned().unwrap_or(json!({}));
                Some(handle_resources_read_with_directory(
                    id,
                    &params,
                    &server_root,
                    &server_directory,
                ))
            }
            "ping" => Some(json!({ "jsonrpc": "2.0", "id": id, "result": {} })),
            _ => Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": format!("Method not found: {method}") }
            })),
        };

        if let Some(resp) = response {
            write_mcp_response(&mut stdout, &resp)?;
        }
    }

    Ok(())
}

fn open_server_root_capability(
    root: &Path,
    after_identity_bound: impl FnOnce() -> Result<(), String>,
) -> Result<(PathBuf, Dir), String> {
    let server_directory = Dir::open_ambient_dir(root, ambient_authority()).map_err(|error| {
        if error.kind() == io::ErrorKind::NotADirectory {
            format!("MCP server root is not a directory: {}", root.display())
        } else {
            format!(
                "Cannot open initial MCP server root capability {}: {error}",
                root.display()
            )
        }
    })?;
    let expected_identity = directory_identity(&server_directory).map_err(|error| {
        format!(
            "Cannot bind initial MCP server root identity {}: {error}",
            root.display()
        )
    })?;
    after_identity_bound()?;

    let server_root = root
        .canonicalize()
        .map_err(|error| format!("Cannot resolve MCP server root {}: {error}", root.display()))?;
    let observed_directory = match (server_root.parent(), server_root.file_name()) {
        (Some(parent), Some(name)) => {
            let parent_directory =
                Dir::open_ambient_dir(parent, ambient_authority()).map_err(|error| {
                    format!(
                        "Cannot open MCP server-root parent capability {}: {error}",
                        parent.display()
                    )
                })?;
            parent_directory.open_dir(name).map_err(|error| {
                format!(
                    "Cannot reopen MCP server root through its parent capability {}: {error}",
                    server_root.display()
                )
            })?
        }
        _ => Dir::open_ambient_dir(&server_root, ambient_authority()).map_err(|error| {
            format!(
                "Cannot reopen filesystem-root MCP capability {}: {error}",
                server_root.display()
            )
        })?,
    };
    let observed_identity = directory_identity(&observed_directory).map_err(|error| {
        format!(
            "Cannot verify MCP server root identity {}: {error}",
            server_root.display()
        )
    })?;
    if expected_identity != observed_identity {
        return Err(format!(
            "MCP server root changed while its identity was being resolved: {}",
            server_root.display()
        ));
    }

    Ok((server_root, server_directory))
}

#[cfg(debug_assertions)]
fn wait_for_mcp_startup_test_barrier() -> Result<(), String> {
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
fn wait_for_mcp_snapshot_file_test_barrier(relative: &Path) -> Result<(), String> {
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
fn wait_for_mcp_snapshot_directory_test_barrier(relative: &Path) -> Result<(), String> {
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
fn wait_for_mcp_read_root_revalidation_test_barrier() -> Result<(), String> {
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

#[cfg(unix)]
type DirectoryIdentity = (u64, u64);

#[cfg(unix)]
type FileIdentity = (u64, u64, [u8; 32]);

#[cfg(unix)]
fn directory_identity(directory: &Dir) -> io::Result<DirectoryIdentity> {
    use cap_std::fs::MetadataExt;

    let metadata = directory.dir_metadata()?;
    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(unix)]
fn file_identity(file: &cap_std::fs::File) -> io::Result<FileIdentity> {
    use cap_std::fs::MetadataExt;

    let metadata = file.metadata()?;
    Ok((metadata.dev(), metadata.ino(), file_content_digest(file)?))
}

#[cfg(windows)]
type DirectoryIdentity = (u32, u64);

#[cfg(windows)]
type FileIdentity = (u32, u64, [u8; 32]);

#[cfg(windows)]
fn directory_identity(directory: &Dir) -> io::Result<DirectoryIdentity> {
    use std::os::windows::io::AsRawHandle;

    let file = directory.try_clone()?.into_std_file();
    windows_handle_identity(file.as_raw_handle().cast())
}

#[cfg(windows)]
fn file_identity(file: &cap_std::fs::File) -> io::Result<FileIdentity> {
    use std::os::windows::io::AsRawHandle;

    let (volume, index) = windows_handle_identity(file.as_raw_handle().cast())?;
    Ok((volume, index, file_content_digest(file)?))
}

#[cfg(windows)]
fn windows_handle_identity(handle: *mut std::ffi::c_void) -> io::Result<(u32, u64)> {
    use std::ffi::c_void;
    use std::mem::MaybeUninit;

    #[repr(C)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    #[repr(C)]
    struct ByHandleFileInformation {
        file_attributes: u32,
        creation_time: FileTime,
        last_access_time: FileTime,
        last_write_time: FileTime,
        volume_serial_number: u32,
        file_size_high: u32,
        file_size_low: u32,
        number_of_links: u32,
        file_index_high: u32,
        file_index_low: u32,
    }

    unsafe extern "system" {
        fn GetFileInformationByHandle(
            file: *mut c_void,
            information: *mut ByHandleFileInformation,
        ) -> i32;
    }

    let mut information = MaybeUninit::<ByHandleFileInformation>::uninit();
    // SAFETY: the caller owns a valid file or directory handle for the duration of the call, and
    // `information` points to writable storage of the exact Win32 structure layout.
    let success = unsafe { GetFileInformationByHandle(handle, information.as_mut_ptr()) };
    if success == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: a successful GetFileInformationByHandle call initializes every field.
    let information = unsafe { information.assume_init() };
    let file_index =
        (u64::from(information.file_index_high) << 32) | u64::from(information.file_index_low);
    Ok((information.volume_serial_number, file_index))
}

#[cfg(not(any(unix, windows)))]
type DirectoryIdentity = (u64, Option<std::time::SystemTime>);

#[cfg(not(any(unix, windows)))]
type FileIdentity = (u64, Option<std::time::SystemTime>, [u8; 32]);

#[cfg(not(any(unix, windows)))]
fn directory_identity(directory: &Dir) -> io::Result<DirectoryIdentity> {
    let metadata = directory.dir_metadata()?;
    Ok((metadata.len(), metadata.modified().ok()))
}

#[cfg(not(any(unix, windows)))]
fn file_identity(file: &cap_std::fs::File) -> io::Result<FileIdentity> {
    let metadata = file.metadata()?;
    Ok((
        metadata.len(),
        metadata.modified().ok(),
        file_content_digest(file)?,
    ))
}

fn file_content_digest(file: &cap_std::fs::File) -> io::Result<[u8; 32]> {
    file_content_digest_with_limit(file, MAX_GENERATED_OUTPUT_BYTES)
}

fn file_content_digest_with_limit(
    file: &cap_std::fs::File,
    maximum_bytes: u64,
) -> io::Result<[u8; 32]> {
    let mut reader = file.try_clone()?;
    reader.seek(SeekFrom::Start(0))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8 * 1024];
    let mut total_bytes = 0_u64;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        total_bytes = total_bytes.saturating_add(read as u64);
        if total_bytes > maximum_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("file identity input exceeds the {maximum_bytes}-byte limit"),
            ));
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().into())
}

fn validate_request_envelope(request: &Value) -> Result<Option<Value>, Value> {
    let Some(object) = request.as_object() else {
        return Err(invalid_request("JSON-RPC request must be an object"));
    };
    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err(invalid_request("JSON-RPC version must be exactly `2.0`"));
    }
    if !object.get("method").is_some_and(Value::is_string) {
        return Err(invalid_request("JSON-RPC method must be a string"));
    }
    if let Some(id) = object.get("id")
        && !(id.is_string() || id.as_i64().is_some() || id.as_u64().is_some())
    {
        return Err(invalid_request("JSON-RPC id must be a string or integer"));
    }
    if let Some(id) = object.get("id")
        && serde_json::to_vec(id).is_ok_and(|encoded| encoded.len() > MAX_JSON_RPC_ID_BYTES)
    {
        return Err(invalid_request("JSON-RPC id exceeds the 4 KiB limit"));
    }
    if let Some(params) = object.get("params")
        && !(params.is_object() || params.is_array())
    {
        return Err(invalid_request(
            "JSON-RPC params must be an object or array",
        ));
    }
    Ok(object.get("id").cloned())
}

fn invalid_request(message: impl Into<String>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": null,
        "error": { "code": -32600, "message": message.into() }
    })
}

fn write_mcp_response(writer: &mut impl Write, response: &Value) -> Result<(), String> {
    let mut encoded = serde_json::to_vec(response)
        .map_err(|error| format!("Failed to encode MCP response: {error}"))?;
    if encoded.len() > MAX_JSON_RPC_RESPONSE_BYTES {
        let id = response.get("id").cloned().unwrap_or(Value::Null);
        encoded = serde_json::to_vec(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32603,
                "message": "MCP response exceeds the 1 MiB output limit"
            }
        }))
        .map_err(|error| format!("Failed to encode bounded MCP response: {error}"))?;
        if encoded.len() > MAX_JSON_RPC_RESPONSE_BYTES {
            encoded = serde_json::to_vec(&json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": {
                    "code": -32603,
                    "message": "MCP response exceeds the 1 MiB output limit"
                }
            }))
            .map_err(|error| format!("Failed to encode null-ID MCP response: {error}"))?;
        }
    }
    writer
        .write_all(&encoded)
        .and_then(|_| writer.write_all(b"\n"))
        .and_then(|_| writer.flush())
        .map_err(|error| format!("Failed to write MCP stdout: {error}"))
}

fn read_mcp_line<Reader: BufRead>(
    reader: &mut Reader,
) -> io::Result<Option<Result<String, &'static str>>> {
    let mut bytes = Vec::new();
    let mut saw_input = false;
    let mut too_large = false;

    loop {
        let (consumed, reached_newline) = {
            let available = reader.fill_buf()?;
            if available.is_empty() {
                if !saw_input {
                    return Ok(None);
                }
                break;
            }
            saw_input = true;

            let newline = available.iter().position(|byte| *byte == b'\n');
            let content_end = newline.unwrap_or(available.len());
            if !too_large {
                let remaining = MAX_JSON_RPC_LINE_BYTES.saturating_sub(bytes.len());
                let copy_len = content_end.min(remaining);
                bytes.extend_from_slice(&available[..copy_len]);
                too_large = content_end > remaining;
            }

            (
                newline.map_or(available.len(), |index| index + 1),
                newline.is_some(),
            )
        };
        reader.consume(consumed);
        if reached_newline {
            break;
        }
    }

    if too_large {
        return Ok(Some(Err("JSON-RPC request exceeds the 1 MiB line limit")));
    }
    match String::from_utf8(bytes) {
        Ok(line) => Ok(Some(Ok(line))),
        Err(_) => Ok(Some(Err("JSON-RPC request is not valid UTF-8"))),
    }
}

struct ProjectSnapshot {
    directory: tempfile::TempDir,
    git_freshness_available: bool,
}

impl ProjectSnapshot {
    #[cfg(test)]
    fn create(root: &Path) -> Result<Self, String> {
        let source = Dir::open_ambient_dir(root, ambient_authority()).map_err(|error| {
            format!(
                "Cannot open MCP server root capability {}: {error}",
                root.display()
            )
        })?;
        Self::create_from_directory(&source)
    }

    fn create_from_directory(source: &Dir) -> Result<Self, String> {
        Self::create_from_directory_in(source, None)
    }

    #[cfg(test)]
    fn create_in(root: &Path, temp_parent: &Path) -> Result<Self, String> {
        let source = Dir::open_ambient_dir(root, ambient_authority()).map_err(|error| {
            format!(
                "Cannot open MCP server root capability {}: {error}",
                root.display()
            )
        })?;
        Self::create_from_directory_in(&source, Some(temp_parent))
    }

    fn create_from_directory_in(source: &Dir, temp_parent: Option<&Path>) -> Result<Self, String> {
        let mut builder = tempfile::Builder::new();
        builder.prefix("specsync-mcp-");
        let directory = if let Some(parent) = temp_parent {
            builder.tempdir_in(parent)
        } else {
            builder.tempdir()
        }
        .map_err(|error| format!("Cannot create bounded MCP project snapshot: {error}"))?;
        let destination = Dir::open_ambient_dir(directory.path(), ambient_authority())
            .map_err(|error| format!("Cannot open MCP snapshot capability: {error}"))?;
        let mut budget = SnapshotBudget::default();
        copy_snapshot_configuration(source, &destination, directory.path(), &mut budget)?;
        let snapshot_config = load_config(directory.path());
        let configured_exclusions: HashSet<String> =
            snapshot_config.exclude_dirs.iter().cloned().collect();
        let mut configured_inputs: Vec<PathBuf> = snapshot_config
            .source_dirs
            .iter()
            .map(PathBuf::from)
            .collect();
        configured_inputs.push(PathBuf::from(&snapshot_config.specs_dir));
        if let Some(schema_dir) = snapshot_config.schema_dir.as_deref() {
            configured_inputs.push(PathBuf::from(schema_dir));
        }
        configured_inputs.extend(
            snapshot_config
                .modules
                .values()
                .flat_map(|module| module.files.iter().map(PathBuf::from)),
        );
        collect_snapshot_manifest_inputs(source, &mut configured_inputs, &mut budget)?;
        copy_preloaded_snapshot_files(&destination, &mut budget)?;
        if configured_inputs.iter().any(|path| {
            path.components().any(|component| {
                component
                    .as_os_str()
                    .to_str()
                    .is_some_and(|name| name.eq_ignore_ascii_case(".git"))
            })
        }) {
            return Err(
                "MCP project configuration must not use Git metadata as a project input"
                    .to_string(),
            );
        }
        copy_snapshot_directory(
            source,
            &destination,
            Path::new("."),
            &configured_exclusions,
            &configured_inputs,
            &mut budget,
        )?;
        Ok(Self {
            directory,
            git_freshness_available: false,
        })
    }

    fn root(&self) -> &Path {
        self.directory.path()
    }

    fn git_freshness_available(&self) -> bool {
        self.git_freshness_available
    }
}

/// Pin git discovery to `parent(snapshot)` for the duration of `f`.
///
/// Snapshots copy the project without `.git`. If the tempfile sits inside a
/// host worktree (`TMPDIR` pointing at the repo), default walk-up would find
/// that host. Empirically ceiling=`parent(snapshot)` stops the walk; ceiling
/// equal to the snapshot path itself does not. CLI callers never enter here.
fn with_isolated_snapshot_git<T>(snapshot: &ProjectSnapshot, f: impl FnOnce() -> T) -> T {
    match snapshot.root().parent() {
        Some(ceiling) => crate::git_utils::with_discovery_ceiling(ceiling, f),
        None => f(),
    }
}

fn collect_snapshot_manifest_inputs(
    source: &Dir,
    configured_inputs: &mut Vec<PathBuf>,
    budget: &mut SnapshotBudget,
) -> Result<(), String> {
    let mut configured_input_set: HashSet<PathBuf> = configured_inputs
        .iter()
        .map(|path| normalize_snapshot_input(path))
        .collect();
    let gradle_modules = preload_gradle_manifests(source, budget)?
        .map(|content| parse_gradle_settings(&content))
        .transpose()?
        .unwrap_or_default();
    let mut cargo_manifests = vec![PathBuf::from("Cargo.toml")];
    let mut seen_cargo_manifests = HashSet::new();
    let mut seen_cargo_members = HashSet::new();
    while let Some(manifest) = cargo_manifests.pop() {
        if !seen_cargo_manifests.insert(manifest.clone()) {
            continue;
        }
        if seen_cargo_manifests.len() > MAX_MANIFEST_PREFLIGHTS {
            return Err(format!(
                "MCP snapshot manifest discovery exceeds {MAX_MANIFEST_PREFLIGHTS} manifests"
            ));
        }
        let Some(content) = read_capability_text_if_exists(source, &manifest, budget)? else {
            continue;
        };
        let manifest_dir = manifest.parent().unwrap_or_else(|| Path::new(""));
        let cargo = parse_cargo_snapshot_manifest(&content, &manifest)?;
        for member in cargo.workspace_members {
            budget.charge_entries(1, "MCP snapshot manifest discovery")?;
            let member = snapshot_manifest_input(manifest_dir, &member, "Cargo workspace member")?;
            if seen_cargo_members.insert(member.clone()) {
                push_unique_snapshot_input(
                    configured_inputs,
                    &mut configured_input_set,
                    member.clone(),
                );
                cargo_manifests.push(member.join("Cargo.toml"));
            }
        }
        for path in cargo.paths {
            if path.is_empty() {
                continue;
            }
            let path = snapshot_manifest_input(manifest_dir, &path, "Cargo target path")?;
            push_unique_snapshot_input(
                configured_inputs,
                &mut configured_input_set,
                path.parent().unwrap_or(&path).to_path_buf(),
            );
        }
    }

    if let Some(content) =
        read_capability_text_if_exists(source, Path::new("package.json"), budget)?
    {
        let package: Value = serde_json::from_str(&content)
            .map_err(|error| format!("Cannot parse MCP package.json as JSON: {error}"))?;
        let declarations = package_workspace_declarations(&package, "MCP snapshot package.json")?;
        budget.charge_entries(declarations.len(), "MCP snapshot manifest discovery")?;
        let patterns = declarations
            .iter()
            .map(|declaration| {
                declaration.as_str().ok_or_else(|| {
                    "MCP snapshot package.json workspace entries must be strings".to_string()
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut seen_patterns = HashSet::new();
        let mut seen_bases = HashSet::new();
        for pattern in patterns {
            if !seen_patterns.insert(pattern.to_string()) {
                continue;
            }
            let base = pattern.trim_end_matches("/*").trim_end_matches("/**");
            let base = snapshot_manifest_input(
                Path::new(""),
                if base.is_empty() { "." } else { base },
                "package workspace base",
            )?;
            if seen_bases.insert(base.clone()) {
                push_unique_snapshot_input(configured_inputs, &mut configured_input_set, base);
            }
        }
        let main = package.get("main").and_then(Value::as_str).unwrap_or("");
        if main.starts_with("./") {
            let main = snapshot_manifest_input(Path::new(""), main, "package main path")?;
            push_unique_snapshot_input(
                configured_inputs,
                &mut configured_input_set,
                main.parent().unwrap_or(&main).to_path_buf(),
            );
        }
        if source
            .try_exists("src")
            .map_err(|error| format!("Cannot inspect MCP package source directory: {error}"))?
        {
            push_unique_snapshot_input(
                configured_inputs,
                &mut configured_input_set,
                PathBuf::from("src"),
            );
        } else if source
            .try_exists("lib")
            .map_err(|error| format!("Cannot inspect MCP package library directory: {error}"))?
        {
            push_unique_snapshot_input(
                configured_inputs,
                &mut configured_input_set,
                PathBuf::from("lib"),
            );
        }
    }

    if let Some(content) =
        read_capability_text_if_exists(source, Path::new("Package.swift"), budget)?
    {
        let mut search = content.as_str();
        while let Some(path_start) = search.find("path:") {
            let rest = &search[path_start + "path:".len()..];
            let Some(quote_start) = rest.find('"') else {
                break;
            };
            let quoted = &rest[quote_start + 1..];
            let Some(quote_end) = quoted.find('"') else {
                break;
            };
            configured_inputs.push(snapshot_manifest_input(
                Path::new(""),
                &quoted[..quote_end],
                "Swift target path",
            )?);
            search = &quoted[quote_end + 1..];
        }
        if !configured_inputs
            .iter()
            .any(|path| path == Path::new("Sources"))
            && source
                .try_exists("Sources")
                .map_err(|error| format!("Cannot inspect MCP Swift Sources directory: {error}"))?
        {
            configured_inputs.push(PathBuf::from("Sources"));
        }
    }

    for module in gradle_modules {
        configured_inputs.push(snapshot_manifest_input(
            Path::new(""),
            &module.path,
            "Gradle module",
        )?);
    }

    if read_capability_text_if_exists(source, Path::new("pubspec.yaml"), budget)?.is_some() {
        configured_inputs.push(PathBuf::from("lib"));
    }

    if read_capability_text_if_exists(source, Path::new("go.mod"), budget)?.is_some() {
        let mut found = false;
        for path in ["cmd", "internal", "pkg", "api"] {
            if source.try_exists(path).map_err(|error| {
                format!("Cannot inspect MCP Go source directory {path}: {error}")
            })? {
                configured_inputs.push(PathBuf::from(path));
                found = true;
            }
        }
        if !found {
            configured_inputs.push(PathBuf::new());
        }
    }

    if let Some(content) =
        read_capability_text_if_exists(source, Path::new("pyproject.toml"), budget)?
    {
        if source
            .try_exists("src")
            .map_err(|error| format!("Cannot inspect MCP Python src directory: {error}"))?
        {
            configured_inputs.push(PathBuf::from("src"));
        } else {
            let name = extract_manifest_toml_value(&content, "name", "[project]")
                .or_else(|| extract_manifest_toml_value(&content, "name", "[tool.poetry]"));
            match name {
                Some(name) => {
                    let package =
                        snapshot_manifest_input(Path::new(""), &name, "Python package path")?;
                    if source.try_exists(&package).map_err(|error| {
                        format!(
                            "Cannot inspect MCP Python package directory {}: {error}",
                            package.display()
                        )
                    })? {
                        configured_inputs.push(package);
                    } else {
                        configured_inputs.push(PathBuf::new());
                    }
                }
                None => configured_inputs.push(PathBuf::new()),
            }
        }
    }

    if configured_inputs.len() > MAX_CONFIGURED_PATHS {
        return Err(format!(
            "MCP snapshot configuration exceeds {MAX_CONFIGURED_PATHS} input paths"
        ));
    }
    Ok(())
}

fn package_workspace_declarations<'a>(
    package: &'a Value,
    label: &str,
) -> Result<&'a [Value], String> {
    let object = package
        .as_object()
        .ok_or_else(|| format!("{label} root must be a JSON object"))?;
    match object.get("workspaces") {
        None => Ok(&[]),
        Some(Value::Array(values)) => Ok(values),
        Some(Value::Object(workspaces)) => match workspaces.get("packages") {
            None => Err(format!(
                "{label} `workspaces` object must contain a `packages` array"
            )),
            Some(Value::Array(values)) => Ok(values),
            Some(_) => Err(format!("{label} `workspaces.packages` must be an array")),
        },
        Some(_) => Err(format!(
            "{label} `workspaces` must be an array or an object containing `packages`"
        )),
    }
}

fn push_unique_snapshot_input(
    configured_inputs: &mut Vec<PathBuf>,
    configured_input_set: &mut HashSet<PathBuf>,
    input: PathBuf,
) {
    let normalized = normalize_snapshot_input(&input);
    if configured_input_set.insert(normalized.clone()) {
        configured_inputs.push(normalized);
    }
}

fn preload_gradle_manifests(
    source: &Dir,
    budget: &mut SnapshotBudget,
) -> Result<Option<String>, String> {
    let mut selected_settings = None;
    for name in [
        "build.gradle.kts",
        "build.gradle",
        "settings.gradle.kts",
        "settings.gradle",
    ] {
        let content = read_capability_text_if_exists_with_limit(
            source,
            Path::new(name),
            budget,
            MAX_GRADLE_MANIFEST_BYTES,
        )?;
        if selected_settings.is_none() && name.starts_with("settings.") {
            selected_settings = content;
        }
    }
    Ok(selected_settings)
}

struct CargoSnapshotManifest {
    workspace_members: Vec<String>,
    paths: Vec<String>,
}

fn parse_cargo_snapshot_manifest(
    content: &str,
    manifest: &Path,
) -> Result<CargoSnapshotManifest, String> {
    let document = toml::from_str::<toml::Table>(content).map_err(|error| {
        format!(
            "Cannot parse MCP Cargo workspace manifest {} as TOML: {error}",
            manifest.display()
        )
    })?;
    let workspace_members = match document.get("workspace") {
        Some(toml::Value::Table(workspace)) => match workspace.get("members") {
            Some(toml::Value::Array(members)) => members
                .iter()
                .map(|member| {
                    member.as_str().map(str::to_string).ok_or_else(|| {
                        format!(
                            "MCP Cargo workspace manifest {} has a non-string workspace member",
                            manifest.display()
                        )
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
            Some(_) => {
                return Err(format!(
                    "MCP Cargo workspace manifest {} must define `workspace.members` as a string array",
                    manifest.display()
                ));
            }
            None => Vec::new(),
        },
        Some(_) => {
            return Err(format!(
                "MCP Cargo workspace manifest {} must define `workspace` as a TOML table",
                manifest.display()
            ));
        }
        None => Vec::new(),
    };
    let mut paths = Vec::new();
    collect_cargo_manifest_paths(&document, manifest, &mut paths)?;
    Ok(CargoSnapshotManifest {
        workspace_members,
        paths,
    })
}

fn collect_cargo_manifest_paths(
    document: &toml::Table,
    manifest: &Path,
    paths: &mut Vec<String>,
) -> Result<(), String> {
    for target in ["lib", "bin", "example", "test", "bench"] {
        if let Some(value) = document.get(target) {
            collect_cargo_path_fields(value, manifest, paths)?;
        }
    }

    for dependencies in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(value) = document.get(dependencies) {
            collect_cargo_dependency_paths(value, manifest, paths)?;
        }
    }

    if let Some(workspace) = document.get("workspace").and_then(toml::Value::as_table)
        && let Some(dependencies) = workspace.get("dependencies")
    {
        collect_cargo_dependency_paths(dependencies, manifest, paths)?;
    }

    if let Some(targets) = document.get("target").and_then(toml::Value::as_table) {
        for target in targets.values().filter_map(toml::Value::as_table) {
            for dependencies in ["dependencies", "dev-dependencies", "build-dependencies"] {
                if let Some(value) = target.get(dependencies) {
                    collect_cargo_dependency_paths(value, manifest, paths)?;
                }
            }
        }
    }

    if let Some(patches) = document.get("patch").and_then(toml::Value::as_table) {
        for registry in patches.values() {
            collect_cargo_dependency_paths(registry, manifest, paths)?;
        }
    }

    if let Some(replacements) = document.get("replace") {
        collect_cargo_dependency_paths(replacements, manifest, paths)?;
    }

    Ok(())
}

fn collect_cargo_dependency_paths(
    dependencies: &toml::Value,
    manifest: &Path,
    paths: &mut Vec<String>,
) -> Result<(), String> {
    let Some(dependencies) = dependencies.as_table() else {
        return Ok(());
    };
    for dependency in dependencies.values() {
        if dependency.is_table() {
            collect_cargo_path_fields(dependency, manifest, paths)?;
        }
    }
    Ok(())
}

fn collect_cargo_path_fields(
    value: &toml::Value,
    manifest: &Path,
    paths: &mut Vec<String>,
) -> Result<(), String> {
    match value {
        toml::Value::Table(table) => {
            if let Some(value) = table.get("path") {
                let path = value.as_str().ok_or_else(|| {
                    format!(
                        "MCP Cargo workspace manifest {} has a non-string `path` value",
                        manifest.display()
                    )
                })?;
                paths.push(path.to_string());
            }
        }
        toml::Value::Array(values) => {
            for value in values {
                collect_cargo_path_fields(value, manifest, paths)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn read_capability_text_if_exists(
    source: &Dir,
    relative: &Path,
    budget: &mut SnapshotBudget,
) -> Result<Option<String>, String> {
    read_capability_text_if_exists_with_limit_and_hook(
        source,
        relative,
        budget,
        MAX_PROJECT_FILE_BYTES,
        || {},
    )
}

#[cfg(test)]
fn read_capability_text_if_exists_with_hook(
    source: &Dir,
    relative: &Path,
    budget: &mut SnapshotBudget,
    after_retained_open: impl FnOnce(),
) -> Result<Option<String>, String> {
    read_capability_text_if_exists_with_limit_and_hook(
        source,
        relative,
        budget,
        MAX_PROJECT_FILE_BYTES,
        after_retained_open,
    )
}

fn read_capability_text_if_exists_with_limit(
    source: &Dir,
    relative: &Path,
    budget: &mut SnapshotBudget,
    max_bytes: u64,
) -> Result<Option<String>, String> {
    read_capability_text_if_exists_with_limit_and_hook(source, relative, budget, max_bytes, || {})
}

fn read_capability_text_if_exists_with_limit_and_hook(
    source: &Dir,
    relative: &Path,
    budget: &mut SnapshotBudget,
    max_bytes: u64,
    after_retained_open: impl FnOnce(),
) -> Result<Option<String>, String> {
    let Some(bytes) = read_retained_snapshot_file_if_exists_with_hook(
        source,
        relative,
        relative,
        "MCP snapshot manifest",
        max_bytes,
        || {
            after_retained_open();
            Ok(())
        },
    )?
    else {
        return Ok(None);
    };
    let normalized = normalize_snapshot_input(relative);
    budget.charge_file(&normalized, bytes.len() as u64)?;
    let content = String::from_utf8(bytes.clone()).map_err(|_| {
        format!(
            "MCP snapshot manifest is not valid UTF-8: {}",
            relative.display()
        )
    })?;
    budget.preloaded_files.insert(normalized, bytes);
    Ok(Some(content))
}

fn read_retained_snapshot_file_if_exists_with_hook(
    source: &Dir,
    relative: &Path,
    diagnostic_relative: &Path,
    label: &str,
    max_bytes: u64,
    after_retained_open: impl FnOnce() -> Result<(), String>,
) -> Result<Option<Vec<u8>>, String> {
    let before = match source.symlink_metadata(relative) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "Cannot inspect {label} {}: {error}",
                diagnostic_relative.display()
            ));
        }
    };
    if snapshot_metadata_is_link(&before) || !before.is_file() {
        return Err(format!(
            "{label} must be a regular file and must not be a symlink or reparse point: {}",
            diagnostic_relative.display()
        ));
    }
    let expected_identity = snapshot_metadata_identity(&before).map_err(|error| {
        format!(
            "Cannot identify {label} {}: {error}",
            diagnostic_relative.display()
        )
    })?;

    let options = read_only_nofollow_options();
    let mut file = source.open_with(relative, &options).map_err(|error| {
        format!(
            "Cannot open {label} {} as a no-follow, non-blocking regular file: {error}",
            diagnostic_relative.display()
        )
    })?;
    let opened_metadata = file.metadata().map_err(|error| {
        format!(
            "Cannot inspect opened {label} {}: {error}",
            diagnostic_relative.display()
        )
    })?;
    if snapshot_metadata_is_link(&opened_metadata) || !opened_metadata.is_file() {
        return Err(format!(
            "{label} must be a regular file and must not be a symlink or reparse point: {}",
            diagnostic_relative.display()
        ));
    }
    let opened_identity = snapshot_file_identity(&file).map_err(|error| {
        format!(
            "Cannot identify opened {label} {}: {error}",
            diagnostic_relative.display()
        )
    })?;
    if opened_identity != expected_identity {
        return Err(format!(
            "{label} changed during inspection: {}",
            diagnostic_relative.display()
        ));
    }

    after_retained_open()?;
    let after_open = source.symlink_metadata(relative).map_err(|error| {
        format!(
            "Cannot re-inspect {label} {}: {error}",
            diagnostic_relative.display()
        )
    })?;
    if snapshot_metadata_is_link(&after_open)
        || !after_open.is_file()
        || snapshot_metadata_identity(&after_open).ok() != Some(expected_identity)
        || snapshot_file_identity(&file).ok() != Some(opened_identity)
    {
        return Err(format!(
            "{label} changed during inspection: {}",
            diagnostic_relative.display()
        ));
    }

    let mut bytes = Vec::with_capacity(before.len().min(max_bytes) as usize);
    Read::by_ref(&mut file)
        .take(max_bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            format!(
                "Cannot read {label} {}: {error}",
                diagnostic_relative.display()
            )
        })?;
    if bytes.len() as u64 > max_bytes {
        return Err(format!(
            "{label} exceeds the {} MiB per-file limit: {}",
            max_bytes / (1024 * 1024),
            diagnostic_relative.display()
        ));
    }

    let after_read = source.symlink_metadata(relative).map_err(|error| {
        format!(
            "Cannot re-inspect {label} {} after reading: {error}",
            diagnostic_relative.display()
        )
    })?;
    if snapshot_metadata_is_link(&after_read)
        || !after_read.is_file()
        || snapshot_metadata_identity(&after_read).ok() != Some(expected_identity)
        || snapshot_file_identity(&file).ok() != Some(opened_identity)
    {
        return Err(format!(
            "{label} changed while it was being read: {}",
            diagnostic_relative.display()
        ));
    }
    Ok(Some(bytes))
}

fn snapshot_manifest_input(base: &Path, configured: &str, label: &str) -> Result<PathBuf, String> {
    let bytes = configured.as_bytes();
    let has_windows_drive_prefix =
        bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    if has_windows_drive_prefix || configured.starts_with('\\') || configured.starts_with('/') {
        return Err(format!(
            "MCP {label} must use a safe project-relative path: {configured}"
        ));
    }

    let portable = configured.replace('\\', "/");
    let path = Path::new(&portable);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::RootDir | Component::Prefix(_)))
    {
        return Err(format!(
            "MCP {label} must use a safe project-relative path: {configured}"
        ));
    }

    let mut normalized = normalize_snapshot_input(base);
    for component in path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(format!(
                        "MCP {label} escapes the configured server root: {configured}"
                    ));
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!(
                    "MCP {label} must use a safe project-relative path: {configured}"
                ));
            }
        }
    }
    Ok(normalized)
}

fn normalize_snapshot_input(path: &Path) -> PathBuf {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part),
            Component::CurDir => None,
            _ => None,
        })
        .collect()
}

fn snapshot_input_overlaps(input: &Path, child: &Path) -> bool {
    let input = normalize_snapshot_input(input);
    input.as_os_str().is_empty() || input.starts_with(child) || child.starts_with(input)
}

fn copy_snapshot_configuration(
    source: &Dir,
    destination: &Dir,
    destination_root: &Path,
    budget: &mut SnapshotBudget,
) -> Result<(), String> {
    for relative in [
        ".specsync/config.toml",
        ".specsync/config.json",
        ".specsync.toml",
        "specsync.json",
    ] {
        let path = Path::new(relative);
        let Some(bytes) = read_snapshot_configuration(source, path)? else {
            continue;
        };
        budget.charge_file(path, bytes.len() as u64)?;
        validate_snapshot_configuration(destination_root, relative, &bytes)?;
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            destination.create_dir_all(parent).map_err(|error| {
                format!("Cannot create MCP snapshot configuration directory: {error}")
            })?;
        }
        destination.write(path, bytes).map_err(|error| {
            format!("Cannot copy MCP configuration {relative} into the snapshot: {error}")
        })?;
        budget.copied_paths.insert(path.to_path_buf());
        break;
    }
    Ok(())
}

#[cfg(unix)]
type SnapshotEntryIdentity = (u64, u64);

#[cfg(windows)]
type SnapshotEntryIdentity = (u32, u64);

#[cfg(not(any(unix, windows)))]
type SnapshotEntryIdentity = (u64, Option<std::time::SystemTime>);

#[cfg(unix)]
fn snapshot_metadata_identity(metadata: &Metadata) -> io::Result<SnapshotEntryIdentity> {
    use cap_std::fs::MetadataExt;

    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(windows)]
fn snapshot_metadata_identity(metadata: &Metadata) -> io::Result<SnapshotEntryIdentity> {
    use cap_primitives::fs::_WindowsByHandle;

    let volume = metadata.volume_serial_number().ok_or_else(|| {
        io::Error::other("Windows metadata does not expose a volume serial number")
    })?;
    let index = metadata
        .file_index()
        .ok_or_else(|| io::Error::other("Windows metadata does not expose a file index"))?;
    Ok((volume, index))
}

#[cfg(not(any(unix, windows)))]
fn snapshot_metadata_identity(metadata: &Metadata) -> io::Result<SnapshotEntryIdentity> {
    Ok((metadata.len(), metadata.modified().ok()))
}

#[cfg(unix)]
fn snapshot_file_identity(file: &cap_std::fs::File) -> io::Result<SnapshotEntryIdentity> {
    snapshot_metadata_identity(&file.metadata()?)
}

#[cfg(windows)]
fn snapshot_file_identity(file: &cap_std::fs::File) -> io::Result<SnapshotEntryIdentity> {
    use std::os::windows::io::AsRawHandle;

    windows_handle_identity(file.as_raw_handle().cast())
}

#[cfg(not(any(unix, windows)))]
fn snapshot_file_identity(file: &cap_std::fs::File) -> io::Result<SnapshotEntryIdentity> {
    snapshot_metadata_identity(&file.metadata()?)
}

#[cfg(windows)]
fn snapshot_metadata_is_link(metadata: &Metadata) -> bool {
    use cap_std::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn snapshot_metadata_is_link(metadata: &Metadata) -> bool {
    metadata.file_type().is_symlink()
}

fn snapshot_configuration_identities_match<Identity: PartialEq>(
    expected: &Identity,
    observed: &[Option<Identity>],
) -> bool {
    observed
        .iter()
        .all(|identity| identity.as_ref() == Some(expected))
}

#[cfg(not(windows))]
fn open_snapshot_configuration_directory(
    parent: &Dir,
    name: &OsStr,
    relative: &Path,
) -> Result<Option<(Dir, SnapshotEntryIdentity)>, String> {
    open_snapshot_configuration_directory_with_hook(parent, name, relative, || {})
}

#[cfg(not(windows))]
fn open_snapshot_configuration_directory_with_hook(
    parent: &Dir,
    name: &OsStr,
    relative: &Path,
    after_pre_open_identity: impl FnOnce(),
) -> Result<Option<(Dir, SnapshotEntryIdentity)>, String> {
    let before = match parent.symlink_metadata(name) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "Cannot inspect MCP configuration directory {}: {error}",
                relative.display()
            ));
        }
    };
    if snapshot_metadata_is_link(&before) || !before.is_dir() {
        return Err(format!(
            "Selected MCP configuration {} must not traverse a symlink and must use regular directories",
            relative.display()
        ));
    }
    let expected = snapshot_metadata_identity(&before).map_err(|error| {
        format!(
            "Cannot identify MCP configuration directory {}: {error}",
            relative.display()
        )
    })?;
    after_pre_open_identity();
    let directory = parent.open_dir(name).map_err(|error| {
        format!(
            "Cannot open MCP configuration directory {}: {error}",
            relative.display()
        )
    })?;
    let opened =
        snapshot_metadata_identity(&directory.dir_metadata().map_err(|error| {
            format!("Cannot inspect opened MCP configuration directory: {error}")
        })?)
        .map_err(|error| format!("Cannot identify opened MCP configuration directory: {error}"))?;
    let after = parent.symlink_metadata(name).map_err(|error| {
        format!(
            "Cannot re-inspect MCP configuration directory {}: {error}",
            relative.display()
        )
    })?;
    if snapshot_metadata_is_link(&after)
        || !after.is_dir()
        || !snapshot_configuration_identities_match(
            &expected,
            &[Some(opened), snapshot_metadata_identity(&after).ok()],
        )
    {
        return Err(format!(
            "Selected MCP configuration directory changed during inspection: {}",
            relative.display()
        ));
    }
    Ok(Some((directory, expected)))
}

#[cfg(windows)]
fn open_snapshot_configuration_directory(
    parent: &Dir,
    name: &OsStr,
    relative: &Path,
) -> Result<Option<(Dir, SnapshotEntryIdentity)>, String> {
    open_snapshot_configuration_directory_with_hook(parent, name, relative, || {})
}

#[cfg(windows)]
fn open_snapshot_configuration_directory_with_hook(
    parent: &Dir,
    name: &OsStr,
    relative: &Path,
    after_pre_open_identity: impl FnOnce(),
) -> Result<Option<(Dir, SnapshotEntryIdentity)>, String> {
    let before = match parent.symlink_metadata(name) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "Cannot inspect MCP configuration directory {}: {error}",
                relative.display()
            ));
        }
    };
    if snapshot_metadata_is_link(&before) || !before.is_dir() {
        return Err(format!(
            "Selected MCP configuration {} must not traverse a reparse point and must use regular directories",
            relative.display()
        ));
    }
    let expected = snapshot_metadata_identity(&before).map_err(|error| {
        format!(
            "Cannot identify MCP configuration directory {}: {error}",
            relative.display()
        )
    })?;
    after_pre_open_identity();
    let directory = parent.open_dir(name).map_err(|error| {
        format!(
            "Cannot open MCP configuration directory {}: {error}",
            relative.display()
        )
    })?;
    let opened = directory_identity(&directory)
        .map_err(|error| format!("Cannot identify opened MCP configuration directory: {error}"))?;
    let observed = parent.open_dir(name).map_err(|error| {
        format!(
            "Cannot re-open MCP configuration directory {}: {error}",
            relative.display()
        )
    })?;
    let after = parent.symlink_metadata(name).map_err(|error| {
        format!(
            "Cannot re-inspect MCP configuration directory {}: {error}",
            relative.display()
        )
    })?;
    if snapshot_metadata_is_link(&after)
        || !after.is_dir()
        || !snapshot_configuration_identities_match(
            &expected,
            &[
                Some(opened),
                snapshot_metadata_identity(&after).ok(),
                directory_identity(&observed).ok(),
            ],
        )
    {
        return Err(format!(
            "Selected MCP configuration directory changed during inspection: {}",
            relative.display()
        ));
    }
    Ok(Some((directory, expected)))
}

fn revalidate_snapshot_configuration_parents(
    source: &Dir,
    parents: &[(PathBuf, SnapshotEntryIdentity)],
) -> Result<(), String> {
    let mut parent = source
        .try_clone()
        .map_err(|error| format!("Cannot clone MCP server root capability: {error}"))?;
    for (relative, expected) in parents {
        let Some(name) = relative.file_name() else {
            return Err(format!(
                "Selected MCP configuration parent path is invalid: {}",
                relative.display()
            ));
        };
        let Some((directory, observed)) =
            open_snapshot_configuration_directory(&parent, name, relative)?
        else {
            return Err(format!(
                "Selected MCP configuration directory changed while the configuration was being read: {}",
                relative.display()
            ));
        };
        if &observed != expected {
            return Err(format!(
                "Selected MCP configuration directory changed while the configuration was being read: {}",
                relative.display()
            ));
        }
        parent = directory;
    }
    Ok(())
}

fn read_snapshot_configuration(source: &Dir, relative: &Path) -> Result<Option<Vec<u8>>, String> {
    read_snapshot_configuration_with_hook(source, relative, || {})
}

fn read_snapshot_configuration_with_hook(
    source: &Dir,
    relative: &Path,
    after_retained_open: impl FnOnce(),
) -> Result<Option<Vec<u8>>, String> {
    let components = relative.components().collect::<Vec<_>>();
    let Some((file_name, parent_components)) = components.split_last() else {
        return Err("Selected MCP configuration path is empty".to_string());
    };
    let Component::Normal(file_name) = file_name else {
        return Err(format!(
            "Selected MCP configuration path is not project-relative: {}",
            relative.display()
        ));
    };

    let mut parent = source
        .try_clone()
        .map_err(|error| format!("Cannot clone MCP server root capability: {error}"))?;
    let mut traversed = PathBuf::new();
    let mut parents = Vec::new();
    for component in parent_components {
        let Component::Normal(name) = component else {
            return Err(format!(
                "Selected MCP configuration path is not project-relative: {}",
                relative.display()
            ));
        };
        traversed.push(name);
        parent = match open_snapshot_configuration_directory(&parent, name, &traversed)? {
            Some((directory, identity)) => {
                parents.push((traversed.clone(), identity));
                directory
            }
            None => return Ok(None),
        };
    }

    let bytes = read_retained_snapshot_file_if_exists_with_hook(
        &parent,
        Path::new(file_name),
        relative,
        "Selected MCP configuration",
        MAX_PROJECT_FILE_BYTES,
        || {
            after_retained_open();
            Ok(())
        },
    )?;
    revalidate_snapshot_configuration_parents(source, &parents)?;
    Ok(bytes)
}

fn validate_snapshot_configuration(
    destination_root: &Path,
    relative: &str,
    bytes: &[u8],
) -> Result<(), String> {
    let content = std::str::from_utf8(bytes)
        .map_err(|_| format!("Selected MCP configuration {relative} is not valid UTF-8"))?;
    let content = content.trim_start_matches('\u{feff}');
    if relative.ends_with(".toml") {
        let value = toml::from_str::<toml::Value>(content).map_err(|error| {
            format!("Selected MCP configuration {relative} is malformed TOML: {error}")
        })?;
        validate_toml_snapshot_path_selectors(relative, &value)?;
    } else {
        let value = serde_json::from_str::<Value>(content).map_err(|error| {
            format!("Selected MCP configuration {relative} is malformed JSON: {error}")
        })?;
        if !value.is_object() {
            return Err(format!(
                "Selected MCP configuration {relative} is malformed JSON: root must be an object"
            ));
        }
        validate_json_snapshot_path_selectors(relative, &value)?;
    }
    parse_config_content_checked(&destination_root.join(relative), content, destination_root)
        .map(|_| ())
        .map_err(|error| {
            let format = if relative.ends_with(".toml") {
                "TOML"
            } else {
                "JSON"
            };
            format!("Selected MCP configuration {relative} is malformed {format}: {error}")
        })
}

fn validate_json_snapshot_path_selectors(relative: &str, config: &Value) -> Result<(), String> {
    if config
        .get("specsDir")
        .is_some_and(|value| !value.is_string())
    {
        return Err(format!(
            "Selected MCP configuration {relative} path selector `specsDir` must be a string"
        ));
    }
    if config.get("sourceDirs").is_some_and(|value| {
        !value
            .as_array()
            .is_some_and(|entries| entries.iter().all(Value::is_string))
    }) {
        return Err(format!(
            "Selected MCP configuration {relative} path selector `sourceDirs` must be an array of strings"
        ));
    }
    Ok(())
}

fn validate_toml_snapshot_path_selectors(
    relative: &str,
    config: &toml::Value,
) -> Result<(), String> {
    if config.get("specs_dir").is_some_and(|value| !value.is_str()) {
        return Err(format!(
            "Selected MCP configuration {relative} path selector `specs_dir` must be a string"
        ));
    }
    if config.get("source_dirs").is_some_and(|value| {
        !value
            .as_array()
            .is_some_and(|entries| entries.iter().all(toml::Value::is_str))
    }) {
        return Err(format!(
            "Selected MCP configuration {relative} path selector `source_dirs` must be an array of strings"
        ));
    }
    Ok(())
}

struct SnapshotBudget {
    bytes: u64,
    max_bytes: u64,
    entries: usize,
    charged_paths: HashSet<PathBuf>,
    copied_paths: HashSet<PathBuf>,
    preloaded_files: HashMap<PathBuf, Vec<u8>>,
}

impl Default for SnapshotBudget {
    fn default() -> Self {
        Self {
            bytes: 0,
            max_bytes: MAX_PROJECT_INPUT_BYTES,
            entries: 0,
            charged_paths: HashSet::new(),
            copied_paths: HashSet::new(),
            preloaded_files: HashMap::new(),
        }
    }
}

fn copy_preloaded_snapshot_files(
    destination: &Dir,
    budget: &mut SnapshotBudget,
) -> Result<(), String> {
    let mut preloaded: Vec<(PathBuf, Vec<u8>)> = budget.preloaded_files.drain().collect();
    preloaded.sort_by(|left, right| left.0.cmp(&right.0));
    for (relative, bytes) in preloaded {
        if let Some(parent) = relative.parent()
            && !parent.as_os_str().is_empty()
        {
            destination.create_dir_all(parent).map_err(|error| {
                format!(
                    "Cannot create MCP snapshot manifest directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        destination.write(&relative, bytes).map_err(|error| {
            format!(
                "Cannot copy preflighted MCP snapshot manifest {}: {error}",
                relative.display()
            )
        })?;
        budget.copied_paths.insert(relative);
    }
    Ok(())
}

impl SnapshotBudget {
    fn charge_entries(&mut self, count: usize, label: &str) -> Result<(), String> {
        self.entries = self.entries.saturating_add(count);
        if self.entries > MAX_CONFINEMENT_ENTRIES {
            return Err(format!("{label} exceeds {MAX_CONFINEMENT_ENTRIES} entries"));
        }
        Ok(())
    }

    fn charge_file(&mut self, relative: &Path, bytes: u64) -> Result<(), String> {
        if !self.charged_paths.insert(relative.to_path_buf()) {
            return Ok(());
        }
        self.bytes = self.bytes.saturating_add(bytes);
        if self.bytes > self.max_bytes {
            let limit = if self.max_bytes.is_multiple_of(1024 * 1024) {
                format!("{} MiB", self.max_bytes / (1024 * 1024))
            } else {
                format!("{} bytes", self.max_bytes)
            };
            return Err(format!(
                "MCP project inputs exceed the {limit} cumulative limit"
            ));
        }
        Ok(())
    }
}

fn read_snapshot_project_file_from_directory(
    source: &Dir,
    relative: &Path,
    diagnostic_relative: &Path,
) -> Result<Vec<u8>, String> {
    read_snapshot_project_file_from_directory_with_result_hook(
        source,
        relative,
        diagnostic_relative,
        || {
            #[cfg(debug_assertions)]
            wait_for_mcp_snapshot_file_test_barrier(diagnostic_relative)?;
            Ok(())
        },
    )
}

#[cfg(all(test, unix))]
fn read_snapshot_project_file_with_hook(
    source: &Dir,
    relative: &Path,
    after_retained_open: impl FnOnce(),
) -> Result<Vec<u8>, String> {
    read_snapshot_project_file_with_result_hook(source, relative, || {
        after_retained_open();
        Ok(())
    })
}

#[cfg(all(test, unix))]
fn read_snapshot_project_file_with_result_hook(
    source: &Dir,
    relative: &Path,
    after_retained_open: impl FnOnce() -> Result<(), String>,
) -> Result<Vec<u8>, String> {
    read_snapshot_project_file_from_directory_with_result_hook(
        source,
        relative,
        relative,
        after_retained_open,
    )
}

fn read_snapshot_project_file_from_directory_with_result_hook(
    source: &Dir,
    relative: &Path,
    diagnostic_relative: &Path,
    after_retained_open: impl FnOnce() -> Result<(), String>,
) -> Result<Vec<u8>, String> {
    read_retained_snapshot_file_if_exists_with_hook(
        source,
        relative,
        diagnostic_relative,
        "MCP project input",
        MAX_PROJECT_FILE_BYTES,
        after_retained_open,
    )?
    .ok_or_else(|| {
        format!(
            "MCP project input changed during inspection: {}",
            diagnostic_relative.display()
        )
    })
}

struct RetainedSnapshotDirectory {
    directory: Dir,
    identity: SnapshotEntryIdentity,
}

struct SnapshotDirectoryEntry {
    name: std::ffi::OsString,
    child: PathBuf,
    directory_identity: Option<SnapshotEntryIdentity>,
}

fn copy_snapshot_directory(
    source: &Dir,
    destination: &Dir,
    relative: &Path,
    configured_exclusions: &HashSet<String>,
    configured_inputs: &[PathBuf],
    budget: &mut SnapshotBudget,
) -> Result<(), String> {
    copy_snapshot_directory_with_enumeration_hook(
        source,
        destination,
        relative,
        configured_exclusions,
        configured_inputs,
        budget,
        &mut |enumerated| {
            #[cfg(debug_assertions)]
            wait_for_mcp_snapshot_directory_test_barrier(enumerated)?;
            #[cfg(not(debug_assertions))]
            let _ = enumerated;
            Ok(())
        },
    )
}

#[cfg(all(test, unix))]
fn copy_snapshot_directory_with_hook(
    source: &Dir,
    destination: &Dir,
    relative: &Path,
    configured_exclusions: &HashSet<String>,
    configured_inputs: &[PathBuf],
    budget: &mut SnapshotBudget,
    mut after_enumeration: impl FnMut(&Path) -> Result<(), String>,
) -> Result<(), String> {
    copy_snapshot_directory_with_enumeration_hook(
        source,
        destination,
        relative,
        configured_exclusions,
        configured_inputs,
        budget,
        &mut after_enumeration,
    )
}

fn copy_snapshot_directory_with_enumeration_hook(
    source: &Dir,
    destination: &Dir,
    relative: &Path,
    configured_exclusions: &HashSet<String>,
    configured_inputs: &[PathBuf],
    budget: &mut SnapshotBudget,
    after_enumeration: &mut impl FnMut(&Path) -> Result<(), String>,
) -> Result<(), String> {
    destination.create_dir_all(relative).map_err(|error| {
        format!(
            "Cannot create MCP snapshot directory {}: {error}",
            relative.display()
        )
    })?;
    let entries = source.read_dir(".").map_err(|error| {
        format!(
            "Cannot read MCP project directory {} through its root capability: {error}",
            relative.display()
        )
    })?;

    let mut retained_entries = Vec::new();
    for entry in entries {
        budget.charge_entries(1, "MCP project snapshot")?;
        let entry = entry.map_err(|error| {
            format!(
                "Cannot inspect MCP project entry beneath {}: {error}",
                relative.display()
            )
        })?;
        let child = if relative == Path::new(".") {
            PathBuf::from(entry.file_name())
        } else {
            relative.join(entry.file_name())
        };
        let name = entry.file_name();
        if name
            .to_str()
            .is_some_and(|name| name.eq_ignore_ascii_case(".git"))
        {
            continue;
        }
        let metadata = source.symlink_metadata(&name).map_err(|error| {
            format!(
                "Cannot inspect MCP project input type {}: {error}",
                child.display()
            )
        })?;
        let file_type = metadata.file_type();
        if file_type.is_dir() || file_type.is_symlink() {
            let is_configured_input = configured_inputs
                .iter()
                .any(|input| snapshot_input_overlaps(input, &child));
            let has_direct_configured_input = configured_inputs.iter().any(|input| {
                let input = normalize_snapshot_input(input);
                !input.as_os_str().is_empty() && input.starts_with(&child)
            });
            let is_fixed_ignored = name
                .to_str()
                .is_some_and(|name| SNAPSHOT_IGNORED_DIRS.contains(&name));
            let is_configured_excluded = name
                .to_str()
                .is_some_and(|name| configured_exclusions.contains(name));
            if (is_configured_excluded && !has_direct_configured_input)
                || (is_fixed_ignored && !is_configured_input)
            {
                continue;
            }
        }

        if budget.copied_paths.contains(&child) {
            continue;
        }

        let directory_identity = if file_type.is_dir() {
            Some(snapshot_metadata_identity(&metadata).map_err(|error| {
                format!(
                    "Cannot identify MCP project directory {} during enumeration: {error}",
                    child.display()
                )
            })?)
        } else {
            None
        };
        retained_entries.push(SnapshotDirectoryEntry {
            name,
            child,
            directory_identity,
        });
    }

    after_enumeration(relative)?;

    for entry in retained_entries {
        if let Some(expected_identity) = entry.directory_identity {
            let retained =
                retain_snapshot_directory(source, &entry.name, &entry.child, expected_identity)?;
            verify_retained_snapshot_directory(source, &entry.name, &entry.child, &retained)?;
            copy_snapshot_directory_with_enumeration_hook(
                &retained.directory,
                destination,
                &entry.child,
                configured_exclusions,
                configured_inputs,
                budget,
                after_enumeration,
            )?;
            verify_retained_snapshot_directory(source, &entry.name, &entry.child, &retained)?;
            continue;
        }

        let metadata = source.symlink_metadata(&entry.name).map_err(|error| {
            format!(
                "Cannot inspect MCP project input {} through its retained parent capability: {error}",
                entry.child.display()
            )
        })?;
        if snapshot_metadata_is_link(&metadata) || !metadata.is_file() {
            return Err(format!(
                "MCP project input must be a regular file or directory and must not be a symlink or reparse point: {}",
                entry.child.display()
            ));
        }
        if metadata.len() > MAX_PROJECT_FILE_BYTES {
            return Err(format!(
                "MCP project input exceeds the {} MiB per-file limit: {}",
                MAX_PROJECT_FILE_BYTES / (1024 * 1024),
                entry.child.display()
            ));
        }
        let bytes = read_snapshot_project_file_from_directory(
            source,
            Path::new(&entry.name),
            &entry.child,
        )?;
        budget.charge_file(&entry.child, bytes.len() as u64)?;
        destination.write(&entry.child, bytes).map_err(|error| {
            format!(
                "Cannot copy MCP project input {} into the bounded snapshot: {error}",
                entry.child.display()
            )
        })?;
        budget.copied_paths.insert(entry.child);
    }

    Ok(())
}

fn retain_snapshot_directory(
    parent: &Dir,
    name: &OsStr,
    relative: &Path,
    expected_identity: SnapshotEntryIdentity,
) -> Result<RetainedSnapshotDirectory, String> {
    let before = parent.symlink_metadata(name).map_err(|error| {
        format!(
            "Cannot inspect MCP project directory {} through its retained parent capability: {error}",
            relative.display()
        )
    })?;
    if snapshot_metadata_is_link(&before) || !before.is_dir() {
        return Err(format!(
            "MCP project input must be a regular directory and must not be a symlink or reparse point: {}",
            relative.display()
        ));
    }
    let expected = snapshot_metadata_identity(&before).map_err(|error| {
        format!(
            "Cannot identify MCP project directory {}: {error}",
            relative.display()
        )
    })?;
    if expected != expected_identity {
        return Err(format!(
            "MCP project directory changed during snapshot traversal: {}",
            relative.display()
        ));
    }
    let directory = parent.open_dir(name).map_err(|error| {
        format!(
            "Cannot open MCP project directory {} through its retained parent capability: {error}",
            relative.display()
        )
    })?;
    let opened = directory_identity(&directory).map_err(|error| {
        format!(
            "Cannot identify opened MCP project directory {}: {error}",
            relative.display()
        )
    })?;
    let retained = RetainedSnapshotDirectory {
        directory,
        identity: expected,
    };
    if opened != expected {
        return Err(format!(
            "MCP project directory changed during enumeration: {}",
            relative.display()
        ));
    }
    verify_retained_snapshot_directory(parent, name, relative, &retained)?;
    Ok(retained)
}

fn verify_retained_snapshot_directory(
    parent: &Dir,
    name: &OsStr,
    relative: &Path,
    retained: &RetainedSnapshotDirectory,
) -> Result<(), String> {
    let current = parent.symlink_metadata(name).map_err(|error| {
        format!(
            "Cannot re-inspect MCP project directory {} through its retained parent capability: {error}",
            relative.display()
        )
    })?;
    if snapshot_metadata_is_link(&current)
        || !current.is_dir()
        || snapshot_metadata_identity(&current).ok() != Some(retained.identity)
        || directory_identity(&retained.directory).ok() != Some(retained.identity)
    {
        return Err(format!(
            "MCP project directory changed during snapshot traversal: {}",
            relative.display()
        ));
    }
    Ok(())
}

fn handle_initialize(id: Option<Value>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {
                "tools": {},
                "resources": {}
            },
            "serverInfo": {
                "name": SERVER_NAME,
                "version": SERVER_VERSION
            }
        }
    })
}

fn handle_initialize_request(id: Option<Value>, params: Option<&Value>) -> Value {
    let Some(Value::Object(params)) = params else {
        return invalid_params(id, "initialize params must be an object");
    };
    if params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .is_none_or(str::is_empty)
    {
        return invalid_params(
            id,
            "initialize parameter `protocolVersion` must be a non-empty string",
        );
    }
    if !params.get("capabilities").is_some_and(Value::is_object) {
        return invalid_params(id, "initialize parameter `capabilities` must be an object");
    }
    let Some(client_info) = params.get("clientInfo").and_then(Value::as_object) else {
        return invalid_params(id, "initialize parameter `clientInfo` must be an object");
    };
    for field in ["name", "version"] {
        if client_info
            .get(field)
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            return invalid_params(
                id,
                format!("initialize parameter `clientInfo.{field}` must be a non-empty string"),
            );
        }
    }
    handle_initialize(id)
}

/// Same tool-definition list the MCP server registers for `tools/list`.
/// One catalog — lock/diff must call this rather than duplicating schemas.
pub fn mcp_tool_definitions(allow_write: bool) -> Vec<Value> {
    let root_property = json!({
        "type": "string",
        "description": "Existing project directory at or below the server root"
    });
    let mut tools = vec![
        json!({
            "name": "specsync_check",
            "description": "Validate all spec files against source code. Returns errors, warnings, and pass/fail status for each spec.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "root": root_property.clone(),
                    "strict": {
                        "type": "boolean",
                        "description": "Treat warnings as errors (default: false)"
                    }
                },
                "additionalProperties": false
            }
        }),
        read_tool_schema(
            "specsync_coverage",
            "Get file and LOC coverage metrics. Shows which source files and modules have specs and which don't.",
            &root_property,
        ),
        read_tool_schema(
            "specsync_list_specs",
            "List all spec files found in the project with their module names and status.",
            &root_property,
        ),
        read_tool_schema(
            "specsync_score",
            "Score spec quality (0-100) with letter grades, breakdown by category, and improvement suggestions.",
            &root_property,
        ),
        read_tool_schema(
            "specsync_issues",
            "Verify GitHub issue references in spec frontmatter. Checks that linked issues exist and reports their status (open/closed).",
            &root_property,
        ),
    ];

    if allow_write {
        tools.push(write_tool_schema(
            "specsync_generate",
            "Deterministically scaffold spec files for uncovered source modules at the server root. Returns paths of generated specs.",
        ));
        tools.push(write_tool_schema(
            "specsync_init",
            "Initialize a specsync.json config file at the server root with auto-detected source directories.",
        ));
    }

    tools
}

fn handle_tools_list(id: Option<Value>, allow_write: bool) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": { "tools": mcp_tool_definitions(allow_write) }
    })
}

fn read_tool_schema(name: &str, description: &str, root_property: &Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": { "root": root_property },
            "additionalProperties": false
        }
    })
}

fn write_tool_schema(name: &str, description: &str) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": {},
            "additionalProperties": false
        }
    })
}

#[cfg(test)]
fn handle_tools_call(
    id: Option<Value>,
    params: &Value,
    server_root: &Path,
    allow_write: bool,
) -> Value {
    let server_directory = match Dir::open_ambient_dir(server_root, ambient_authority()) {
        Ok(directory) => directory,
        Err(error) => {
            return tool_error(
                id,
                format!(
                    "Cannot open MCP server root capability {}: {error}",
                    server_root.display()
                ),
            );
        }
    };
    handle_tools_call_with_directory(
        id,
        params,
        server_root,
        server_root,
        &server_directory,
        allow_write,
    )
}

fn handle_tools_call_with_directory(
    id: Option<Value>,
    params: &Value,
    server_root: &Path,
    requested_server_root: &Path,
    server_directory: &Dir,
    allow_write: bool,
) -> Value {
    let Some(params) = params.as_object() else {
        return invalid_params(id, "tools/call params must be an object");
    };
    if let Some(key) = params
        .keys()
        .find(|key| key.as_str() != "name" && key.as_str() != "arguments")
    {
        return invalid_params(id, format!("Unknown tools/call parameter `{key}`"));
    }
    let Some(tool_name) = params.get("name").and_then(Value::as_str) else {
        return invalid_params(id, "tools/call parameter `name` must be a string");
    };
    let empty_arguments = serde_json::Map::new();
    let arguments = match params.get("arguments") {
        Some(Value::Object(arguments)) => arguments,
        Some(_) => {
            return invalid_params(id, "tools/call parameter `arguments` must be an object");
        }
        None => &empty_arguments,
    };

    let is_mutating = matches!(tool_name, "specsync_generate" | "specsync_init");
    let is_known = matches!(
        tool_name,
        "specsync_check"
            | "specsync_coverage"
            | "specsync_generate"
            | "specsync_list_specs"
            | "specsync_init"
            | "specsync_score"
            | "specsync_issues"
    );
    if !is_known {
        return tool_error(id, format!("Unknown tool: {tool_name}"));
    }
    if let Err(message) = validate_tool_arguments(tool_name, arguments) {
        return invalid_params(id, message);
    }
    if is_mutating && !allow_write {
        return tool_error(
            id,
            format!("Tool `{tool_name}` requires starting the MCP server with --allow-write"),
        );
    }

    let operation_relative = if is_mutating {
        PathBuf::from(".")
    } else {
        match resolve_read_root(
            server_root,
            requested_server_root,
            arguments.get("root").and_then(Value::as_str),
        ) {
            Ok(relative) => relative,
            Err(message) => return tool_error(id, message),
        }
    };
    let operation_route = match open_confined_read_root(server_directory, &operation_relative) {
        Ok(route) => route,
        Err(error) => return tool_error(id, error),
    };
    let arguments = Value::Object(arguments.clone());
    let snapshot = match ProjectSnapshot::create_from_directory(operation_route.directory()) {
        Ok(snapshot) => snapshot,
        Err(message) => return tool_error(id, message),
    };
    let result = with_isolated_snapshot_git(&snapshot, || {
        let operation_root = snapshot.root();
        match tool_name {
            "specsync_check" => tool_check(operation_root, &arguments),
            "specsync_coverage" => tool_coverage(operation_root),
            "specsync_generate" => tool_generate(operation_root, server_directory, &arguments),
            "specsync_list_specs" => tool_list_specs(operation_root),
            "specsync_init" => tool_init(operation_root, server_directory),
            "specsync_score" => tool_score(operation_root, snapshot.git_freshness_available()),
            "specsync_issues" => tool_issues(operation_root),
            _ => unreachable!("known tool was validated before dispatch"),
        }
    });

    match result {
        Ok(content) => {
            if let Err(message) = operation_route.revalidate_before_success() {
                return tool_error(id, message);
            }
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&content).unwrap_or_default()
                    }]
                }
            })
        }
        Err(msg) => tool_error(id, msg),
    }
}

fn validate_tool_arguments(
    tool_name: &str,
    arguments: &serde_json::Map<String, Value>,
) -> Result<(), String> {
    const RETIRED_GENERATE_ARGUMENTS: &[&str] = &[
        "ai",
        "provider",
        "aiProvider",
        "ai_provider",
        "model",
        "aiModel",
        "ai_model",
        "apiKey",
        "api_key",
        "aiApiKey",
        "ai_api_key",
        "credential",
        "credentials",
        "baseUrl",
        "base_url",
        "aiBaseUrl",
        "ai_base_url",
        "timeout",
        "timeoutSecs",
        "timeout_secs",
        "aiTimeout",
        "ai_timeout",
        "command",
        "aiCommand",
        "ai_command",
    ];

    if tool_name == "specsync_generate"
        && let Some(name) = RETIRED_GENERATE_ARGUMENTS
            .iter()
            .find(|name| arguments.contains_key(**name))
    {
        return Err(format!(
            "MCP argument `{name}` was removed in spec-sync 5.0; `specsync_generate` is deterministic"
        ));
    }

    let allowed: &[(&str, &str)] = match tool_name {
        "specsync_check" => &[("root", "string"), ("strict", "boolean")],
        "specsync_coverage" | "specsync_list_specs" | "specsync_score" | "specsync_issues" => {
            &[("root", "string")]
        }
        "specsync_generate" | "specsync_init" => &[],
        _ => &[],
    };

    for (name, value) in arguments {
        let Some((_, expected_type)) = allowed.iter().find(|(allowed, _)| allowed == name) else {
            return Err(format!("Unknown argument `{name}` for tool `{tool_name}`"));
        };
        let valid = match *expected_type {
            "string" => value.is_string(),
            "boolean" => value.is_boolean(),
            _ => false,
        };
        if !valid {
            return Err(format!(
                "Argument `{name}` for tool `{tool_name}` must be a {expected_type}"
            ));
        }
    }

    Ok(())
}

fn resolve_read_root(
    server_root: &Path,
    requested_server_root: &Path,
    requested_root: Option<&str>,
) -> Result<PathBuf, String> {
    let Some(requested_root) = requested_root else {
        return Ok(PathBuf::from("."));
    };
    let requested_path = Path::new(requested_root);
    let components: Vec<Component<'_>> = requested_path.components().collect();
    if components.contains(&Component::ParentDir) {
        return Err("Read root override must not contain parent traversal".to_string());
    }
    if !requested_path.is_absolute()
        && components
            .iter()
            .any(|component| matches!(*component, Component::RootDir | Component::Prefix(_)))
    {
        return Err("Read root override must not use a rooted or drive-relative path".to_string());
    }

    let relative = if requested_path.is_absolute() {
        relative_read_root_suffix_from_aliases(server_root, requested_server_root, requested_path)
            .ok_or_else(|| "Read root override escapes the configured server root".to_string())?
    } else {
        requested_path.to_path_buf()
    };
    if relative.components().any(|component| {
        matches!(
            component,
            Component::Normal(name)
                if name
                    .to_str()
                    .is_some_and(|name| name.eq_ignore_ascii_case(".git"))
        )
    }) {
        return Err("Read root override must not select Git metadata".to_string());
    }
    Ok(if relative.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        relative
    })
}

fn relative_read_root_suffix_from_aliases(
    canonical_root: &Path,
    requested_root: &Path,
    candidate: &Path,
) -> Option<PathBuf> {
    // Windows canonicalization can expand an operator-supplied DOS 8.3 root spelling. Accept the
    // candidate beneath either startup spelling, but never canonicalize the candidate before
    // authorization. The derived suffix is still opened only through the retained root capability.
    relative_read_root_suffix(canonical_root, candidate)
        .or_else(|| relative_read_root_suffix(requested_root, candidate))
}

#[cfg(not(windows))]
fn relative_read_root_suffix(root: &Path, candidate: &Path) -> Option<PathBuf> {
    candidate.strip_prefix(root).ok().map(Path::to_path_buf)
}

#[cfg(windows)]
fn relative_read_root_suffix(root: &Path, candidate: &Path) -> Option<PathBuf> {
    let (root_prefix, root_components) = parse_windows_absolute_path_native(root)?;
    let (candidate_prefix, candidate_components) = parse_windows_absolute_path_native(candidate)?;
    if !windows_prefixes_equal(&root_prefix, &candidate_prefix)
        || candidate_components.len() < root_components.len()
        || !root_components
            .iter()
            .zip(&candidate_components)
            .all(|(root, candidate)| windows_ordinal_ignore_case(root, candidate))
    {
        return None;
    }
    Some(
        candidate_components[root_components.len()..]
            .iter()
            .collect(),
    )
}

#[cfg(windows)]
enum WindowsPathPrefix {
    Disk(u8),
    Unc(std::ffi::OsString, std::ffi::OsString),
}

#[cfg(windows)]
fn parse_windows_absolute_path_native(
    path: &Path,
) -> Option<(WindowsPathPrefix, Vec<std::ffi::OsString>)> {
    use std::path::Prefix;

    let mut components = path.components();
    let Component::Prefix(prefix) = components.next()? else {
        return None;
    };
    let prefix = match prefix.kind() {
        Prefix::Disk(drive) | Prefix::VerbatimDisk(drive) => WindowsPathPrefix::Disk(drive),
        Prefix::UNC(server, share) | Prefix::VerbatimUNC(server, share) => {
            WindowsPathPrefix::Unc(server.to_os_string(), share.to_os_string())
        }
        Prefix::Verbatim(_) | Prefix::DeviceNS(_) => return None,
    };
    if !matches!(components.next(), Some(Component::RootDir)) {
        return None;
    }
    let mut relative = Vec::new();
    for component in components {
        match component {
            Component::Normal(component) => relative.push(component.to_os_string()),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some((prefix, relative))
}

#[cfg(windows)]
fn windows_prefixes_equal(left: &WindowsPathPrefix, right: &WindowsPathPrefix) -> bool {
    match (left, right) {
        (WindowsPathPrefix::Disk(left), WindowsPathPrefix::Disk(right)) => {
            left.eq_ignore_ascii_case(right)
        }
        (
            WindowsPathPrefix::Unc(left_server, left_share),
            WindowsPathPrefix::Unc(right_server, right_share),
        ) => {
            windows_ordinal_ignore_case(left_server, right_server)
                && windows_ordinal_ignore_case(left_share, right_share)
        }
        _ => false,
    }
}

#[cfg(windows)]
fn windows_ordinal_ignore_case(left: &std::ffi::OsStr, right: &std::ffi::OsStr) -> bool {
    use std::os::windows::ffi::OsStrExt;

    unsafe extern "system" {
        fn CompareStringOrdinal(
            left: *const u16,
            left_len: i32,
            right: *const u16,
            right_len: i32,
            ignore_case: i32,
        ) -> i32;
    }

    let left: Vec<u16> = left.encode_wide().collect();
    let right: Vec<u16> = right.encode_wide().collect();
    let (Ok(left_len), Ok(right_len)) = (i32::try_from(left.len()), i32::try_from(right.len()))
    else {
        return false;
    };
    // SAFETY: both pointers reference initialized UTF-16 buffers for the exact lengths supplied;
    // CompareStringOrdinal does not retain or mutate either buffer.
    unsafe { CompareStringOrdinal(left.as_ptr(), left_len, right.as_ptr(), right_len, 1) == 2 }
}

#[cfg(test)]
fn windows_relative_suffix_text(root: &str, candidate: &str) -> Option<PathBuf> {
    let root = parse_windows_absolute_path(root)?;
    let candidate = parse_windows_absolute_path(candidate)?;
    if !windows_text_ignore_case(&root.prefix, &candidate.prefix)
        || candidate.components.len() < root.components.len()
        || !root
            .components
            .iter()
            .zip(&candidate.components)
            .all(|(root, candidate)| windows_text_ignore_case(root, candidate))
    {
        return None;
    }
    Some(
        candidate.components[root.components.len()..]
            .iter()
            .collect(),
    )
}

#[cfg(test)]
fn windows_text_ignore_case(left: &str, right: &str) -> bool {
    left.to_lowercase() == right.to_lowercase()
}

#[cfg(test)]
struct WindowsAbsolutePath {
    prefix: String,
    components: Vec<String>,
}

#[cfg(test)]
fn parse_windows_absolute_path(path: &str) -> Option<WindowsAbsolutePath> {
    let mut path = path.replace('/', "\\");
    if let Some(without_prefix) = strip_ascii_case_prefix(&path, r"\\?\UNC\") {
        path = format!(r"\\{without_prefix}");
    } else if let Some(without_prefix) = strip_ascii_case_prefix(&path, r"\\?\") {
        path = without_prefix.to_string();
    }

    if let Some(unc) = path.strip_prefix(r"\\") {
        let mut components = unc.split('\\').filter(|component| !component.is_empty());
        let server = components.next()?;
        let share = components.next()?;
        return Some(WindowsAbsolutePath {
            prefix: format!(r"\\{server}\{share}"),
            components: components.map(str::to_string).collect(),
        });
    }

    let bytes = path.as_bytes();
    if bytes.len() < 3 || bytes[1] != b':' || bytes[2] != b'\\' {
        return None;
    }
    Some(WindowsAbsolutePath {
        prefix: path[..2].to_string(),
        components: path[3..]
            .split('\\')
            .filter(|component| !component.is_empty())
            .map(str::to_string)
            .collect(),
    })
}

#[cfg(test)]
fn strip_ascii_case_prefix<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    let candidate = value.get(..prefix.len())?;
    candidate
        .eq_ignore_ascii_case(prefix)
        .then(|| &value[prefix.len()..])
}

fn invalid_params(id: Option<Value>, message: impl Into<String>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": -32602, "message": message.into() }
    })
}

fn tool_error(id: Option<Value>, message: impl Into<String>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": [{
                "type": "text",
                "text": message.into()
            }],
            "isError": true
        }
    })
}

// ─── Resource Handlers ──────────────────────────────────────────────────

fn handle_resources_list(id: Option<Value>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "resources": [
                {
                    "uri": "specsync:///specs",
                    "name": "All Specs",
                    "description": "List all spec modules with metadata (name, path, version, status, score)",
                    "mimeType": "application/json"
                },
                {
                    "uri": "specsync:///graph",
                    "name": "Dependency Graph",
                    "description": "Cross-module dependency graph with edges, cycles, and topological order",
                    "mimeType": "application/json"
                },
                {
                    "uri": "specsync:///config",
                    "name": "Configuration",
                    "description": "Current specsync.json configuration",
                    "mimeType": "application/json"
                },
                {
                    "uri": "specsync:///coverage",
                    "name": "Coverage Report",
                    "description": "File and LOC coverage metrics — which modules have specs and which don't",
                    "mimeType": "application/json"
                }
            ],
            "resourceTemplates": [
                {
                    "uriTemplate": "specsync:///specs/{module}",
                    "name": "Spec by Module",
                    "description": "Read a specific spec's full content with parsed frontmatter and score",
                    "mimeType": "text/markdown"
                }
            ]
        }
    })
}

#[cfg(test)]
fn handle_resources_read(id: Option<Value>, params: &Value, root: &Path) -> Value {
    let directory = match Dir::open_ambient_dir(root, ambient_authority()) {
        Ok(directory) => directory,
        Err(error) => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32602,
                    "message": format!("Cannot open MCP server root capability {}: {error}", root.display())
                }
            });
        }
    };
    handle_resources_read_with_directory(id, params, root, &directory)
}

fn handle_resources_read_with_directory(
    id: Option<Value>,
    params: &Value,
    _root: &Path,
    directory: &Dir,
) -> Value {
    let Some(arguments) = params.as_object() else {
        return invalid_params(id, "resources/read params must be an object");
    };
    if let Some(key) = arguments.keys().find(|key| key.as_str() != "uri") {
        return invalid_params(id, format!("Unknown resources/read parameter `{key}`"));
    }
    let Some(uri) = arguments.get("uri").and_then(Value::as_str) else {
        return invalid_params(id, "resources/read parameter `uri` must be a string");
    };

    let operation_route = match open_confined_read_root(directory, Path::new(".")) {
        Ok(route) => route,
        Err(message) => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32602, "message": message }
            });
        }
    };
    let snapshot = match ProjectSnapshot::create_from_directory(operation_route.directory()) {
        Ok(snapshot) => snapshot,
        Err(message) => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32602, "message": message }
            });
        }
    };
    let result = with_isolated_snapshot_git(&snapshot, || {
        let operation_root = snapshot.root();
        match uri {
            "specsync:///specs" => {
                resource_specs_list(operation_root, snapshot.git_freshness_available())
            }
            "specsync:///graph" => resource_graph(operation_root),
            "specsync:///config" => resource_config(operation_root),
            "specsync:///coverage" => resource_coverage(operation_root),
            _ if uri.starts_with("specsync:///specs/") => {
                let module = &uri["specsync:///specs/".len()..];
                resource_spec_by_module(operation_root, module)
            }
            _ => Err(format!("Unknown resource URI: {uri}")),
        }
    });

    match result {
        Ok((content, mime_type)) => {
            if let Err(message) = operation_route.revalidate_before_success() {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32602, "message": message }
                });
            }
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "contents": [{
                        "uri": uri,
                        "mimeType": mime_type,
                        "text": content
                    }]
                }
            })
        }
        Err(msg) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32602, "message": msg }
        }),
    }
}

// ─── Resource Implementations ───────────────────────────────────────────

fn resource_specs_list(
    root: &Path,
    git_freshness_available: bool,
) -> Result<(String, &'static str), String> {
    let (config, spec_files) = load_and_discover(root, true)?;

    let specs: Vec<Value> = spec_files
        .iter()
        .map(|f| -> Result<Value, String> {
            let content = read_file_bounded(root, f, "spec file")?;
            let parsed = crate::parser::parse_frontmatter(&content);
            let score = score_spec_for_mcp(f, root, &config, git_freshness_available);
            let relative = f
                .strip_prefix(root)
                .unwrap_or(f)
                .to_string_lossy()
                .to_string();

            if let Some(parsed) = parsed {
                let fm = parsed.frontmatter;
                Ok(json!({
                    "path": relative,
                    "module": fm.module,
                    "version": fm.version,
                    "status": fm.status,
                    "files": fm.files,
                    "depends_on": fm.depends_on,
                    "score": score.total,
                    "grade": score.grade,
                    "git_freshness_available": git_freshness_available,
                }))
            } else {
                Ok(json!({
                    "path": relative,
                    "module": null,
                    "score": score.total,
                    "grade": score.grade,
                    "git_freshness_available": git_freshness_available,
                }))
            }
        })
        .collect::<Result<_, _>>()?;

    let output = json!({
        "specs": specs,
        "count": specs.len(),
        "git_freshness_available": git_freshness_available,
    });
    Ok((
        serde_json::to_string_pretty(&output).unwrap(),
        "application/json",
    ))
}

fn resource_spec_by_module(root: &Path, module: &str) -> Result<(String, &'static str), String> {
    let (_config, spec_files) = load_and_discover(root, true)?;

    // Find the spec file matching this module name
    for f in &spec_files {
        let content = read_file_bounded(root, f, "spec file")?;

        let parsed = crate::parser::parse_frontmatter(&content);
        let matches = parsed
            .as_ref()
            .and_then(|p| p.frontmatter.module.as_deref())
            .map(|m| m == module)
            .unwrap_or(false);

        if matches {
            return Ok((content, "text/markdown"));
        }
    }

    Err(format!("No spec found for module: {module}"))
}

fn resource_graph(root: &Path) -> Result<(String, &'static str), String> {
    let config = load_confined_config(root)?;
    let graph = build_dep_graph(root, &config.specs_dir);

    let nodes: Vec<Value> = graph
        .values()
        .map(|node| {
            json!({
                "module": node.module,
                "spec_path": node.spec_path,
                "depends_on": node.declared_deps,
                "files": node.files,
            })
        })
        .collect();

    // Build edges list
    let mut edges: Vec<Value> = Vec::new();
    for node in graph.values() {
        for dep in &node.declared_deps {
            edges.push(json!({
                "from": node.module,
                "to": dep,
            }));
        }
    }

    // Detect cycles
    let cycles = crate::deps::validate_deps(root, &config.specs_dir).cycles;
    let cycle_values: Vec<Value> = cycles.iter().map(|c| json!(c)).collect();

    // Topological order
    let topo = crate::deps::topological_sort(&graph);

    let output = json!({
        "modules": nodes,
        "edges": edges,
        "module_count": graph.len(),
        "edge_count": edges.len(),
        "cycles": cycle_values,
        "topological_order": topo,
    });

    Ok((
        serde_json::to_string_pretty(&output).unwrap(),
        "application/json",
    ))
}

fn resource_config(root: &Path) -> Result<(String, &'static str), String> {
    let config = load_confined_config(root)?;
    let output = json!({
        "specs_dir": config.specs_dir,
        "source_dirs": config.source_dirs,
        "required_sections": config.required_sections,
        "exclude_dirs": config.exclude_dirs,
        "exclude_patterns": config.exclude_patterns,
        "schema_dir": config.schema_dir,
    });

    Ok((
        serde_json::to_string_pretty(&output).unwrap(),
        "application/json",
    ))
}

fn resource_coverage(root: &Path) -> Result<(String, &'static str), String> {
    // Byte-identical to the `specsync_coverage` tool: one constructor, one
    // validation pass, one payload. This resource is what an agent reads when
    // it asks the project how it is doing, and it used to answer with
    // percentages alone — no findings, no `passed`. An agent has no exit code
    // and no second channel, so "nothing reported" is the only thing it can
    // read as "nothing wrong".
    let output = mcp_coverage_payload(root)?;
    Ok((
        serde_json::to_string_pretty(&output).unwrap(),
        "application/json",
    ))
}

/// Every spec's validation outcome, collected once for whichever MCP payload
/// needs it.
///
/// `tool_check`, `tool_coverage` and `resource_coverage` used to answer three
/// different questions from three different amounts of work: check validated,
/// the other two validated nothing and reported no findings at all. One
/// collector means a finding cannot be present in one payload and absent from
/// another.
#[derive(Default)]
struct McpFindings {
    total_errors: usize,
    total_warnings: usize,
    specs_passed: usize,
    errors: Vec<String>,
    warnings: Vec<String>,
    notices: Vec<String>,
    specs: Vec<Value>,
}

fn validate_all(root: &Path, config: &SpecSyncConfig, spec_files: &[PathBuf]) -> McpFindings {
    let schema_tables = get_schema_table_names(root, config);
    let schema_columns = config
        .schema_dir
        .as_deref()
        .map(|schema_dir| crate::schema::build_schema(&root.join(schema_dir)))
        .unwrap_or_default();

    let mut collected = McpFindings::default();
    for spec_file in spec_files {
        let result = validate_spec(spec_file, root, &schema_tables, &schema_columns, config);
        let spec_passed = result.errors.is_empty();

        collected.specs.push(json!({
            "spec": result.spec_path,
            "passed": spec_passed,
            "errors": result.errors,
            "warnings": result.warnings,
            "export_summary": result.export_summary,
        }));

        for e in &result.errors {
            collected.errors.push(format!("{}: {e}", result.spec_path));
        }
        for w in &result.warnings {
            collected
                .warnings
                .push(format!("{}: {w}", result.spec_path));
        }
        for n in &result.notices {
            collected.notices.push(format!("{}: {n}", result.spec_path));
        }

        collected.total_errors += result.errors.len();
        collected.total_warnings += result.warnings.len();
        if spec_passed {
            collected.specs_passed += 1;
        }
    }
    collected
}

/// The coverage payload both MCP surfaces return, built by the one shared
/// constructor in `output`.
///
/// `passed` is the gate verdict, the same semantics the CLI uses: "would
/// `specsync coverage` exit 0 on this project?" — evaluated with no CLI
/// overrides and the project's configured enforcement, since MCP has neither
/// flags nor an exit code. The counts and arrays beside it answer the factual
/// question independently of that policy.
fn mcp_coverage_payload(root: &Path) -> Result<Value, String> {
    let (config, spec_files) = load_and_discover(root, true)?;
    let coverage = compute_coverage_checked(root, &spec_files, &config)
        .map_err(|error| format!("MCP coverage is inconclusive: {error}"))?;
    let collected = validate_all(root, &config, &spec_files);
    let passed = crate::commands::compute_exit_code(
        collected.total_errors,
        collected.total_warnings,
        false,
        config.enforcement,
        &coverage,
        None,
    ) == 0;

    Ok(coverage_json(
        &coverage,
        &CoverageFindings {
            passed,
            specs_checked: spec_files.len(),
            specs_passed: collected.specs_passed,
            errors: collected.errors,
            warnings: collected.warnings,
            notices: collected.notices,
        },
    ))
}

// ─── Tool Implementations ────────────────────────────────────────────────

fn load_and_discover(
    root: &Path,
    allow_empty: bool,
) -> Result<(SpecSyncConfig, Vec<PathBuf>), String> {
    let config = load_confined_config(root)?;
    let specs_dir = root.join(&config.specs_dir);
    let spec_files: Vec<PathBuf> = find_spec_files(&specs_dir)
        .into_iter()
        .filter(|f| {
            f.file_name()
                .and_then(|n| n.to_str())
                .map(|n| !n.starts_with('_'))
                .unwrap_or(true)
        })
        .collect();

    validate_spec_file_mappings(root, &spec_files, &config.exclude_dirs)?;

    if spec_files.is_empty() && !allow_empty {
        return Err(format!(
            "No spec files found in {}/. Run specsync generate to scaffold specs.",
            config.specs_dir
        ));
    }

    Ok((config, spec_files))
}

fn load_confined_config(root: &Path) -> Result<SpecSyncConfig, String> {
    validate_known_config_files(root)?;
    validate_manifest_inputs(root)?;
    if config_requires_source_autodetection(root) {
        validate_autodetection_tree(root)?;
    }
    let config = load_config(root);

    let configured_path_count = 1usize
        + config.source_dirs.len()
        + usize::from(config.schema_dir.is_some())
        + config
            .modules
            .values()
            .map(|module| module.files.len())
            .sum::<usize>();
    if configured_path_count > MAX_CONFIGURED_PATHS {
        return Err(format!(
            "MCP configuration exceeds {MAX_CONFIGURED_PATHS} path entries"
        ));
    }

    let no_exclusions = HashSet::new();
    let source_exclusions: HashSet<String> = config.exclude_dirs.iter().cloned().collect();
    let mut confinement_entries_seen = 0usize;

    validate_configured_path_with_budget(
        root,
        &config.specs_dir,
        "specs_dir",
        Some(&no_exclusions),
        &mut confinement_entries_seen,
    )?;
    for source_dir in &config.source_dirs {
        validate_configured_path_with_budget(
            root,
            source_dir,
            "source_dirs entry",
            Some(&source_exclusions),
            &mut confinement_entries_seen,
        )?;
    }
    if let Some(schema_dir) = config.schema_dir.as_deref()
        && !schema_dir.is_empty()
    {
        validate_configured_path_with_budget(
            root,
            schema_dir,
            "schema_dir",
            Some(&no_exclusions),
            &mut confinement_entries_seen,
        )?;
    }
    for (module_name, module) in &config.modules {
        validate_module_name(module_name)?;
        for file in &module.files {
            validate_configured_path_with_budget(
                root,
                file,
                "configured module file",
                Some(&source_exclusions),
                &mut confinement_entries_seen,
            )?;
        }
    }

    validate_project_input_budget(root, &config)?;

    Ok(config)
}

fn validate_known_config_files(root: &Path) -> Result<(), String> {
    for relative in [
        ".specsync/config.toml",
        ".specsync/config.json",
        ".specsync/config.local.toml",
        ".specsync.toml",
        "specsync.json",
        ".specsync/hashes.json",
        ".git",
        "Cargo.toml",
        "Package.swift",
        "build.gradle.kts",
        "build.gradle",
        "settings.gradle.kts",
        "settings.gradle",
        "package.json",
        "pubspec.yaml",
        "go.mod",
        "pyproject.toml",
    ] {
        let candidate = root.join(relative);
        match fs::symlink_metadata(&candidate) {
            Ok(_) => validate_existing_path(root, &candidate, "project metadata file", None)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP configuration path {}: {error}",
                    candidate.display()
                ));
            }
        }
    }
    Ok(())
}

fn selected_config_path(root: &Path) -> Option<PathBuf> {
    [
        ".specsync/config.toml",
        ".specsync/config.json",
        ".specsync.toml",
        "specsync.json",
    ]
    .iter()
    .map(|relative| root.join(relative))
    .find(|path| path.exists())
}

fn config_requires_source_autodetection(root: &Path) -> bool {
    let Some(config_path) = selected_config_path(root) else {
        return true;
    };
    let Some(content) = crate::config::read_config_file(&config_path) else {
        return false;
    };
    if config_path
        .extension()
        .and_then(|extension| extension.to_str())
        == Some("json")
    {
        return serde_json::from_str::<Value>(&content).is_ok()
            && !content.contains("\"sourceDirs\"");
    }

    let mut top_level = true;
    for line in content.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            top_level = false;
            continue;
        }
        if top_level
            && line
                .split_once('=')
                .is_some_and(|(key, _)| key.trim() == "source_dirs")
        {
            return false;
        }
    }
    true
}

fn validate_manifest_inputs(root: &Path) -> Result<(), String> {
    let mut visiting = HashSet::new();
    let mut validated = HashSet::new();
    let mut manifests_seen = 0usize;
    let mut traversal_entries_seen = 0usize;
    validate_cargo_workspace_manifest(
        root,
        root,
        &mut visiting,
        &mut validated,
        &mut manifests_seen,
        &mut traversal_entries_seen,
    )?;
    validate_package_workspaces(root, &mut traversal_entries_seen)?;
    validate_gradle_modules(root)?;
    validate_python_package_path(root)?;
    Ok(())
}

fn validate_autodetection_tree(root: &Path) -> Result<(), String> {
    let canonical_root = root.canonicalize().map_err(|error| {
        format!(
            "Cannot resolve MCP server root {} during source autodetection preflight: {error}",
            root.display()
        )
    })?;
    let ignored: HashSet<&str> = AUTODETECT_IGNORED_DIRS.iter().copied().collect();
    let mut entries_seen = 0usize;
    let entries = WalkDir::new(&canonical_root)
        .max_depth(4)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            if entry.depth() == 0 {
                return true;
            }
            let name = entry.file_name().to_str().unwrap_or("");
            if entry.depth() == 1 && (name.starts_with('.') || ignored.contains(name)) {
                return false;
            }
            !entry.file_type().is_dir() || (!name.starts_with('.') && !ignored.contains(name))
        });

    for entry in entries {
        entries_seen += 1;
        if entries_seen > MAX_CONFINEMENT_ENTRIES {
            return Err(format!(
                "MCP source autodetection preflight exceeds {MAX_CONFINEMENT_ENTRIES} entries"
            ));
        }
        let entry = entry
            .map_err(|error| format!("Cannot inspect MCP source autodetection input: {error}"))?;
        if !entry.file_type().is_symlink() {
            continue;
        }
        let canonical = entry.path().canonicalize().map_err(|error| {
            format!(
                "Cannot resolve symlink used by MCP source autodetection {}: {error}",
                entry.path().display()
            )
        })?;
        if !canonical.starts_with(&canonical_root) {
            return Err(format!(
                "Symlink used by MCP source autodetection escapes the configured server root: {}",
                entry.path().display()
            ));
        }
    }

    Ok(())
}

fn validate_cargo_workspace_manifest(
    server_root: &Path,
    manifest_dir: &Path,
    visiting: &mut HashSet<PathBuf>,
    validated: &mut HashSet<PathBuf>,
    manifests_seen: &mut usize,
    traversal_entries_seen: &mut usize,
) -> Result<(), String> {
    let manifest_path = manifest_dir.join("Cargo.toml");
    match fs::symlink_metadata(&manifest_path) {
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(format!(
                "Cannot inspect MCP Cargo workspace manifest {}: {error}",
                manifest_path.display()
            ));
        }
    }
    validate_existing_path(
        server_root,
        &manifest_path,
        "Cargo workspace manifest",
        None,
    )?;
    let canonical_manifest = manifest_path.canonicalize().map_err(|error| {
        format!(
            "Cannot resolve MCP Cargo workspace manifest {}: {error}",
            manifest_path.display()
        )
    })?;
    if validated.contains(&canonical_manifest) {
        return Ok(());
    }
    if !visiting.insert(canonical_manifest.clone()) {
        return Err(format!(
            "MCP Cargo workspace manifest cycle detected at {}",
            manifest_path.display()
        ));
    }
    *manifests_seen += 1;
    if *manifests_seen > MAX_MANIFEST_PREFLIGHTS {
        return Err(format!(
            "MCP Cargo workspace preflight exceeds {MAX_MANIFEST_PREFLIGHTS} manifests"
        ));
    }

    let content = read_file_bounded(server_root, &manifest_path, "Cargo workspace manifest")?;
    let cargo = parse_cargo_snapshot_manifest(&content, &manifest_path)?;
    let mut member_roots = HashSet::new();
    for member in cargo.workspace_members {
        charge_manifest_traversal_entry(traversal_entries_seen, "MCP Cargo workspace preflight")?;
        let member_root = validate_manifest_relative_candidate(
            server_root,
            manifest_dir,
            &member,
            "Cargo workspace member",
        )?;
        let member_key = member_root.canonicalize().unwrap_or_else(|_| {
            member_root
                .strip_prefix(server_root)
                .map(normalize_snapshot_input)
                .unwrap_or_else(|_| member_root.clone())
        });
        if !member_roots.insert(member_key) {
            continue;
        }
        let nested_manifest = member_root.join("Cargo.toml");
        match fs::symlink_metadata(&nested_manifest) {
            Ok(_) => validate_cargo_workspace_manifest(
                server_root,
                &member_root,
                visiting,
                validated,
                manifests_seen,
                traversal_entries_seen,
            )?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP Cargo workspace member manifest {}: {error}",
                    nested_manifest.display()
                ));
            }
        }
    }

    visiting.remove(&canonical_manifest);
    validated.insert(canonical_manifest);
    Ok(())
}

fn validate_package_workspaces(
    root: &Path,
    traversal_entries_seen: &mut usize,
) -> Result<(), String> {
    let package_path = root.join("package.json");
    let content = match read_file_bounded(root, &package_path, "package workspace manifest") {
        Ok(content) => content,
        Err(_) if !package_path.exists() => return Ok(()),
        Err(error) => return Err(error),
    };
    let json: Value = serde_json::from_str(&content)
        .map_err(|error| format!("Cannot parse MCP package.json as JSON: {error}"))?;
    let declarations = package_workspace_declarations(&json, "MCP package.json")?;
    for _ in declarations {
        charge_manifest_traversal_entry(traversal_entries_seen, "MCP package workspace preflight")?;
    }
    let workspace_patterns = declarations
        .iter()
        .map(|declaration| {
            declaration
                .as_str()
                .ok_or_else(|| "MCP package.json workspace entries must be strings".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut seen_patterns = HashSet::new();
    let mut seen_bases = HashSet::new();
    let mut seen_workspaces = HashSet::new();
    for pattern in workspace_patterns {
        if !seen_patterns.insert(pattern.to_string()) {
            continue;
        }
        let base = pattern.trim_end_matches("/*").trim_end_matches("/**");
        let base_dir = if base.is_empty() {
            root.to_path_buf()
        } else {
            validate_manifest_relative_candidate(root, root, base, "package workspace base")?
        };
        match fs::symlink_metadata(&base_dir) {
            Ok(_) => validate_existing_path(root, &base_dir, "package workspace base", None)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP package workspace base {}: {error}",
                    base_dir.display()
                ));
            }
        }
        let Ok(canonical_base) = base_dir.canonicalize() else {
            continue;
        };
        if !canonical_base.is_dir() || !seen_bases.insert(canonical_base) {
            continue;
        }
        let entries = match fs::read_dir(&base_dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries {
            charge_manifest_traversal_entry(
                traversal_entries_seen,
                "MCP package workspace preflight",
            )?;
            let entry = entry.map_err(|error| {
                format!(
                    "Cannot inspect entry beneath MCP package workspace base {}: {error}",
                    base_dir.display()
                )
            })?;
            validate_existing_path(root, &entry.path(), "package workspace entry", None)?;
            let canonical_entry = entry.path().canonicalize().map_err(|error| {
                format!(
                    "Cannot resolve MCP package workspace entry {}: {error}",
                    entry.path().display()
                )
            })?;
            if !canonical_entry.is_dir() {
                continue;
            }
            if !seen_workspaces.insert(canonical_entry) {
                continue;
            }
            let nested_package = entry.path().join("package.json");
            match fs::symlink_metadata(&nested_package) {
                Ok(_) => {
                    validate_existing_path(
                        root,
                        &nested_package,
                        "package workspace manifest",
                        None,
                    )?;
                    let nested_content =
                        read_file_bounded(root, &nested_package, "package workspace manifest")?;
                    let nested_json: Value =
                        serde_json::from_str(&nested_content).map_err(|error| {
                            format!(
                                "Cannot parse MCP package workspace manifest {} as JSON: {error}",
                                nested_package.display()
                            )
                        })?;
                    package_workspace_declarations(
                        &nested_json,
                        &format!(
                            "MCP package workspace manifest {}",
                            nested_package.display()
                        ),
                    )?;
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(format!(
                        "Cannot inspect MCP package workspace manifest {}: {error}",
                        nested_package.display()
                    ));
                }
            }
        }
    }
    Ok(())
}

fn charge_manifest_traversal_entry(entries_seen: &mut usize, label: &str) -> Result<(), String> {
    *entries_seen = entries_seen.saturating_add(1);
    if *entries_seen > MAX_CONFINEMENT_ENTRIES {
        return Err(format!("{label} exceeds {MAX_CONFINEMENT_ENTRIES} entries"));
    }
    Ok(())
}

fn validate_gradle_modules(root: &Path) -> Result<(), String> {
    let settings_path = ["settings.gradle.kts", "settings.gradle"]
        .iter()
        .map(|name| root.join(name))
        .find(|path| path.exists());
    let Some(settings_path) = settings_path else {
        return Ok(());
    };
    let settings = read_file_bounded(root, &settings_path, "Gradle settings manifest")?;
    let modules = parse_gradle_settings(&settings)
        .map_err(|error| format!("MCP Gradle discovery is inconclusive: {error}"))?;
    for module in modules {
        validate_manifest_relative_candidate(root, root, &module.path, "Gradle module path")?;
    }
    Ok(())
}

fn validate_python_package_path(root: &Path) -> Result<(), String> {
    let src = root.join("src");
    match fs::symlink_metadata(&src) {
        Ok(_) => {
            validate_existing_path(root, &src, "Python source directory", None)?;
            return Ok(());
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "Cannot inspect MCP Python source directory {}: {error}",
                src.display()
            ));
        }
    }

    let pyproject_path = root.join("pyproject.toml");
    let content = match read_file_bounded(root, &pyproject_path, "Python workspace manifest") {
        Ok(content) => content,
        Err(_) if !pyproject_path.exists() => return Ok(()),
        Err(error) => return Err(error),
    };
    let name = extract_manifest_toml_value(&content, "name", "[project]")
        .or_else(|| extract_manifest_toml_value(&content, "name", "[tool.poetry]"));
    if let Some(name) = name {
        validate_manifest_relative_candidate(root, root, &name, "Python package path")?;
    }
    Ok(())
}

fn validate_manifest_relative_candidate(
    server_root: &Path,
    base: &Path,
    configured: &str,
    label: &str,
) -> Result<PathBuf, String> {
    let relative = Path::new(configured);
    if configured.is_empty()
        || relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!(
            "MCP {label} must be a non-empty project-relative path without traversal: {configured}"
        ));
    }
    let candidate = base.join(relative);
    validate_path_or_ancestor(server_root, &candidate, label, None)?;
    Ok(candidate)
}

fn manifest_toml_section<'a>(content: &'a str, header: &str) -> Option<&'a str> {
    let start = content.find(header)?;
    let after = &content[start + header.len()..];
    let end = after
        .find("\n[")
        .map(|position| position + 1)
        .unwrap_or(after.len());
    Some(&after[..end])
}

fn extract_manifest_toml_value(content: &str, key: &str, section: &str) -> Option<String> {
    let section = manifest_toml_section(content, section)?;
    for line in section.lines().map(str::trim) {
        let Some((candidate_key, value)) = line.split_once('=') else {
            continue;
        };
        if candidate_key.trim() != key {
            continue;
        }
        let value = value.trim();
        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            return Some(value[1..value.len() - 1].to_string());
        }
        return Some(value.to_string());
    }
    None
}

fn validate_configured_path_with_budget(
    root: &Path,
    configured: &str,
    label: &str,
    tree_exclusions: Option<&HashSet<String>>,
    confinement_entries_seen: &mut usize,
) -> Result<(), String> {
    let path = Path::new(configured);
    if configured.is_empty() || path.is_absolute() {
        return Err(format!(
            "MCP {label} must be a non-empty project-relative path: {configured}"
        ));
    }
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(format!(
            "MCP {label} must not contain traversal or an absolute prefix: {configured}"
        ));
    }

    validate_path_or_ancestor_with_budget(
        root,
        &root.join(path),
        label,
        tree_exclusions,
        confinement_entries_seen,
    )
}

fn validate_path_or_ancestor(
    root: &Path,
    candidate: &Path,
    label: &str,
    tree_exclusions: Option<&HashSet<String>>,
) -> Result<(), String> {
    let mut confinement_entries_seen = 0usize;
    validate_path_or_ancestor_with_budget(
        root,
        candidate,
        label,
        tree_exclusions,
        &mut confinement_entries_seen,
    )
}

fn validate_path_or_ancestor_with_budget(
    root: &Path,
    candidate: &Path,
    label: &str,
    tree_exclusions: Option<&HashSet<String>>,
    confinement_entries_seen: &mut usize,
) -> Result<(), String> {
    let mut current = candidate;
    loop {
        match fs::symlink_metadata(current) {
            Ok(_) => {
                let exclusions = (current == candidate).then_some(tree_exclusions).flatten();
                validate_existing_path_with_budget(
                    root,
                    current,
                    label,
                    exclusions,
                    confinement_entries_seen,
                )?;
                return Ok(());
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                current = current.parent().ok_or_else(|| {
                    format!(
                        "MCP {label} has no resolvable ancestor: {}",
                        candidate.display()
                    )
                })?;
            }
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP {label} {}: {error}",
                    current.display()
                ));
            }
        }
    }
}

fn validate_existing_path(
    root: &Path,
    candidate: &Path,
    label: &str,
    tree_exclusions: Option<&HashSet<String>>,
) -> Result<(), String> {
    let mut confinement_entries_seen = 0usize;
    validate_existing_path_with_budget(
        root,
        candidate,
        label,
        tree_exclusions,
        &mut confinement_entries_seen,
    )
}

fn validate_existing_path_with_budget(
    root: &Path,
    candidate: &Path,
    label: &str,
    tree_exclusions: Option<&HashSet<String>>,
    confinement_entries_seen: &mut usize,
) -> Result<(), String> {
    let canonical_root = root.canonicalize().map_err(|error| {
        format!(
            "Cannot resolve MCP server root {} while validating {label}: {error}",
            root.display()
        )
    })?;
    let canonical = candidate.canonicalize().map_err(|error| {
        format!(
            "Cannot resolve MCP {label} {} (including symlink targets): {error}",
            candidate.display()
        )
    })?;
    if !canonical.starts_with(&canonical_root) {
        return Err(format!(
            "MCP {label} escapes the configured server root: {}",
            candidate.display()
        ));
    }
    let metadata = fs::metadata(&canonical).map_err(|error| {
        format!(
            "Cannot inspect MCP {label} {} after canonicalization: {error}",
            candidate.display()
        )
    })?;
    if metadata.is_file() && metadata.len() > MAX_PROJECT_FILE_BYTES {
        return Err(format!(
            "MCP {label} exceeds the {} MiB project-file limit: {}",
            MAX_PROJECT_FILE_BYTES / (1024 * 1024),
            candidate.display()
        ));
    }
    if let Some(exclusions) = tree_exclusions
        && canonical.is_dir()
    {
        validate_tree_confinement(
            &canonical_root,
            &canonical,
            label,
            exclusions,
            confinement_entries_seen,
        )?;
    }
    Ok(())
}

fn read_file_bounded(root: &Path, path: &Path, label: &str) -> Result<String, String> {
    validate_existing_path(root, path, label, None)?;
    let metadata = fs::metadata(path)
        .map_err(|error| format!("Cannot inspect MCP {label} {}: {error}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("MCP {label} is not a file: {}", path.display()));
    }
    if metadata.len() > MAX_PROJECT_FILE_BYTES {
        return Err(format!(
            "MCP {label} exceeds the {} MiB project-file limit: {}",
            MAX_PROJECT_FILE_BYTES / (1024 * 1024),
            path.display()
        ));
    }

    let file = fs::File::open(path)
        .map_err(|error| format!("Cannot read MCP {label} {}: {error}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(MAX_PROJECT_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Cannot read MCP {label} {}: {error}", path.display()))?;
    if bytes.len() as u64 > MAX_PROJECT_FILE_BYTES {
        return Err(format!(
            "MCP {label} exceeds the {} MiB project-file limit: {}",
            MAX_PROJECT_FILE_BYTES / (1024 * 1024),
            path.display()
        ));
    }
    String::from_utf8(bytes)
        .map_err(|_| format!("MCP {label} is not valid UTF-8: {}", path.display()))
}

fn validate_project_input_budget(root: &Path, config: &SpecSyncConfig) -> Result<(), String> {
    let mut candidates = vec![root.join(&config.specs_dir)];
    candidates.extend(config.source_dirs.iter().map(|path| root.join(path)));
    if let Some(schema_dir) = config.schema_dir.as_deref()
        && !schema_dir.is_empty()
    {
        candidates.push(root.join(schema_dir));
    }
    for module in config.modules.values() {
        candidates.extend(module.files.iter().map(|path| root.join(path)));
    }

    let mut exclusions: HashSet<String> = AUTODETECT_IGNORED_DIRS
        .iter()
        .map(|name| (*name).to_string())
        .collect();
    exclusions.extend(config.exclude_dirs.iter().cloned());
    let canonical_root = root
        .canonicalize()
        .map_err(|error| format!("Cannot resolve MCP server root {}: {error}", root.display()))?;
    let mut seen_files = HashSet::new();
    let mut total_bytes = 0u64;
    let mut entries_seen = 0usize;

    for candidate in candidates {
        if !candidate.exists() {
            continue;
        }
        let entries = WalkDir::new(&candidate)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| {
                entry.depth() == 0
                    || !entry.file_type().is_dir()
                    || !entry
                        .file_name()
                        .to_str()
                        .is_some_and(|name| exclusions.contains(name))
            });
        for entry in entries {
            entries_seen += 1;
            if entries_seen > MAX_CONFINEMENT_ENTRIES {
                return Err(format!(
                    "MCP project input scan exceeds {MAX_CONFINEMENT_ENTRIES} entries"
                ));
            }
            let entry =
                entry.map_err(|error| format!("Cannot inspect MCP project input: {error}"))?;
            if !entry.file_type().is_file() {
                continue;
            }
            let canonical = entry.path().canonicalize().map_err(|error| {
                format!(
                    "Cannot resolve MCP project input {}: {error}",
                    entry.path().display()
                )
            })?;
            if !canonical.starts_with(&canonical_root) {
                return Err(format!(
                    "MCP project input escapes the configured server root: {}",
                    entry.path().display()
                ));
            }
            if !seen_files.insert(canonical) {
                continue;
            }
            let bytes = entry
                .metadata()
                .map_err(|error| {
                    format!(
                        "Cannot inspect MCP project input {}: {error}",
                        entry.path().display()
                    )
                })?
                .len();
            if bytes > MAX_PROJECT_FILE_BYTES {
                return Err(format!(
                    "MCP project input exceeds the {} MiB per-file limit: {}",
                    MAX_PROJECT_FILE_BYTES / (1024 * 1024),
                    entry.path().display()
                ));
            }
            total_bytes = total_bytes.saturating_add(bytes);
            if total_bytes > MAX_PROJECT_INPUT_BYTES {
                return Err(format!(
                    "MCP project inputs exceed the {} MiB cumulative limit",
                    MAX_PROJECT_INPUT_BYTES / (1024 * 1024)
                ));
            }
        }
    }

    Ok(())
}

fn validate_tree_confinement(
    root: &Path,
    start: &Path,
    label: &str,
    exclusions: &HashSet<String>,
    entries_seen: &mut usize,
) -> Result<(), String> {
    let entries = WalkDir::new(start)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            entry.depth() == 0
                || !entry.file_type().is_dir()
                || !entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| exclusions.contains(name))
        });

    for entry in entries {
        *entries_seen += 1;
        if *entries_seen > MAX_CONFINEMENT_ENTRIES {
            return Err(format!(
                "MCP {label} confinement scan exceeds {MAX_CONFINEMENT_ENTRIES} entries"
            ));
        }
        let entry =
            entry.map_err(|error| format!("Cannot inspect entry beneath MCP {label}: {error}"))?;
        if !entry.file_type().is_symlink() {
            continue;
        }
        let canonical = entry.path().canonicalize().map_err(|error| {
            format!(
                "Cannot resolve symlink beneath MCP {label} {}: {error}",
                entry.path().display()
            )
        })?;
        if !canonical.starts_with(root) {
            return Err(format!(
                "Symlink beneath MCP {label} escapes the configured server root: {}",
                entry.path().display()
            ));
        }
    }

    Ok(())
}

fn validate_spec_file_mappings(
    root: &Path,
    spec_files: &[PathBuf],
    exclude_dirs: &[String],
) -> Result<(), String> {
    let mut confinement_entries_seen = 0usize;
    let exclusions: HashSet<String> = exclude_dirs.iter().cloned().collect();
    for spec_file in spec_files {
        validate_existing_path(root, spec_file, "spec file", None)?;
        let content = read_file_bounded(root, spec_file, "spec file")?;
        let Some(parsed) = crate::parser::parse_frontmatter(&content) else {
            continue;
        };
        for file in &parsed.frontmatter.files {
            validate_configured_path_with_budget(
                root,
                file,
                "spec file mapping",
                Some(&exclusions),
                &mut confinement_entries_seen,
            )?;
        }
        for dependency in &parsed.frontmatter.depends_on {
            if crate::validator::is_cross_project_ref(dependency) {
                continue;
            }
            if !dependency.contains('/') && !dependency.contains('.') {
                validate_module_name(dependency)?;
            } else {
                validate_configured_path_with_budget(
                    root,
                    dependency,
                    "dependency reference",
                    None,
                    &mut confinement_entries_seen,
                )?;
            }
        }
        validate_consumed_by_paths(root, &parsed.body, &mut confinement_entries_seen)?;
    }
    Ok(())
}

fn validate_consumed_by_paths(
    root: &Path,
    body: &str,
    confinement_entries_seen: &mut usize,
) -> Result<(), String> {
    let mut in_section = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed == "### Consumed By" {
            in_section = true;
            continue;
        }
        if in_section && (trimmed.starts_with("## ") || trimmed.starts_with("### ")) {
            break;
        }
        if !in_section || !trimmed.starts_with('|') {
            continue;
        }
        for (index, value) in line.split('`').enumerate() {
            if index % 2 == 1 && value.rsplit_once('.').is_some() {
                validate_configured_path_with_budget(
                    root,
                    value,
                    "Consumed By file reference",
                    None,
                    confinement_entries_seen,
                )?;
            }
        }
    }
    Ok(())
}

fn validate_module_name(module_name: &str) -> Result<(), String> {
    crate::commands::validate_module_name(module_name)
        .map_err(|error| format!("MCP module name is unsafe: {error}"))
}

fn validate_generated_module_names(module_names: &[String]) -> Result<(), String> {
    for module_name in module_names {
        validate_module_name(module_name)?;
    }
    Ok(())
}

fn tool_check(root: &Path, arguments: &Value) -> Result<Value, String> {
    let (config, spec_files) = load_and_discover(root, false)?;
    let strict = arguments
        .get("strict")
        .and_then(|s| s.as_bool())
        .unwrap_or(false);

    // Classify changes for staleness detection
    let cache = crate::hash_cache::HashCache::load(root);
    let classifications = crate::hash_cache::classify_all_changes(root, &spec_files, &cache);
    let mut stale_entries: Vec<Value> = Vec::new();
    for classification in &classifications {
        let spec_rel = classification
            .spec_path
            .strip_prefix(root)
            .unwrap_or(&classification.spec_path)
            .to_string_lossy()
            .to_string();
        if classification.has(&crate::hash_cache::ChangeKind::Requirements) {
            stale_entries.push(json!({
                "spec": spec_rel,
                "reason": "requirements_changed",
                "message": "requirements changed — spec may need re-validation"
            }));
        }
    }

    // Same collector the coverage payloads use — one validation implementation
    // for every MCP surface, so a finding cannot exist in one and not another.
    let collected = validate_all(root, &config, &spec_files);
    let total_errors = collected.total_errors;
    let total_warnings = collected.total_warnings;
    let passed = collected.specs_passed;
    let all_errors = collected.errors;
    let mut all_warnings = collected.warnings;
    let spec_results = collected.specs;

    // Add staleness warnings into the warnings array for consistency
    for entry in &stale_entries {
        if let Some(msg) = entry["message"].as_str() {
            let spec = entry["spec"].as_str().unwrap_or("unknown");
            all_warnings.push(format!("{spec}: {msg}"));
        }
    }

    let coverage = compute_coverage_checked(root, &spec_files, &config)
        .map_err(|error| format!("MCP coverage is inconclusive: {error}"))?;
    let staleness_warnings = stale_entries.len();
    let effective_warnings = total_warnings + staleness_warnings;
    let overall_passed = total_errors == 0 && (!strict || effective_warnings == 0);

    Ok(json!({
        "passed": overall_passed,
        "specs_checked": spec_files.len(),
        "specs_passed": passed,
        "total_errors": total_errors,
        "total_warnings": effective_warnings,
        "errors": all_errors,
        "warnings": all_warnings,
        "stale": stale_entries,
        "specs": spec_results,
        "coverage": {
            // Null, not 100, when nothing was measured (#582).
            "file_percent": coverage.file_coverage_percent(),
            "loc_percent": coverage.loc_coverage_percent(),
        }
    }))
}

fn tool_coverage(root: &Path) -> Result<Value, String> {
    mcp_coverage_payload(root)
}

fn tool_generate(root: &Path, write_directory: &Dir, arguments: &Value) -> Result<Value, String> {
    const RETIRED_ARGUMENTS: &[&str] = &[
        "ai",
        "provider",
        "aiProvider",
        "ai_provider",
        "model",
        "aiModel",
        "ai_model",
        "apiKey",
        "api_key",
        "aiApiKey",
        "ai_api_key",
        "credential",
        "credentials",
        "baseUrl",
        "base_url",
        "aiBaseUrl",
        "ai_base_url",
        "timeout",
        "timeoutSecs",
        "timeout_secs",
        "aiTimeout",
        "ai_timeout",
        "command",
        "aiCommand",
        "ai_command",
    ];
    if let Some(name) = RETIRED_ARGUMENTS
        .iter()
        .find(|name| arguments.get(**name).is_some())
    {
        return Err(format!(
            "MCP argument `{name}` was removed in spec-sync 5.0. `specsync_generate` is deterministic; use your coding agent to enrich the generated spec."
        ));
    }
    if let Some(name) = arguments
        .as_object()
        .and_then(|object| object.keys().find(|name| name.as_str() != "root"))
    {
        return Err(format!(
            "Unknown MCP generate argument `{name}`. `specsync_generate` accepts only `root`."
        ));
    }

    let (config, spec_files) = load_and_discover(root, true)?;
    let coverage = compute_coverage_checked(root, &spec_files, &config)
        .map_err(|error| format!("MCP coverage is inconclusive: {error}"))?;
    if coverage.unspecced_modules.len() > MAX_GENERATED_SPECS {
        return Err(format!(
            "MCP generation exceeds the {MAX_GENERATED_SPECS}-spec output limit"
        ));
    }
    validate_generated_module_names(&coverage.unspecced_modules)?;
    let mut expected_destinations = Vec::new();
    for module_name in &coverage.unspecced_modules {
        let destination = root
            .join(&config.specs_dir)
            .join(module_name)
            .join(format!("{module_name}.spec.md"));
        validate_path_or_ancestor(root, &destination, "generation destination", None)?;
        match fs::symlink_metadata(&destination) {
            Ok(_) => {
                return Err(format!(
                    "MCP generation destination already exists for uncovered module `{module_name}`: {}",
                    destination.display()
                ));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP generation destination {}: {error}",
                    destination.display()
                ));
            }
        }
        expected_destinations.push((module_name.clone(), destination));
    }
    let outcome = generate_specs_for_unspecced_modules_paths(root, &coverage, &config);

    let generated_specs = validate_generation_outcome(root, &expected_destinations, &outcome)?;
    let generated_paths: Vec<String> = generated_specs
        .iter()
        .map(|(path, _)| path.to_string_lossy().to_string())
        .collect();
    let result = json!({
        "generated": generated_paths,
        "count": outcome.generated,
    });
    validate_tool_content_response_size(&result)?;
    write_generated_specs_transactionally(write_directory, generated_specs)?;

    Ok(result)
}

fn validate_generation_outcome(
    root: &Path,
    expected_destinations: &[(String, PathBuf)],
    outcome: &crate::generator::GenerationOutcome,
) -> Result<Vec<(PathBuf, String)>, String> {
    validate_generation_outcome_with_limits(
        root,
        expected_destinations,
        outcome,
        MAX_GENERATED_SPECS,
        MAX_GENERATED_OUTPUT_BYTES,
    )
}

fn validate_generation_outcome_with_limits(
    root: &Path,
    expected_destinations: &[(String, PathBuf)],
    outcome: &crate::generator::GenerationOutcome,
    max_generated_specs: usize,
    max_generated_output_bytes: u64,
) -> Result<Vec<(PathBuf, String)>, String> {
    if expected_destinations.len() > max_generated_specs || outcome.generated > max_generated_specs
    {
        return Err(format!(
            "MCP generation exceeds the {max_generated_specs}-spec output limit"
        ));
    }
    if outcome.generated != expected_destinations.len() {
        return Err(format!(
            "MCP generation created {} of {} required specs",
            outcome.generated,
            expected_destinations.len()
        ));
    }
    let mut generated_specs = Vec::with_capacity(expected_destinations.len());
    let mut generated_bytes = 0u64;
    for (module_name, destination) in expected_destinations {
        if !destination.is_file() {
            return Err(format!(
                "MCP generation did not create the required spec for module `{module_name}`: {}",
                destination.display()
            ));
        }
        let content = read_file_bounded(root, destination, "generated spec")?;
        let relative = destination.strip_prefix(root).map_err(|_| {
            format!(
                "Generated MCP spec is not relative to the operation root: {}",
                destination.display()
            )
        })?;
        generated_bytes = generated_bytes.saturating_add(content.len() as u64);
        if generated_bytes > max_generated_output_bytes {
            return Err(format!(
                "MCP generated specs exceed the {} MiB cumulative output limit",
                max_generated_output_bytes / (1024 * 1024)
            ));
        }
        generated_specs.push((relative.to_path_buf(), content));
    }
    let expected_paths: Vec<String> = generated_specs
        .iter()
        .map(|(path, _)| path.to_string_lossy().to_string())
        .collect();
    if outcome.generated_paths != expected_paths {
        return Err("MCP generator reported paths that do not match its confined outputs".into());
    }
    Ok(generated_specs)
}

fn validate_tool_content_response_size(content: &Value) -> Result<(), String> {
    validate_tool_content_response_size_with_limit(content, MAX_TOOL_CONTENT_RESPONSE_BYTES)
}

fn validate_tool_content_response_size_with_limit(
    content: &Value,
    max_response_bytes: usize,
) -> Result<(), String> {
    let text = serde_json::to_string_pretty(content)
        .map_err(|error| format!("Cannot encode MCP tool result: {error}"))?;
    let response = json!({
        "jsonrpc": "2.0",
        "id": null,
        "result": { "content": [{ "type": "text", "text": text }] }
    });
    let encoded = serde_json::to_vec(&response)
        .map_err(|error| format!("Cannot preflight MCP tool response: {error}"))?;
    if encoded.len() > max_response_bytes {
        return Err("MCP generated-spec result exceeds the bounded response limit".into());
    }
    Ok(())
}

fn tool_list_specs(root: &Path) -> Result<Value, String> {
    let (_config, spec_files) = load_and_discover(root, true)?;

    let specs: Vec<Value> = spec_files
        .iter()
        .map(|f| -> Result<Value, String> {
            let content = read_file_bounded(root, f, "spec file")?;
            let parsed = crate::parser::parse_frontmatter(&content);
            let relative = f
                .strip_prefix(root)
                .unwrap_or(f)
                .to_string_lossy()
                .to_string();

            if let Some(parsed) = parsed {
                let fm = parsed.frontmatter;
                Ok(json!({
                    "path": relative,
                    "module": fm.module,
                    "version": fm.version,
                    "status": fm.status,
                    "files": fm.files,
                }))
            } else {
                Ok(json!({
                    "path": relative,
                    "module": null,
                    "version": null,
                    "status": null,
                    "files": [],
                }))
            }
        })
        .collect::<Result<_, _>>()?;

    Ok(json!({
        "specs": specs,
        "count": specs.len(),
    }))
}

fn tool_init(root: &Path, write_directory: &Dir) -> Result<Value, String> {
    validate_known_config_files(root)?;
    if write_directory
        .try_exists("specsync.json")
        .map_err(|error| format!("Cannot inspect MCP initialization destination: {error}"))?
    {
        return Ok(json!({
            "created": false,
            "message": "specsync.json already exists"
        }));
    }

    validate_autodetection_tree(root)?;
    validate_manifest_inputs(root)?;
    let detected_dirs = detect_source_dirs(root);

    let default = json!({
        "specsDir": "specs",
        "sourceDirs": detected_dirs,
        "requiredSections": [
            "Purpose",
            "Public API",
            "Invariants",
            "Behavioral Examples",
            "Error Cases",
            "Dependencies",
            "Change Log"
        ],
        "excludeDirs": ["__tests__"],
        "excludePatterns": ["**/__tests__/**", "**/*.test.ts", "**/*.spec.ts"]
    });

    let content = serde_json::to_string_pretty(&default).unwrap() + "\n";
    write_new_confined(
        write_directory,
        Path::new("specsync.json"),
        content.as_bytes(),
        "initialization destination",
    )?;

    Ok(json!({
        "created": true,
        "source_dirs": detected_dirs,
        "message": "Created specsync.json"
    }))
}

fn write_new_confined(
    directory: &Dir,
    relative: &Path,
    content: &[u8],
    label: &str,
) -> Result<(), String> {
    let mut staged = stage_new_confined(directory, relative, content, label)?;
    if let Err(error) = publish_staged_file(&mut staged, label) {
        return Err(rollback_staged_batch(&[staged], error));
    }
    Ok(())
}

fn write_generated_specs_transactionally(
    directory: &Dir,
    generated_specs: Vec<(PathBuf, String)>,
) -> Result<(), String> {
    if generated_specs.len() > MAX_GENERATED_SPECS {
        return Err(format!(
            "MCP generation exceeds the {MAX_GENERATED_SPECS}-spec output limit"
        ));
    }
    let mut staged = Vec::with_capacity(generated_specs.len());
    for (relative, content) in generated_specs {
        match stage_new_confined(directory, &relative, content.as_bytes(), "generated spec") {
            Ok(file) => staged.push(file),
            Err(error) => {
                return Err(rollback_staged_batch(&staged, error));
            }
        }
    }

    for index in 0..staged.len() {
        if let Err(error) = publish_staged_file(&mut staged[index], "generated spec") {
            return Err(rollback_staged_batch(&staged, error));
        }
    }
    Ok(())
}

struct StagedFile<'root> {
    root: &'root Dir,
    parent: Dir,
    parent_path: PathBuf,
    parent_identity: DirectoryIdentity,
    destination_name: PathBuf,
    destination_display: PathBuf,
    temporary_name: PathBuf,
    temporary_display: PathBuf,
    identity: FileIdentity,
    temporary_present: bool,
    published: bool,
}

fn stage_new_confined<'root>(
    directory: &'root Dir,
    destination: &Path,
    content: &[u8],
    label: &str,
) -> Result<StagedFile<'root>, String> {
    stage_new_confined_with_hook(directory, destination, content, label, |_, _| {})
}

fn stage_new_confined_with_hook<'root, Hook>(
    directory: &'root Dir,
    destination: &Path,
    content: &[u8],
    label: &str,
    after_identity: Hook,
) -> Result<StagedFile<'root>, String>
where
    Hook: FnOnce(&Dir, &Path),
{
    validate_confined_relative(destination, label)?;
    let parent_path = destination.parent().unwrap_or_else(|| Path::new(""));
    let destination_name = destination
        .file_name()
        .map(PathBuf::from)
        .ok_or_else(|| format!("MCP {label} must name a destination file"))?;
    let parent = create_confined_directories(directory, parent_path, label)?;
    let parent_identity = directory_identity(&parent).map_err(|error| {
        format!(
            "Cannot identify confined MCP {label} parent {}: {error}",
            parent_path.display()
        )
    })?;

    for _ in 0..128 {
        let sequence = STAGED_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary_name = PathBuf::from(format!(
            ".specsync-mcp-stage-{}-{sequence}",
            std::process::id()
        ));
        let temporary_display = parent_path.join(&temporary_name);
        let mut options = OpenOptions::new();
        options.read(true).write(true).create_new(true);
        let mut file = match parent.open_with(&temporary_name, &options) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                let failure = format!(
                    "Cannot create confined MCP staged {label} {}: {error}",
                    temporary_display.display()
                );
                return Err(failure);
            }
        };
        let write_result = file
            .write_all(content)
            .and_then(|_| file.flush())
            .and_then(|_| file.sync_all());
        let (identity, identity_error) = match file_identity(&file) {
            Ok(identity) => (identity, None),
            Err(error) => (invalid_file_identity(), Some(error)),
        };
        drop(file);
        after_identity(&parent, &temporary_name);
        let staged = StagedFile {
            root: directory,
            parent,
            parent_path: parent_path.to_path_buf(),
            parent_identity,
            destination_name,
            destination_display: destination.to_path_buf(),
            temporary_name,
            temporary_display,
            identity,
            temporary_present: true,
            published: false,
        };
        if let Err(error) = write_result {
            let failure = format!(
                "Cannot stage confined MCP {label} {}: {error}",
                destination.display()
            );
            return Err(rollback_staged_batch(&[staged], failure));
        }
        if let Some(error) = identity_error {
            let failure = format!(
                "Cannot identify confined MCP staged {label} {}: {error}",
                destination.display()
            );
            return Err(rollback_staged_batch(&[staged], failure));
        }
        return Ok(staged);
    }

    let failure = format!(
        "Cannot reserve a unique staged MCP {label} beside {}",
        destination.display()
    );
    Err(failure)
}

#[cfg(unix)]
fn invalid_file_identity() -> FileIdentity {
    (u64::MAX, u64::MAX, [0_u8; 32])
}

#[cfg(windows)]
fn invalid_file_identity() -> FileIdentity {
    (u32::MAX, u64::MAX, [0_u8; 32])
}

#[cfg(not(any(unix, windows)))]
fn invalid_file_identity() -> FileIdentity {
    (u64::MAX, None, [0_u8; 32])
}

fn create_confined_directories(directory: &Dir, parent: &Path, label: &str) -> Result<Dir, String> {
    // Parent directory creation has no portable create-and-open primitive. Retain empty
    // parents after a failed batch rather than risk claiming and deleting a concurrently
    // replaced directory.
    let mut current_directory = directory
        .try_clone()
        .map_err(|error| format!("Cannot clone MCP root capability: {error}"))?;
    let mut current_display = PathBuf::new();
    for component in parent.components() {
        let name = PathBuf::from(component.as_os_str());
        current_display.push(&name);
        let exists = match current_directory.try_exists(&name) {
            Ok(exists) => exists,
            Err(error) => {
                return Err(format!(
                    "Cannot inspect confined MCP {label} directory {}: {error}",
                    current_display.display()
                ));
            }
        };
        if !exists {
            match current_directory.create_dir(&name) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => {
                    return Err(format!(
                        "Cannot create confined MCP {label} directory {}: {error}",
                        current_display.display()
                    ));
                }
            }
        }
        let next = match current_directory.open_dir(&name) {
            Ok(next) => next,
            Err(error) => {
                return Err(format!(
                    "Cannot open confined MCP {label} directory {}: {error}",
                    current_display.display()
                ));
            }
        };
        current_directory = next;
    }
    Ok(current_directory)
}

fn publish_staged_file(staged: &mut StagedFile<'_>, label: &str) -> Result<(), String> {
    publish_staged_file_with_hooks(staged, label, |_, _| {}, |_, _| {})
}

#[cfg(test)]
fn publish_staged_file_with_hook<Hook>(
    staged: &mut StagedFile<'_>,
    label: &str,
    before_quarantine: Hook,
) -> Result<(), String>
where
    Hook: FnOnce(&Dir, &Path),
{
    publish_staged_file_with_hooks(staged, label, before_quarantine, |_, _| {})
}

fn publish_staged_file_with_hooks<BeforeQuarantine, AfterHardLink>(
    staged: &mut StagedFile<'_>,
    label: &str,
    before_quarantine: BeforeQuarantine,
    after_hard_link: AfterHardLink,
) -> Result<(), String>
where
    BeforeQuarantine: FnOnce(&Dir, &Path),
    AfterHardLink: FnOnce(&Dir, &Path),
{
    publish_staged_file_with_verification_hooks(
        staged,
        label,
        before_quarantine,
        after_hard_link,
        file_identity,
    )
}
