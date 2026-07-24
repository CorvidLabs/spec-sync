use crate::config::{detect_source_dirs, load_config, parse_config_content_checked};
use crate::deps::build_dep_graph;
use crate::generator::generate_specs_for_unspecced_modules_paths;
use crate::manifest::{MAX_GRADLE_MANIFEST_BYTES, parse_gradle_settings};
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
            "initialize" => Some(handle_initialize(id)),
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
        && !(id.is_string() || id.is_number() || id.is_null())
    {
        return Err(invalid_request(
            "JSON-RPC id must be a string, number, or null",
        ));
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
        let directory = tempfile::Builder::new()
            .prefix("specsync-mcp-")
            .tempdir()
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

fn handle_tools_list(id: Option<Value>, allow_write: bool) -> Value {
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

    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": { "tools": tools }
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

    let operation_directory = if is_mutating {
        match server_directory.try_clone() {
            Ok(directory) => directory,
            Err(error) => {
                return tool_error(
                    id,
                    format!("Cannot clone MCP server root capability: {error}"),
                );
            }
        }
    } else {
        match resolve_read_root(
            server_root,
            requested_server_root,
            arguments.get("root").and_then(Value::as_str),
        ) {
            Ok(relative) => match server_directory.open_dir(&relative) {
                Ok(directory) => directory,
                Err(error) => {
                    return tool_error(
                        id,
                        format!(
                            "Read root override does not resolve to a confined existing directory {}: {error}",
                            relative.display()
                        ),
                    );
                }
            },
            Err(message) => return tool_error(id, message),
        }
    };
    let arguments = Value::Object(arguments.clone());
    let snapshot = match ProjectSnapshot::create_from_directory(&operation_directory) {
        Ok(snapshot) => snapshot,
        Err(message) => return tool_error(id, message),
    };
    let operation_root = snapshot.root();

    let result = match tool_name {
        "specsync_check" => tool_check(operation_root, &arguments),
        "specsync_coverage" => tool_coverage(operation_root),
        "specsync_generate" => tool_generate(operation_root, server_directory, &arguments),
        "specsync_list_specs" => tool_list_specs(operation_root),
        "specsync_init" => tool_init(operation_root, server_directory),
        "specsync_score" => tool_score(operation_root, snapshot.git_freshness_available()),
        "specsync_issues" => tool_issues(operation_root),
        _ => unreachable!("known tool was validated before dispatch"),
    };

    match result {
        Ok(content) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&content).unwrap_or_default()
                }]
            }
        }),
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

    let snapshot = match ProjectSnapshot::create_from_directory(directory) {
        Ok(snapshot) => snapshot,
        Err(message) => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32602, "message": message }
            });
        }
    };
    let operation_root = snapshot.root();
    let result = match uri {
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
    };

    match result {
        Ok((content, mime_type)) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "contents": [{
                    "uri": uri,
                    "mimeType": mime_type,
                    "text": content
                }]
            }
        }),
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
    let (config, spec_files) = load_and_discover(root, true)?;
    let coverage = compute_coverage_checked(root, &spec_files, &config)
        .map_err(|error| format!("MCP coverage is inconclusive: {error}"))?;

    let file_coverage = if coverage.total_source_files == 0 {
        100.0
    } else {
        (coverage.specced_file_count as f64 / coverage.total_source_files as f64) * 100.0
    };

    let loc_coverage = if coverage.total_loc == 0 {
        100.0
    } else {
        (coverage.specced_loc as f64 / coverage.total_loc as f64) * 100.0
    };

    let uncovered_modules: Vec<Value> = coverage
        .unspecced_modules
        .iter()
        .map(|m| json!({ "name": m }))
        .collect();

    let uncovered_files: Vec<Value> = coverage
        .unspecced_file_loc
        .iter()
        .map(|(f, loc)| json!({ "file": f, "loc": loc }))
        .collect();

    let output = json!({
        "file_coverage_percent": (file_coverage * 100.0).round() / 100.0,
        "files_covered": coverage.specced_file_count,
        "files_total": coverage.total_source_files,
        "loc_coverage_percent": (loc_coverage * 100.0).round() / 100.0,
        "loc_covered": coverage.specced_loc,
        "loc_total": coverage.total_loc,
        "uncovered_modules": uncovered_modules,
        "uncovered_files": uncovered_files,
    });

    Ok((
        serde_json::to_string_pretty(&output).unwrap(),
        "application/json",
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
    let schema_tables = get_schema_table_names(root, &config);
    let schema_columns = config
        .schema_dir
        .as_deref()
        .map(|schema_dir| crate::schema::build_schema(&root.join(schema_dir)))
        .unwrap_or_default();
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

    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut passed = 0;
    let mut all_errors: Vec<Value> = Vec::new();
    let mut all_warnings: Vec<Value> = Vec::new();
    let mut spec_results: Vec<Value> = Vec::new();

    for spec_file in &spec_files {
        let result = validate_spec(spec_file, root, &schema_tables, &schema_columns, &config);
        let spec_passed = result.errors.is_empty();

        spec_results.push(json!({
            "spec": result.spec_path,
            "passed": spec_passed,
            "errors": result.errors,
            "warnings": result.warnings,
            "export_summary": result.export_summary,
        }));

        for e in &result.errors {
            all_errors.push(json!(format!("{}: {e}", result.spec_path)));
        }
        for w in &result.warnings {
            all_warnings.push(json!(format!("{}: {w}", result.spec_path)));
        }

        total_errors += result.errors.len();
        total_warnings += result.warnings.len();
        if spec_passed {
            passed += 1;
        }
    }

    // Add staleness warnings into the warnings array for consistency
    for entry in &stale_entries {
        if let Some(msg) = entry["message"].as_str() {
            let spec = entry["spec"].as_str().unwrap_or("unknown");
            all_warnings.push(json!(format!("{spec}: {msg}")));
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
            "file_percent": coverage.coverage_percent,
            "loc_percent": coverage.loc_coverage_percent,
        }
    }))
}

fn tool_coverage(root: &Path) -> Result<Value, String> {
    let (config, spec_files) = load_and_discover(root, true)?;
    let coverage = compute_coverage_checked(root, &spec_files, &config)
        .map_err(|error| format!("MCP coverage is inconclusive: {error}"))?;

    let file_coverage = if coverage.total_source_files == 0 {
        100.0
    } else {
        (coverage.specced_file_count as f64 / coverage.total_source_files as f64) * 100.0
    };

    let loc_coverage = if coverage.total_loc == 0 {
        100.0
    } else {
        (coverage.specced_loc as f64 / coverage.total_loc as f64) * 100.0
    };

    let modules: Vec<Value> = coverage
        .unspecced_modules
        .iter()
        .map(|m| json!({ "name": m, "has_spec": false }))
        .collect();

    let uncovered_files: Vec<Value> = coverage
        .unspecced_file_loc
        .iter()
        .map(|(f, loc)| json!({ "file": f, "loc": loc }))
        .collect();

    Ok(json!({
        "file_coverage": (file_coverage * 100.0).round() / 100.0,
        "files_covered": coverage.specced_file_count,
        "files_total": coverage.total_source_files,
        "loc_coverage": (loc_coverage * 100.0).round() / 100.0,
        "loc_covered": coverage.specced_loc,
        "loc_total": coverage.total_loc,
        "uncovered_modules": modules,
        "uncovered_files": uncovered_files,
    }))
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

struct StagedFile {
    parent: Dir,
    destination_name: PathBuf,
    destination_display: PathBuf,
    temporary_name: PathBuf,
    temporary_display: PathBuf,
    identity: FileIdentity,
    temporary_present: bool,
    published: bool,
}

fn stage_new_confined(
    directory: &Dir,
    destination: &Path,
    content: &[u8],
    label: &str,
) -> Result<StagedFile, String> {
    stage_new_confined_with_hook(directory, destination, content, label, |_, _| {})
}

fn stage_new_confined_with_hook<Hook>(
    directory: &Dir,
    destination: &Path,
    content: &[u8],
    label: &str,
    after_identity: Hook,
) -> Result<StagedFile, String>
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
            parent,
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

fn publish_staged_file(staged: &mut StagedFile, label: &str) -> Result<(), String> {
    publish_staged_file_with_hook(staged, label, |_, _| {})
}

fn publish_staged_file_with_hook<Hook>(
    staged: &mut StagedFile,
    label: &str,
    before_quarantine: Hook,
) -> Result<(), String>
where
    Hook: FnOnce(&Dir, &Path),
{
    let temporary = staged
        .parent
        .open(&staged.temporary_name)
        .map_err(|error| {
            format!(
                "Cannot open staged MCP {label} {} before publication: {error}",
                staged.temporary_display.display()
            )
        })?;
    let observed = file_identity(&temporary).map_err(|error| {
        format!(
            "Cannot identify staged MCP {label} {} before publication: {error}",
            staged.temporary_display.display()
        )
    })?;
    if observed != staged.identity {
        return Err(format!(
            "Refusing to publish replacement staged MCP {label} {}",
            staged.temporary_display.display()
        ));
    }
    drop(temporary);
    before_quarantine(&staged.parent, &staged.temporary_name);

    let quarantined = quarantine_entry(
        &staged.parent,
        &staged.temporary_name,
        &staged.temporary_display,
        "staged file",
    )?;
    staged.temporary_present = false;
    let quarantined_file = quarantined.directory.open("entry").map_err(|error| {
        format!(
            "Cannot open quarantined staged MCP {label} {}: {error}",
            staged.temporary_display.display()
        )
    })?;
    let quarantined_identity = file_identity(&quarantined_file).map_err(|error| {
        format!(
            "Cannot identify quarantined staged MCP {label} {}: {error}",
            staged.temporary_display.display()
        )
    })?;
    if quarantined_identity != staged.identity {
        return Err(format!(
            "Refusing to publish replaced staged MCP {label} {}; quarantined replacement was preserved",
            staged.temporary_display.display()
        ));
    }

    drop(quarantined_file);

    if let Err(error) =
        quarantined
            .directory
            .hard_link("entry", &staged.parent, &staged.destination_name)
    {
        let failure = format!(
            "Cannot atomically publish confined MCP {label} {}: {error}",
            staged.destination_display.display()
        );
        return match cleanup_quarantined_file(
            quarantined,
            staged.identity,
            &staged.temporary_display,
        ) {
            Ok(()) => Err(failure),
            Err(cleanup) => Err(format!("{failure}; {cleanup}")),
        };
    }
    staged.published = true;
    let destination = staged
        .parent
        .open(&staged.destination_name)
        .map_err(|error| {
            format!(
                "Cannot verify published MCP {label} {}: {error}",
                staged.destination_display.display()
            )
        })?;
    let destination_identity = file_identity(&destination).map_err(|error| {
        format!(
            "Cannot identify published MCP {label} {}: {error}",
            staged.destination_display.display()
        )
    })?;
    if destination_identity != staged.identity {
        return Err(format!(
            "Published MCP {label} {} was replaced before identity verification",
            staged.destination_display.display()
        ));
    }
    drop(destination);
    cleanup_quarantined_file(quarantined, staged.identity, &staged.temporary_display)
}

fn validate_confined_relative(relative: &Path, label: &str) -> Result<(), String> {
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!(
            "MCP {label} must be a project-relative path without traversal: {}",
            relative.display()
        ));
    }
    Ok(())
}

fn remove_identity_bound_file(
    parent: &Dir,
    name: &Path,
    display: &Path,
    expected: FileIdentity,
) -> Result<(), String> {
    remove_identity_bound_file_with_hook(parent, name, display, expected, |_, _| {})
}

fn remove_identity_bound_file_with_hook<Hook>(
    parent: &Dir,
    name: &Path,
    display: &Path,
    expected: FileIdentity,
    before_quarantine: Hook,
) -> Result<(), String>
where
    Hook: FnOnce(&Dir, &Path),
{
    let file = match parent.open(name) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("{}: {error}", display.display())),
    };
    let observed = file_identity(&file).map_err(|error| {
        format!(
            "Cannot identify {} during rollback: {error}",
            display.display()
        )
    })?;
    if observed != expected {
        return Err(format!(
            "Refusing to remove replacement file {} during rollback",
            display.display()
        ));
    }
    drop(file);
    before_quarantine(parent, name);
    let quarantined = match quarantine_entry(parent, name, display, "file") {
        Ok(quarantined) => quarantined,
        Err(error) if error.contains("not found") => return Ok(()),
        Err(error) => return Err(error),
    };
    let file = quarantined.directory.open("entry").map_err(|error| {
        format!(
            "Cannot open quarantined {} during rollback: {error}",
            display.display()
        )
    })?;
    let observed = file_identity(&file).map_err(|error| {
        format!(
            "Cannot identify quarantined {} during rollback: {error}",
            display.display()
        )
    })?;
    if observed != expected {
        return Err(format!(
            "Refusing to remove replacement file {} during rollback; quarantined replacement was preserved",
            display.display()
        ));
    }
    drop(file);
    cleanup_quarantined_file(quarantined, expected, display)
}

struct QuarantinedEntry {
    directory: Dir,
    directory_identity: DirectoryIdentity,
}

fn quarantine_entry(
    parent: &Dir,
    name: &Path,
    display: &Path,
    kind: &str,
) -> Result<QuarantinedEntry, String> {
    for _ in 0..128 {
        let sequence = STAGED_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let directory_name = PathBuf::from(format!(
            ".specsync-mcp-quarantine-{}-{sequence}",
            std::process::id()
        ));
        match parent.create_dir(&directory_name) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "Cannot reserve rollback quarantine for {}: {error}",
                    display.display()
                ));
            }
        }
        let directory = parent.open_dir(&directory_name).map_err(|error| {
            format!(
                "Cannot open rollback quarantine for {}: {error}",
                display.display()
            )
        })?;
        let directory_identity = match directory_identity(&directory) {
            Ok(identity) => identity,
            Err(error) => {
                let failure = format!(
                    "Cannot identify rollback quarantine for {}: {error}",
                    display.display()
                );
                return match directory.remove_open_dir() {
                    Ok(()) => Err(failure),
                    Err(cleanup) => Err(format!(
                        "{failure}; cannot remove unidentifiable rollback quarantine: {cleanup}"
                    )),
                };
            }
        };
        if let Err(error) = parent.rename(name, &directory, "entry") {
            let failure = format!(
                "Cannot atomically quarantine {kind} {} before deletion: {error}",
                display.display()
            );
            return match directory.remove_open_dir() {
                Ok(()) => Err(failure),
                Err(cleanup) => Err(format!(
                    "{failure}; cannot remove unused rollback quarantine: {cleanup}"
                )),
            };
        }
        return Ok(QuarantinedEntry {
            directory,
            directory_identity,
        });
    }
    Err(format!(
        "Cannot reserve a unique rollback quarantine for {}",
        display.display()
    ))
}

fn cleanup_quarantined_file(
    quarantined: QuarantinedEntry,
    expected: FileIdentity,
    display: &Path,
) -> Result<(), String> {
    let file = quarantined
        .directory
        .open("entry")
        .map_err(|error| format!("Cannot reopen quarantined {}: {error}", display.display()))?;
    if file_identity(&file).map_err(|error| {
        format!(
            "Cannot reidentify quarantined {}: {error}",
            display.display()
        )
    })? != expected
    {
        return Err(format!(
            "Refusing to delete a replacement in the private quarantine for {}",
            display.display()
        ));
    }
    drop(file);
    quarantined
        .directory
        .remove_file("entry")
        .map_err(|error| format!("Cannot remove quarantined {}: {error}", display.display()))?;
    cleanup_quarantine_directory(quarantined, display)
}

fn cleanup_quarantine_directory(
    quarantined: QuarantinedEntry,
    display: &Path,
) -> Result<(), String> {
    let observed = directory_identity(&quarantined.directory).map_err(|error| {
        format!(
            "Cannot reidentify rollback quarantine for {}: {error}",
            display.display()
        )
    })?;
    if observed != quarantined.directory_identity {
        return Err(format!(
            "Refusing to remove a replaced rollback quarantine for {}",
            display.display()
        ));
    }
    quarantined.directory.remove_open_dir().map_err(|error| {
        format!(
            "Cannot remove rollback quarantine for {}: {error}",
            display.display()
        )
    })
}

fn rollback_staged_batch(staged: &[StagedFile], error: String) -> String {
    let mut cleanup_failures = Vec::new();
    for file in staged.iter().rev() {
        if file.published
            && let Err(cleanup_error) = remove_identity_bound_file(
                &file.parent,
                &file.destination_name,
                &file.destination_display,
                file.identity,
            )
        {
            cleanup_failures.push(cleanup_error);
        }
        if file.temporary_present
            && let Err(cleanup_error) = remove_identity_bound_file(
                &file.parent,
                &file.temporary_name,
                &file.temporary_display,
                file.identity,
            )
        {
            cleanup_failures.push(cleanup_error);
        }
    }
    append_rollback_failures(error, cleanup_failures)
}

fn append_rollback_failures(error: String, cleanup_failures: Vec<String>) -> String {
    if cleanup_failures.is_empty() {
        error
    } else {
        format!(
            "{error}; failed to roll back generated outputs: {}",
            cleanup_failures.join(", ")
        )
    }
}

fn tool_score(root: &Path, git_freshness_available: bool) -> Result<Value, String> {
    let (config, spec_files) = load_and_discover(root, false)?;

    let scores: Vec<scoring::SpecScore> = spec_files
        .iter()
        .map(|f| score_spec_for_mcp(f, root, &config, git_freshness_available))
        .collect();
    let project = scoring::compute_project_score(scores);

    let specs_json: Vec<Value> = project
        .spec_scores
        .iter()
        .map(|s| {
            json!({
                "spec": s.spec_path,
                "total": s.total,
                "grade": s.grade,
                "frontmatter": s.frontmatter_score,
                "sections": s.sections_score,
                "api": s.api_score,
                "depth": s.depth_score,
                "freshness": s.freshness_score,
                "git_freshness_available": git_freshness_available,
                "suggestions": s.suggestions,
            })
        })
        .collect();

    Ok(json!({
        "average_score": (project.average_score * 10.0).round() / 10.0,
        "grade": project.grade,
        "total_specs": project.total_specs,
        "git_freshness_available": git_freshness_available,
        "distribution": {
            "A": project.grade_distribution[0],
            "B": project.grade_distribution[1],
            "C": project.grade_distribution[2],
            "D": project.grade_distribution[3],
            "F": project.grade_distribution[4],
        },
        "specs": specs_json,
    }))
}

fn score_spec_for_mcp(
    spec_path: &Path,
    root: &Path,
    config: &crate::types::SpecSyncConfig,
    git_freshness_available: bool,
) -> scoring::SpecScore {
    let mut score = scoring::score_spec(spec_path, root, config);
    if git_freshness_available {
        return score;
    }

    const UNAVAILABLE_GIT_FRESHNESS_POINTS: u32 = 5;
    let penalty = score.freshness_score.min(UNAVAILABLE_GIT_FRESHNESS_POINTS);
    score.freshness_score = score.freshness_score.saturating_sub(penalty);
    score.total = score.total.saturating_sub(penalty);
    score.grade = mcp_letter_grade(score.total);
    score.suggestions.push(
        "Freshness (-5pts): Git history is intentionally unavailable in the confined MCP snapshot"
            .to_string(),
    );
    if let Some(freshness) = score
        .explain
        .iter_mut()
        .find(|detail| detail.dimension == "Freshness")
    {
        freshness.score = score.freshness_score;
        if let Some(criterion) = freshness
            .criteria
            .iter_mut()
            .find(|criterion| criterion.name == "git_freshness")
        {
            criterion.passed = false;
            criterion.points = 0;
            criterion.detail = Some("unavailable in confined MCP snapshot".to_string());
        }
    }
    score
}

fn mcp_letter_grade(score: u32) -> &'static str {
    match score {
        90.. => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    }
}

fn tool_issues(root: &Path) -> Result<Value, String> {
    use crate::github;

    let (config, references) = discover_issue_references(root)?;

    let mut results: Vec<Value> = Vec::new();
    let mut total_valid = 0usize;
    let mut total_closed = 0usize;
    let mut total_not_found = 0usize;

    if references.is_empty() {
        return Ok(json!({
            "repo": null,
            "total_valid": 0,
            "total_closed": 0,
            "total_not_found": 0,
            "specs": results,
        }));
    }

    let repo = config
        .github
        .as_ref()
        .and_then(|github| github.repo.as_deref())
        .filter(|repo| !repo.trim().is_empty())
        .ok_or_else(|| {
            "MCP specsync_issues requires an explicit `github.repo`; Git metadata auto-detection is disabled by the server security boundary".to_string()
        })?
        .to_string();

    for verification in github::verify_issue_batch(&repo, &references) {
        let rel_path = verification.spec_path.clone();
        ensure_issue_verification_complete(&rel_path, &verification)?;

        total_valid += verification.valid.len();
        total_closed += verification.closed.len();
        total_not_found += verification.not_found.len();

        results.push(json!({
            "spec": rel_path,
            "valid": verification.valid.iter().map(|i| json!({
                "number": i.number,
                "title": i.title,
                "state": i.state,
            })).collect::<Vec<_>>(),
            "closed": verification.closed.iter().map(|i| json!({
                "number": i.number,
                "title": i.title,
            })).collect::<Vec<_>>(),
            "not_found": verification.not_found,
        }));
    }

    Ok(json!({
        "repo": repo,
        "total_valid": total_valid,
        "total_closed": total_closed,
        "total_not_found": total_not_found,
        "specs": results,
    }))
}

type IssueReferences = (String, Vec<u64>, Vec<u64>);

fn is_issue_spec_file_name(name: &OsStr) -> Result<bool, ()> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;

        let bytes = name.as_bytes();
        if !bytes.ends_with(b".spec.md") {
            return Ok(false);
        }
        if name.to_str().is_none() {
            return Err(());
        }
        Ok(!bytes.starts_with(b"_"))
    }

    #[cfg(not(unix))]
    {
        Ok(name
            .to_str()
            .is_some_and(|name| name.ends_with(".spec.md") && !name.starts_with('_')))
    }
}

fn find_issue_spec_files_checked(root: &Path, specs_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut spec_files = Vec::new();
    for result in WalkDir::new(specs_dir)
        .follow_links(false)
        .sort_by_file_name()
    {
        let entry = result.map_err(|error| {
            let rel_path = issue_relative_spec_path(root, error.path().unwrap_or(specs_dir));
            format!(
                "MCP specsync_issues issue discovery is inconclusive for {rel_path}: spec directory entry could not be inspected"
            )
        })?;
        let is_spec = is_issue_spec_file_name(entry.file_name()).map_err(|()| {
            let rel_path = issue_relative_spec_path(root, entry.path());
            format!(
                "MCP specsync_issues issue discovery is inconclusive for {rel_path}: spec filename is not valid UTF-8"
            )
        })?;
        if !is_spec {
            continue;
        }
        if !entry.file_type().is_file() {
            let rel_path = issue_relative_spec_path(root, entry.path());
            return Err(format!(
                "MCP specsync_issues issue discovery is inconclusive for {rel_path}: spec path is not a regular file"
            ));
        }
        spec_files.push(entry.into_path());
    }
    spec_files.sort();
    Ok(spec_files)
}

fn discover_issue_references(
    root: &Path,
) -> Result<(SpecSyncConfig, Vec<IssueReferences>), String> {
    let config = load_confined_config(root).map_err(|error| {
        format!(
            "MCP specsync_issues issue discovery is inconclusive: {}",
            issue_project_error_reason(&error)
        )
    })?;
    let specs_dir = root.join(&config.specs_dir);
    let spec_files = find_issue_spec_files_checked(root, &specs_dir)?;

    if spec_files.is_empty() {
        return Err(
            "MCP specsync_issues issue discovery is inconclusive: no spec files were found"
                .to_string(),
        );
    }

    let mut references = Vec::new();
    for spec_path in &spec_files {
        let rel_path = issue_relative_spec_path(root, spec_path);
        let content = read_file_bounded(root, spec_path, "spec file").map_err(|error| {
            format!(
                "MCP specsync_issues issue discovery is inconclusive for {rel_path}: {}",
                issue_spec_read_error_reason(&error)
            )
        })?;
        let (implements, tracks) = crate::parser::parse_checked_issue_references(&content)
            .map_err(|reason| {
                format!(
                    "MCP specsync_issues issue discovery is inconclusive for {rel_path}: {reason}"
                )
            })?;

        if !implements.is_empty() || !tracks.is_empty() {
            references.push((rel_path, implements, tracks));
        }
    }

    validate_spec_file_mappings(root, &spec_files, &config.exclude_dirs).map_err(|error| {
        format!(
            "MCP specsync_issues issue discovery is inconclusive: {}",
            issue_project_error_reason(&error)
        )
    })?;

    Ok((config, references))
}

fn is_unsafe_issue_diagnostic_character(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '\u{061c}'
                | '\u{200e}'
                | '\u{200f}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2066}'..='\u{2069}'
        )
}

fn issue_relative_spec_path(root: &Path, spec_path: &Path) -> String {
    let Some(relative) = spec_path.strip_prefix(root).ok() else {
        return "<spec>".to_string();
    };
    let relative = relative.to_string_lossy();
    let mut sanitized = String::with_capacity(relative.len());
    for character in relative.chars() {
        if is_unsafe_issue_diagnostic_character(character) {
            sanitized.push_str(&format!("\\u{{{:04X}}}", character as u32));
        } else if cfg!(windows) && character == '\\' {
            sanitized.push('/');
        } else {
            sanitized.push(character);
        }
    }
    if sanitized.chars().count() <= MAX_ISSUE_DIAGNOSTIC_PATH_CHARS {
        return sanitized;
    }

    let mut bounded: String = sanitized
        .chars()
        .take(MAX_ISSUE_DIAGNOSTIC_PATH_CHARS.saturating_sub(3))
        .collect();
    bounded.push_str("...");
    bounded
}

fn issue_spec_read_error_reason(error: &str) -> &'static str {
    if error.contains("not valid UTF-8") {
        "spec file is not valid UTF-8"
    } else if error.contains("exceeds") {
        "spec file exceeds the configured size limit"
    } else if error.contains("not a file") {
        "spec path is not a regular file"
    } else if error.contains("Cannot inspect") {
        "spec file metadata could not be inspected"
    } else if error.contains("Cannot read") {
        "spec file could not be read"
    } else {
        "spec path failed safety validation"
    }
}

fn issue_project_error_reason(error: &str) -> &'static str {
    if error.contains("exceeds") || error.contains("limit") {
        "project input exceeds a configured safety limit"
    } else if error.contains("Cannot read") || error.contains("not valid UTF-8") {
        "project metadata could not be read"
    } else if error.contains("Cannot inspect") {
        "project metadata could not be inspected"
    } else if error.contains("escape")
        || error.contains("outside")
        || error.contains("symlink")
        || error.contains("junction")
        || error.contains("relative path")
    {
        "project input failed confinement validation"
    } else {
        "project discovery failed safety validation"
    }
}

fn ensure_issue_verification_complete(
    spec_path: &str,
    verification: &crate::github::IssueVerification,
) -> Result<(), String> {
    if verification.errors.is_empty() {
        return Ok(());
    }
    Err(format!(
        "GitHub issue verification was inconclusive for {spec_path}: {}",
        verification.errors.join("; ")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn open_test_directory(path: &Path) -> Dir {
        Dir::open_ambient_dir(path, ambient_authority()).unwrap()
    }

    fn assert_no_mcp_transaction_debris(root: &Path) {
        assert!(
            WalkDir::new(root)
                .into_iter()
                .filter_map(Result::ok)
                .all(|entry| {
                    let name = entry.file_name().to_string_lossy();
                    !name.starts_with(".specsync-mcp-stage-")
                        && !name.starts_with(".specsync-mcp-quarantine-")
                }),
            "successful MCP publication must remove private staging and quarantine entries"
        );
    }

    fn setup_project() -> TempDir {
        let tmp = TempDir::new().unwrap();
        let config = json!({
            "specsDir": "specs",
            "sourceDirs": ["src"],
            "requiredSections": ["Purpose", "Public API"]
        });
        std::fs::write(
            tmp.path().join("specsync.json"),
            serde_json::to_string_pretty(&config).unwrap(),
        )
        .unwrap();
        std::fs::create_dir_all(tmp.path().join("specs")).unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        tmp
    }

    fn setup_project_with_spec(spec_name: &str, spec_content: &str) -> TempDir {
        let tmp = setup_project();
        let spec_dir = tmp.path().join("specs").join(spec_name);
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(spec_dir.join(format!("{spec_name}.spec.md")), spec_content).unwrap();
        tmp
    }

    #[test]
    fn issue_verification_errors_fail_closed_instead_of_returning_empty_success() {
        let verification = crate::github::IssueVerification {
            spec_path: "specs/mcp/mcp.spec.md".to_string(),
            valid: Vec::new(),
            closed: Vec::new(),
            not_found: Vec::new(),
            errors: vec!["#414: authentication failed".to_string()],
        };

        let error =
            ensure_issue_verification_complete("specs/mcp/mcp.spec.md", &verification).unwrap_err();
        assert!(error.contains("inconclusive"));
        assert!(error.contains("#414"));
    }

    #[test]
    fn issue_tool_enforces_one_deduplicated_invocation_cap_across_specs() {
        let tmp = setup_project();
        fs::write(
            tmp.path().join("specsync.json"),
            r#"{"specsDir":"specs","sourceDirs":["src"],"github":{"repo":"CorvidLabs/spec-sync"}}"#,
        )
        .unwrap();
        let first: Vec<String> = (1..=60).map(|number| number.to_string()).collect();
        let second: Vec<String> = (41..=101).map(|number| number.to_string()).collect();
        for (module, issues) in [("one", first), ("two", second)] {
            let directory = tmp.path().join("specs").join(module);
            fs::create_dir_all(&directory).unwrap();
            fs::write(
                directory.join(format!("{module}.spec.md")),
                format!(
                    "---\nmodule: {module}\nversion: 1\nstatus: draft\nfiles: []\ntracks: [{}]\n---\n\n# Purpose\nTest\n",
                    issues.join(", ")
                ),
            )
            .unwrap();
        }

        let error = tool_issues(tmp.path())
            .expect_err("the global issue cap must fail before provider selection");

        assert!(error.contains("100-issue invocation limit"));
    }

    #[test]
    fn issue_tool_fails_inconclusive_for_malformed_frontmatter() {
        let tmp = setup_project_with_spec("malformed", "# Missing frontmatter\n");

        let error = tool_issues(tmp.path())
            .expect_err("malformed frontmatter must not produce a zero-reference success");

        assert_eq!(
            error,
            "MCP specsync_issues issue discovery is inconclusive for specs/malformed/malformed.spec.md: missing or malformed YAML frontmatter"
        );
    }

    #[test]
    fn issue_tool_fails_inconclusive_for_unreadable_spec_text() {
        let tmp = setup_project();
        let spec_dir = tmp.path().join("specs").join("unreadable");
        fs::create_dir_all(&spec_dir).unwrap();
        let spec_path = spec_dir.join("unreadable.spec.md");
        fs::write(&spec_path, [0xff]).unwrap();

        let error = tool_issues(tmp.path())
            .expect_err("unreadable spec text must not produce a zero-reference success");

        assert_eq!(
            error,
            "MCP specsync_issues issue discovery is inconclusive for specs/unreadable/unreadable.spec.md: spec file is not valid UTF-8"
        );
        assert!(!error.contains(&tmp.path().to_string_lossy().to_string()));
    }

    #[test]
    fn issue_tool_fails_inconclusive_for_malformed_known_issue_fields() {
        let cases = [
            ("implements:", "`implements` must be a list"),
            ("implements: null", "`implements` must be a list"),
            ("implements: nope", "`implements` must be a list"),
            ("tracks: 42", "`tracks` must be a list"),
            ("implements : [nope]", "`implements` contains an invalid"),
            ("implements: [0]", "`implements` contains an invalid"),
            ("implements: [-1]", "`implements` contains an invalid"),
            (
                "implements: [18446744073709551616]",
                "`implements` contains an invalid",
            ),
            (
                "implements: [41, not-a-number, 42]",
                "`implements` contains an invalid unsigned issue number",
            ),
            (
                "tracks:\n  - 41\n  - nope\n  - 42",
                "`tracks` contains an invalid unsigned issue number",
            ),
            (
                "tracks:\n- 41\n- nope",
                "`tracks` contains an invalid unsigned issue number",
            ),
        ];

        for (field, expected_reason) in cases {
            let spec = format!(
                "---\nmodule: malformed\nversion: 1\nstatus: draft\nfiles: []\n{field}\n---\n\n# Purpose\nTest\n"
            );
            let tmp = setup_project_with_spec("malformed", &spec);

            let error = tool_issues(tmp.path()).expect_err(
                "malformed known issue fields must not produce a zero-reference success",
            );

            assert!(
                error.contains(expected_reason),
                "unexpected error for {field}: {error}"
            );
            assert!(!error.contains(&tmp.path().to_string_lossy().to_string()));
        }
    }

    #[test]
    fn issue_reference_field_validation_accepts_supported_list_forms() {
        let content = "---\nmodule: valid\nimplements: [41, 42]\ntracks:\n  - 43\n  - 44 # retained comment\n---\n\n# Purpose\nTest\n";

        assert_eq!(
            crate::parser::parse_checked_issue_references(content).unwrap(),
            (vec![41, 42], vec![43, 44])
        );
    }

    #[test]
    fn issue_discovery_accepts_yaml_trailing_comma_and_inline_comment() {
        let content = "---\nmodule: valid\nversion: 1\nstatus: draft\nfiles: []\nimplements: [42,] # valid YAML\n---\n\n# Purpose\nTest\n";
        let tmp = setup_project_with_spec("valid", content);

        let (_, references) = discover_issue_references(tmp.path()).unwrap();

        assert_eq!(
            references,
            vec![(
                "specs/valid/valid.spec.md".to_string(),
                vec![42],
                Vec::new()
            )]
        );
    }

    #[test]
    fn issue_discovery_accepts_crlf_frontmatter_and_retains_references() {
        let content = "---\r\nmodule: crlf\r\nversion: 1\r\nstatus: draft\r\nfiles: []\r\nimplements: [41, 42]\r\ntracks:\r\n  - 43\r\n---\r\n\r\n# Purpose\r\nTest\r\n";
        let tmp = setup_project_with_spec("crlf", content);

        let (_, references) = discover_issue_references(tmp.path()).unwrap();

        assert_eq!(
            references,
            vec![(
                "specs/crlf/crlf.spec.md".to_string(),
                vec![41, 42],
                vec![43],
            )]
        );
    }

    #[test]
    fn issue_reference_field_validation_ignores_nested_extensions_and_block_scalars() {
        let content = "\
---
module: valid
extensions:
  implements: [900]
  nested:
    tracks: invalid
extension_sequence:
  - implements: [901]
    tracks: [902]
notes: |
  implements: invalid
  tracks:
    - 903
folded: >
  tracks: [904]
implements: [41, 42]
tracks:
  - 43
  - 44
---

# Purpose
Test
";

        assert_eq!(
            crate::parser::parse_checked_issue_references(content).unwrap(),
            (vec![41, 42], vec![43, 44])
        );
    }

    #[test]
    fn issue_reference_field_validation_keeps_top_level_known_fields_strict() {
        let content = "\
---
module: invalid
extensions:
  implements: [900]
implements:
  nested: invalid
---

# Purpose
Test
";

        assert_eq!(
            crate::parser::parse_checked_issue_references(content).unwrap_err(),
            "`implements` must be a list of unsigned issue numbers"
        );
    }

    #[test]
    fn issue_tool_rejects_duplicate_issue_reference_keys() {
        let content = "---\nmodule: invalid\nversion: 1\nstatus: draft\nfiles: []\nimplements: [41]\nimplements: [42]\n---\n\n# Purpose\nTest\n";
        let tmp = setup_project_with_spec("invalid", content);

        let error = tool_issues(tmp.path()).unwrap_err();

        assert_eq!(
            error,
            "MCP specsync_issues issue discovery is inconclusive for specs/invalid/invalid.spec.md: duplicate `implements` issue-reference field"
        );
    }

    #[test]
    fn issue_tool_rejects_reviewer_reproducer_without_leaking_content() {
        let content = "\
---
module: invalid
version: 1
status: draft
files: []
implements:
private_extension:
  secret: [reviewer-reproducer
---

# Purpose
Test
";
        let tmp = setup_project_with_spec("invalid", content);

        let error = tool_issues(tmp.path()).unwrap_err();

        assert!(error.contains("issue discovery is inconclusive"));
        assert!(
            error.contains("invalid YAML frontmatter")
                || error.contains("`implements` must be a list")
        );
        assert!(!error.contains("reviewer-reproducer"));
        assert!(!error.contains("private_extension"));
    }

    #[cfg(unix)]
    #[test]
    fn issue_spec_file_name_rejects_non_utf8_spec_suffix() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let mut file_name = b"hidden-".to_vec();
        file_name.push(0xff);
        file_name.extend_from_slice(b".spec.md");

        assert_eq!(
            is_issue_spec_file_name(&OsString::from_vec(file_name)),
            Err(())
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn issue_tool_rejects_non_utf8_spec_filename_after_snapshot_copy() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let tmp = setup_project();
        let spec_dir = tmp.path().join("specs").join("hostile");
        fs::create_dir_all(&spec_dir).unwrap();
        let mut file_name = b"hidden-".to_vec();
        file_name.push(0xff);
        file_name.extend_from_slice(b".spec.md");
        let spec_path = spec_dir.join(OsString::from_vec(file_name));
        fs::write(
            &spec_path,
            "---\nmodule: hostile\nversion: 1\nstatus: draft\nfiles: []\n---\n\n# Purpose\nTest\n",
        )
        .unwrap();

        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();
        let error = tool_issues(snapshot.root())
            .expect_err("a non-UTF-8 spec filename must make issue discovery inconclusive");

        assert!(error.contains("spec filename is not valid UTF-8"));
        assert!(error.contains("specs/hostile/hidden-"));
        assert!(!error.contains(&tmp.path().to_string_lossy().to_string()));
        assert!(!error.contains("# Purpose"));
        assert!(error.chars().count() < 400);
    }

    #[test]
    fn issue_read_diagnostics_are_bounded_relative_and_content_free() {
        let tmp = setup_project();
        let long_name = "x".repeat(MAX_ISSUE_DIAGNOSTIC_PATH_CHARS + 80);
        let path = tmp.path().join("specs").join(long_name);

        let relative = issue_relative_spec_path(tmp.path(), &path);
        let reason = issue_spec_read_error_reason(&format!(
            "Cannot read MCP spec file {}: secret operating system detail",
            path.display()
        ));

        assert!(relative.chars().count() <= MAX_ISSUE_DIAGNOSTIC_PATH_CHARS);
        assert!(!relative.contains(&tmp.path().to_string_lossy().to_string()));
        assert_eq!(reason, "spec file could not be read");
        assert!(!reason.contains("secret"));
    }

    #[test]
    fn issue_relative_spec_path_escapes_every_unsafe_display_character() {
        let root = Path::new("project");
        let dangerous_code_points = (0x0000u32..=0x001f)
            .chain(0x007f..=0x009f)
            .chain([0x061c, 0x200e, 0x200f, 0x2028, 0x2029])
            .chain(0x202a..=0x202e)
            .chain(0x2066..=0x2069);

        for code_point in dangerous_code_points {
            let character =
                char::from_u32(code_point).expect("test code point must be valid Unicode");
            let path = root
                .join("specs")
                .join(format!("before{character}after.spec.md"));
            let escaped = format!("\\u{{{code_point:04X}}}");

            assert_eq!(
                issue_relative_spec_path(root, &path),
                format!("specs/before{escaped}after.spec.md"),
                "unsafe diagnostic character U+{code_point:04X} was not escaped"
            );
        }

        let path = root.join(r"specs\windows\path.spec.md");
        #[cfg(windows)]
        assert_eq!(
            issue_relative_spec_path(root, &path),
            "specs/windows/path.spec.md"
        );
        #[cfg(not(windows))]
        assert_eq!(
            issue_relative_spec_path(root, &path),
            r"specs\windows\path.spec.md"
        );
    }

    #[cfg(unix)]
    #[test]
    fn issue_tool_escapes_adversarial_filename_formatting_in_diagnostic() {
        let tmp = setup_project();
        let spec_dir = tmp.path().join("specs").join("hostile");
        fs::create_dir_all(&spec_dir).unwrap();
        let dangerous_characters = [
            '\n', '\u{001b}', '\u{009b}', '\u{061c}', '\u{200e}', '\u{200f}', '\u{2028}',
            '\u{2029}', '\u{202a}', '\u{202b}', '\u{202c}', '\u{202d}', '\u{202e}', '\u{2066}',
            '\u{2067}', '\u{2068}', '\u{2069}',
        ];
        let hostile: String = dangerous_characters.iter().copied().collect();
        let filename = format!(r"segment\before{hostile}after.spec.md");
        fs::write(spec_dir.join(filename), "# Missing frontmatter\n").unwrap();

        let error = tool_issues(tmp.path())
            .expect_err("unsafe filename formatting must not produce a successful issue result");

        assert!(error.contains(r"specs/hostile/segment\before"));
        for character in dangerous_characters {
            assert!(
                error.contains(&format!("\\u{{{:04X}}}", character as u32)),
                "diagnostic omitted escape for U+{:04X}",
                character as u32
            );
            assert!(
                !error.contains(character),
                "diagnostic retained unsafe character U+{:04X}",
                character as u32
            );
        }
        assert!(error.ends_with("after.spec.md: missing or malformed YAML frontmatter"));
    }

    #[cfg(unix)]
    #[test]
    fn server_root_capability_rejects_a_root_replaced_before_canonicalization() {
        use std::os::unix::fs::symlink;

        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("server");
        let moved = tmp.path().join("moved-server");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();

        let error = open_server_root_capability(&root, || {
            fs::rename(&root, &moved).unwrap();
            symlink(&outside, &root).unwrap();
            Ok(())
        })
        .expect_err("a swapped root must be rejected");

        assert!(error.contains("root") || error.contains("capability"));
    }

    #[test]
    fn windows_read_root_suffix_normalizes_drive_extended_unc_and_case() {
        assert_eq!(
            windows_relative_suffix_text(r"C:\Project", r"c:\PROJECT\Specs\Auth"),
            Some(PathBuf::from("Specs").join("Auth"))
        );
        assert_eq!(
            windows_relative_suffix_text(r"\\?\C:\Project", r"C:\Project\src"),
            Some(PathBuf::from("src"))
        );
        assert_eq!(
            windows_relative_suffix_text(
                r"\\server\share\Project",
                r"\\?\UNC\SERVER\SHARE\project\specs"
            ),
            Some(PathBuf::from("specs"))
        );
        assert_eq!(
            windows_relative_suffix_text(r"C:\Project", r"C:\ProjectElsewhere\src"),
            None
        );
        assert_eq!(
            windows_relative_suffix_text(r"\\server\share\Project", r"\\server\other\Project"),
            None
        );
        assert_eq!(
            windows_relative_suffix_text(r"C:\Projekt\Ärger", r"c:\PROJEKT\ärger\Specs"),
            Some(PathBuf::from("Specs"))
        );
    }

    #[test]
    fn read_root_suffix_accepts_the_identity_bound_startup_alias_only_as_a_prefix() {
        #[cfg(windows)]
        let canonical_root = Path::new(r"C:\Users\runneradmin\AppData\Local\Temp\server");
        #[cfg(windows)]
        let requested_root = Path::new(r"C:\Users\RUNNER~1\AppData\Local\Temp\server");
        #[cfg(windows)]
        let child = r"C:\Users\RUNNER~1\AppData\Local\Temp\server\child";
        #[cfg(windows)]
        let sibling = r"C:\Users\RUNNER~1\AppData\Local\Temp\server-sibling\child";

        #[cfg(not(windows))]
        let canonical_root = Path::new("/resolved/temp/server");
        #[cfg(not(windows))]
        let requested_root = Path::new("/startup-alias/temp/server");
        #[cfg(not(windows))]
        let child = "/startup-alias/temp/server/child";
        #[cfg(not(windows))]
        let sibling = "/startup-alias/temp/server-sibling/child";

        assert_eq!(
            resolve_read_root(canonical_root, requested_root, Some(child)).unwrap(),
            PathBuf::from("child")
        );
        assert_eq!(
            resolve_read_root(canonical_root, requested_root, Some(sibling)).unwrap_err(),
            "Read root override escapes the configured server root"
        );
    }

    #[test]
    fn read_root_rejects_git_metadata_components_case_insensitively() {
        #[cfg(not(windows))]
        let canonical_root = Path::new("/resolved/temp/server");
        #[cfg(not(windows))]
        let requested_root = Path::new("/startup-alias/temp/server");
        #[cfg(not(windows))]
        let absolute_git_root = "/startup-alias/temp/server/.gIt";

        #[cfg(windows)]
        let canonical_root = Path::new(r"C:\resolved\temp\server");
        #[cfg(windows)]
        let requested_root = Path::new(r"C:\startup-alias\temp\server");
        #[cfg(windows)]
        let absolute_git_root = r"C:\startup-alias\temp\server\.gIt";

        for candidate in [".git", ".GIT", "child/.GiT", absolute_git_root] {
            assert_eq!(
                resolve_read_root(canonical_root, requested_root, Some(candidate)).unwrap_err(),
                "Read root override must not select Git metadata"
            );
        }
        assert_eq!(
            resolve_read_root(canonical_root, requested_root, Some("child")).unwrap(),
            PathBuf::from("child")
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_read_root_accepts_runtime_absolute_variants() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("Child")).unwrap();
        let canonical = tmp.path().canonicalize().unwrap();
        let child = canonical.join("Child");
        let ordinary = child.to_string_lossy().replace(r"\\?\", "");
        let case_varied = ordinary.to_ascii_uppercase();

        assert_eq!(
            resolve_read_root(&canonical, &canonical, Some(&ordinary)).unwrap(),
            PathBuf::from("Child")
        );
        assert_eq!(
            resolve_read_root(&canonical, &canonical, Some(&case_varied)).unwrap(),
            PathBuf::from("CHILD")
        );

        let unicode_root = canonical.join("Ärger");
        fs::create_dir_all(unicode_root.join("Child")).unwrap();
        let unicode_candidate = unicode_root
            .join("Child")
            .to_string_lossy()
            .replace("Ärger", "ärger");
        assert_eq!(
            resolve_read_root(&unicode_root, &unicode_root, Some(&unicode_candidate)).unwrap(),
            PathBuf::from("Child")
        );
    }

    #[test]
    fn snapshot_root_source_dir_includes_otherwise_ignored_inputs() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("target")).unwrap();
        fs::write(
            tmp.path().join("specsync.json"),
            r#"{"specsDir":"specs","sourceDirs":["."]}"#,
        )
        .unwrap();
        fs::File::create(tmp.path().join("target/oversized.rs"))
            .unwrap()
            .set_len(MAX_PROJECT_FILE_BYTES + 1)
            .unwrap();

        let error = ProjectSnapshot::create(tmp.path())
            .err()
            .expect("explicit root inputs must not disappear from the snapshot");
        assert!(error.contains("per-file limit"));
    }

    #[cfg(unix)]
    #[test]
    fn snapshot_skips_ignored_and_configured_exclusion_symlinks_before_following() {
        use std::os::unix::fs::symlink;

        let source_temp = TempDir::new().unwrap();
        let destination_temp = TempDir::new().unwrap();
        let explicit_destination_temp = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        fs::write(outside.path().join("secret.rs"), "outside\n").unwrap();
        fs::create_dir_all(source_temp.path().join("kept")).unwrap();
        fs::write(source_temp.path().join("kept/module.rs"), "inside\n").unwrap();
        symlink(outside.path(), source_temp.path().join("target")).unwrap();
        symlink("kept", source_temp.path().join("generated")).unwrap();
        let source = open_test_directory(source_temp.path());
        let destination = open_test_directory(destination_temp.path());
        let mut budget = SnapshotBudget::default();

        copy_snapshot_directory(
            &source,
            &destination,
            Path::new("."),
            &HashSet::from(["generated".to_string()]),
            &[],
            &mut budget,
        )
        .expect("excluded symlink names must be skipped without following their targets");

        assert!(!destination_temp.path().join("target").exists());
        assert!(!destination_temp.path().join("generated").exists());

        let explicit_destination = open_test_directory(explicit_destination_temp.path());
        let mut explicit_budget = SnapshotBudget::default();
        let error = copy_snapshot_directory(
            &source,
            &explicit_destination,
            Path::new("."),
            &HashSet::from(["generated".to_string()]),
            &[PathBuf::from("generated")],
            &mut explicit_budget,
        )
        .expect_err("an explicit input must inspect and reject the excluded symlink");
        assert!(
            error.contains("regular file or directory"),
            "unexpected explicit symlink rejection: {error}"
        );
    }

    #[test]
    fn snapshot_includes_manifest_derived_member_under_ignored_directory() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        fs::create_dir_all(tmp.path().join("vendor/member/src")).unwrap();
        fs::write(
            tmp.path().join("specsync.json"),
            r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
        )
        .unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"vendor/member\"]\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("vendor/member/Cargo.toml"),
            "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("vendor/member/src/lib.rs"),
            "pub fn member() {}\n",
        )
        .unwrap();

        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();
        assert!(snapshot.root().join("vendor/member/src/lib.rs").is_file());
    }

    #[test]
    fn snapshot_normalizes_confined_cargo_sibling_dependency() {
        let tmp = setup_project();
        fs::create_dir_all(tmp.path().join("crates/a/src")).unwrap();
        fs::create_dir_all(tmp.path().join("crates/b/src")).unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/a\"]\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("crates/a/Cargo.toml"),
            "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[dependencies]\nb = { path = \"../b\" }\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("crates/b/Cargo.toml"),
            "[package]\nname = \"b\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(tmp.path().join("crates/a/src/lib.rs"), "pub fn a() {}\n").unwrap();
        fs::write(tmp.path().join("crates/b/src/lib.rs"), "pub fn b() {}\n").unwrap();

        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();

        assert!(snapshot.root().join("crates/b/src/lib.rs").is_file());
    }

    #[test]
    fn snapshot_normalizes_confined_windows_native_cargo_paths() {
        let tmp = setup_project();
        fs::create_dir_all(tmp.path().join("crates/a/src")).unwrap();
        fs::create_dir_all(tmp.path().join("crates/b/src")).unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = ['crates\\a']\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("crates/a/Cargo.toml"),
            "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[dependencies]\nb = { path = '..\\b' }\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("crates/b/Cargo.toml"),
            "[package]\nname = \"b\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(tmp.path().join("crates/a/src/lib.rs"), "pub fn a() {}\n").unwrap();
        fs::write(tmp.path().join("crates/b/src/lib.rs"), "pub fn b() {}\n").unwrap();

        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();

        assert!(snapshot.root().join("crates/a/src/lib.rs").is_file());
        assert!(snapshot.root().join("crates/b/src/lib.rs").is_file());
    }

    #[test]
    fn snapshot_ignores_nonsemantic_cargo_metadata_paths() {
        let tmp = setup_project();
        fs::create_dir_all(tmp.path().join("vendor")).unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"metadata-path\"\nversion = \"0.1.0\"\n\n[lib]\npath = \"vendor/lib.rs\"\n\n[package.metadata.example]\npath = \"/benign/absolute/metadata/value\"\n",
        )
        .unwrap();
        fs::write(tmp.path().join("src/lib.rs"), "pub fn value() {}\n").unwrap();
        fs::write(
            tmp.path().join("vendor/lib.rs"),
            "pub fn custom_target() {}\n",
        )
        .unwrap();

        let snapshot = ProjectSnapshot::create(tmp.path())
            .expect("non-semantic Cargo metadata paths must not affect snapshot discovery");

        assert!(snapshot.root().join("src/lib.rs").is_file());
        assert!(snapshot.root().join("vendor/lib.rs").is_file());
    }

    #[test]
    fn snapshot_manifest_input_rejects_true_root_and_cross_platform_lexical_escapes() {
        let error = snapshot_manifest_input(
            Path::new("crates/a"),
            "../../../outside",
            "Cargo target path",
        )
        .expect_err("parent traversal beyond the retained root must be rejected");
        assert!(error.contains("escapes the configured server root"));

        for configured in [
            "/outside",
            "C:/outside",
            "C:outside",
            r"\\server\share\outside",
            r"\outside",
        ] {
            let error =
                snapshot_manifest_input(Path::new("crates/a"), configured, "Cargo target path")
                    .expect_err(
                        "absolute, drive, UNC, and backslash paths must be rejected portably",
                    );
            assert!(error.contains("safe project-relative path"));
        }

        let error = snapshot_manifest_input(
            Path::new("crates/a"),
            r"..\..\..\outside",
            "Cargo target path",
        )
        .expect_err("Windows-native traversal beyond the retained root must be rejected");
        assert!(error.contains("escapes the configured server root"));
    }

    #[cfg(unix)]
    #[test]
    fn snapshot_rejects_confined_sibling_dependency_symlink_escape() {
        use std::os::unix::fs::symlink;

        let tmp = setup_project();
        let outside = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("crates/a/src")).unwrap();
        fs::create_dir_all(outside.path().join("src")).unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/a\"]\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("crates/a/Cargo.toml"),
            "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[dependencies]\nb = { path = '..\\b' }\n",
        )
        .unwrap();
        fs::write(
            outside.path().join("Cargo.toml"),
            "[package]\nname = \"b\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(outside.path().join("src/lib.rs"), "pub fn outside() {}\n").unwrap();
        symlink(outside.path(), tmp.path().join("crates/b")).unwrap();

        ProjectSnapshot::create(tmp.path())
            .err()
            .expect("a normalized sibling dependency must not follow a symlink outside root");
    }

    #[test]
    fn snapshot_includes_multiline_cargo_members_beneath_ignored_directories() {
        let tmp = setup_project();
        fs::create_dir_all(tmp.path().join("vendor/member/src")).unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\n  \"vendor/member\", # kept\n]\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("vendor/member/Cargo.toml"),
            "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::File::create(tmp.path().join("vendor/member/src/lib.rs"))
            .unwrap()
            .set_len(MAX_PROJECT_FILE_BYTES + 1)
            .unwrap();

        let error = ProjectSnapshot::create(tmp.path())
            .err()
            .expect("a multiline workspace member must remain visible and bounded");

        assert!(error.contains("per-file limit"));
        let expected_path = PathBuf::from("vendor")
            .join("member")
            .join("src")
            .join("lib.rs")
            .display()
            .to_string();
        assert!(error.contains(&expected_path));
    }

    #[test]
    fn snapshot_uses_toml_structure_when_comments_precede_workspace_header() {
        let tmp = setup_project();
        fs::create_dir_all(tmp.path().join("vendor/member/src")).unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "# [workspace]\n# members = [\"ignored\"]\n\n[workspace]\nmembers = [\"vendor/member\"]\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("vendor/member/Cargo.toml"),
            "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("vendor/member/src/lib.rs"),
            "pub fn member() {}\n",
        )
        .unwrap();

        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();

        assert!(snapshot.root().join("vendor/member/src/lib.rs").is_file());
    }

    #[test]
    fn snapshot_rejects_malformed_cargo_toml_without_partial_discovery() {
        let tmp = setup_project();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"vendor/member\"\n",
        )
        .unwrap();

        let error = ProjectSnapshot::create(tmp.path())
            .err()
            .expect("malformed Cargo TOML must make snapshot discovery inconclusive");

        assert!(error.contains("Cannot parse MCP Cargo workspace manifest Cargo.toml as TOML"));
    }

    #[test]
    fn manifest_discovery_charges_the_shared_cumulative_snapshot_budget() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("vendor/one")).unwrap();
        fs::create_dir_all(tmp.path().join("vendor/two")).unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"vendor/one\", \"vendor/two\"]\n",
        )
        .unwrap();
        fs::write(tmp.path().join("vendor/one/Cargo.toml"), vec![b' '; 300]).unwrap();
        fs::write(tmp.path().join("vendor/two/Cargo.toml"), vec![b' '; 300]).unwrap();
        let source = open_test_directory(tmp.path());
        let mut inputs = Vec::new();
        let mut budget = SnapshotBudget {
            max_bytes: 512,
            ..SnapshotBudget::default()
        };

        let error = collect_snapshot_manifest_inputs(&source, &mut inputs, &mut budget)
            .expect_err("manifest parsing must not bypass the cumulative byte budget");

        assert!(error.contains("512 bytes cumulative limit"));
    }

    #[test]
    fn snapshot_workspace_declarations_are_charged_before_deduplication() {
        let cargo = TempDir::new().unwrap();
        fs::create_dir_all(cargo.path().join("crates/member")).unwrap();
        fs::write(
            cargo.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/member\", \"./crates/member\"]\n",
        )
        .unwrap();
        fs::write(
            cargo.path().join("crates/member/Cargo.toml"),
            "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        let cargo_source = open_test_directory(cargo.path());
        let mut cargo_inputs = Vec::new();
        let mut cargo_budget = SnapshotBudget {
            entries: MAX_CONFINEMENT_ENTRIES - 2,
            ..SnapshotBudget::default()
        };

        collect_snapshot_manifest_inputs(&cargo_source, &mut cargo_inputs, &mut cargo_budget)
            .expect("two duplicate Cargo declarations must fit exactly at the traversal limit");

        assert_eq!(cargo_budget.entries, MAX_CONFINEMENT_ENTRIES);
        assert_eq!(
            cargo_inputs
                .iter()
                .filter(|path| path.as_path() == Path::new("crates/member"))
                .count(),
            1
        );

        let node = TempDir::new().unwrap();
        fs::write(
            node.path().join("package.json"),
            r#"{"workspaces":["packages/*","packages/*","packages/**"]}"#,
        )
        .unwrap();
        let node_source = open_test_directory(node.path());
        let mut node_inputs = Vec::new();
        let mut node_budget = SnapshotBudget {
            entries: MAX_CONFINEMENT_ENTRIES - 2,
            ..SnapshotBudget::default()
        };

        let error =
            collect_snapshot_manifest_inputs(&node_source, &mut node_inputs, &mut node_budget)
                .expect_err(
                    "a duplicate Node declaration beyond the traversal limit must be rejected",
                );

        assert!(error.contains("exceeds 100000 entries"), "{error}");
        assert_eq!(node_budget.entries, MAX_CONFINEMENT_ENTRIES + 1);
        assert_eq!(
            node_inputs
                .iter()
                .filter(|path| path.as_path() == Path::new("packages"))
                .count(),
            0
        );
    }

    #[test]
    fn snapshot_package_json_fails_closed_and_charges_entries_before_type_validation() {
        for (content, expected) in [
            ("{", "Cannot parse MCP package.json as JSON"),
            ("[]", "root must be a JSON object"),
            (
                r#"{"workspaces":7}"#,
                "`workspaces` must be an array or an object",
            ),
            (
                r#"{"workspaces":{"packages":7}}"#,
                "`workspaces.packages` must be an array",
            ),
            (
                r#"{"workspaces":{}}"#,
                "`workspaces` object must contain a `packages` array",
            ),
        ] {
            let tmp = TempDir::new().unwrap();
            fs::write(tmp.path().join("package.json"), content).unwrap();
            let error = ProjectSnapshot::create(tmp.path())
                .err()
                .expect("invalid package.json must make snapshot discovery inconclusive");
            assert!(error.contains(expected), "{content}: {error}");
        }

        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("package.json"),
            r#"{"workspaces":[7,"packages/*"]}"#,
        )
        .unwrap();
        let source = open_test_directory(tmp.path());
        let mut inputs = Vec::new();
        let mut budget = SnapshotBudget {
            entries: MAX_CONFINEMENT_ENTRIES - 2,
            ..SnapshotBudget::default()
        };
        let error = collect_snapshot_manifest_inputs(&source, &mut inputs, &mut budget)
            .expect_err("wrong-typed workspace entries must fail after declaration charging");

        assert!(
            error.contains("workspace entries must be strings"),
            "{error}"
        );
        assert_eq!(budget.entries, MAX_CONFINEMENT_ENTRIES);
    }

    #[test]
    fn snapshot_copies_the_exact_manifest_bytes_charged_during_discovery() {
        let source_temp = TempDir::new().unwrap();
        let destination_temp = TempDir::new().unwrap();
        let original = b"[workspace]\nmembers = []\n";
        fs::write(source_temp.path().join("Cargo.toml"), original).unwrap();
        let source = open_test_directory(source_temp.path());
        let destination = open_test_directory(destination_temp.path());
        let mut budget = SnapshotBudget {
            max_bytes: original.len() as u64,
            ..SnapshotBudget::default()
        };

        let loaded =
            read_capability_text_if_exists(&source, Path::new("Cargo.toml"), &mut budget).unwrap();
        assert!(loaded.is_some());
        fs::write(
            source_temp.path().join("Cargo.toml"),
            vec![b'x'; MAX_PROJECT_FILE_BYTES as usize],
        )
        .unwrap();

        copy_preloaded_snapshot_files(&destination, &mut budget).unwrap();
        copy_snapshot_directory(
            &source,
            &destination,
            Path::new("."),
            &HashSet::new(),
            &[PathBuf::from("Cargo.toml")],
            &mut budget,
        )
        .unwrap();

        assert_eq!(
            fs::read(destination_temp.path().join("Cargo.toml")).unwrap(),
            original
        );
        assert_eq!(budget.bytes, original.len() as u64);
    }

    #[cfg(unix)]
    #[test]
    fn snapshot_manifest_reader_rejects_special_files_without_blocking() {
        use std::process::Command;
        use std::time::{Duration, Instant};

        let tmp = TempDir::new().unwrap();
        assert!(
            Command::new("mkfifo")
                .arg(tmp.path().join("Cargo.toml"))
                .status()
                .unwrap()
                .success()
        );
        let source = open_test_directory(tmp.path());
        let mut budget = SnapshotBudget::default();
        let started = Instant::now();

        let error = read_capability_text_if_exists(&source, Path::new("Cargo.toml"), &mut budget)
            .expect_err("special-file manifests must fail before a blocking open");

        assert!(error.contains("regular file"), "{error}");
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[cfg(unix)]
    #[test]
    fn generic_snapshot_rejects_fifo_and_socket_sources_without_blocking() {
        use std::os::unix::net::UnixListener;
        use std::process::Command;
        use std::time::{Duration, Instant};

        let fifo = setup_project();
        assert!(
            Command::new("mkfifo")
                .arg(fifo.path().join("src/special.rs"))
                .status()
                .unwrap()
                .success()
        );
        let started = Instant::now();
        let fifo_error = ProjectSnapshot::create(fifo.path())
            .err()
            .expect("a configured FIFO source must fail the generic snapshot");
        assert!(
            fifo_error.contains("regular file or directory"),
            "{fifo_error}"
        );
        assert!(started.elapsed() < Duration::from_secs(2));

        let socket = setup_project();
        match UnixListener::bind(socket.path().join("src/special.rs")) {
            Ok(_listener) => {
                let socket_error = ProjectSnapshot::create(socket.path())
                    .err()
                    .expect("a configured socket source must fail the generic snapshot");
                assert!(
                    socket_error.contains("regular file or directory"),
                    "{socket_error}"
                );
            }
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {}
            Err(error) => panic!("cannot create generic snapshot socket fixture: {error}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn generic_snapshot_reader_rejects_fifo_symlink_and_regular_replacements() {
        use std::os::unix::fs::symlink;
        use std::process::Command;
        use std::time::{Duration, Instant};

        const ATTACKER_BYTES: &str = "GENERIC_SNAPSHOT_ATTACKER_BYTES";
        for replacement in ["fifo", "symlink", "regular"] {
            let tmp = setup_project();
            let root = tmp.path();
            let source_path = root.join("src/lib.rs");
            let attacker_path = root.join("attacker.rs");
            fs::write(&source_path, "pub fn retained() {}\n").unwrap();
            fs::write(&attacker_path, ATTACKER_BYTES).unwrap();
            let source = open_test_directory(root);
            let started = Instant::now();

            let result =
                read_snapshot_project_file_with_hook(&source, Path::new("src/lib.rs"), || {
                    fs::rename(&source_path, root.join("src/original.rs")).unwrap();
                    match replacement {
                        "fifo" => {
                            assert!(
                                Command::new("mkfifo")
                                    .arg(&source_path)
                                    .status()
                                    .unwrap()
                                    .success()
                            );
                        }
                        "symlink" => symlink(&attacker_path, &source_path).unwrap(),
                        "regular" => fs::rename(&attacker_path, &source_path).unwrap(),
                        _ => unreachable!(),
                    }
                });

            let error = result.expect_err("every post-open path replacement must fail closed");
            assert!(
                error.contains("changed during inspection"),
                "{replacement}: {error}"
            );
            assert!(!error.contains(ATTACKER_BYTES), "{replacement}: {error}");
            assert!(
                started.elapsed() < Duration::from_secs(2),
                "{replacement} replacement blocked the retained reader"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn snapshot_rejects_directory_replacement_after_enumeration_before_recursion() {
        let source_temp = setup_project();
        let destination_temp = TempDir::new().unwrap();
        let source_root = source_temp.path();
        let original = source_root.join("src");
        let retained = source_root.join("retained-src");
        let destination = open_test_directory(destination_temp.path());
        let source = open_test_directory(source_root);
        let mut budget = SnapshotBudget::default();
        let mut replaced = false;

        let error = copy_snapshot_directory_with_hook(
            &source,
            &destination,
            Path::new("."),
            &HashSet::new(),
            &[PathBuf::from("src"), PathBuf::from("specs")],
            &mut budget,
            |relative| {
                if relative == Path::new(".") && !replaced {
                    fs::rename(&original, &retained).unwrap();
                    fs::create_dir(&original).unwrap();
                    fs::write(original.join("attacker.rs"), "pub fn attacker() {}\n").unwrap();
                    replaced = true;
                }
                Ok(())
            },
        )
        .expect_err("a post-enumeration directory replacement must fail closed");

        assert!(
            error.contains("changed during snapshot traversal"),
            "{error}"
        );
        assert!(error.contains("src"), "{error}");
        assert!(!destination_temp.path().join("src/attacker.rs").exists());
    }

    #[cfg(unix)]
    #[test]
    fn snapshot_preflight_rejects_special_and_linked_gradle_manifests_without_blocking() {
        use std::os::unix::fs::symlink;
        use std::os::unix::net::UnixListener;
        use std::process::Command;
        use std::time::{Duration, Instant};

        let fifo = TempDir::new().unwrap();
        assert!(
            Command::new("mkfifo")
                .arg(fifo.path().join("build.gradle.kts"))
                .status()
                .unwrap()
                .success()
        );
        let started = Instant::now();
        let error = ProjectSnapshot::create(fifo.path())
            .err()
            .expect("a Gradle FIFO must fail before a blocking snapshot open");
        assert!(error.contains("regular file"), "{error}");
        assert!(started.elapsed() < Duration::from_secs(2));

        let socket = TempDir::new().unwrap();
        match UnixListener::bind(socket.path().join("settings.gradle.kts")) {
            Ok(_listener) => {
                let error = ProjectSnapshot::create(socket.path())
                    .err()
                    .expect("a Gradle socket must fail snapshot preflight");
                assert!(error.contains("regular file"), "{error}");
            }
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {}
            Err(error) => panic!("cannot create Gradle socket fixture: {error}"),
        }

        for manifest in ["build.gradle.kts", "settings.gradle.kts"] {
            let linked = TempDir::new().unwrap();
            fs::write(linked.path().join("real.gradle"), "plugins {}\n").unwrap();
            symlink(
                linked.path().join("real.gradle"),
                linked.path().join(manifest),
            )
            .unwrap();
            let error = ProjectSnapshot::create(linked.path())
                .err()
                .expect("a linked Gradle manifest must fail snapshot preflight");
            assert!(error.contains("symlink or reparse point"), "{error}");
        }
    }

    #[test]
    fn snapshot_preflight_applies_the_gradle_limit_before_discovery() {
        for manifest in [
            "build.gradle.kts",
            "build.gradle",
            "settings.gradle.kts",
            "settings.gradle",
        ] {
            let tmp = TempDir::new().unwrap();
            fs::write(
                tmp.path().join(manifest),
                vec![b' '; MAX_GRADLE_MANIFEST_BYTES as usize + 1],
            )
            .unwrap();

            let error = ProjectSnapshot::create(tmp.path())
                .err()
                .expect("an oversized Gradle manifest must fail snapshot preflight");
            assert!(error.contains("4 MiB"), "{manifest}: {error}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn gradle_manifest_path_replacement_cannot_block_or_change_preloaded_bytes() {
        use std::process::Command;
        use std::time::{Duration, Instant};

        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
        let source = open_test_directory(root);
        let mut budget = SnapshotBudget::default();
        let started = Instant::now();

        let result = read_capability_text_if_exists_with_limit_and_hook(
            &source,
            Path::new("build.gradle.kts"),
            &mut budget,
            MAX_GRADLE_MANIFEST_BYTES,
            || {
                fs::rename(
                    root.join("build.gradle.kts"),
                    root.join("original.gradle.kts"),
                )
                .unwrap();
                assert!(
                    Command::new("mkfifo")
                        .arg(root.join("build.gradle.kts"))
                        .status()
                        .unwrap()
                        .success()
                );
            },
        );

        assert!(result.is_err());
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn snapshot_manifest_reader_binds_the_retained_file_identity() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
        fs::write(
            root.join("replacement.toml"),
            "[workspace]\nmembers = [\"outside\"]\n",
        )
        .unwrap();
        let source = open_test_directory(root);
        let mut budget = SnapshotBudget::default();

        let result = read_capability_text_if_exists_with_hook(
            &source,
            Path::new("Cargo.toml"),
            &mut budget,
            || {
                fs::rename(root.join("Cargo.toml"), root.join("original.toml")).unwrap();
                fs::rename(root.join("replacement.toml"), root.join("Cargo.toml")).unwrap();
            },
        );

        assert!(
            result.is_err(),
            "a manifest path replaced after the retained open must be rejected"
        );
    }

    #[test]
    fn selected_config_directory_identity_stays_bound_to_the_pre_open_observation() {
        assert!(snapshot_configuration_identities_match(
            &1_u64,
            &[Some(1), Some(1), Some(1)]
        ));
        assert!(!snapshot_configuration_identities_match(
            &1_u64,
            &[Some(2), Some(2), Some(2)]
        ));
        assert!(!snapshot_configuration_identities_match(
            &1_u64,
            &[Some(1), None, Some(1)]
        ));
    }

    #[test]
    fn selected_config_parent_open_binds_the_pre_open_identity() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir(root.join("selected")).unwrap();
        fs::create_dir(root.join("replacement")).unwrap();
        let source = open_test_directory(root);

        let result = open_snapshot_configuration_directory_with_hook(
            &source,
            OsStr::new("selected"),
            Path::new("selected"),
            || {
                fs::rename(root.join("selected"), root.join("original")).unwrap();
                fs::rename(root.join("replacement"), root.join("selected")).unwrap();
            },
        );

        let error =
            result.expect_err("a replacement opened after metadata inspection must be rejected");
        assert!(error.contains("changed during inspection"), "{error}");
    }

    #[test]
    fn selected_config_open_binds_the_retained_file_identity() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::write(
            root.join("specsync.json"),
            r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
        )
        .unwrap();
        fs::write(
            root.join("other.json"),
            r#"{"specsDir":"other","sourceDirs":["other"]}"#,
        )
        .unwrap();
        let source = open_test_directory(root);

        let result =
            read_snapshot_configuration_with_hook(&source, Path::new("specsync.json"), || {
                fs::rename(root.join("specsync.json"), root.join("original.json")).unwrap();
                fs::rename(root.join("other.json"), root.join("specsync.json")).unwrap();
            });

        assert!(
            result.is_err(),
            "a regular selected config replaced after discovery must not be read"
        );
    }

    #[test]
    fn selected_config_read_revalidates_each_parent_identity() {
        for replaced_parent in ["selected", "selected/nested"] {
            let tmp = TempDir::new().unwrap();
            let root = tmp.path();
            fs::create_dir_all(root.join("selected/nested")).unwrap();
            fs::write(
                root.join("selected/nested/specsync.json"),
                r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
            )
            .unwrap();
            let replacement = root.join("replacement");
            if replaced_parent == "selected" {
                fs::create_dir_all(replacement.join("nested")).unwrap();
                fs::write(
                    replacement.join("nested/specsync.json"),
                    r#"{"specsDir":"outside","sourceDirs":["outside"]}"#,
                )
                .unwrap();
            } else {
                fs::create_dir(&replacement).unwrap();
                fs::write(
                    replacement.join("specsync.json"),
                    r#"{"specsDir":"outside","sourceDirs":["outside"]}"#,
                )
                .unwrap();
            }
            let source = open_test_directory(root);

            let result = read_snapshot_configuration_with_hook(
                &source,
                Path::new("selected/nested/specsync.json"),
                || {
                    let selected_parent = root.join(replaced_parent);
                    let original_parent = if replaced_parent == "selected" {
                        root.join("original-selected")
                    } else {
                        root.join("selected/original-nested")
                    };
                    fs::rename(&selected_parent, original_parent).unwrap();
                    fs::rename(&replacement, selected_parent).unwrap();
                },
            );

            let error = result
                .expect_err("every selected-config parent replacement must fail after the read");
            assert!(
                error.contains("changed while the configuration was being read"),
                "{replaced_parent}: {error}"
            );
        }
    }

    #[test]
    fn snapshot_includes_python_package_derived_under_ignored_directory() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("vendor")).unwrap();
        fs::write(
            tmp.path().join("specsync.json"),
            r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
        )
        .unwrap();
        fs::write(
            tmp.path().join("pyproject.toml"),
            "[project]\nname = \"vendor\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(tmp.path().join("vendor/module.py"), "def public(): pass\n").unwrap();

        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();
        assert!(snapshot.root().join("vendor/module.py").is_file());
    }

    #[test]
    fn snapshot_includes_all_standard_gradle_module_forms_under_ignored_directories() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("build.gradle"), "plugins {}\n").unwrap();
        fs::write(
            tmp.path().join("settings.gradle"),
            r#"
include ':vendor:groovy',
        ':nested:second'
include(
    ":kotlin:member",
    ':override'
)
project(':override').projectDir = file('vendor/custom')
"#,
        )
        .unwrap();
        for path in [
            "vendor/groovy/src/main/kotlin/one.kt",
            "nested/second/src/main/java/Two.java",
            "kotlin/member/src/main/kotlin/three.kt",
            "vendor/custom/src/main/kotlin/four.kt",
        ] {
            let path = tmp.path().join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "public class Module {}\n").unwrap();
        }

        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();

        assert!(
            snapshot
                .root()
                .join("vendor/groovy/src/main/kotlin/one.kt")
                .is_file()
        );
        assert!(
            snapshot
                .root()
                .join("nested/second/src/main/java/Two.java")
                .is_file()
        );
        assert!(
            snapshot
                .root()
                .join("kotlin/member/src/main/kotlin/three.kt")
                .is_file()
        );
        assert!(
            snapshot
                .root()
                .join("vendor/custom/src/main/kotlin/four.kt")
                .is_file()
        );
    }

    // --- handle_initialize ---

    #[test]
    fn test_handle_initialize_response_format() {
        let resp = handle_initialize(Some(json!(1)));
        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 1);
        assert_eq!(resp["result"]["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(resp["result"]["serverInfo"]["name"], "specsync");
        assert!(resp["result"]["capabilities"]["tools"].is_object());
        assert!(resp["result"]["capabilities"]["resources"].is_object());
    }

    #[test]
    fn test_handle_initialize_null_id() {
        let resp = handle_initialize(None);
        assert_eq!(resp["jsonrpc"], "2.0");
        assert!(resp["result"]["protocolVersion"].is_string());
    }

    #[test]
    fn test_handle_initialize_string_id() {
        let resp = handle_initialize(Some(json!("req-42")));
        assert_eq!(resp["id"], "req-42");
    }

    // --- handle_tools_list ---

    #[test]
    fn test_handle_tools_list_returns_all_tools() {
        let resp = handle_tools_list(Some(json!(2)), true);
        let tools = resp["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 7);
    }

    #[test]
    fn test_handle_tools_list_defaults_to_read_only_tools() {
        let resp = handle_tools_list(Some(json!(2)), false);
        let tools = resp["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 5);
        assert!(
            tools
                .iter()
                .all(|tool| tool["name"] != "specsync_generate" && tool["name"] != "specsync_init")
        );
    }

    #[test]
    fn test_handle_tools_list_tool_names() {
        let resp = handle_tools_list(Some(json!(1)), true);
        let tools = resp["result"]["tools"].as_array().unwrap();
        let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"specsync_check"));
        assert!(names.contains(&"specsync_coverage"));
        assert!(names.contains(&"specsync_generate"));
        assert!(names.contains(&"specsync_list_specs"));
        assert!(names.contains(&"specsync_init"));
        assert!(names.contains(&"specsync_score"));
        assert!(names.contains(&"specsync_issues"));
    }

    #[test]
    fn test_handle_tools_list_all_have_schemas() {
        let resp = handle_tools_list(Some(json!(1)), true);
        let tools = resp["result"]["tools"].as_array().unwrap();
        for tool in tools {
            assert!(
                tool["inputSchema"].is_object(),
                "Tool {} missing inputSchema",
                tool["name"]
            );
            assert_eq!(tool["inputSchema"]["type"], "object");
            assert_eq!(tool["inputSchema"]["additionalProperties"], false);
        }
    }

    // --- handle_tools_call ---

    #[test]
    fn test_handle_tools_call_unknown_tool() {
        let tmp = setup_project();
        let params = json!({ "name": "nonexistent_tool", "arguments": {} });
        let resp = handle_tools_call(Some(json!(1)), &params, tmp.path(), false);
        assert_eq!(resp["result"]["isError"], true);
        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Unknown tool"));
    }

    #[test]
    fn test_handle_tools_call_custom_root() {
        let tmp = setup_project();
        let server_root = tmp.path().canonicalize().unwrap();
        // coverage tool should work with empty specs
        let params = json!({
            "name": "specsync_coverage",
            "arguments": { "root": server_root.to_string_lossy() }
        });
        let resp = handle_tools_call(Some(json!(1)), &params, &server_root, false);
        assert!(!resp["result"]["isError"].as_bool().unwrap_or(false));
    }

    // --- load_and_discover ---

    #[test]
    fn test_load_and_discover_empty_allowed() {
        let tmp = setup_project();
        let result = load_and_discover(tmp.path(), true);
        assert!(result.is_ok());
        let (config, specs) = result.unwrap();
        assert_eq!(config.specs_dir, "specs");
        assert!(specs.is_empty());
    }

    #[test]
    fn test_load_and_discover_empty_not_allowed() {
        let tmp = setup_project();
        let result = load_and_discover(tmp.path(), false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No spec files found"));
    }

    #[test]
    fn test_load_and_discover_filters_private_specs() {
        let tmp = setup_project();
        let spec_dir = tmp.path().join("specs").join("_private");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(
            spec_dir.join("_private.spec.md"),
            "---\nmodule: private\n---",
        )
        .unwrap();

        let result = load_and_discover(tmp.path(), true);
        assert!(result.is_ok());
        let (_config, specs) = result.unwrap();
        // Private specs (starting with _) should be filtered out
        assert!(specs.is_empty());
    }

    #[test]
    fn test_load_and_discover_finds_real_specs() {
        let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth module\n\n# Public API\nNone\n";
        let tmp = setup_project_with_spec("auth", spec_content);

        let result = load_and_discover(tmp.path(), false);
        assert!(result.is_ok());
        let (_config, specs) = result.unwrap();
        assert_eq!(specs.len(), 1);
    }

    // --- tool_init ---

    #[test]
    fn test_tool_init_creates_config() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();

        let directory = open_test_directory(tmp.path());
        let result = tool_init(tmp.path(), &directory);
        assert!(
            result.is_ok(),
            "unexpected initialization error: {result:?}"
        );
        let val = result.unwrap();
        assert_eq!(val["created"], true);
        assert!(tmp.path().join("specsync.json").exists());
        assert_no_mcp_transaction_debris(tmp.path());
    }

    #[test]
    fn test_tool_init_already_exists() {
        let tmp = setup_project();
        let directory = open_test_directory(tmp.path());
        let result = tool_init(tmp.path(), &directory);
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["created"], false);
        assert!(val["message"].as_str().unwrap().contains("already exists"));
    }

    #[test]
    fn test_generation_outcome_rejects_missing_required_output() {
        let tmp = TempDir::new().unwrap();
        let expected = vec![(
            "missing".to_string(),
            tmp.path().join("specs/missing/missing.spec.md"),
        )];
        let outcome = crate::generator::GenerationOutcome {
            generated: 1,
            generated_paths: vec!["specs/missing/missing.spec.md".to_string()],
        };

        let error = validate_generation_outcome(tmp.path(), &expected, &outcome).unwrap_err();
        assert!(error.contains("did not create the required spec"));
    }

    #[test]
    fn test_generation_outcome_rejects_incomplete_count() {
        let tmp = TempDir::new().unwrap();
        let expected = vec![(
            "missing".to_string(),
            tmp.path().join("specs/missing/missing.spec.md"),
        )];
        let outcome = crate::generator::GenerationOutcome::default();

        let error = validate_generation_outcome(tmp.path(), &expected, &outcome).unwrap_err();
        assert!(error.contains("created 0 of 1 required specs"));
    }

    // --- tool_coverage ---

    #[test]
    fn test_tool_coverage_empty_project() {
        let tmp = setup_project();
        let result = tool_coverage(tmp.path());
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["file_coverage"], 100.0);
        assert_eq!(val["files_total"], 0);
    }

    #[test]
    fn test_tool_coverage_with_unspecced_files() {
        let tmp = setup_project();
        // Create a source file without a spec
        std::fs::write(tmp.path().join("src").join("main.rs"), "fn main() {}").unwrap();

        let result = tool_coverage(tmp.path());
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(val["files_total"].as_u64().unwrap() > 0);
    }

    #[test]
    fn malformed_gradle_makes_every_mcp_coverage_consumer_inconclusive() {
        let tmp = setup_project();
        fs::write(
            tmp.path().join("settings.gradle.kts"),
            "include(\"member\"\n",
        )
        .unwrap();
        fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
        let directory = open_test_directory(tmp.path());

        let errors = [
            resource_coverage(tmp.path()).unwrap_err(),
            tool_check(tmp.path(), &json!({})).unwrap_err(),
            tool_coverage(tmp.path()).unwrap_err(),
            tool_generate(tmp.path(), &directory, &json!({})).unwrap_err(),
        ];

        for error in errors {
            assert!(error.contains("Gradle"), "unexpected error: {error}");
            assert!(
                error.contains("inconclusive") || error.contains("Cannot parse"),
                "malformed Gradle must not become a compatibility coverage report: {error}"
            );
        }
    }

    // --- tool_list_specs ---

    #[test]
    fn test_tool_list_specs_empty() {
        let tmp = setup_project();
        let result = tool_list_specs(tmp.path());
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["count"], 0);
        assert!(val["specs"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_tool_list_specs_with_frontmatter() {
        let spec_content = "---\nmodule: auth\nversion: 2.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth\n";
        let tmp = setup_project_with_spec("auth", spec_content);

        let result = tool_list_specs(tmp.path());
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["count"], 1);
        let spec = &val["specs"][0];
        assert_eq!(spec["module"], "auth");
        assert_eq!(spec["version"], "2.0.0");
        assert_eq!(spec["status"], "stable");
    }

    #[test]
    fn test_tool_list_specs_malformed_frontmatter() {
        let tmp = setup_project();
        let spec_dir = tmp.path().join("specs").join("bad");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(spec_dir.join("bad.spec.md"), "no frontmatter here").unwrap();

        let result = tool_list_specs(tmp.path());
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["count"], 1);
        let spec = &val["specs"][0];
        assert!(spec["module"].is_null());
    }

    // --- tool_check ---

    #[test]
    fn test_tool_check_strict_mode() {
        let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth module\n\n# Public API\nNone\n";
        let tmp = setup_project_with_spec("auth", spec_content);
        std::fs::write(tmp.path().join("src").join("auth.rs"), "pub fn login() {}").unwrap();

        let result_normal = tool_check(tmp.path(), &json!({ "strict": false }));
        assert!(result_normal.is_ok());

        let result_strict = tool_check(tmp.path(), &json!({ "strict": true }));
        assert!(result_strict.is_ok());
    }

    #[test]
    fn test_tool_check_no_specs_error() {
        let tmp = setup_project();
        let result = tool_check(tmp.path(), &json!({}));
        assert!(result.is_err());
    }

    // --- tool_score ---

    #[test]
    fn test_tool_score_no_specs_error() {
        let tmp = setup_project();
        let result = tool_score(tmp.path(), true);
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_score_with_spec() {
        let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth module\n\n# Public API\nNone\n\n# Invariants\nNone\n\n# Behavioral Examples\nNone\n\n# Error Cases\nNone\n\n# Dependencies\nNone\n\n# Change Log\nNone\n";
        let tmp = setup_project_with_spec("auth", spec_content);
        std::fs::write(tmp.path().join("src").join("auth.rs"), "pub fn login() {}").unwrap();

        let result = tool_score(tmp.path(), true);
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(val["average_score"].as_f64().unwrap() >= 0.0);
        assert!(val["grade"].is_string());
        assert_eq!(val["total_specs"], 1);
        assert!(val["distribution"].is_object());
    }

    // --- tool_generate ---

    #[test]
    fn test_tool_generate_no_uncovered() {
        let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth\n";
        let tmp = setup_project_with_spec("auth", spec_content);
        std::fs::write(tmp.path().join("src").join("auth.rs"), "pub fn login() {}").unwrap();

        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();
        let directory = open_test_directory(tmp.path());
        let result = tool_generate(snapshot.root(), &directory, &json!({}));
        assert!(result.is_ok());
    }

    #[test]
    fn test_tool_generate_creates_spec() {
        let tmp = setup_project();
        std::fs::write(tmp.path().join("src").join("auth.rs"), "pub fn login() {}").unwrap();

        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();
        let directory = open_test_directory(tmp.path());
        let result = tool_generate(snapshot.root(), &directory, &json!({}));
        assert!(result.is_ok(), "unexpected generation error: {result:?}");
        let val = result.unwrap();
        assert_eq!(val["count"].as_u64(), Some(1));
        let generated = val["generated"].as_array().unwrap();
        assert_eq!(generated.len(), 1);
        let generated_path = std::path::Path::new(generated[0].as_str().unwrap());
        assert!(!generated_path.is_absolute());
        assert!(tmp.path().join(generated_path).is_file());
        assert!(
            generated_path.ends_with(
                std::path::Path::new("specs")
                    .join("auth")
                    .join("auth.spec.md")
            )
        );
        assert_no_mcp_transaction_debris(tmp.path());
    }

    #[test]
    fn failed_generated_batch_rolls_back_files_and_retains_ambiguous_empty_parent() {
        let tmp = setup_project();
        let directory = open_test_directory(tmp.path());
        fs::create_dir_all(tmp.path().join("specs/blocked")).unwrap();
        fs::write(
            tmp.path().join("specs/blocked/blocked.spec.md"),
            "existing\n",
        )
        .unwrap();

        let error = write_generated_specs_transactionally(
            &directory,
            vec![
                (
                    PathBuf::from("specs/created/created.spec.md"),
                    "created\n".to_string(),
                ),
                (
                    PathBuf::from("specs/blocked/blocked.spec.md"),
                    "replacement\n".to_string(),
                ),
            ],
        )
        .unwrap_err();

        assert!(
            error.contains("Cannot atomically publish confined MCP generated spec"),
            "unexpected publication error: {error}"
        );
        assert!(!tmp.path().join("specs/created/created.spec.md").exists());
        assert!(tmp.path().join("specs/created").is_dir());
        assert!(
            fs::read_dir(tmp.path().join("specs/created"))
                .unwrap()
                .next()
                .is_none()
        );
        assert_eq!(
            fs::read_to_string(tmp.path().join("specs/blocked/blocked.spec.md")).unwrap(),
            "existing\n"
        );
        assert_no_mcp_transaction_debris(tmp.path());
    }

    #[cfg(unix)]
    #[test]
    fn generation_publication_and_rollback_remain_bound_to_a_replaced_parent() {
        let tmp = setup_project();
        fs::create_dir_all(tmp.path().join("specs")).unwrap();
        let directory = open_test_directory(tmp.path());
        let mut staged = stage_new_confined(
            &directory,
            Path::new("specs/target/generated.spec.md"),
            b"generated\n",
            "generated spec",
        )
        .unwrap();

        fs::rename(
            tmp.path().join("specs/target"),
            tmp.path().join("specs/moved-target"),
        )
        .unwrap();
        fs::create_dir_all(tmp.path().join("specs/target")).unwrap();
        fs::write(
            tmp.path().join("specs/target/generated.spec.md"),
            "replacement\n",
        )
        .unwrap();

        publish_staged_file(&mut staged, "generated spec").unwrap();
        assert_eq!(
            fs::read_to_string(tmp.path().join("specs/moved-target/generated.spec.md")).unwrap(),
            "generated\n"
        );
        let rollback = rollback_staged_batch(&[staged], "forced rollback".to_string());

        assert_eq!(rollback, "forced rollback");
        assert_eq!(
            fs::read_to_string(tmp.path().join("specs/target/generated.spec.md")).unwrap(),
            "replacement\n"
        );
        assert!(
            !tmp.path()
                .join("specs/moved-target/generated.spec.md")
                .exists()
        );
        assert!(
            fs::read_dir(tmp.path().join("specs/moved-target"))
                .unwrap()
                .next()
                .is_none()
        );
    }

    #[test]
    fn staging_detects_a_same_entry_replacement_after_identity_capture() {
        let tmp = setup_project();
        let directory = open_test_directory(tmp.path());
        let mut staged = stage_new_confined_with_hook(
            &directory,
            Path::new("specs/replaced/replaced.spec.md"),
            b"generated\n",
            "generated spec",
            |parent, name| {
                parent.remove_file(name).unwrap();
                parent.write(name, b"replacement\n").unwrap();
            },
        )
        .unwrap();

        let error = publish_staged_file(&mut staged, "generated spec").unwrap_err();

        assert!(error.contains("Refusing to publish replacement staged MCP"));
        assert_eq!(
            staged.parent.read(&staged.temporary_name).unwrap(),
            b"replacement\n"
        );
        assert!(!tmp.path().join("specs/replaced/replaced.spec.md").exists());
    }

    #[test]
    fn publication_quarantines_a_replacement_swapped_after_verification() {
        let tmp = setup_project();
        let directory = open_test_directory(tmp.path());
        let mut staged = stage_new_confined(
            &directory,
            Path::new("specs/replaced/replaced.spec.md"),
            b"generated\n",
            "generated spec",
        )
        .unwrap();

        let error = publish_staged_file_with_hook(&mut staged, "generated spec", |parent, name| {
            parent.remove_file(name).unwrap();
            parent.write(name, b"replacement\n").unwrap();
        })
        .unwrap_err();

        assert!(error.contains("quarantined replacement was preserved"));
        assert!(!tmp.path().join("specs/replaced/replaced.spec.md").exists());
        assert!(quarantine_contains(&staged.parent, b"replacement\n"));
    }

    #[test]
    fn file_rollback_quarantines_a_replacement_swapped_after_verification() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("generated.spec.md"), "generated\n").unwrap();
        let parent = open_test_directory(tmp.path());
        let expected = file_identity(&parent.open("generated.spec.md").unwrap()).unwrap();

        let error = remove_identity_bound_file_with_hook(
            &parent,
            Path::new("generated.spec.md"),
            Path::new("generated.spec.md"),
            expected,
            |parent, name| {
                parent.remove_file(name).unwrap();
                parent.write(name, b"replacement\n").unwrap();
            },
        )
        .unwrap_err();

        assert!(error.contains("quarantined replacement was preserved"));
        assert!(!tmp.path().join("generated.spec.md").exists());
        assert!(quarantine_contains(&parent, b"replacement\n"));
    }

    #[test]
    fn file_identity_rejects_changed_bytes_on_the_same_filesystem_entry() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("generated.spec.md"), "generated\n").unwrap();
        let parent = open_test_directory(tmp.path());
        let original = file_identity(&parent.open("generated.spec.md").unwrap()).unwrap();

        fs::write(tmp.path().join("generated.spec.md"), "replacement\n").unwrap();
        let replacement = file_identity(&parent.open("generated.spec.md").unwrap()).unwrap();

        assert_ne!(original, replacement);
    }

    #[test]
    fn file_identity_digest_fails_closed_at_the_generated_output_bound() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("oversized"), b"123456789").unwrap();
        let parent = open_test_directory(tmp.path());
        let file = parent.open("oversized").unwrap();

        let error = file_content_digest_with_limit(&file, 8).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("8-byte limit"));
    }

    fn quarantine_contains(parent: &Dir, expected: &[u8]) -> bool {
        parent
            .read_dir(".")
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".specsync-mcp-quarantine-")
            })
            .any(|entry| {
                let Ok(directory) = entry.open_dir() else {
                    return false;
                };
                if directory.read("entry").is_ok_and(|bytes| bytes == expected) {
                    return true;
                }
                let Ok(nested) = directory.open_dir("entry") else {
                    return false;
                };
                nested
                    .read("replacement.txt")
                    .is_ok_and(|bytes| bytes == expected)
            })
    }

    #[test]
    fn generation_rejects_an_unbounded_output_count_before_reading_files() {
        let tmp = TempDir::new().unwrap();
        let expected = (0..=MAX_GENERATED_SPECS)
            .map(|index| {
                (
                    format!("module-{index}"),
                    tmp.path()
                        .join(format!("specs/module-{index}/module-{index}.spec.md")),
                )
            })
            .collect::<Vec<_>>();
        let outcome = crate::generator::GenerationOutcome {
            generated: expected.len(),
            generated_paths: Vec::new(),
        };

        let error = validate_generation_outcome(tmp.path(), &expected, &outcome).unwrap_err();
        assert!(error.contains("1000-spec output limit"));
    }

    #[test]
    fn generation_rejects_cumulative_output_bytes_before_publication() {
        let tmp = TempDir::new().unwrap();
        let first = tmp.path().join("specs/first/first.spec.md");
        let second = tmp.path().join("specs/second/second.spec.md");
        fs::create_dir_all(first.parent().unwrap()).unwrap();
        fs::create_dir_all(second.parent().unwrap()).unwrap();
        fs::write(&first, "12345").unwrap();
        fs::write(&second, "67890").unwrap();
        let expected = vec![("first".to_string(), first), ("second".to_string(), second)];
        let outcome = crate::generator::GenerationOutcome {
            generated: 2,
            generated_paths: vec![
                "specs/first/first.spec.md".to_string(),
                "specs/second/second.spec.md".to_string(),
            ],
        };

        assert_eq!(MAX_GENERATED_OUTPUT_BYTES, 64 * 1024 * 1024);
        let error = validate_generation_outcome_with_limits(
            tmp.path(),
            &expected,
            &outcome,
            MAX_GENERATED_SPECS,
            8,
        )
        .unwrap_err();
        assert!(error.contains("cumulative output limit"));
    }

    #[test]
    fn generation_rejects_an_oversized_result_during_response_preflight() {
        let result = json!({
            "generated": ["specs/a/very-long-generated-spec-name.spec.md"],
            "count": 1,
        });

        let error = validate_tool_content_response_size_with_limit(&result, 64).unwrap_err();
        assert!(error.contains("bounded response limit"));
    }

    #[test]
    fn test_tool_generate_rejects_retired_ai_arguments_without_echoing_values() {
        let tmp = setup_project();
        let secret = "sk-do-not-echo";
        let directory = open_test_directory(tmp.path());
        let error =
            tool_generate(tmp.path(), &directory, &json!({ "apiKey": secret })).unwrap_err();
        assert!(error.contains("removed in spec-sync 5.0"));
        assert!(!error.contains(secret));
    }

    #[test]
    fn test_tool_generate_rejects_unknown_arguments() {
        let tmp = setup_project();
        let directory = open_test_directory(tmp.path());
        let error =
            tool_generate(tmp.path(), &directory, &json!({ "unexpected": true })).unwrap_err();
        assert!(error.contains("Unknown MCP generate argument `unexpected`"));
    }

    // --- JSONRPC response structure ---

    #[test]
    fn test_tools_call_success_response_structure() {
        let tmp = setup_project();
        let params = json!({ "name": "specsync_coverage", "arguments": {} });
        let resp = handle_tools_call(Some(json!(42)), &params, tmp.path(), false);

        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 42);
        assert!(resp["result"]["content"].is_array());
        assert_eq!(resp["result"]["content"][0]["type"], "text");
    }

    #[test]
    fn test_tools_call_error_response_structure() {
        let tmp = setup_project();
        let params = json!({ "name": "bogus", "arguments": {} });
        let resp = handle_tools_call(Some(json!(99)), &params, tmp.path(), false);

        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 99);
        assert_eq!(resp["result"]["isError"], true);
    }

    // --- handle_resources_list ---

    #[test]
    fn test_handle_resources_list_returns_resources() {
        let resp = handle_resources_list(Some(json!(1)));
        assert_eq!(resp["jsonrpc"], "2.0");
        let resources = resp["result"]["resources"].as_array().unwrap();
        assert_eq!(resources.len(), 4);
        let uris: Vec<&str> = resources
            .iter()
            .map(|r| r["uri"].as_str().unwrap())
            .collect();
        assert!(uris.contains(&"specsync:///specs"));
        assert!(uris.contains(&"specsync:///graph"));
        assert!(uris.contains(&"specsync:///config"));
        assert!(uris.contains(&"specsync:///coverage"));
    }

    #[test]
    fn test_handle_resources_list_has_templates() {
        let resp = handle_resources_list(Some(json!(1)));
        let templates = resp["result"]["resourceTemplates"].as_array().unwrap();
        assert_eq!(templates.len(), 1);
        assert_eq!(
            templates[0]["uriTemplate"].as_str().unwrap(),
            "specsync:///specs/{module}"
        );
    }

    // --- handle_resources_read ---

    #[test]
    fn test_resource_specs_list_empty() {
        let tmp = setup_project();
        let params = json!({ "uri": "specsync:///specs" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        assert_eq!(resp["jsonrpc"], "2.0");
        let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["count"], 0);
    }

    #[test]
    fn test_resource_specs_list_with_spec() {
        let spec_content = "---\nmodule: auth\nversion: 2.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth\n";
        let tmp = setup_project_with_spec("auth", spec_content);

        let params = json!({ "uri": "specsync:///specs" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["count"], 1);
        assert_eq!(parsed["specs"][0]["module"], "auth");
        assert!(parsed["specs"][0]["score"].is_number());
        assert_eq!(parsed["git_freshness_available"], false);
        assert_eq!(parsed["specs"][0]["git_freshness_available"], false);
    }

    #[test]
    fn test_resource_spec_by_module() {
        let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth module\n";
        let tmp = setup_project_with_spec("auth", spec_content);

        let params = json!({ "uri": "specsync:///specs/auth" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
        assert!(text.contains("module: auth"));
        assert_eq!(
            resp["result"]["contents"][0]["mimeType"].as_str().unwrap(),
            "text/markdown"
        );
    }

    #[test]
    fn test_resource_spec_by_module_not_found() {
        let tmp = setup_project();
        let params = json!({ "uri": "specsync:///specs/nonexistent" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        assert!(
            resp["error"]["message"]
                .as_str()
                .unwrap()
                .contains("No spec found")
        );
    }

    #[test]
    fn test_resource_graph_empty() {
        let tmp = setup_project();
        let params = json!({ "uri": "specsync:///graph" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["module_count"], 0);
        assert_eq!(parsed["edge_count"], 0);
    }

    #[test]
    fn test_resource_config() {
        let tmp = setup_project();
        let params = json!({ "uri": "specsync:///config" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["specs_dir"], "specs");
    }

    #[test]
    fn test_resource_coverage_empty() {
        let tmp = setup_project();
        let params = json!({ "uri": "specsync:///coverage" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["file_coverage_percent"], 100.0);
    }

    #[test]
    fn test_resource_unknown_uri() {
        let tmp = setup_project();
        let params = json!({ "uri": "specsync:///bogus" });
        let resp = handle_resources_read(Some(json!(1)), &params, tmp.path());
        assert!(
            resp["error"]["message"]
                .as_str()
                .unwrap()
                .contains("Unknown resource URI")
        );
    }

    #[test]
    fn test_run_mcp_server_rejects_a_non_directory_root() {
        let tmp = TempDir::new().unwrap();
        let file_root = tmp.path().join("not-a-directory");
        fs::write(&file_root, "not a project root").unwrap();

        let error = run_mcp_server(&file_root, false).unwrap_err();
        assert!(error.contains("not a directory"));
    }

    #[cfg(unix)]
    #[test]
    fn test_retained_server_capability_survives_root_path_swap() {
        use std::os::unix::fs::symlink;

        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("server");
        let moved = tmp.path().join("server-moved");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn local() {}\n").unwrap();
        fs::write(outside.join("victim"), "outside\n").unwrap();
        let directory = open_test_directory(&root);
        fs::rename(&root, &moved).unwrap();
        symlink(&outside, &root).unwrap();

        let response = handle_tools_call_with_directory(
            Some(json!(1)),
            &json!({
                "name": "specsync_init",
                "arguments": {}
            }),
            &root,
            &root,
            &directory,
            true,
        );

        assert_eq!(response["result"]["isError"].as_bool(), None);
        assert!(moved.join("specsync.json").is_file());
        assert!(!outside.join("specsync.json").exists());
        assert_eq!(
            fs::read_to_string(outside.join("victim")).unwrap(),
            "outside\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn retained_read_root_resolution_ignores_a_replacement_server_path() {
        use std::os::unix::fs::symlink;

        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("server");
        let moved = tmp.path().join("server-moved");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(root.join("child/src")).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(
            root.join("child/specsync.json"),
            r#"{"specsDir":"specs","sourceDirs":["src"]}"#,
        )
        .unwrap();
        fs::write(root.join("child/src/lib.rs"), "pub fn retained() {}\n").unwrap();
        let directory = open_test_directory(&root);
        fs::rename(&root, &moved).unwrap();
        symlink(&outside, &root).unwrap();

        let retained = handle_tools_call_with_directory(
            Some(json!(1)),
            &json!({
                "name": "specsync_coverage",
                "arguments": { "root": "child" }
            }),
            &root,
            &root,
            &directory,
            false,
        );
        assert_eq!(retained["result"]["isError"], Value::Null);

        let missing_before = handle_tools_call_with_directory(
            Some(json!(2)),
            &json!({
                "name": "specsync_coverage",
                "arguments": { "root": "probe" }
            }),
            &root,
            &root,
            &directory,
            false,
        );
        fs::create_dir_all(outside.join("probe/src")).unwrap();
        fs::write(outside.join("probe/src/outside.rs"), "outside\n").unwrap();
        let missing_after = handle_tools_call_with_directory(
            Some(json!(3)),
            &json!({
                "name": "specsync_coverage",
                "arguments": { "root": "probe" }
            }),
            &root,
            &root,
            &directory,
            false,
        );
        assert_eq!(missing_before["result"]["isError"], true);
        assert_eq!(missing_after["result"]["isError"], true);
        assert_eq!(
            missing_before["result"]["content"][0]["text"],
            missing_after["result"]["content"][0]["text"]
        );
    }

    #[test]
    fn test_snapshot_excludes_mixed_case_git_metadata() {
        let tmp = setup_project();
        fs::create_dir_all(tmp.path().join(".GIT")).unwrap();
        fs::write(tmp.path().join(".GIT/config"), "gitdir = outside\n").unwrap();

        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();

        assert!(!snapshot.root().join(".GIT").exists());
        assert!(!snapshot.root().join(".git").exists());
        assert!(!snapshot.git_freshness_available());
    }

    #[test]
    fn test_read_mcp_line_rejects_and_drains_an_oversized_request() {
        let mut input = vec![b'x'; MAX_JSON_RPC_LINE_BYTES + 1];
        input.extend_from_slice(b"\n{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"ping\"}\n");
        let mut reader = std::io::Cursor::new(input);

        let first = read_mcp_line(&mut reader).unwrap().unwrap().unwrap_err();
        assert!(first.contains("1 MiB"));
        let second = read_mcp_line(&mut reader).unwrap().unwrap().unwrap();
        assert!(second.contains("\"id\":2"));
        assert!(read_mcp_line(&mut reader).unwrap().is_none());
    }

    #[test]
    fn test_read_mcp_line_propagates_reader_failures() {
        struct FailingReader;

        impl Read for FailingReader {
            fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("reader failed"))
            }
        }

        impl BufRead for FailingReader {
            fn fill_buf(&mut self) -> io::Result<&[u8]> {
                Err(io::Error::other("reader failed"))
            }

            fn consume(&mut self, _amount: usize) {}
        }

        let error = read_mcp_line(&mut FailingReader).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert_eq!(error.to_string(), "reader failed");
    }

    #[test]
    fn test_mcp_score_always_marks_git_freshness_unavailable_and_fails_closed() {
        let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuthentication behavior.\n\n# Requirements\n- Authenticate users.\n\n# Public API\n| Name | Kind | Description |\n| --- | --- | --- |\n| `login` | Function | Authenticates a user. |\n\n# Invariants\n- Credentials are validated.\n\n# Behavioral Examples\nCalling `login` validates credentials.\n\n# Error Cases\nInvalid credentials return an error.\n\n# Dependencies\nNone.\n\n# Change Log\n- 1.0.0: Initial specification.\n";
        let tmp = setup_project_with_spec("auth", spec_content);
        fs::write(tmp.path().join("src/auth.rs"), "pub fn login() {}\n").unwrap();
        let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();
        assert!(!snapshot.git_freshness_available());
        let config = load_config(snapshot.root());
        let spec_path = snapshot.root().join("specs/auth/auth.spec.md");
        let baseline = scoring::score_spec(&spec_path, snapshot.root(), &config);
        let confined = score_spec_for_mcp(&spec_path, snapshot.root(), &config, false);

        assert_eq!(confined.total, baseline.total.saturating_sub(5));
        assert_eq!(
            confined.freshness_score,
            baseline.freshness_score.saturating_sub(5)
        );
        assert!(
            confined
                .suggestions
                .iter()
                .any(|suggestion| suggestion.contains("Git history is intentionally unavailable"))
        );
        let output = tool_score(snapshot.root(), snapshot.git_freshness_available()).unwrap();
        assert_eq!(output["git_freshness_available"], false);
        assert_eq!(output["specs"][0]["git_freshness_available"], false);
    }

    #[test]
    fn test_invalid_json_rpc_envelopes_fail_before_dispatch() {
        for request in [
            json!([]),
            json!({"id": 1, "method": "ping"}),
            json!({"jsonrpc": "1.0", "id": 1, "method": "ping"}),
            json!({"jsonrpc": "2.0", "id": 1, "method": 7}),
            json!({"jsonrpc": "2.0", "id": true, "method": "ping"}),
            json!({"jsonrpc": "2.0", "id": 1, "method": "ping", "params": true}),
        ] {
            let response = validate_request_envelope(&request).unwrap_err();
            assert_eq!(response["error"]["code"], -32600);
            assert!(response["id"].is_null());
        }

        let notification = json!({"jsonrpc": "2.0", "method": "ping"});
        assert_eq!(validate_request_envelope(&notification).unwrap(), None);
        let request = json!({"jsonrpc": "2.0", "id": "safe", "method": "ping"});
        assert_eq!(
            validate_request_envelope(&request).unwrap(),
            Some(json!("safe"))
        );
    }

    #[test]
    fn test_oversized_response_is_replaced_with_a_bounded_error() {
        let response = json!({
            "jsonrpc": "2.0",
            "id": 19,
            "result": "x".repeat(MAX_JSON_RPC_RESPONSE_BYTES + 1)
        });
        let mut output = Vec::new();

        write_mcp_response(&mut output, &response).unwrap();

        assert!(output.len() < MAX_JSON_RPC_RESPONSE_BYTES);
        let bounded: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(bounded["id"], 19);
        assert_eq!(bounded["error"]["code"], -32603);
    }

    #[test]
    fn test_response_writer_failures_are_propagated() {
        struct FailingWriter;

        impl Write for FailingWriter {
            fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("intentional writer failure"))
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let error = write_mcp_response(
            &mut FailingWriter,
            &json!({"jsonrpc": "2.0", "id": 1, "result": {}}),
        )
        .unwrap_err();
        assert!(error.contains("intentional writer failure"));
    }

    #[test]
    fn test_resources_read_rejects_non_exact_arguments() {
        let tmp = setup_project();
        for params in [
            json!([]),
            json!({}),
            json!({"uri": 7}),
            json!({"uri": "specsync:///config", "extra": true}),
        ] {
            let response = handle_resources_read(Some(json!(1)), &params, tmp.path());
            assert_eq!(response["error"]["code"], -32602);
        }
    }

    #[test]
    fn test_repeated_tree_scans_share_one_confinement_budget() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        fs::write(tmp.path().join("src/lib.rs"), "pub fn value() {}\n").unwrap();
        let exclusions = HashSet::new();
        let mut entries_seen = MAX_CONFINEMENT_ENTRIES - 2;

        validate_configured_path_with_budget(
            tmp.path(),
            "src",
            "source_dirs entry",
            Some(&exclusions),
            &mut entries_seen,
        )
        .unwrap();
        let error = validate_configured_path_with_budget(
            tmp.path(),
            "src",
            "source_dirs entry",
            Some(&exclusions),
            &mut entries_seen,
        )
        .unwrap_err();

        assert!(error.contains("confinement scan exceeds"));
    }

    #[test]
    fn test_cargo_manifest_preflight_enforces_manifest_bound() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
        let mut visiting = HashSet::new();
        let mut validated = HashSet::new();
        let mut manifests_seen = MAX_MANIFEST_PREFLIGHTS;
        let mut traversal_entries_seen = 0;

        let error = validate_cargo_workspace_manifest(
            tmp.path(),
            tmp.path(),
            &mut visiting,
            &mut validated,
            &mut manifests_seen,
            &mut traversal_entries_seen,
        )
        .unwrap_err();
        assert!(error.contains("preflight exceeds"));
    }

    #[test]
    fn cargo_preflight_charges_duplicate_members_without_replaying_manifests() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("crates/member")).unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/member\", \"./crates/member\"]\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("crates/member/Cargo.toml"),
            "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        let mut visiting = HashSet::new();
        let mut validated = HashSet::new();
        let mut manifests_seen = 0;
        let mut traversal_entries_seen = MAX_CONFINEMENT_ENTRIES - 2;

        validate_cargo_workspace_manifest(
            tmp.path(),
            tmp.path(),
            &mut visiting,
            &mut validated,
            &mut manifests_seen,
            &mut traversal_entries_seen,
        )
        .expect("duplicate members must fit exactly at the declaration limit");

        assert_eq!(traversal_entries_seen, MAX_CONFINEMENT_ENTRIES);
        assert_eq!(manifests_seen, 2);

        fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/member\", \"./crates/member\", \"crates/member\"]\n",
        )
        .unwrap();
        let mut visiting = HashSet::new();
        let mut validated = HashSet::new();
        let mut manifests_seen = 0;
        let mut traversal_entries_seen = MAX_CONFINEMENT_ENTRIES - 2;
        let error = validate_cargo_workspace_manifest(
            tmp.path(),
            tmp.path(),
            &mut visiting,
            &mut validated,
            &mut manifests_seen,
            &mut traversal_entries_seen,
        )
        .expect_err("the limit-plus-one duplicate declaration must be rejected");

        assert!(error.contains("exceeds 100000 entries"), "{error}");
        assert_eq!(manifests_seen, 2);
    }

    #[test]
    fn node_preflight_deduplicates_patterns_bases_and_workspace_paths() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("packages/member")).unwrap();
        fs::write(
            tmp.path().join("packages/member/package.json"),
            r#"{"name":"member"}"#,
        )
        .unwrap();
        fs::write(
            tmp.path().join("package.json"),
            r#"{"workspaces":["packages/*","packages/*","packages/**"]}"#,
        )
        .unwrap();
        let mut exact_entries_seen = MAX_CONFINEMENT_ENTRIES - 4;

        validate_package_workspaces(tmp.path(), &mut exact_entries_seen)
            .expect("three declarations plus one unique workspace must fit exactly");

        assert_eq!(exact_entries_seen, MAX_CONFINEMENT_ENTRIES);

        let mut overflow_entries_seen = MAX_CONFINEMENT_ENTRIES - 3;
        let error = validate_package_workspaces(tmp.path(), &mut overflow_entries_seen)
            .expect_err("the limit-plus-one Node declaration must be rejected");

        assert!(error.contains("exceeds 100000 entries"), "{error}");
    }

    #[test]
    fn node_preflight_fails_closed_and_charges_entries_before_type_validation() {
        for (content, expected) in [
            ("{", "Cannot parse MCP package.json as JSON"),
            ("false", "root must be a JSON object"),
            (
                r#"{"workspaces":"packages/*"}"#,
                "`workspaces` must be an array or an object",
            ),
            (
                r#"{"workspaces":{"packages":{}}}"#,
                "`workspaces.packages` must be an array",
            ),
            (
                r#"{"workspaces":{}}"#,
                "`workspaces` object must contain a `packages` array",
            ),
        ] {
            let tmp = TempDir::new().unwrap();
            fs::write(tmp.path().join("package.json"), content).unwrap();
            let mut entries_seen = 0;
            let error = validate_package_workspaces(tmp.path(), &mut entries_seen)
                .expect_err("invalid package.json must make MCP preflight inconclusive");
            assert!(error.contains(expected), "{content}: {error}");
        }

        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("package.json"),
            r#"{"workspaces":[false,"packages/*"]}"#,
        )
        .unwrap();
        let mut entries_seen = MAX_CONFINEMENT_ENTRIES - 2;
        let error = validate_package_workspaces(tmp.path(), &mut entries_seen)
            .expect_err("wrong-typed workspace entries must fail after declaration charging");

        assert!(
            error.contains("workspace entries must be strings"),
            "{error}"
        );
        assert_eq!(entries_seen, MAX_CONFINEMENT_ENTRIES);
    }
}
