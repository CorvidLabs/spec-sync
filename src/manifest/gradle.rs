//! Gradle discovery and effective project-directory interpretation.

use cap_std::ambient_authority;
use cap_std::fs::Dir;
use std::collections::HashMap;
use std::path::{Component, Path};

use super::{GradleSettingsModule, ManifestDiscovery, ManifestModule};

mod access;
mod guards;
mod syntax;

use access::{gradle_confined_directory_exists, gradle_confined_manifest_text};
use guards::{
    gradle_brace_depth, gradle_follows_detached_control_header, reject_non_leading_gradle_includes,
    reject_unsupported_gradle_project_dir_mutations,
};
use syntax::{
    find_gradle_project_call, find_unquoted_gradle_fragment, gradle_paren_balance,
    gradle_parenthesized, gradle_string_arguments, gradle_string_literal, is_gradle_include_start,
    parse_gradle_include_statement, parse_gradle_root_dir_file_arguments,
    require_gradle_expression_end, strip_gradle_comments,
};

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
    parse_gradle_checked_with_root(root, &project_root)
}

pub(super) fn parse_gradle_checked_with_root(
    root: &Path,
    project_root: &Dir,
) -> Result<Option<ManifestDiscovery>, String> {
    // Try Kotlin DSL first, then Groovy. A settings manifest is independently sufficient for a
    // multi-project Gradle workspace; do not require a root build script before parsing it. Every
    // recognized variant is preflighted before precedence is selected so a shadowed unsafe entry
    // cannot evade the retained reader.
    let build_kotlin = gradle_confined_manifest_text(project_root, "build.gradle.kts", "build")?;
    let build_groovy = gradle_confined_manifest_text(project_root, "build.gradle", "build")?;
    let settings_kotlin =
        gradle_confined_manifest_text(project_root, "settings.gradle.kts", "settings")?;
    let settings_groovy =
        gradle_confined_manifest_text(project_root, "settings.gradle", "settings")?;
    let build = build_kotlin
        .map(|content| ("build.gradle.kts", content))
        .or_else(|| build_groovy.map(|content| ("build.gradle", content)));
    let settings = settings_kotlin
        .map(|content| ("settings.gradle.kts", content))
        .or_else(|| settings_groovy.map(|content| ("settings.gradle", content)));
    if build.is_none() && settings.is_none() {
        return Ok(None);
    }
    let content = build.map_or_else(String::new, |(_, content)| content);
    let modules = if let Some((settings_name, settings_text)) = settings.as_ref() {
        parse_gradle_settings(settings_text).map_err(|error| {
            format!("Cannot parse Gradle settings manifest {settings_name}: {error}")
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
            if gradle_confined_directory_exists(project_root, dir)? {
                discovery.source_dirs.push(dir.to_string());
            }
        }
    } else {
        // Standard Gradle: src/main/kotlin or src/main/java
        for dir in &["src/main/kotlin", "src/main/java", "src/main/scala"] {
            if gradle_confined_directory_exists(project_root, dir)? {
                discovery.source_dirs.push(dir.to_string());
            }
        }
    }

    let has_included_modules = !modules.is_empty();
    for module in modules {
        let module_src = format!("{}/src/main", module.path);
        let kotlin_source = format!("{}/src/main/kotlin", module.path);
        let java_source = format!("{}/src/main/java", module.path);
        let source_path = if gradle_confined_directory_exists(project_root, &kotlin_source)? {
            kotlin_source
        } else if gradle_confined_directory_exists(project_root, &java_source)? {
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

    // Single-project Gradle has no `include`s. Name it from the settings identity
    // (`rootProject.name`) or, when that is unset, the project directory — the
    // same default Gradle uses. Do not invent a name from package path segments.
    if !has_included_modules {
        let settings_name = match settings.as_ref() {
            Some((_, settings_text)) => parse_gradle_root_project_name(settings_text)?,
            None => None,
        };
        if let Some(name) = settings_name
            .as_deref()
            .and_then(gradle_identity_module_name)
            .or_else(|| {
                root.file_name()
                    .and_then(|name| name.to_str())
                    .and_then(gradle_identity_module_name)
            })
        {
            discovery.modules.insert(
                name.clone(),
                ManifestModule {
                    name,
                    source_paths: discovery.source_dirs.clone(),
                    dependencies: Vec::new(),
                },
            );
        }
    }

    if discovery.modules.is_empty() && discovery.source_dirs.is_empty() {
        Ok(None)
    } else {
        Ok(Some(discovery))
    }
}

/// True when `source_dir` is a JVM language root (`src/main/kotlin`, …).
/// Immediate children of that root are package segments, not modules.
pub(super) fn is_jvm_package_source_root(source_dir: &str) -> bool {
    let normalized = source_dir.replace('\\', "/");
    let normalized = normalized.trim_end_matches('/');
    normalized.ends_with("src/main/kotlin")
        || normalized.ends_with("src/main/java")
        || normalized.ends_with("src/main/scala")
        || normalized.ends_with("src/main")
        || normalized.ends_with("src/test/kotlin")
        || normalized.ends_with("src/test/java")
        || normalized.ends_with("src/test/scala")
}

fn gradle_identity_module_name(name: &str) -> Option<String> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let mut components = Path::new(name).components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(segment)), None) => {
            let text = segment.to_str()?;
            if text.contains('/') || text.contains('\\') {
                return None;
            }
            Some(text.to_string())
        }
        _ => None,
    }
}

/// Read a literal `rootProject.name = "..."` assignment from settings.
///
/// Dynamic or otherwise unsupported right-hand sides are ignored so existing
/// settings stay parseable; callers then use the project directory name.
fn parse_gradle_root_project_name(content: &str) -> Result<Option<String>, String> {
    let content = strip_gradle_comments(content)?;
    let mut search_start = 0usize;
    let mut found = None;
    while let Some(index) =
        find_unquoted_gradle_fragment(&content, "rootProject.name", search_start)
    {
        let token_end = index + "rootProject.name".len();
        search_start = token_end;
        let before_is_boundary = content[..index]
            .chars()
            .next_back()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_');
        let after_is_boundary = content[token_end..]
            .chars()
            .next()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_');
        if !before_is_boundary || !after_is_boundary {
            continue;
        }
        let after = content[token_end..].trim_start();
        let Some(rhs) = after.strip_prefix('=') else {
            continue;
        };
        if rhs.starts_with('=') {
            continue;
        }
        let Ok((name, remainder)) = gradle_string_literal(rhs.trim_start()) else {
            continue;
        };
        let remainder = remainder.trim_start();
        if !(remainder.is_empty()
            || remainder.starts_with(';')
            || remainder.starts_with('\n')
            || remainder.starts_with('\r'))
        {
            continue;
        }
        found = gradle_identity_module_name(&name);
    }
    Ok(found)
}

/// Parse Groovy or Kotlin Gradle settings into effective module directories.
///
/// Supports parenthesized and bare `include` forms, single or double quotes,
/// multiline declarations, nested `:module:name` paths, and the common
/// `project(...).projectDir = ...` and `project(...).setProjectDir(...)`
/// overrides with `file(...)` / `new File(rootDir, ...)` path expressions.
pub(super) fn parse_gradle_settings(content: &str) -> Result<Vec<GradleSettingsModule>, String> {
    let content = strip_gradle_comments(content)?;
    reject_non_leading_gradle_includes(&content)?;
    reject_unsupported_gradle_project_dir_mutations(&content)?;
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
        let after_marker = content[project_dir_index + marker.len()..].trim_start();
        if matches!(syntax, GradleProjectDirSyntax::Assignment)
            && !is_gradle_simple_assignment(after_marker)?
        {
            search_start = project_dir_index + marker.len();
            continue;
        }
        let before = &content[..project_dir_index];
        let Some(project_index) = find_gradle_project_call(before) else {
            return Err("Unsupported indirect Gradle projectDir mutation".to_string());
        };
        let line_start = before[..project_index]
            .rfind('\n')
            .map_or(0, |index| index + 1);
        if !before[line_start..project_index].trim().is_empty() {
            return Err(
                "Unsupported qualified or conditional Gradle projectDir mutation".to_string(),
            );
        }
        if gradle_brace_depth(before)? != 0 {
            return Err("Unsupported block-scoped Gradle projectDir mutation".to_string());
        }
        if gradle_follows_detached_control_header(content, project_index)? {
            return Err("Unsupported conditional Gradle projectDir mutation".to_string());
        }

        let project_call = before[project_index + "project".len()..].trim_start();
        let (project_arguments, project_remainder) = gradle_parenthesized(project_call)?;
        if !project_remainder.trim().is_empty() {
            return Err("Unsupported indirect Gradle projectDir mutation".to_string());
        }
        let module_values = gradle_string_arguments(project_arguments)?;
        if module_values.len() != 1 {
            return Err("Gradle projectDir assignment must identify one module".to_string());
        }

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

fn is_gradle_simple_assignment(after_marker: &str) -> Result<bool, String> {
    if let Some(remainder) = after_marker.strip_prefix('=') {
        return Ok(!remainder.starts_with('=') && !remainder.starts_with('~'));
    }
    let mutation_prefix = after_marker
        .chars()
        .take_while(|character| !character.is_whitespace())
        .collect::<String>();
    if mutation_prefix.ends_with('=')
        && mutation_prefix.chars().any(|character| {
            matches!(
                character,
                '+' | '-' | '*' | '/' | '%' | '?' | '&' | '|' | '^'
            )
        })
    {
        return Err("Unsupported Gradle projectDir mutation operator".to_string());
    }
    Ok(false)
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
        find_unquoted_gradle_fragment(content, syntax.marker(), search_start)
            .map(|index| (index, syntax))
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
        if file_call
            .chars()
            .next()
            .is_none_or(|character| !character.is_whitespace())
        {
            return Err("Unsupported Gradle projectDir assignment".to_string());
        }
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
    let trimmed = value.trim();
    let rooted_gradle_identity = trimmed.starts_with(':');
    let value = trimmed.trim_start_matches(':');
    let bytes = value.as_bytes();
    if bytes.get(1) == Some(&b':')
        && bytes.first().is_some_and(u8::is_ascii_alphabetic)
        && (!rooted_gradle_identity
            || bytes
                .get(2)
                .is_some_and(|byte| matches!(byte, b'/' | b'\\')))
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

#[cfg(test)]
mod tests;
