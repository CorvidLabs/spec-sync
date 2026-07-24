use crate::types::RegistryEntry;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use walkdir::WalkDir;

const REGISTRY_FILENAME: &str = "specsync-registry.toml";
const V4_REGISTRY_RELATIVE: &str = ".specsync/registry.toml";

/// Resolve the local registry path for a project.
/// Prefers the v4 location (`.specsync/registry.toml`) when it exists or when
/// the project uses the v4 layout; falls back to the legacy root-level
/// `specsync-registry.toml` only for un-migrated 3.x projects.
pub fn local_registry_path(root: &Path) -> PathBuf {
    let v4_path = root.join(V4_REGISTRY_RELATIVE);
    if v4_path.exists() {
        return v4_path;
    }
    let legacy_path = root.join(REGISTRY_FILENAME);
    if legacy_path.exists() {
        return legacy_path;
    }
    // Neither exists yet: default to the v4 location unless the project is
    // still on the legacy 3.x layout (root-level config, no version stamp).
    if crate::config::is_legacy_layout(root) {
        legacy_path
    } else {
        v4_path
    }
}

/// A parsed remote registry (fetched over HTTPS).
#[derive(Debug, Clone)]
pub struct RemoteRegistry {
    #[allow(dead_code)]
    pub name: String,
    pub specs: Vec<(String, String)>,
}

impl RemoteRegistry {
    /// Check whether a module name exists in this registry.
    pub fn has_spec(&self, module: &str) -> bool {
        self.specs.iter().any(|(m, _)| m == module)
    }

    /// Get the spec file path for a module.
    pub fn spec_path(&self, module: &str) -> Option<&str> {
        self.specs
            .iter()
            .find(|(m, _)| m == module)
            .map(|(_, p)| p.as_str())
    }
}

/// Fetched remote spec content with parsed metadata.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RemoteSpec {
    pub module: String,
    pub status: Option<String>,
    pub depends_on: Vec<String>,
    pub exports: Vec<String>,
    pub body: String,
}

/// Fetch a spec file's raw content from a GitHub repo.
///
/// `repo` is `owner/repo`, `spec_path` is the relative path from the registry.
pub fn fetch_remote_spec(repo: &str, spec_path: &str) -> Result<String, String> {
    let url = format!("https://raw.githubusercontent.com/{repo}/HEAD/{spec_path}");

    let agent = ureq::Agent::new_with_config(
        ureq::config::Config::builder()
            .timeout_global(Some(Duration::from_secs(10)))
            .build(),
    );

    let mut response = agent
        .get(&url)
        .call()
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    if response.status() != 200 {
        return Err(format!(
            "HTTP {} — could not fetch {spec_path} from {repo}",
            response.status()
        ));
    }

    response
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("Failed to read response body: {e}"))
}

/// Parse a fetched spec into its relevant metadata for verification.
pub fn parse_remote_spec(module: &str, content: &str) -> Option<RemoteSpec> {
    use crate::parser;

    let parsed = parser::parse_frontmatter(content)?;
    let exports = parser::get_spec_symbols(&parsed.body);

    Some(RemoteSpec {
        module: parsed
            .frontmatter
            .module
            .unwrap_or_else(|| module.to_string()),
        status: parsed.frontmatter.status,
        depends_on: parsed.frontmatter.depends_on,
        exports,
        body: parsed.body,
    })
}

/// Fetch `specsync-registry.toml` from a GitHub repo's default branch.
///
/// `repo` is in `owner/repo` format (e.g. `corvid-labs/algochat`).
/// Tries the GitHub raw content URL for the file at repo root.
pub fn fetch_remote_registry(repo: &str) -> Result<RemoteRegistry, String> {
    let url = format!("https://raw.githubusercontent.com/{repo}/HEAD/{REGISTRY_FILENAME}");

    let agent = ureq::Agent::new_with_config(
        ureq::config::Config::builder()
            .timeout_global(Some(Duration::from_secs(10)))
            .build(),
    );

    let mut response = agent
        .get(&url)
        .call()
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    if response.status() != 200 {
        return Err(format!(
            "HTTP {} — {repo} may not have a {REGISTRY_FILENAME}",
            response.status()
        ));
    }

    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("Failed to read response body: {e}"))?;

    let entry =
        parse_registry(&body).ok_or_else(|| format!("Failed to parse registry from {repo}"))?;

    Ok(RemoteRegistry {
        name: entry.name,
        specs: entry.specs,
    })
}

/// Return true when content has no registry `name` and no `[specs]` mappings.
///
/// Matches inert 5.0.1-era placeholders such as `version = 1` plus an empty
/// `[modules]` table that never carried module authority under 5.1.x parsing.
pub fn is_inert_legacy_registry_stub(content: &str) -> bool {
    match parse_registry_toml(content) {
        Ok((name, specs)) => name.is_empty() && specs.is_empty(),
        // Malformed TOML is not inert — it must fail closed in
        // load_local_registry, not silently vanish.
        Err(_) => false,
    }
}

/// Load the local registry with explicit missing/inert/invalid discrimination.
///
/// - `Ok(None)` when the file is missing or an inert legacy stub
/// - `Ok(Some(entry))` when parse succeeds
/// - `Err(...)` when the file exists, is not inert, and cannot parse
pub fn load_local_registry(root: &Path) -> Result<Option<RegistryEntry>, String> {
    let path = local_registry_path(root);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read local registry {}: {error}", path.display()))?;
    if is_inert_legacy_registry_stub(&content) {
        return Ok(None);
    }
    parse_registry(&content)
        .map(Some)
        .ok_or_else(|| format!("failed to parse local registry {}", path.display()))
}

/// Load a registry from the local registry file
/// (`.specsync/registry.toml`, falling back to legacy `specsync-registry.toml`).
///
/// Best-effort: returns `None` for missing, inert, unreadable, or unparsable content.
#[allow(dead_code)]
pub fn load_registry(root: &Path) -> Option<RegistryEntry> {
    load_local_registry(root).ok().flatten()
}

/// Parse registry TOML into `(name, spec mappings)` with a real TOML parser.
///
/// Two mapping shapes are supported:
/// - the generated `[specs]` table (`module = "path"`), emitted by
///   `init-registry` / `generate_registry`
/// - the documented `[[modules]]` array-of-tables (`name` + `spec` keys) from
///   `cross-project-refs.md`
///
/// The registry `name` is read from `[registry] name` first, then a top-level
/// `name`. Returns `Err` on malformed TOML or malformed `[[modules]]` entries
/// so callers fail closed instead of silently dropping mappings.
fn parse_registry_toml(content: &str) -> Result<(String, Vec<(String, String)>), String> {
    let value: toml::Value =
        toml::from_str(content).map_err(|e| format!("invalid TOML: {e}"))?;

    let mut name = String::new();
    if let Some(n) = value.get("name").and_then(|v| v.as_str()) {
        name = n.to_string();
    }
    if let Some(n) = value
        .get("registry")
        .and_then(|r| r.get("name"))
        .and_then(|v| v.as_str())
    {
        name = n.to_string();
    }

    let mut specs: Vec<(String, String)> = Vec::new();

    // Generated shape: [specs] table of module = "path".
    if let Some(table) = value.get("specs").and_then(|v| v.as_table()) {
        for (module, path) in table {
            if let Some(path) = path.as_str() {
                specs.push((module.clone(), path.to_string()));
            }
        }
    }

    // Documented shape: [[modules]] array of { name, spec }.
    if let Some(modules) = value.get("modules").and_then(|v| v.as_array()) {
        for module in modules {
            let module_name = module.get("name").and_then(|v| v.as_str());
            let spec_path = module.get("spec").and_then(|v| v.as_str());
            match (module_name, spec_path) {
                (Some(m), Some(p)) => specs.push((m.to_string(), p.to_string())),
                _ => {
                    return Err(
                        "every [[modules]] entry needs both `name` and `spec` keys".to_string()
                    );
                }
            }
        }
    }

    Ok((name, specs))
}

/// Parse registry TOML content.
///
/// Returns `None` when the content is malformed TOML or carries no registry
/// `name` — callers treat that as "failed to parse" and fail closed.
fn parse_registry(content: &str) -> Option<RegistryEntry> {
    let (name, specs) = parse_registry_toml(content).ok()?;
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
    output.push_str(&format!("name = \"{}\"\n", toml_escape(project_name)));
    output.push_str("\n[specs]\n");
    for (module, path) in &specs {
        output.push_str(&format!("{module} = \"{}\"\n", toml_escape(path)));
    }

    output
}

/// Escape a string for embedding inside a TOML basic string (`"..."`).
/// Without this, a project name containing `"`, `\`, or control characters
/// (e.g. newlines) produces a registry that fails TOML parsing — or worse,
/// injects extra keys into it.
fn toml_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Add a module entry to an existing local registry file
/// (`.specsync/registry.toml`, falling back to legacy `specsync-registry.toml`).
/// If the module already exists, it is not duplicated.
/// Returns `true` if the entry was added, `false` if it already existed or the file is missing.
pub fn register_module(root: &Path, module_name: &str, spec_rel_path: &str) -> bool {
    let path = local_registry_path(root);
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    // Check if module already registered
    if let Some(entry) = parse_registry(&content)
        && entry.specs.iter().any(|(m, _)| m == module_name)
    {
        return false;
    }

    // Append to the [specs] section
    let new_line = format!("{module_name} = \"{spec_rel_path}\"\n");

    // If there's a [specs] section, append after it
    if content.contains("[specs]") {
        let updated = format!("{}\n{new_line}", content.trim_end());
        if fs::write(&path, updated).is_ok() {
            return true;
        }
    }

    false
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
    fn inert_legacy_registry_stub_is_detected() {
        let stub = "version = 1\n\n[modules]\n";
        assert!(is_inert_legacy_registry_stub(stub));
        assert!(is_inert_legacy_registry_stub(""));
        assert!(is_inert_legacy_registry_stub("[registry]\n"));
        assert!(is_inert_legacy_registry_stub("[specs]\n"));
    }

    #[test]
    fn named_or_mapped_registries_are_not_inert() {
        assert!(!is_inert_legacy_registry_stub(
            "[registry]\nname = \"fixture\"\n\n[specs]\n"
        ));
        assert!(!is_inert_legacy_registry_stub(
            "[specs]\nauth = \"specs/auth/auth.spec.md\"\n"
        ));
    }

    #[test]
    fn load_local_registry_treats_inert_stub_as_absent() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::write(
            root.join(".specsync/registry.toml"),
            "version = 1\n\n[modules]\n",
        )
        .unwrap();

        assert!(load_local_registry(root).unwrap().is_none());
        assert!(load_registry(root).is_none());
    }

    #[test]
    fn load_local_registry_loads_named_registry() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::write(
            root.join(".specsync/registry.toml"),
            "[registry]\nname = \"fixture\"\n\n[specs]\nauth = \"specs/auth/auth.spec.md\"\n",
        )
        .unwrap();

        let entry = load_local_registry(root).unwrap().unwrap();
        assert_eq!(entry.name, "fixture");
        assert_eq!(entry.specs.len(), 1);
        assert_eq!(entry.specs[0].0, "auth");
    }

    #[test]
    fn load_local_registry_fails_closed_on_non_inert_unparsable() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::write(
            root.join(".specsync/registry.toml"),
            "[specs]\nauth = \"specs/auth/auth.spec.md\"\n",
        )
        .unwrap();

        let error = load_local_registry(root).unwrap_err();
        assert!(error.contains("failed to parse local registry"));
        assert!(load_registry(root).is_none());
    }

    #[test]
    fn test_extract_module_name() {
        let content = "---\nmodule: auth\nversion: 1\n---\n# Auth\n";
        assert_eq!(extract_module_name(content), Some("auth".to_string()));
    }

    #[test]
    fn test_remote_registry_has_spec() {
        let reg = RemoteRegistry {
            name: "test".to_string(),
            specs: vec![
                ("auth".to_string(), "specs/auth/auth.spec.md".to_string()),
                (
                    "messaging".to_string(),
                    "specs/messaging/messaging.spec.md".to_string(),
                ),
            ],
        };
        assert!(reg.has_spec("auth"));
        assert!(reg.has_spec("messaging"));
        assert!(!reg.has_spec("nonexistent"));
    }

    #[test]
    fn generate_registry_escapes_toml_injection() {
        // #440: a --name with quotes/newlines must produce valid TOML.
        let tmp = tempfile::TempDir::new().unwrap();
        let content = generate_registry(tmp.path(), "evil\"\n[specs]\npwned=\"x", "specs");
        let parsed: Result<toml::Value, toml::de::Error> = toml::from_str(&content);
        assert!(parsed.is_ok(), "generated registry must be valid TOML: {content}");
        // No injected [specs] key survived.
        let value = parsed.unwrap();
        assert!(value.get("specs").is_none() || value["specs"].get("pwned").is_none());
    }

    #[test]
    fn parse_registry_supports_documented_modules_shape() {
        // #413 facet 1: [[modules]] array-of-tables must not drop mappings.
        let content = "[registry]\nname = \"probe\"\n\n[[modules]]\nname = \"lib\"\nspec = \"custom/lib.spec.md\"\n";
        let entry = parse_registry(content).expect("documented shape parses");
        assert_eq!(entry.name, "probe");
        assert_eq!(
            entry.specs,
            vec![("lib".to_string(), "custom/lib.spec.md".to_string())]
        );
    }

    #[test]
    fn load_local_registry_fails_closed_on_malformed_toml_with_name_line() {
        // #413 facet 2: `name = {{{` is not valid TOML and must not parse.
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::write(root.join(".specsync/registry.toml"), "version = 1\nname = {{{\n").unwrap();

        assert!(!is_inert_legacy_registry_stub(
            "version = 1\nname = {{{\n"
        ));
        let error = load_local_registry(root).unwrap_err();
        assert!(error.contains("failed to parse local registry"), "{error}");
    }

    #[test]
    fn modules_entry_missing_spec_key_is_an_error() {
        let content = "[registry]\nname = \"probe\"\n\n[[modules]]\nname = \"lib\"\n";
        assert!(parse_registry(content).is_none());
        assert!(!is_inert_legacy_registry_stub(content));
    }
}
