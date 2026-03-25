//! Manifest-aware module detection.
//!
//! Parses language-specific manifest files (Package.swift, Cargo.toml,
//! build.gradle.kts, package.json, etc.) to discover targets, source paths,
//! and module names instead of relying on directory scanning alone.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A module discovered from a manifest file.
#[derive(Debug, Clone)]
pub struct ManifestModule {
    /// Module/target name.
    pub name: String,
    /// Source paths relative to project root.
    pub source_paths: Vec<String>,
    /// Dependencies (other module names).
    pub dependencies: Vec<String>,
}

/// Result of parsing all manifest files in a project.
#[derive(Debug, Default)]
pub struct ManifestDiscovery {
    /// Modules discovered from manifest files, keyed by name.
    pub modules: HashMap<String, ManifestModule>,
    /// Source directories discovered from manifests.
    pub source_dirs: Vec<String>,
}

/// Discover modules from all supported manifest files in the project root.
pub fn discover_from_manifests(root: &Path) -> ManifestDiscovery {
    let mut discovery = ManifestDiscovery::default();

    // Try each manifest type in order
    if let Some(d) = parse_cargo_toml(root) {
        merge_discovery(&mut discovery, d);
    }
    if let Some(d) = parse_package_swift(root) {
        merge_discovery(&mut discovery, d);
    }
    if let Some(d) = parse_gradle(root) {
        merge_discovery(&mut discovery, d);
    }
    if let Some(d) = parse_package_json(root) {
        merge_discovery(&mut discovery, d);
    }
    if let Some(d) = parse_pubspec_yaml(root) {
        merge_discovery(&mut discovery, d);
    }
    if let Some(d) = parse_go_mod(root) {
        merge_discovery(&mut discovery, d);
    }
    if let Some(d) = parse_pyproject_toml(root) {
        merge_discovery(&mut discovery, d);
    }

    discovery
}

fn merge_discovery(target: &mut ManifestDiscovery, source: ManifestDiscovery) {
    for (name, module) in source.modules {
        target.modules.entry(name).or_insert(module);
    }
    for dir in source.source_dirs {
        if !target.source_dirs.contains(&dir) {
            target.source_dirs.push(dir);
        }
    }
}

// ─── Cargo.toml (Rust) ──────────────────────────────────────────────────

fn parse_cargo_toml(root: &Path) -> Option<ManifestDiscovery> {
    let path = root.join("Cargo.toml");
    let content = fs::read_to_string(&path).ok()?;
    let mut discovery = ManifestDiscovery::default();

    // Extract package name
    if let Some(name) = extract_toml_value(&content, "name", Some("[package]")) {
        let src_path = "src";
        discovery.modules.insert(
            name.clone(),
            ManifestModule {
                name,
                source_paths: vec![src_path.to_string()],
                dependencies: Vec::new(),
            },
        );
        if !discovery.source_dirs.contains(&src_path.to_string()) {
            discovery.source_dirs.push(src_path.to_string());
        }
    }

    // Extract [[bin]] targets
    for section in split_toml_array_sections(&content, "[[bin]]") {
        if let Some(name) = extract_toml_value(&section, "name", None) {
            let path = extract_toml_value(&section, "path", None)
                .unwrap_or_else(|| format!("src/bin/{name}.rs"));
            let dir = Path::new(&path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "src".to_string());
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

    // Check for workspace members
    if let Some(members_str) = extract_toml_array(&content, "members", Some("[workspace]")) {
        for member in members_str {
            // Workspace members are subdirectories with their own Cargo.toml
            let member_root = root.join(&member);
            if member_root.join("Cargo.toml").exists() {
                if let Some(sub) = parse_cargo_toml(&member_root) {
                    for (_, mut module) in sub.modules {
                        // Prefix paths with workspace member dir
                        module.source_paths = module
                            .source_paths
                            .iter()
                            .map(|p| format!("{member}/{p}"))
                            .collect();
                        discovery
                            .modules
                            .insert(module.name.clone(), module.clone());
                    }
                }
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
        None
    } else {
        Some(discovery)
    }
}

// ─── Package.swift (Swift) ───────────────────────────────────────────────

fn parse_package_swift(root: &Path) -> Option<ManifestDiscovery> {
    let path = root.join("Package.swift");
    let content = fs::read_to_string(&path).ok()?;
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
    if discovery.modules.is_empty() && root.join("Sources").exists() {
        discovery.source_dirs.push("Sources".to_string());
    }

    if discovery.modules.is_empty() && discovery.source_dirs.is_empty() {
        None
    } else {
        Some(discovery)
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

// ─── build.gradle.kts / build.gradle (Kotlin/Java) ──────────────────────

fn parse_gradle(root: &Path) -> Option<ManifestDiscovery> {
    // Try Kotlin DSL first, then Groovy
    let path = if root.join("build.gradle.kts").exists() {
        root.join("build.gradle.kts")
    } else if root.join("build.gradle").exists() {
        root.join("build.gradle")
    } else {
        return None;
    };

    let content = fs::read_to_string(&path).ok()?;
    let mut discovery = ManifestDiscovery::default();

    // Detect Android project vs plain Kotlin/Java
    let is_android = content.contains("android {") || content.contains("android{");

    if is_android {
        // Android: source in app/src/main/java or app/src/main/kotlin
        for dir in &[
            "app/src/main/java",
            "app/src/main/kotlin",
            "src/main/java",
            "src/main/kotlin",
        ] {
            if root.join(dir).exists() {
                discovery.source_dirs.push(dir.to_string());
            }
        }
    } else {
        // Standard Gradle: src/main/kotlin or src/main/java
        for dir in &["src/main/kotlin", "src/main/java", "src/main/scala"] {
            if root.join(dir).exists() {
                discovery.source_dirs.push(dir.to_string());
            }
        }
    }

    // Extract project name from settings.gradle.kts or settings.gradle
    let settings_path = if root.join("settings.gradle.kts").exists() {
        Some(root.join("settings.gradle.kts"))
    } else if root.join("settings.gradle").exists() {
        Some(root.join("settings.gradle"))
    } else {
        None
    };

    if let Some(settings_path) = settings_path
        && let Ok(settings) = fs::read_to_string(&settings_path)
    {
        // Parse include(":module1", ":module2") or include ":module1", ":module2"
        for line in settings.lines() {
            let line = line.trim();
            if line.starts_with("include") {
                // Extract quoted module names
                let mut search = line;
                while let Some(quote_start) = search.find('"') {
                    let rest = &search[quote_start + 1..];
                    if let Some(quote_end) = rest.find('"') {
                        let module = rest[..quote_end].trim_start_matches(':');
                        if !module.is_empty() {
                            let module_src = format!("{module}/src/main");
                            let source_path =
                                if root.join(format!("{module}/src/main/kotlin")).exists() {
                                    format!("{module}/src/main/kotlin")
                                } else if root.join(format!("{module}/src/main/java")).exists() {
                                    format!("{module}/src/main/java")
                                } else {
                                    module_src
                                };

                            discovery.modules.insert(
                                module.to_string(),
                                ManifestModule {
                                    name: module.to_string(),
                                    source_paths: vec![source_path.clone()],
                                    dependencies: Vec::new(),
                                },
                            );
                            if !discovery.source_dirs.contains(&source_path) {
                                discovery.source_dirs.push(source_path);
                            }
                        }
                        search = &rest[quote_end + 1..];
                    } else {
                        break;
                    }
                }
            }
        }
    }

    if discovery.modules.is_empty() && discovery.source_dirs.is_empty() {
        None
    } else {
        Some(discovery)
    }
}

// ─── package.json (TypeScript/JavaScript) ────────────────────────────────

fn parse_package_json(root: &Path) -> Option<ManifestDiscovery> {
    let path = root.join("package.json");
    let content = fs::read_to_string(&path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let mut discovery = ManifestDiscovery::default();

    let name = json.get("name").and_then(|v| v.as_str()).unwrap_or("app");

    // Check for workspaces (monorepo)
    if let Some(workspaces) = json.get("workspaces") {
        let workspace_patterns: Vec<&str> = match workspaces {
            serde_json::Value::Array(arr) => arr.iter().filter_map(|v| v.as_str()).collect(),
            serde_json::Value::Object(obj) => {
                if let Some(serde_json::Value::Array(arr)) = obj.get("packages") {
                    arr.iter().filter_map(|v| v.as_str()).collect()
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        };

        for pattern in workspace_patterns {
            // Simple glob: "packages/*" → look for subdirs
            let base = pattern.trim_end_matches("/*").trim_end_matches("/**");
            let base_dir = root.join(base);
            if base_dir.exists()
                && base_dir.is_dir()
                && let Ok(entries) = fs::read_dir(&base_dir)
            {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        let pkg_json = entry.path().join("package.json");
                        if pkg_json.exists() {
                            let ws_name = entry.file_name().to_string_lossy().to_string();
                            let src_dir = if entry.path().join("src").exists() {
                                format!("{base}/{ws_name}/src")
                            } else {
                                format!("{base}/{ws_name}")
                            };
                            discovery.modules.insert(
                                ws_name.clone(),
                                ManifestModule {
                                    name: ws_name,
                                    source_paths: vec![src_dir.clone()],
                                    dependencies: Vec::new(),
                                },
                            );
                            if !discovery.source_dirs.contains(&src_dir) {
                                discovery.source_dirs.push(src_dir);
                            }
                        }
                    }
                }
            }
        }
    }

    // Detect main source directory
    let main_field = json.get("main").and_then(|v| v.as_str()).unwrap_or("");
    let src_dir = if root.join("src").exists() {
        "src"
    } else if root.join("lib").exists() {
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

    Some(discovery)
}

// ─── pubspec.yaml (Dart/Flutter) ─────────────────────────────────────────

fn parse_pubspec_yaml(root: &Path) -> Option<ManifestDiscovery> {
    let path = root.join("pubspec.yaml");
    let content = fs::read_to_string(&path).ok()?;
    let mut discovery = ManifestDiscovery::default();

    // Extract name from `name: my_package`
    let name = content
        .lines()
        .find(|l| l.starts_with("name:"))
        .and_then(|l| l.strip_prefix("name:"))
        .map(|n| n.trim().to_string())
        .unwrap_or_else(|| "app".to_string());

    let src_dir = "lib";

    discovery.modules.insert(
        name.clone(),
        ManifestModule {
            name,
            source_paths: vec![src_dir.to_string()],
            dependencies: Vec::new(),
        },
    );
    discovery.source_dirs.push(src_dir.to_string());

    Some(discovery)
}

// ─── go.mod (Go) ─────────────────────────────────────────────────────────

fn parse_go_mod(root: &Path) -> Option<ManifestDiscovery> {
    let path = root.join("go.mod");
    let content = fs::read_to_string(&path).ok()?;
    let mut discovery = ManifestDiscovery::default();

    // Extract module name: `module github.com/user/repo`
    let module_name = content
        .lines()
        .find(|l| l.starts_with("module "))
        .and_then(|l| l.strip_prefix("module "))
        .map(|m| {
            // Use last segment as module name
            m.trim().rsplit('/').next().unwrap_or(m.trim()).to_string()
        })
        .unwrap_or_else(|| "app".to_string());

    // Go projects: scan for directories with .go files as packages
    // Common patterns: cmd/, internal/, pkg/
    let mut source_dirs = Vec::new();
    for dir_name in &["cmd", "internal", "pkg", "api"] {
        if root.join(dir_name).exists() {
            source_dirs.push(dir_name.to_string());
        }
    }

    // If none of the standard dirs exist, use "." (root)
    if source_dirs.is_empty() {
        source_dirs.push(".".to_string());
    }

    discovery.modules.insert(
        module_name.clone(),
        ManifestModule {
            name: module_name,
            source_paths: source_dirs.clone(),
            dependencies: Vec::new(),
        },
    );
    discovery.source_dirs = source_dirs;

    Some(discovery)
}

// ─── pyproject.toml (Python) ─────────────────────────────────────────────

fn parse_pyproject_toml(root: &Path) -> Option<ManifestDiscovery> {
    let path = root.join("pyproject.toml");
    let content = fs::read_to_string(&path).ok()?;
    let mut discovery = ManifestDiscovery::default();

    // Try [project] name first, then [tool.poetry] name
    let name = extract_toml_value(&content, "name", Some("[project]"))
        .or_else(|| extract_toml_value(&content, "name", Some("[tool.poetry]")))
        .unwrap_or_else(|| "app".to_string());

    // Check for packages in [tool.setuptools.packages.find]
    let src_dir = if root.join("src").exists() {
        "src".to_string()
    } else if root.join(&name).exists() {
        name.clone()
    } else {
        ".".to_string()
    };

    discovery.modules.insert(
        name.clone(),
        ManifestModule {
            name,
            source_paths: vec![src_dir.to_string()],
            dependencies: Vec::new(),
        },
    );
    discovery.source_dirs.push(src_dir.to_string());

    Some(discovery)
}

// ─── TOML Helpers ────────────────────────────────────────────────────────

/// Extract a string value from a TOML key, optionally within a specific section.
fn extract_toml_value(content: &str, key: &str, section: Option<&str>) -> Option<String> {
    let search_content = if let Some(section_header) = section {
        extract_section(content, section_header)?
    } else {
        content.to_string()
    };

    for line in search_content.lines() {
        let line = line.trim();
        if let Some(eq_pos) = line.find('=') {
            let k = line[..eq_pos].trim();
            if k == key {
                let val = line[eq_pos + 1..].trim();
                // Strip quotes
                if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
                    return Some(val[1..val.len() - 1].to_string());
                }
                return Some(val.to_string());
            }
        }
    }
    None
}

/// Extract an array of strings from a TOML key within a section.
fn extract_toml_array(content: &str, key: &str, section: Option<&str>) -> Option<Vec<String>> {
    let search_content = if let Some(section_header) = section {
        extract_section(content, section_header)?
    } else {
        content.to_string()
    };

    for line in search_content.lines() {
        let line = line.trim();
        if let Some(eq_pos) = line.find('=') {
            let k = line[..eq_pos].trim();
            if k == key {
                let val = line[eq_pos + 1..].trim();
                if val.starts_with('[') && val.ends_with(']') {
                    let inner = &val[1..val.len() - 1];
                    let items: Vec<String> = inner
                        .split(',')
                        .map(|s| {
                            let s = s.trim();
                            if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
                                s[1..s.len() - 1].to_string()
                            } else {
                                s.to_string()
                            }
                        })
                        .filter(|s| !s.is_empty())
                        .collect();
                    return Some(items);
                }
            }
        }
    }
    None
}

/// Extract the content of a TOML section (from header to next section or EOF).
fn extract_section(content: &str, header: &str) -> Option<String> {
    let start = content.find(header)?;
    let after = &content[start + header.len()..];
    // Find the next section header
    let end = after.find("\n[").map(|pos| pos + 1).unwrap_or(after.len());
    Some(after[..end].to_string())
}

/// Split TOML content into repeated array-of-table sections (e.g., [[bin]]).
fn split_toml_array_sections(content: &str, header: &str) -> Vec<String> {
    let mut sections = Vec::new();
    let mut search_from = 0;

    while let Some(start) = content[search_from..].find(header) {
        let abs_start = search_from + start + header.len();
        let rest = &content[abs_start..];

        // Find end: next [[...]] or [...]  section
        let end = rest
            .find("\n[[")
            .or_else(|| rest.find("\n["))
            .map(|pos| pos + 1)
            .unwrap_or(rest.len());

        sections.push(rest[..end].to_string());
        search_from = abs_start + end;
    }

    sections
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_parse_cargo_toml_basic() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        fs::write(tmp.path().join("src/lib.rs"), "").unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            r#"
[package]
name = "my-crate"
version = "0.1.0"

[dependencies]
serde = "1.0"
regex = "1.0"
"#,
        )
        .unwrap();

        let result = parse_cargo_toml(tmp.path()).unwrap();
        assert!(result.modules.contains_key("my-crate"));
        let module = &result.modules["my-crate"];
        assert_eq!(module.source_paths, vec!["src"]);
        assert!(module.dependencies.contains(&"serde".to_string()));
        assert!(module.dependencies.contains(&"regex".to_string()));
    }

    #[test]
    fn test_parse_package_swift_basic() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("Sources/MyLib")).unwrap();
        fs::write(
            tmp.path().join("Package.swift"),
            r#"
// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "MyPackage",
    targets: [
        .target(name: "MyLib", dependencies: ["Logging"]),
        .target(name: "MyApp", dependencies: [.target(name: "MyLib")], path: "Sources/App"),
        .testTarget(name: "MyLibTests", dependencies: ["MyLib"]),
    ]
)
"#,
        )
        .unwrap();

        let result = parse_package_swift(tmp.path()).unwrap();
        assert!(result.modules.contains_key("MyLib"));
        assert!(result.modules.contains_key("MyApp"));
        // testTarget should NOT be in modules
        assert!(!result.modules.contains_key("MyLibTests"));

        let mylib = &result.modules["MyLib"];
        assert_eq!(mylib.source_paths, vec!["Sources/MyLib"]);
        assert!(mylib.dependencies.contains(&"Logging".to_string()));

        let myapp = &result.modules["MyApp"];
        assert_eq!(myapp.source_paths, vec!["Sources/App"]);
    }

    #[test]
    fn test_parse_package_json_workspaces() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("packages/core/src")).unwrap();
        fs::create_dir_all(tmp.path().join("packages/web/src")).unwrap();
        fs::write(
            tmp.path().join("packages/core/package.json"),
            r#"{"name": "@app/core"}"#,
        )
        .unwrap();
        fs::write(
            tmp.path().join("packages/web/package.json"),
            r#"{"name": "@app/web"}"#,
        )
        .unwrap();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        fs::write(
            tmp.path().join("package.json"),
            r#"{"name": "my-app", "workspaces": ["packages/*"]}"#,
        )
        .unwrap();

        let result = parse_package_json(tmp.path()).unwrap();
        assert!(result.modules.contains_key("core"));
        assert!(result.modules.contains_key("web"));
        assert!(
            result
                .source_dirs
                .contains(&"packages/core/src".to_string())
        );
    }

    #[test]
    fn test_parse_go_mod() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("cmd")).unwrap();
        fs::create_dir_all(tmp.path().join("internal")).unwrap();
        fs::write(
            tmp.path().join("go.mod"),
            "module github.com/user/myproject\n\ngo 1.21\n",
        )
        .unwrap();

        let result = parse_go_mod(tmp.path()).unwrap();
        assert!(result.modules.contains_key("myproject"));
        assert!(result.source_dirs.contains(&"cmd".to_string()));
        assert!(result.source_dirs.contains(&"internal".to_string()));
    }

    #[test]
    fn test_extract_balanced_parens() {
        assert_eq!(
            extract_balanced_parens("name: \"Foo\", path: \"bar\")"),
            Some("name: \"Foo\", path: \"bar\"".to_string())
        );
        assert_eq!(
            extract_balanced_parens("a(b), c)"),
            Some("a(b), c".to_string())
        );
        assert_eq!(extract_balanced_parens("no close paren"), None);
    }
}
