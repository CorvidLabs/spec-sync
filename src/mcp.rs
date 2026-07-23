use crate::config::{detect_source_dirs, load_config};
use crate::deps::build_dep_graph;
use crate::generator::generate_specs_for_unspecced_modules_paths;
use crate::manifest::parse_gradle_settings;
use crate::scoring;
use crate::types::SpecSyncConfig;
use crate::validator::{
    compute_coverage_checked, find_spec_files, get_schema_table_names, validate_spec,
};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
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

/// Run the MCP server on stdio.
pub fn run_mcp_server(root: &Path, allow_write: bool) -> Result<(), String> {
    let (server_root, server_directory) = open_server_root_capability(root, || {})?;
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
    before_confined_open: impl FnOnce(),
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
    before_confined_open();

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
        copy_snapshot_configuration(source, &destination, &mut budget)?;
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
    let mut cargo_manifests = vec![PathBuf::from("Cargo.toml")];
    let mut seen = HashSet::new();
    while let Some(manifest) = cargo_manifests.pop() {
        if !seen.insert(manifest.clone()) {
            continue;
        }
        if seen.len() > MAX_MANIFEST_PREFLIGHTS {
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
            let member = snapshot_manifest_input(manifest_dir, &member, "Cargo workspace member")?;
            configured_inputs.push(member.clone());
            cargo_manifests.push(member.join("Cargo.toml"));
        }
        for path in cargo.paths {
            if path.is_empty() {
                continue;
            }
            let path = snapshot_manifest_input(manifest_dir, &path, "Cargo target path")?;
            configured_inputs.push(path.parent().unwrap_or(&path).to_path_buf());
        }
    }

    if let Some(content) =
        read_capability_text_if_exists(source, Path::new("package.json"), budget)?
        && let Ok(package) = serde_json::from_str::<Value>(&content)
    {
        let patterns: Vec<&str> = match package.get("workspaces") {
            Some(Value::Array(values)) => values.iter().filter_map(Value::as_str).collect(),
            Some(Value::Object(object)) => object
                .get("packages")
                .and_then(Value::as_array)
                .map(|values| values.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        for pattern in patterns {
            let base = pattern.trim_end_matches("/*").trim_end_matches("/**");
            configured_inputs.push(snapshot_manifest_input(
                Path::new(""),
                if base.is_empty() { "." } else { base },
                "package workspace base",
            )?);
        }
        let main = package.get("main").and_then(Value::as_str).unwrap_or("");
        if main.starts_with("./") {
            let main = snapshot_manifest_input(Path::new(""), main, "package main path")?;
            configured_inputs.push(main.parent().unwrap_or(&main).to_path_buf());
        }
        if source
            .try_exists("src")
            .map_err(|error| format!("Cannot inspect MCP package source directory: {error}"))?
        {
            configured_inputs.push(PathBuf::from("src"));
        } else if source
            .try_exists("lib")
            .map_err(|error| format!("Cannot inspect MCP package library directory: {error}"))?
        {
            configured_inputs.push(PathBuf::from("lib"));
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

    for settings in ["settings.gradle.kts", "settings.gradle"] {
        let Some(content) = read_capability_text_if_exists(source, Path::new(settings), budget)?
        else {
            continue;
        };
        for module in parse_gradle_settings(&content)? {
            configured_inputs.push(snapshot_manifest_input(
                Path::new(""),
                &module.path,
                "Gradle module",
            )?);
        }
        break;
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
    let document = toml::Value::Table(document);
    collect_cargo_manifest_paths(&document, manifest, &mut paths)?;
    Ok(CargoSnapshotManifest {
        workspace_members,
        paths,
    })
}

fn collect_cargo_manifest_paths(
    value: &toml::Value,
    manifest: &Path,
    paths: &mut Vec<String>,
) -> Result<(), String> {
    match value {
        toml::Value::Table(table) => {
            for (key, value) in table {
                if key == "path" {
                    let path = value.as_str().ok_or_else(|| {
                        format!(
                            "MCP Cargo workspace manifest {} has a non-string `path` value",
                            manifest.display()
                        )
                    })?;
                    paths.push(path.to_string());
                } else {
                    collect_cargo_manifest_paths(value, manifest, paths)?;
                }
            }
        }
        toml::Value::Array(values) => {
            for value in values {
                collect_cargo_manifest_paths(value, manifest, paths)?;
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
    if !source.try_exists(relative).map_err(|error| {
        format!(
            "Cannot inspect MCP snapshot manifest {}: {error}",
            relative.display()
        )
    })? {
        return Ok(None);
    }
    let file = source.open(relative).map_err(|error| {
        format!(
            "Cannot open MCP snapshot manifest {} through its root capability: {error}",
            relative.display()
        )
    })?;
    let mut bytes = Vec::new();
    file.take(MAX_PROJECT_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            format!(
                "Cannot read MCP snapshot manifest {}: {error}",
                relative.display()
            )
        })?;
    if bytes.len() as u64 > MAX_PROJECT_FILE_BYTES {
        return Err(format!(
            "MCP snapshot manifest exceeds the {} MiB per-file limit: {}",
            MAX_PROJECT_FILE_BYTES / (1024 * 1024),
            relative.display()
        ));
    }
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

fn snapshot_manifest_input(base: &Path, configured: &str, label: &str) -> Result<PathBuf, String> {
    let path = Path::new(configured);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!(
            "MCP {label} must be a project-relative path without traversal: {configured}"
        ));
    }
    Ok(normalize_snapshot_input(&base.join(path)))
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
    budget: &mut SnapshotBudget,
) -> Result<(), String> {
    for relative in [
        ".specsync/config.toml",
        ".specsync/config.json",
        ".specsync.toml",
        "specsync.json",
    ] {
        if !source.try_exists(relative).map_err(|error| {
            format!(
                "Cannot inspect MCP configuration {relative} through its root capability: {error}"
            )
        })? {
            continue;
        }
        let input = source.open(relative).map_err(|error| {
            format!("Cannot open MCP configuration {relative} through its root capability: {error}")
        })?;
        let mut bytes = Vec::new();
        input
            .take(MAX_PROJECT_FILE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| format!("Cannot read MCP configuration {relative}: {error}"))?;
        if bytes.len() as u64 > MAX_PROJECT_FILE_BYTES {
            return Err(format!(
                "MCP configuration exceeds the {} MiB per-file limit: {relative}",
                MAX_PROJECT_FILE_BYTES / (1024 * 1024)
            ));
        }
        let path = Path::new(relative);
        budget.charge_file(path, bytes.len() as u64)?;
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

fn copy_snapshot_directory(
    source: &Dir,
    destination: &Dir,
    relative: &Path,
    configured_exclusions: &HashSet<String>,
    configured_inputs: &[PathBuf],
    budget: &mut SnapshotBudget,
) -> Result<(), String> {
    destination.create_dir_all(relative).map_err(|error| {
        format!(
            "Cannot create MCP snapshot directory {}: {error}",
            relative.display()
        )
    })?;
    let entries = source.read_dir(relative).map_err(|error| {
        format!(
            "Cannot read MCP project directory {} through its root capability: {error}",
            relative.display()
        )
    })?;

    for entry in entries {
        budget.entries += 1;
        if budget.entries > MAX_CONFINEMENT_ENTRIES {
            return Err(format!(
                "MCP project snapshot exceeds {MAX_CONFINEMENT_ENTRIES} entries"
            ));
        }
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
        let file_type = entry.file_type().map_err(|error| {
            format!(
                "Cannot inspect MCP project input type {}: {error}",
                child.display()
            )
        })?;
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

        let metadata = source.metadata(&child).map_err(|error| {
            format!(
                "Cannot inspect MCP project input {} through its root capability: {error}",
                child.display()
            )
        })?;
        if metadata.is_dir() {
            copy_snapshot_directory(
                source,
                destination,
                &child,
                configured_exclusions,
                configured_inputs,
                budget,
            )?;
            continue;
        }
        if !metadata.is_file() {
            continue;
        }
        if metadata.len() > MAX_PROJECT_FILE_BYTES {
            return Err(format!(
                "MCP project input exceeds the {} MiB per-file limit: {}",
                MAX_PROJECT_FILE_BYTES / (1024 * 1024),
                child.display()
            ));
        }
        let input = source.open(&child).map_err(|error| {
            format!(
                "Cannot open MCP project input {} through its root capability: {error}",
                child.display()
            )
        })?;
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        input
            .take(MAX_PROJECT_FILE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| {
                format!("Cannot read MCP project input {}: {error}", child.display())
            })?;
        if bytes.len() as u64 > MAX_PROJECT_FILE_BYTES {
            return Err(format!(
                "MCP project input exceeds the {} MiB per-file limit: {}",
                MAX_PROJECT_FILE_BYTES / (1024 * 1024),
                child.display()
            ));
        }
        budget.charge_file(&child, bytes.len() as u64)?;
        destination.write(&child, bytes).map_err(|error| {
            format!(
                "Cannot copy MCP project input {} into the bounded snapshot: {error}",
                child.display()
            )
        })?;
        budget.copied_paths.insert(child);
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
    handle_tools_call_with_directory(id, params, server_root, &server_directory, allow_write)
}

fn handle_tools_call_with_directory(
    id: Option<Value>,
    params: &Value,
    server_root: &Path,
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
        match resolve_read_root(server_root, arguments.get("root").and_then(Value::as_str)) {
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

fn resolve_read_root(server_root: &Path, requested_root: Option<&str>) -> Result<PathBuf, String> {
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
        relative_read_root_suffix(server_root, requested_path)
            .ok_or_else(|| "Read root override escapes the configured server root".to_string())?
    } else {
        requested_path.to_path_buf()
    };
    Ok(if relative.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        relative
    })
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

#[cfg(any(windows, test))]
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

#[cfg(any(windows, test))]
fn windows_text_ignore_case(left: &str, right: &str) -> bool {
    left.to_lowercase() == right.to_lowercase()
}

#[cfg(any(windows, test))]
struct WindowsAbsolutePath {
    prefix: String,
    components: Vec<String>,
}

#[cfg(any(windows, test))]
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

#[cfg(any(windows, test))]
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
    validate_cargo_workspace_manifest(
        root,
        root,
        &mut visiting,
        &mut validated,
        &mut manifests_seen,
    )?;
    validate_package_workspaces(root)?;
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
    for member in cargo.workspace_members {
        let member_root = validate_manifest_relative_candidate(
            server_root,
            manifest_dir,
            &member,
            "Cargo workspace member",
        )?;
        let nested_manifest = member_root.join("Cargo.toml");
        match fs::symlink_metadata(&nested_manifest) {
            Ok(_) => validate_cargo_workspace_manifest(
                server_root,
                &member_root,
                visiting,
                validated,
                manifests_seen,
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

fn validate_package_workspaces(root: &Path) -> Result<(), String> {
    let package_path = root.join("package.json");
    let content = match read_file_bounded(root, &package_path, "package workspace manifest") {
        Ok(content) => content,
        Err(_) if !package_path.exists() => return Ok(()),
        Err(error) => return Err(error),
    };
    let json: Value = match serde_json::from_str(&content) {
        Ok(json) => json,
        Err(_) => return Ok(()),
    };
    let workspace_patterns: Vec<&str> = match json.get("workspaces") {
        Some(Value::Array(values)) => values.iter().filter_map(Value::as_str).collect(),
        Some(Value::Object(object)) => object
            .get("packages")
            .and_then(Value::as_array)
            .map(|values| values.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default(),
        _ => Vec::new(),
    };

    let mut entries_seen = 0usize;
    for pattern in workspace_patterns {
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
        if !base_dir.canonicalize().is_ok_and(|path| path.is_dir()) {
            continue;
        }
        let entries = match fs::read_dir(&base_dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries {
            entries_seen += 1;
            if entries_seen > MAX_CONFINEMENT_ENTRIES {
                return Err(format!(
                    "MCP package workspace preflight exceeds {MAX_CONFINEMENT_ENTRIES} entries"
                ));
            }
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
            let nested_package = entry.path().join("package.json");
            match fs::symlink_metadata(&nested_package) {
                Ok(_) => validate_existing_path(
                    root,
                    &nested_package,
                    "package workspace manifest",
                    None,
                )?,
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
            &quarantined,
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
    cleanup_quarantined_file(&quarantined, staged.identity, &staged.temporary_display)
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
    cleanup_quarantined_file(&quarantined, expected, display)
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
    quarantined: &QuarantinedEntry,
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
    quarantined: &QuarantinedEntry,
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
    quarantined
        .directory
        .try_clone()
        .map_err(|error| {
            format!(
                "Cannot retain rollback quarantine for handle-relative removal of {}: {error}",
                display.display()
            )
        })?
        .remove_open_dir()
        .map_err(|error| {
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
    use crate::parser::parse_frontmatter;

    let (config, spec_files) = load_and_discover(root, false)?;

    let mut results: Vec<Value> = Vec::new();
    let mut total_valid = 0usize;
    let mut total_closed = 0usize;
    let mut total_not_found = 0usize;
    let mut references = Vec::new();

    for spec_path in &spec_files {
        let content = read_file_bounded(root, spec_path, "spec file")?;

        let parsed = match parse_frontmatter(&content) {
            Some(p) => p,
            None => continue,
        };

        let fm = &parsed.frontmatter;
        if fm.implements.is_empty() && fm.tracks.is_empty() {
            continue;
        }

        let rel_path = spec_path
            .strip_prefix(root)
            .unwrap_or(spec_path)
            .to_string_lossy()
            .to_string();

        references.push((rel_path, fm.implements.clone(), fm.tracks.clone()));
    }

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
        })
        .err()
        .expect("a swapped root must be rejected");

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
            resolve_read_root(&canonical, Some(&ordinary)).unwrap(),
            PathBuf::from("Child")
        );
        assert_eq!(
            resolve_read_root(&canonical, Some(&case_varied)).unwrap(),
            PathBuf::from("CHILD")
        );

        let unicode_root = canonical.join("Ärger");
        fs::create_dir_all(unicode_root.join("Child")).unwrap();
        let unicode_candidate = unicode_root
            .join("Child")
            .to_string_lossy()
            .replace("Ärger", "ärger");
        assert_eq!(
            resolve_read_root(&unicode_root, Some(&unicode_candidate)).unwrap(),
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
        copy_snapshot_directory(
            &source,
            &explicit_destination,
            Path::new("."),
            &HashSet::from(["generated".to_string()]),
            &[PathBuf::from("generated")],
            &mut explicit_budget,
        )
        .expect("an explicit input must override an exclusion with the same symlink basename");

        assert_eq!(
            fs::read_to_string(explicit_destination_temp.path().join("generated/module.rs"))
                .unwrap(),
            "inside\n"
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
        assert!(error.contains("vendor/member/src/lib.rs"));
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
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["created"], true);
        assert!(tmp.path().join("specsync.json").exists());
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
        assert!(result.is_ok());
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

        assert!(error.contains("Cannot atomically publish confined MCP generated spec"));
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
        assert!(
            WalkDir::new(tmp.path())
                .into_iter()
                .filter_map(Result::ok)
                .all(|entry| !entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".specsync-mcp-stage-"))
        );
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

        let error = validate_cargo_workspace_manifest(
            tmp.path(),
            tmp.path(),
            &mut visiting,
            &mut validated,
            &mut manifests_seen,
        )
        .unwrap_err();
        assert!(error.contains("preflight exceeds"));
    }
}
