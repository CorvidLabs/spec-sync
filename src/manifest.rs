//! Manifest-aware module detection.
//!
//! Parses language-specific manifest files (Package.swift, Cargo.toml,
//! build.gradle.kts, package.json, etc.) to discover targets, source paths,
//! and module names instead of relying on directory scanning alone.

use cap_std::ambient_authority;
use cap_std::fs::{Dir, Metadata};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Component, Path};

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

/// A Gradle module name and its effective project-relative directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GradleSettingsModule {
    pub(crate) name: String,
    pub(crate) path: String,
}

/// Discover modules from all supported manifest files in the project root.
#[allow(dead_code)]
pub fn discover_from_manifests(root: &Path) -> ManifestDiscovery {
    discover_from_manifests_checked(root).unwrap_or_default()
}

/// Discover modules while surfacing malformed manifest inputs.
pub fn discover_from_manifests_checked(root: &Path) -> Result<ManifestDiscovery, String> {
    let mut discovery = ManifestDiscovery::default();

    if let Some(d) = parse_cargo_toml(root) {
        merge_discovery(&mut discovery, d);
    }
    if let Some(d) = parse_package_swift(root) {
        merge_discovery(&mut discovery, d);
    }
    if let Some(d) = parse_gradle_checked(root)? {
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

    Ok(discovery)
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

#[allow(dead_code)]
fn parse_gradle(root: &Path) -> Option<ManifestDiscovery> {
    parse_gradle_checked(root).ok().flatten()
}

fn parse_gradle_checked(root: &Path) -> Result<Option<ManifestDiscovery>, String> {
    let project_root = Dir::open_ambient_dir(root, ambient_authority()).map_err(|error| {
        format!(
            "Cannot open Gradle project root {} as a confined directory: {error}",
            root.display()
        )
    })?;
    // Try Kotlin DSL first, then Groovy. A settings manifest is independently sufficient for a
    // multi-project Gradle workspace; do not require a root build script before parsing it.
    let build_path = if root.join("build.gradle.kts").exists() {
        Some(root.join("build.gradle.kts"))
    } else if root.join("build.gradle").exists() {
        Some(root.join("build.gradle"))
    } else {
        None
    };
    let settings_path = if root.join("settings.gradle.kts").exists() {
        Some(root.join("settings.gradle.kts"))
    } else if root.join("settings.gradle").exists() {
        Some(root.join("settings.gradle"))
    } else {
        None
    };
    if build_path.is_none() && settings_path.is_none() {
        return Ok(None);
    }
    let content = if let Some(path) = build_path {
        fs::read_to_string(&path).map_err(|error| {
            format!(
                "Cannot read Gradle build manifest {}: {error}",
                path.display()
            )
        })?
    } else {
        String::new()
    };
    let modules = if let Some(settings_path) = settings_path {
        let settings = fs::read_to_string(&settings_path).map_err(|error| {
            format!(
                "Cannot read Gradle settings manifest {}: {error}",
                settings_path.display()
            )
        })?;
        parse_gradle_settings(&settings).map_err(|error| {
            format!(
                "Cannot parse Gradle settings manifest {}: {error}",
                settings_path.display()
            )
        })?
    } else {
        Vec::new()
    };
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
            if gradle_confined_directory_exists(&project_root, dir)? {
                discovery.source_dirs.push(dir.to_string());
            }
        }
    } else {
        // Standard Gradle: src/main/kotlin or src/main/java
        for dir in &["src/main/kotlin", "src/main/java", "src/main/scala"] {
            if gradle_confined_directory_exists(&project_root, dir)? {
                discovery.source_dirs.push(dir.to_string());
            }
        }
    }

    for module in modules {
        let module_src = format!("{}/src/main", module.path);
        let kotlin_source = format!("{}/src/main/kotlin", module.path);
        let java_source = format!("{}/src/main/java", module.path);
        let source_path = if gradle_confined_directory_exists(&project_root, &kotlin_source)? {
            kotlin_source
        } else if gradle_confined_directory_exists(&project_root, &java_source)? {
            java_source
        } else {
            module_src
        };

        discovery.modules.insert(
            module.name.clone(),
            ManifestModule {
                name: module.name,
                source_paths: vec![source_path.clone()],
                dependencies: Vec::new(),
            },
        );
        if !discovery.source_dirs.contains(&source_path) {
            discovery.source_dirs.push(source_path);
        }
    }

    if discovery.modules.is_empty() && discovery.source_dirs.is_empty() {
        Ok(None)
    } else {
        Ok(Some(discovery))
    }
}

/// Parse Groovy or Kotlin Gradle settings into effective module directories.
///
/// Supports parenthesized and bare `include` forms, single or double quotes,
/// multiline declarations, nested `:module:name` paths, and the common
/// `project(...).projectDir = ...` and `project(...).setProjectDir(...)`
/// overrides with `file(...)` / `new File(rootDir, ...)` path expressions.
pub(crate) fn parse_gradle_settings(content: &str) -> Result<Vec<GradleSettingsModule>, String> {
    let content = strip_gradle_comments(content)?;
    let mut included = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut index = 0usize;
    while index < lines.len() {
        let trimmed = lines[index].trim_start();
        if !is_gradle_include_start(trimmed) {
            index += 1;
            continue;
        }

        let mut statement = trimmed.to_string();
        let mut balance = gradle_paren_balance(trimmed);
        while index + 1 < lines.len() && (balance > 0 || statement.trim_end().ends_with(',')) {
            index += 1;
            statement.push('\n');
            statement.push_str(lines[index].trim());
            balance += gradle_paren_balance(lines[index]);
        }
        if balance != 0 {
            return Err("Gradle include declaration has unbalanced parentheses".to_string());
        }
        for value in parse_gradle_include_statement(&statement)? {
            if !value.trim().trim_start_matches(':').is_empty() {
                included.push(normalize_gradle_module_path(&value, "module path")?);
            }
        }
        index += 1;
    }

    let overrides = parse_gradle_project_dir_overrides(&content)?;

    included.sort();
    included.dedup();
    Ok(included
        .into_iter()
        .map(|name| GradleSettingsModule {
            path: overrides
                .get(&name)
                .cloned()
                .unwrap_or_else(|| name.clone()),
            name,
        })
        .collect())
}

fn parse_gradle_project_dir_overrides(content: &str) -> Result<HashMap<String, String>, String> {
    let mut overrides = HashMap::new();
    let mut search_start = 0usize;

    while let Some((project_dir_index, syntax)) =
        find_next_gradle_project_dir_override(content, search_start)
    {
        let marker = syntax.marker();
        let before = &content[..project_dir_index];
        let Some(project_index) = find_gradle_project_call(before) else {
            search_start = project_dir_index + marker.len();
            continue;
        };

        let project_call = before[project_index + "project".len()..].trim_start();
        let (project_arguments, project_remainder) = gradle_parenthesized(project_call)?;
        if !project_remainder.trim().is_empty() {
            search_start = project_dir_index + marker.len();
            continue;
        }
        let module_values = gradle_string_arguments(project_arguments)?;
        if module_values.len() != 1 {
            return Err("Gradle projectDir assignment must identify one module".to_string());
        }

        let after_marker = content[project_dir_index + marker.len()..].trim_start();
        let expression = match syntax {
            GradleProjectDirSyntax::Assignment => {
                let Some(right_hand_side) = after_marker.strip_prefix('=') else {
                    return Err("Gradle projectDir assignment is missing '='".to_string());
                };
                right_hand_side.trim_start()
            }
            GradleProjectDirSyntax::Setter => {
                let (arguments, remainder) = gradle_parenthesized(after_marker)?;
                require_gradle_expression_end(remainder)?;
                arguments.trim()
            }
        };
        let path = parse_gradle_project_dir_path(expression)?;

        let module = normalize_gradle_module_path(&module_values[0], "projectDir module path")?;
        let path = normalize_gradle_project_relative_path(&path, "projectDir path", true)?;
        overrides.insert(module, path);
        search_start = project_dir_index + marker.len();
    }

    Ok(overrides)
}

#[derive(Clone, Copy)]
enum GradleProjectDirSyntax {
    Assignment,
    Setter,
}

impl GradleProjectDirSyntax {
    fn marker(self) -> &'static str {
        match self {
            Self::Assignment => ".projectDir",
            Self::Setter => ".setProjectDir",
        }
    }
}

fn find_next_gradle_project_dir_override(
    content: &str,
    search_start: usize,
) -> Option<(usize, GradleProjectDirSyntax)> {
    [
        GradleProjectDirSyntax::Assignment,
        GradleProjectDirSyntax::Setter,
    ]
    .into_iter()
    .filter_map(|syntax| {
        content[search_start..]
            .find(syntax.marker())
            .map(|relative| (search_start + relative, syntax))
    })
    .min_by_key(|(index, _)| *index)
}

fn parse_gradle_project_dir_path(expression: &str) -> Result<String, String> {
    if let Some(file_call) = expression.strip_prefix("file") {
        let (arguments, remainder) = gradle_parenthesized(file_call.trim_start())?;
        require_gradle_expression_end(remainder)?;
        let path_values = gradle_string_arguments(arguments)?;
        if path_values.len() != 1 {
            return Err("Gradle projectDir assignment must contain one path".to_string());
        }
        Ok(path_values[0].clone())
    } else if let Some(file_call) = expression.strip_prefix("new") {
        let Some(file_call) = file_call.trim_start().strip_prefix("File") else {
            return Err("Unsupported Gradle projectDir assignment".to_string());
        };
        let (arguments, remainder) = gradle_parenthesized(file_call.trim_start())?;
        require_gradle_expression_end(remainder)?;
        parse_gradle_root_dir_file_arguments(arguments)
    } else {
        Err("Unsupported Gradle projectDir assignment".to_string())
    }
}

fn normalize_gradle_module_path(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim().trim_start_matches(':');
    let bytes = value.as_bytes();
    if bytes.get(1) == Some(&b':')
        && bytes.first().is_some_and(u8::is_ascii_alphabetic)
        && bytes
            .get(2)
            .is_some_and(|byte| matches!(byte, b'/' | b'\\'))
    {
        return Err(format!(
            "Gradle {label} must remain beneath the project root"
        ));
    }
    normalize_gradle_project_relative_path(&value.replace(':', "/"), label, false)
}

fn normalize_gradle_project_relative_path(
    value: &str,
    label: &str,
    allow_project_root: bool,
) -> Result<String, String> {
    let normalized = value.replace('\\', "/");
    if normalized.starts_with('/') {
        return Err(format!(
            "Gradle {label} must remain beneath the project root"
        ));
    }

    let mut components = Vec::new();
    let mut requires_normalization = false;
    for component in normalized.split('/') {
        if component.is_empty() {
            continue;
        }
        if component == "." {
            requires_normalization = true;
            continue;
        }
        if component == ".." {
            requires_normalization = true;
            if components.pop().is_none() {
                return Err(format!(
                    "Gradle {label} must remain beneath the project root"
                ));
            }
            continue;
        }
        if components.is_empty()
            && component.as_bytes().get(1) == Some(&b':')
            && component
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphabetic)
        {
            return Err(format!(
                "Gradle {label} must remain beneath the project root"
            ));
        }
        components.push(component);
    }

    if components.is_empty() {
        if allow_project_root {
            return Ok(".".to_string());
        }
        return Err(format!("Gradle {label} must identify a project path"));
    }
    Ok(if requires_normalization {
        components.join("/")
    } else {
        normalized
    })
}

#[cfg(windows)]
fn gradle_metadata_is_link(metadata: &Metadata) -> bool {
    use cap_std::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn gradle_metadata_is_link(metadata: &Metadata) -> bool {
    metadata.file_type().is_symlink()
}

fn gradle_confined_directory_exists(root: &Dir, relative: &str) -> Result<bool, String> {
    let mut directory = root
        .try_clone()
        .map_err(|error| format!("Cannot retain the Gradle project root: {error}"))?;
    let mut inspected = Vec::new();

    for component in Path::new(relative).components() {
        let name = match component {
            Component::CurDir => continue,
            Component::Normal(name) => name,
            _ => {
                return Err(format!(
                    "Gradle source directory {relative} must remain beneath the project root"
                ));
            }
        };
        inspected.push(name.to_string_lossy().into_owned());
        let display = inspected.join("/");
        let before = match directory.symlink_metadata(name) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => {
                return Err(format!(
                    "Cannot inspect confined Gradle source directory {display}: {error}"
                ));
            }
        };
        if gradle_metadata_is_link(&before) {
            return Err(format!(
                "Gradle source directory {display} must not traverse a symlink or reparse point"
            ));
        }
        if !before.is_dir() {
            return Ok(false);
        }
        let next = directory.open_dir(name).map_err(|error| {
            format!("Cannot open confined Gradle source directory {display}: {error}")
        })?;
        let after = directory.symlink_metadata(name).map_err(|error| {
            format!("Cannot re-inspect confined Gradle source directory {display}: {error}")
        })?;
        if gradle_metadata_is_link(&after) || !after.is_dir() {
            return Err(format!(
                "Gradle source directory {display} changed during confined inspection"
            ));
        }
        directory = next;
    }

    Ok(true)
}

fn find_gradle_project_call(content: &str) -> Option<usize> {
    content.rmatch_indices("project").find_map(|(index, _)| {
        let before_is_boundary = content[..index]
            .chars()
            .next_back()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_');
        let after = content[index + "project".len()..].trim_start();
        (before_is_boundary && after.starts_with('(')).then_some(index)
    })
}

fn gradle_parenthesized(value: &str) -> Result<(&str, &str), String> {
    let Some(value) = value.strip_prefix('(') else {
        return Err("Gradle declaration is missing an opening parenthesis".to_string());
    };
    let mut depth = 1usize;
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in value.char_indices() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }
        if matches!(character, '\'' | '"') {
            quote = Some(character);
            continue;
        }
        match character {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Ok((&value[..index], &value[index + character.len_utf8()..]));
                }
            }
            _ => {}
        }
    }
    Err("Gradle declaration has unbalanced parentheses".to_string())
}

fn parse_gradle_include_statement(statement: &str) -> Result<Vec<String>, String> {
    let Some(arguments) = statement.strip_prefix("include") else {
        return Err("Gradle include declaration is missing 'include'".to_string());
    };
    let arguments = arguments.trim_start();
    if arguments.starts_with('(') {
        let (arguments, remainder) = gradle_parenthesized(arguments)?;
        require_gradle_complete_remainder(remainder, "include declaration")?;
        gradle_string_arguments(arguments)
    } else {
        let arguments = strip_gradle_statement_terminator(arguments, "include declaration")?;
        gradle_string_arguments(arguments)
    }
}

fn parse_gradle_root_dir_file_arguments(arguments: &str) -> Result<String, String> {
    let Some(arguments) = arguments.trim_start().strip_prefix("rootDir") else {
        return Err("Gradle new File projectDir base must be rootDir".to_string());
    };
    let Some(arguments) = arguments.trim_start().strip_prefix(',') else {
        return Err("Gradle new File projectDir base must be exactly rootDir".to_string());
    };
    let path_values = gradle_string_arguments(arguments)?;
    if path_values.len() != 1 {
        return Err("Gradle new File projectDir assignment must contain one path".to_string());
    }
    Ok(path_values[0].clone())
}

fn require_gradle_expression_end(remainder: &str) -> Result<(), String> {
    let statement_remainder = remainder
        .split_once('\n')
        .map_or(remainder, |(line, _)| line);
    require_gradle_complete_remainder(statement_remainder, "projectDir assignment")
}

fn require_gradle_complete_remainder(remainder: &str, context: &str) -> Result<(), String> {
    let remainder = remainder.trim();
    if remainder.is_empty() || remainder == ";" {
        Ok(())
    } else {
        Err(format!("Unsupported trailing Gradle {context} expression"))
    }
}

fn strip_gradle_statement_terminator<'a>(
    statement: &'a str,
    context: &str,
) -> Result<&'a str, String> {
    let statement = statement.trim_end();
    let Some(statement) = statement.strip_suffix(';') else {
        return Ok(statement);
    };
    if statement.trim_end().ends_with(';') {
        return Err(format!("Unsupported trailing Gradle {context} expression"));
    }
    Ok(statement.trim_end())
}

fn strip_gradle_comments(content: &str) -> Result<String, String> {
    let mut cleaned = String::with_capacity(content.len());
    let mut characters = content.chars().peekable();
    let mut quote = None;
    let mut escaped = false;

    while let Some(character) = characters.next() {
        if let Some(delimiter) = quote {
            cleaned.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }
        if matches!(character, '\'' | '"') {
            quote = Some(character);
            cleaned.push(character);
            continue;
        }
        if character != '/' {
            cleaned.push(character);
            continue;
        }

        match characters.peek().copied() {
            Some('/') => {
                characters.next();
                for comment_character in characters.by_ref() {
                    if comment_character == '\n' {
                        cleaned.push('\n');
                        break;
                    }
                }
            }
            Some('*') => {
                characters.next();
                cleaned.push(' ');
                let mut previous = None;
                let mut terminated = false;
                for comment_character in characters.by_ref() {
                    if comment_character == '\n' {
                        cleaned.push('\n');
                    }
                    if previous == Some('*') && comment_character == '/' {
                        terminated = true;
                        break;
                    }
                    previous = Some(comment_character);
                }
                if !terminated {
                    return Err("Gradle settings contain an unterminated block comment".to_string());
                }
            }
            _ => cleaned.push(character),
        }
    }

    if quote.is_some() {
        return Err("Gradle settings contain an unterminated quoted string".to_string());
    }
    if escaped {
        return Err("Gradle settings contain a dangling string escape".to_string());
    }
    Ok(cleaned)
}

fn is_gradle_include_start(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("include") else {
        return false;
    };
    rest.is_empty()
        || rest.chars().next().is_some_and(|character| {
            character.is_whitespace() || matches!(character, '(' | '\'' | '"')
        })
}

fn gradle_paren_balance(value: &str) -> i32 {
    let mut balance = 0i32;
    let mut quote = None;
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && quote.is_some() {
            escaped = true;
            continue;
        }
        if matches!(character, '\'' | '"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            }
            continue;
        }
        if quote.is_none() {
            match character {
                '(' => balance += 1,
                ')' => balance -= 1,
                _ => {}
            }
        }
    }
    balance
}

fn gradle_string_arguments(mut value: &str) -> Result<Vec<String>, String> {
    let mut values = Vec::new();
    loop {
        value = value.trim_start();
        if value.is_empty() {
            return Ok(values);
        }

        let (parsed, remainder) = gradle_string_literal(value)?;
        values.push(parsed);
        value = remainder.trim_start();
        if value.is_empty() {
            return Ok(values);
        }
        let Some(remainder) = value.strip_prefix(',') else {
            return Err("Unsupported or dynamic Gradle expression".to_string());
        };
        value = remainder;
    }
}

fn gradle_string_literal(value: &str) -> Result<(String, &str), String> {
    let Some(delimiter) = value
        .chars()
        .next()
        .filter(|character| matches!(character, '\'' | '"'))
    else {
        return Err("Unsupported or dynamic Gradle expression".to_string());
    };
    let mut parsed = String::new();
    let mut escaped = false;
    for (index, character) in value[delimiter.len_utf8()..].char_indices() {
        if escaped {
            match character {
                '\\' | '\'' | '"' => parsed.push(character),
                'n' => parsed.push('\n'),
                'r' => parsed.push('\r'),
                't' => parsed.push('\t'),
                _ => {
                    parsed.push('\\');
                    parsed.push(character);
                }
            }
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == delimiter {
            let end = delimiter.len_utf8() + index + character.len_utf8();
            return Ok((parsed, &value[end..]));
        } else {
            parsed.push(character);
        }
    }
    if escaped {
        Err("Gradle settings contain a dangling string escape".to_string())
    } else {
        Err("Gradle settings contain an unterminated quoted string".to_string())
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
    fn gradle_settings_support_groovy_kotlin_multiline_and_project_dir_overrides() {
        let modules = parse_gradle_settings(
            r#"
include ':groovy:member',
        ':second'
include(
    ":kotlin:member",
    ':vendor:member'
);
project(':vendor:member').projectDir = file('vendor/custom-member');
project(":second").projectDir = new File(rootDir, "modules/second")
"#,
        )
        .unwrap();

        assert_eq!(
            modules,
            vec![
                GradleSettingsModule {
                    name: "groovy/member".to_string(),
                    path: "groovy/member".to_string(),
                },
                GradleSettingsModule {
                    name: "kotlin/member".to_string(),
                    path: "kotlin/member".to_string(),
                },
                GradleSettingsModule {
                    name: "second".to_string(),
                    path: "modules/second".to_string(),
                },
                GradleSettingsModule {
                    name: "vendor/member".to_string(),
                    path: "vendor/custom-member".to_string(),
                },
            ]
        );
    }

    #[test]
    fn gradle_manifest_discovery_rejects_dynamic_include_without_partial_modules() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("src/main/kotlin")).unwrap();
        fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
        fs::write(
            tmp.path().join("settings.gradle.kts"),
            r#"include(":safe", dynamicModule)"#,
        )
        .unwrap();

        let error = discover_from_manifests_checked(tmp.path()).unwrap_err();
        assert!(error.contains("Unsupported or dynamic Gradle expression"));
        let compatibility_result = discover_from_manifests(tmp.path());
        assert!(compatibility_result.modules.is_empty());
        assert!(compatibility_result.source_dirs.is_empty());
    }

    #[test]
    fn gradle_settings_only_workspace_discovers_included_modules() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("member/src/main/kotlin")).unwrap();
        fs::write(
            tmp.path().join("settings.gradle.kts"),
            "include(\":member\")\n",
        )
        .unwrap();

        let result = discover_from_manifests_checked(tmp.path()).unwrap();

        assert_eq!(
            result.modules["member"].source_paths,
            vec!["member/src/main/kotlin"]
        );
    }

    #[test]
    fn gradle_settings_only_workspace_fails_closed_when_malformed() {
        let tmp = tempdir().unwrap();
        fs::write(tmp.path().join("settings.gradle"), "include(\":member\"\n").unwrap();

        let error = discover_from_manifests_checked(tmp.path()).unwrap_err();

        assert!(error.contains("Cannot parse Gradle settings manifest"));
        assert!(discover_from_manifests(tmp.path()).modules.is_empty());
    }

    #[test]
    fn gradle_settings_reject_unsupported_project_dir_bases_and_suffixes() {
        let outside_base = parse_gradle_settings(
            r#"
include(":outside")
project(":outside").projectDir = new File(rootDir.parentFile, "outside")
"#,
        )
        .unwrap_err();
        assert!(outside_base.contains("base must be exactly rootDir"));

        let project_root = parse_gradle_settings(
            "include(\":root\")\nproject(\":root\").projectDir = file(\".\")\n",
        )
        .unwrap();
        assert_eq!(project_root[0].path, ".");

        for assignment in [
            r#"file("modules/safe").parentFile"#,
            r#"new File(rootDir, "modules/safe") + "/outside""#,
        ] {
            let settings =
                format!("include(\":safe\")\nproject(\":safe\").projectDir = {assignment}\n");
            let error = parse_gradle_settings(&settings).unwrap_err();
            assert!(error.contains("Unsupported trailing Gradle projectDir assignment expression"));
        }
    }

    #[test]
    fn gradle_settings_reject_project_root_escapes() {
        for settings in [
            r#"
include(":outside")
project(":outside").projectDir = file("../outside")
"#,
            r#"
include(":outside")
project(":outside").setProjectDir(file("../outside"))
"#,
            r#"include(":..:outside")"#,
            r#"include("C:/outside")"#,
            r#"include(":C:\outside")"#,
            r#"
include(":outside")
project(":outside").projectDir = file("safe/../../outside")
"#,
            r#"
include(":outside")
project(":outside").projectDir = file("C:\outside")
"#,
            r#"
include(":outside")
project(":outside").projectDir = file("\\server\share")
"#,
            r#"
include(":safe")
project("C:/outside").projectDir = file("modules/safe")
"#,
        ] {
            let error = parse_gradle_settings(settings).unwrap_err();
            assert!(
                error.contains("must remain beneath the project root"),
                "unexpected Gradle confinement error: {error}"
            );
        }
    }

    #[test]
    fn gradle_settings_support_literal_set_project_dir_forms() {
        let modules = parse_gradle_settings(
            r#"
include(":first", ":second")
project(":first").setProjectDir(file("modules/first"))
project(":second").setProjectDir(new File(rootDir, "modules/second"))
"#,
        )
        .unwrap();

        assert_eq!(
            modules,
            vec![
                GradleSettingsModule {
                    name: "first".to_string(),
                    path: "modules/first".to_string(),
                },
                GradleSettingsModule {
                    name: "second".to_string(),
                    path: "modules/second".to_string(),
                },
            ]
        );
    }

    #[test]
    fn gradle_settings_reject_dynamic_or_ambiguous_set_project_dir_forms() {
        for setter in [
            r#"project(":member").setProjectDir(projectDirProvider.get())"#,
            r#"project(":member").setProjectDir(file("modules/member"), file("other"))"#,
            r#"project(":member").setProjectDir(file("modules/member")).parentFile"#,
        ] {
            let settings = format!("include(\":member\")\n{setter}\n");
            let error = parse_gradle_settings(&settings).unwrap_err();
            assert!(
                error.contains("Unsupported Gradle projectDir assignment")
                    || error.contains("must contain one path")
                    || error.contains("Unsupported trailing Gradle projectDir assignment"),
                "unexpected setProjectDir parser error: {error}"
            );
        }
    }

    #[test]
    fn gradle_manifest_discovery_rejects_project_root_escape_without_partial_modules() {
        let tmp = tempdir().unwrap();
        let project = tmp.path().join("project");
        let outside = tmp.path().join("outside/src/main/kotlin");
        fs::create_dir_all(&project).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(
            project.join("settings.gradle.kts"),
            "include(\":outside\")\nproject(\":outside\").projectDir = file(\"../outside\")\n",
        )
        .unwrap();

        let error = discover_from_manifests_checked(&project).unwrap_err();
        assert!(error.contains("must remain beneath the project root"));
        let compatibility_result = discover_from_manifests(&project);
        assert!(compatibility_result.modules.is_empty());
        assert!(compatibility_result.source_dirs.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn gradle_manifest_discovery_rejects_symlinked_module_directories() {
        use std::os::unix::fs::symlink;

        let tmp = tempdir().unwrap();
        let project = tmp.path().join("project");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(&project).unwrap();
        fs::create_dir_all(outside.join("src/main/kotlin")).unwrap();
        symlink(&outside, project.join("linked")).unwrap();
        fs::write(
            project.join("settings.gradle.kts"),
            "include(\":linked\")\n",
        )
        .unwrap();

        let error = discover_from_manifests_checked(&project).unwrap_err();
        assert!(
            error.contains("must not traverse a symlink or reparse point"),
            "unexpected Gradle symlink rejection: {error}"
        );
    }

    #[test]
    fn gradle_manifest_discovery_uses_the_effective_project_directory() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("vendor/custom/src/main/kotlin")).unwrap();
        fs::write(tmp.path().join("build.gradle"), "plugins {}\n").unwrap();
        fs::write(
            tmp.path().join("settings.gradle"),
            "include ':member'\nproject(':member').projectDir = file('vendor/custom')\n",
        )
        .unwrap();

        let result = parse_gradle(tmp.path()).unwrap();
        assert_eq!(
            result.modules["member"].source_paths,
            vec!["vendor/custom/src/main/kotlin"]
        );
        assert!(
            result
                .source_dirs
                .contains(&"vendor/custom/src/main/kotlin".to_string())
        );
    }

    #[test]
    fn gradle_settings_ignore_comments_and_decode_escaped_values() {
        let commented = parse_gradle_settings("include ':member' // \"comment\n").unwrap();
        assert_eq!(commented[0].name, "member");

        let modules = parse_gradle_settings(
            r#"
include(
    ":member", // ignored quote: "
    ':quoted\'member',
    ":slash//member" /* ignored quote: ' */
)
project(":member").projectDir = file("modules\\member")
"#,
        )
        .unwrap();

        assert_eq!(modules[0].name, "member");
        assert_eq!(modules[0].path, "modules/member");
        assert_eq!(modules[1].name, "quoted'member");
        assert_eq!(modules[2].name, "slash//member");
    }

    #[test]
    fn gradle_manifest_discovery_fails_closed_for_malformed_settings() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("src/main/kotlin")).unwrap();
        fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
        fs::write(
            tmp.path().join("settings.gradle.kts"),
            "include(\":member\"\n",
        )
        .unwrap();

        let error = discover_from_manifests_checked(tmp.path()).unwrap_err();
        assert!(error.contains("Cannot parse Gradle settings manifest"));
        let result = discover_from_manifests(tmp.path());
        assert!(result.modules.is_empty());
        assert!(result.source_dirs.is_empty());
    }

    #[test]
    fn gradle_manifest_discovery_accepts_comments_and_escaped_paths() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("modules/member/src/main/kotlin")).unwrap();
        fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
        fs::write(
            tmp.path().join("settings.gradle.kts"),
            r#"include(":member") // ignored unterminated quote: "
project(":member").projectDir = file("modules\\member") /* ignored: ' */
"#,
        )
        .unwrap();

        let result = discover_from_manifests(tmp.path());
        assert_eq!(
            result.modules["member"].source_paths,
            vec!["modules/member/src/main/kotlin"]
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
