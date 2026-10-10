use serde_json::Value;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

pub(super) fn package_workspace_declarations<'a>(
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

pub(super) struct CargoSnapshotManifest {
    pub(super) workspace_members: Vec<String>,
    pub(super) paths: Vec<String>,
}

pub(super) fn parse_cargo_snapshot_manifest(
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

pub(super) fn snapshot_manifest_input(
    base: &Path,
    configured: &str,
    label: &str,
) -> Result<PathBuf, String> {
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

pub(super) fn normalize_snapshot_input(path: &Path) -> PathBuf {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part),
            Component::CurDir => None,
            _ => None,
        })
        .collect()
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

pub(super) fn extract_manifest_toml_value(
    content: &str,
    key: &str,
    section: &str,
) -> Option<String> {
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
