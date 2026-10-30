//! Cargo, Node, and Gradle inputs discovered for a project snapshot.

use super::confinement::MAX_CONFIGURED_PATHS;
use super::preflight::{MAX_MANIFEST_PREFLIGHTS, extract_manifest_toml_value};
use super::snapshot::{
    SnapshotBudget, read_capability_text_if_exists, read_capability_text_if_exists_with_limit,
};
use crate::manifest::{MAX_GRADLE_MANIFEST_BYTES, parse_gradle_settings};
use cap_std::fs::Dir;
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

pub(super) fn collect_snapshot_manifest_inputs(
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

pub(super) struct CargoSnapshotManifest {
    pub(super) workspace_members: Vec<String>,
    paths: Vec<String>,
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

pub(super) fn snapshot_input_overlaps(input: &Path, child: &Path) -> bool {
    let input = normalize_snapshot_input(input);
    input.as_os_str().is_empty() || input.starts_with(child) || child.starts_with(input)
}

#[cfg(test)]
mod tests;
