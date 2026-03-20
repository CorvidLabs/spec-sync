use crate::types::RegistryEntry;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

const REGISTRY_FILENAME: &str = "specsync-registry.toml";

/// Load a registry from a `specsync-registry.toml` file.
pub fn load_registry(root: &Path) -> Option<RegistryEntry> {
    let path = root.join(REGISTRY_FILENAME);
    let content = fs::read_to_string(&path).ok()?;
    parse_registry(&content)
}

/// Parse registry TOML content.
fn parse_registry(content: &str) -> Option<RegistryEntry> {
    let mut name = String::new();
    let mut specs = Vec::new();
    let mut in_specs = false;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line == "[registry]" {
            in_specs = false;
            continue;
        }
        if line == "[specs]" {
            in_specs = true;
            continue;
        }

        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim();
            let value = value.trim_matches('"');

            if in_specs {
                specs.push((key.to_string(), value.to_string()));
            } else if key == "name" {
                name = value.to_string();
            }
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(RegistryEntry { name, specs })
}

/// Generate a registry file by scanning for spec files.
pub fn generate_registry(root: &Path, project_name: &str, specs_dir: &str) -> String {
    let specs_path = root.join(specs_dir);
    let mut specs = Vec::new();

    if specs_path.exists() {
        for entry in WalkDir::new(&specs_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file()
                && path
                    .to_str()
                    .map(|s| s.ends_with(".spec.md"))
                    .unwrap_or(false)
            {
                // Skip template files
                if let Some(name) = path.file_name().and_then(|n| n.to_str())
                    && name.starts_with('_')
                {
                    continue;
                }

                // Extract module name from frontmatter
                if let Ok(content) = fs::read_to_string(path)
                    && let Some(module) = extract_module_name(&content)
                {
                    let rel_path = path
                        .strip_prefix(root)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .replace('\\', "/");
                    specs.push((module, rel_path));
                }
            }
        }
    }

    specs.sort_by(|a, b| a.0.cmp(&b.0));

    let mut output = String::new();
    output.push_str("[registry]\n");
    output.push_str(&format!("name = \"{project_name}\"\n"));
    output.push_str("\n[specs]\n");
    for (module, path) in &specs {
        output.push_str(&format!("{module} = \"{path}\"\n"));
    }

    output
}

/// Extract module name from spec frontmatter.
fn extract_module_name(content: &str) -> Option<String> {
    for line in content.lines() {
        if line == "---" {
            continue;
        }
        if let Some(rest) = line.strip_prefix("module:") {
            let name = rest.trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
        // Stop at end of frontmatter
        if line.starts_with("---") && content.starts_with("---") {
            break;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_registry() {
        let content = r#"
[registry]
name = "algochat"

[specs]
auth = "specs/auth/auth.spec.md"
messaging = "specs/messaging/messaging.spec.md"
"#;
        let entry = parse_registry(content).unwrap();
        assert_eq!(entry.name, "algochat");
        assert_eq!(entry.specs.len(), 2);
        assert_eq!(entry.specs[0].0, "auth");
        assert_eq!(entry.specs[0].1, "specs/auth/auth.spec.md");
    }

    #[test]
    fn test_parse_registry_empty() {
        assert!(parse_registry("").is_none());
        assert!(parse_registry("[registry]").is_none());
    }

    #[test]
    fn test_extract_module_name() {
        let content = "---\nmodule: auth\nversion: 1\n---\n# Auth\n";
        assert_eq!(extract_module_name(content), Some("auth".to_string()));
    }
}
