//! Swift package target discovery and expression helpers.

use std::path::Path;

#[cfg(test)]
use super::access::AmbientManifestAccess;
use super::access::ManifestAccess;
use super::{ManifestDiscovery, ManifestModule};

#[cfg(test)]
fn parse_package_swift(root: &Path) -> Option<ManifestDiscovery> {
    let mut access = AmbientManifestAccess { root };
    parse_package_swift_with_access(&mut access).ok().flatten()
}

pub(super) fn parse_package_swift_with_access(
    access: &mut dyn ManifestAccess,
) -> Result<Option<ManifestDiscovery>, String> {
    let Some(content) = access.read_text(Path::new("Package.swift"), "Swift package manifest")?
    else {
        return Ok(None);
    };
    let mut discovery = ManifestDiscovery::default();

    // Parse .target and .executableTarget declarations
    // Pattern: .target(name: "TargetName", ..., path: "Sources/TargetName", ...)
    // or .target(name: "TargetName", dependencies: [...])
    let target_patterns = [
        ".target(",
        ".executableTarget(",
        ".testTarget(",
        ".systemLibrary(",
    ];

    for pattern in &target_patterns {
        let is_test = *pattern == ".testTarget(";
        let mut search_from = 0;
        while let Some(start) = content[search_from..].find(pattern) {
            let abs_start = search_from + start;
            // Find the matching closing paren (handle nested parens)
            if let Some(block) = extract_balanced_parens(&content[abs_start + pattern.len()..]) {
                let name = extract_swift_string_param(&block, "name");
                let explicit_path = extract_swift_string_param(&block, "path");

                if let Some(name) = name
                    && !is_test
                {
                    let source_path = explicit_path.unwrap_or_else(|| format!("Sources/{name}"));

                    discovery.modules.insert(
                        name.clone(),
                        ManifestModule {
                            name: name.clone(),
                            source_paths: vec![source_path.clone()],
                            dependencies: extract_swift_dependencies(&block),
                        },
                    );

                    if !discovery.source_dirs.contains(&source_path) {
                        discovery.source_dirs.push(source_path);
                    }
                }

                search_from = abs_start + pattern.len() + block.len();
            } else {
                search_from = abs_start + pattern.len();
            }
        }
    }

    // Default: if no targets found, check for Sources/ directory
    if discovery.modules.is_empty()
        && access.directory_exists(Path::new("Sources"), "Swift source")?
    {
        discovery.source_dirs.push("Sources".to_string());
    }

    if discovery.modules.is_empty() && discovery.source_dirs.is_empty() {
        Ok(None)
    } else {
        Ok(Some(discovery))
    }
}

/// Extract the content within balanced parentheses.
fn extract_balanced_parens(s: &str) -> Option<String> {
    let mut depth = 1;
    let mut end = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    end = i;
                    break;
                }
            }
            _ => {}
        }
    }
    if depth == 0 {
        Some(s[..end].to_string())
    } else {
        None
    }
}

/// Extract a named string parameter from a Swift function call body.
/// e.g. `name: "Foo"` → Some("Foo")
fn extract_swift_string_param(block: &str, param: &str) -> Option<String> {
    let pattern = format!("{param}:");
    let start = block.find(&pattern)?;
    let after = &block[start + pattern.len()..];
    let quote_start = after.find('"')?;
    let rest = &after[quote_start + 1..];
    let quote_end = rest.find('"')?;
    Some(rest[..quote_end].to_string())
}

/// Extract dependency names from a Swift target block.
fn extract_swift_dependencies(block: &str) -> Vec<String> {
    let mut deps = Vec::new();
    if let Some(start) = block.find("dependencies:") {
        let after = &block[start..];
        if let Some(bracket_start) = after.find('[') {
            let rest = &after[bracket_start + 1..];
            if let Some(bracket_end) = rest.find(']') {
                let deps_str = &rest[..bracket_end];
                // Parse both string deps and .target/.product deps
                for dep in deps_str.split(',') {
                    let dep = dep.trim();
                    // .target(name: "Foo") or .product(name: "Foo", ...)
                    if let Some(name) = extract_swift_string_param(dep, "name") {
                        deps.push(name);
                    }
                    // Simple string dependency: "Foo"
                    else if dep.starts_with('"') && dep.ends_with('"') && dep.len() > 2 {
                        deps.push(dep[1..dep.len() - 1].to_string());
                    }
                }
            }
        }
    }
    deps
}

#[cfg(test)]
mod tests;
