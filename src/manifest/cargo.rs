//! Cargo package metadata and bounded workspace discovery.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[cfg(test)]
use super::access::AmbientManifestAccess;
use super::access::{
    ManifestAccess, display_manifest_path, manifest_path_text, normalize_retained_manifest_path,
};
use super::simple_manifests::{extract_section, extract_toml_value, split_toml_array_sections};
use super::{ManifestDiscovery, ManifestModule};

#[cfg(test)]
fn parse_cargo_toml(root: &Path) -> Option<ManifestDiscovery> {
    let mut access = AmbientManifestAccess { root };
    parse_cargo_toml_with_access(
        &mut access,
        Path::new(""),
        &mut HashSet::new(),
        &mut HashSet::new(),
    )
    .ok()
    .flatten()
}

pub(super) fn parse_cargo_toml_with_access(
    access: &mut dyn ManifestAccess,
    relative_root: &Path,
    active: &mut HashSet<PathBuf>,
    completed: &mut HashSet<PathBuf>,
) -> Result<Option<ManifestDiscovery>, String> {
    let relative_root = normalize_retained_manifest_path(relative_root, "Cargo workspace")?;
    if completed.contains(&relative_root) {
        return Ok(None);
    }
    if !active.insert(relative_root.clone()) {
        return Err(format!(
            "Retained Cargo workspace cycle revisits {}",
            display_manifest_path(&relative_root)
        ));
    }
    let result = parse_cargo_toml_with_access_inner(access, &relative_root, active, completed);
    active.remove(&relative_root);
    if result.is_ok() {
        completed.insert(relative_root);
    }
    result
}

fn parse_cargo_toml_with_access_inner(
    access: &mut dyn ManifestAccess,
    relative_root: &Path,
    active: &mut HashSet<PathBuf>,
    completed: &mut HashSet<PathBuf>,
) -> Result<Option<ManifestDiscovery>, String> {
    let manifest_path = relative_root.join("Cargo.toml");
    let Some(content) = access.read_text(&manifest_path, "Cargo manifest")? else {
        return Ok(None);
    };
    let document = toml::from_str::<toml::Table>(&content).map_err(|error| {
        format!(
            "Cannot parse retained Cargo manifest {} as TOML: {error}",
            display_manifest_path(&manifest_path)
        )
    })?;
    let mut discovery = ManifestDiscovery::default();

    // Extract package name
    if let Some(name) = extract_toml_value(&content, "name", Some("[package]")) {
        let src_path = manifest_path_text(&relative_root.join("src"), "Cargo source")?;
        discovery.modules.insert(
            name.clone(),
            ManifestModule {
                name,
                source_paths: vec![src_path.clone()],
                dependencies: Vec::new(),
            },
        );
        if !discovery.source_dirs.contains(&src_path) {
            discovery.source_dirs.push(src_path);
        }
    }

    // Extract [[bin]] targets
    for section in split_toml_array_sections(&content, "[[bin]]") {
        if let Some(name) = extract_toml_value(&section, "name", None) {
            let path = extract_toml_value(&section, "path", None)
                .unwrap_or_else(|| format!("src/bin/{name}.rs"));
            let local_dir = Path::new(&path)
                .parent()
                .unwrap_or_else(|| Path::new("src"));
            let dir = manifest_path_text(&relative_root.join(local_dir), "Cargo binary source")?;
            discovery.modules.insert(
                name.clone(),
                ManifestModule {
                    name,
                    source_paths: vec![dir.clone()],
                    dependencies: Vec::new(),
                },
            );
            if !discovery.source_dirs.contains(&dir) {
                discovery.source_dirs.push(dir);
            }
        }
    }

    // Check for workspace members using the same TOML semantics as security preflight.
    if let Some(members) = cargo_workspace_members(&document, &manifest_path, access)? {
        let mut expanded_members = HashSet::new();
        for member in members {
            // Workspace members are subdirectories with their own Cargo.toml
            let member_root = normalize_retained_manifest_path(
                &relative_root.join(&member),
                "Cargo workspace member",
            )?;
            if !expanded_members.insert(member_root.clone()) {
                continue;
            }
            if let Some(sub) =
                parse_cargo_toml_with_access(access, &member_root, active, completed)?
            {
                for (_, module) in sub.modules {
                    discovery
                        .modules
                        .insert(module.name.clone(), module.clone());
                }
                let member = manifest_path_text(&member_root, "Cargo workspace member")?;
                if !discovery.source_dirs.contains(&member) {
                    discovery.source_dirs.push(member);
                }
            }
        }
    }

    // Extract [dependencies] as dependency names
    if let Some(deps_section) = extract_section(&content, "[dependencies]") {
        let dep_names: Vec<String> = deps_section
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
                    return None;
                }
                line.split('=').next().map(|k| k.trim().to_string())
            })
            .filter(|k| !k.is_empty())
            .collect();

        // Assign deps to the main package
        if let Some(pkg_name) = extract_toml_value(&content, "name", Some("[package]"))
            && let Some(module) = discovery.modules.get_mut(&pkg_name)
        {
            module.dependencies = dep_names;
        }
    }

    if discovery.modules.is_empty() {
        Ok(None)
    } else {
        Ok(Some(discovery))
    }
}

fn cargo_workspace_members(
    document: &toml::Table,
    manifest_path: &Path,
    access: &mut dyn ManifestAccess,
) -> Result<Option<Vec<String>>, String> {
    let Some(workspace) = document.get("workspace") else {
        return Ok(None);
    };
    let workspace = workspace.as_table().ok_or_else(|| {
        format!(
            "Retained Cargo manifest {} must define `workspace` as a TOML table",
            display_manifest_path(manifest_path)
        )
    })?;
    let Some(members) = workspace.get("members") else {
        return Ok(Some(Vec::new()));
    };
    let members = members.as_array().ok_or_else(|| {
        format!(
            "Retained Cargo manifest {} must define `workspace.members` as a string array",
            display_manifest_path(manifest_path)
        )
    })?;
    access.charge_entries(members.len(), "Cargo workspace declaration")?;
    members
        .iter()
        .map(|member| {
            member.as_str().map(str::to_string).ok_or_else(|| {
                format!(
                    "Retained Cargo manifest {} has a non-string workspace member",
                    display_manifest_path(manifest_path)
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

#[cfg(test)]
mod tests;
