//! Node package metadata and bounded workspace expansion.

use std::collections::HashSet;
use std::path::Path;

#[cfg(test)]
use super::access::AmbientManifestAccess;
use super::access::{
    ManifestAccess, display_manifest_path, manifest_path_text, normalize_retained_manifest_path,
};
use super::{ManifestDiscovery, ManifestModule};

#[cfg(test)]
fn parse_package_json(root: &Path) -> Option<ManifestDiscovery> {
    let mut access = AmbientManifestAccess { root };
    parse_package_json_with_access(&mut access).ok().flatten()
}

pub(super) fn parse_package_json_with_access(
    access: &mut dyn ManifestAccess,
) -> Result<Option<ManifestDiscovery>, String> {
    parse_package_json_with_access_and_hooks(access, |_| {}, |_, _| {})
}

#[cfg(test)]
pub(super) fn parse_package_json_with_access_and_hook<AfterEnumeration>(
    access: &mut dyn ManifestAccess,
    after_enumeration: AfterEnumeration,
) -> Result<Option<ManifestDiscovery>, String>
where
    AfterEnumeration: FnMut(&Path),
{
    parse_package_json_with_access_and_hooks(access, after_enumeration, |_, _| {})
}

pub(super) fn parse_package_json_with_access_and_hooks<AfterEnumeration, AfterChildRead>(
    access: &mut dyn ManifestAccess,
    mut after_enumeration: AfterEnumeration,
    mut after_child_read: AfterChildRead,
) -> Result<Option<ManifestDiscovery>, String>
where
    AfterEnumeration: FnMut(&Path),
    AfterChildRead: FnMut(&Path, &str),
{
    let Some(content) = access.read_text(Path::new("package.json"), "Node package manifest")?
    else {
        return Ok(None);
    };
    let json = parse_node_package_document(&content, Path::new("package.json"))?;
    let mut discovery = ManifestDiscovery::default();

    let name = json.get("name").and_then(|v| v.as_str()).unwrap_or("app");

    // Check for workspaces (monorepo)
    if let Some(workspace_patterns) =
        node_workspace_patterns(&json, Path::new("package.json"), access)?
    {
        let mut expanded_bases = HashSet::new();
        let mut seen_workspaces = HashSet::new();
        for pattern in workspace_patterns {
            // Simple glob: "packages/*" → look for subdirs
            let base = pattern.trim_end_matches("/*").trim_end_matches("/**");
            let base_dir =
                normalize_retained_manifest_path(Path::new(base), "Node workspace base")?;
            if !expanded_bases.insert(base_dir.clone()) {
                continue;
            }
            let workspace_names = access.child_directories(&base_dir, "Node workspace")?;
            after_enumeration(&base_dir);
            for ws_name in workspace_names {
                let workspace = base_dir.join(&ws_name);
                if !seen_workspaces.insert(workspace.clone()) {
                    continue;
                }
                let workspace_content = access.read_enumerated_child_text(
                    &base_dir,
                    &ws_name,
                    Path::new("package.json"),
                    "Node workspace package manifest",
                )?;
                after_child_read(&base_dir, &ws_name);
                access.verify_enumerated_child(&base_dir, &ws_name, "Node workspace")?;
                if let Some(workspace_content) = workspace_content {
                    let workspace_package = parse_node_package_document(
                        &workspace_content,
                        &workspace.join("package.json"),
                    )?;
                    node_workspace_patterns(
                        &workspace_package,
                        &workspace.join("package.json"),
                        access,
                    )?;
                    let src_dir = if access.enumerated_child_directory_exists(
                        &base_dir,
                        &ws_name,
                        Path::new("src"),
                        "Node workspace source",
                    )? {
                        manifest_path_text(&workspace.join("src"), "Node workspace source")?
                    } else {
                        manifest_path_text(&workspace, "Node workspace source")?
                    };
                    discovery.modules.insert(
                        ws_name.clone(),
                        ManifestModule {
                            name: ws_name.clone(),
                            source_paths: vec![src_dir.clone()],
                            dependencies: Vec::new(),
                        },
                    );
                    if !discovery.source_dirs.contains(&src_dir) {
                        discovery.source_dirs.push(src_dir);
                    }
                }
                access.verify_enumerated_child(&base_dir, &ws_name, "Node workspace")?;
            }
            access.verify_child_directories(&base_dir, "Node workspace")?;
            access.release_child_directories(&base_dir, "Node workspace")?;
        }
    }

    // Detect main source directory
    let main_field = json.get("main").and_then(|v| v.as_str()).unwrap_or("");
    let src_dir = if access.directory_exists(Path::new("src"), "Node source")? {
        "src"
    } else if access.directory_exists(Path::new("lib"), "Node source")? {
        "lib"
    } else if main_field.starts_with("./") {
        Path::new(main_field)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("src")
    } else {
        "src"
    };

    if discovery.modules.is_empty() {
        discovery.modules.insert(
            name.to_string(),
            ManifestModule {
                name: name.to_string(),
                source_paths: vec![src_dir.to_string()],
                dependencies: Vec::new(),
            },
        );
    }

    if !discovery.source_dirs.contains(&src_dir.to_string()) {
        discovery.source_dirs.push(src_dir.to_string());
    }

    Ok(Some(discovery))
}

fn parse_node_package_document(
    content: &str,
    manifest_path: &Path,
) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let value = serde_json::from_str::<serde_json::Value>(content).map_err(|error| {
        format!(
            "Cannot parse retained Node package manifest {} as JSON: {error}",
            display_manifest_path(manifest_path)
        )
    })?;
    value.as_object().cloned().ok_or_else(|| {
        format!(
            "Retained Node package manifest {} must contain a JSON object",
            display_manifest_path(manifest_path)
        )
    })
}

fn node_workspace_patterns(
    package: &serde_json::Map<String, serde_json::Value>,
    manifest_path: &Path,
    access: &mut dyn ManifestAccess,
) -> Result<Option<Vec<String>>, String> {
    let Some(workspaces) = package.get("workspaces") else {
        return Ok(None);
    };
    let entries = match workspaces {
        serde_json::Value::Array(entries) => entries,
        serde_json::Value::Object(object) => object
            .get("packages")
            .ok_or_else(|| {
                format!(
                    "Retained Node package manifest {} must define `workspaces.packages` as a string array",
                    display_manifest_path(manifest_path)
                )
            })?
            .as_array()
            .ok_or_else(|| {
                format!(
                    "Retained Node package manifest {} must define `workspaces.packages` as a string array",
                    display_manifest_path(manifest_path)
                )
            })?,
        _ => {
            return Err(format!(
                "Retained Node package manifest {} must define `workspaces` as a string array or an object with a `packages` string array",
                display_manifest_path(manifest_path)
            ));
        }
    };
    access.charge_entries(entries.len(), "Node workspace declaration")?;
    entries
        .iter()
        .map(|entry| {
            entry.as_str().map(str::to_string).ok_or_else(|| {
                format!(
                    "Retained Node package manifest {} has a non-string workspace entry",
                    display_manifest_path(manifest_path)
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

#[cfg(test)]
mod tests;
