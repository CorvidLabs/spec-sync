use super::super::MAX_CONFIGURED_PATHS;
use super::super::MAX_MANIFEST_PREFLIGHTS;
use super::super::manifest_paths::extract_manifest_toml_value;
use super::super::manifest_paths::normalize_snapshot_input;
use super::super::manifest_paths::package_workspace_declarations;
use super::super::manifest_paths::parse_cargo_snapshot_manifest;
use super::super::manifest_paths::snapshot_manifest_input;
use super::SnapshotBudget;
use super::inputs::read_capability_text_if_exists;
use super::inputs::read_capability_text_if_exists_with_limit;
use crate::manifest::MAX_GRADLE_MANIFEST_BYTES;
use crate::manifest::parse_gradle_settings;
use cap_std::fs::Dir;
use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

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

#[cfg(test)]
mod tests;
