use crate::config::default_schema_pattern;
use crate::exports::{get_exported_symbols, has_extension, is_test_file};
use crate::parser::{get_missing_sections, get_spec_symbols, parse_frontmatter};
use crate::types::{CoverageReport, SpecSyncConfig, ValidationResult};
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// ─── Schema Table Discovery ──────────────────────────────────────────────

/// Extract table names from SQL schema files.
pub fn get_schema_table_names(root: &Path, config: &SpecSyncConfig) -> HashSet<String> {
    let mut tables = HashSet::new();
    let schema_dir = match &config.schema_dir {
        Some(d) => root.join(d),
        None => return tables,
    };

    if !schema_dir.exists() {
        return tables;
    }

    let pattern_str = config
        .schema_pattern
        .as_deref()
        .unwrap_or_else(|| default_schema_pattern());

    let re = match Regex::new(pattern_str) {
        Ok(r) => r,
        Err(_) => return tables,
    };

    if let Ok(entries) = fs::read_dir(&schema_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "ts" && ext != "sql" {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&path) {
                for caps in re.captures_iter(&content) {
                    if let Some(name) = caps.get(1) {
                        tables.insert(name.as_str().to_string());
                    }
                }
            }
        }
    }

    tables
}

// ─── File Discovery ──────────────────────────────────────────────────────

/// Find all *.spec.md files in a directory recursively.
pub fn find_spec_files(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if !dir.exists() {
        return results;
    }

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file()
            && let Some(name) = path.file_name().and_then(|n| n.to_str())
            && name.ends_with(".spec.md")
        {
            results.push(path.to_path_buf());
        }
    }

    results.sort();
    results
}

/// Find source files in a directory, respecting exclusions.
fn find_source_files(
    dir: &Path,
    exclude_dirs: &HashSet<String>,
    config: &SpecSyncConfig,
) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if !dir.exists() {
        return results;
    }

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_str().unwrap_or("");
                !exclude_dirs.contains(name)
            } else {
                true
            }
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && has_extension(path, &config.source_extensions) && !is_test_file(path) {
            results.push(path.to_path_buf());
        }
    }

    results
}

// ─── Single Spec Validation ──────────────────────────────────────────────

/// Validate a single spec file against source code.
pub fn validate_spec(
    spec_path: &Path,
    root: &Path,
    schema_tables: &HashSet<String>,
    config: &SpecSyncConfig,
) -> ValidationResult {
    let rel_path = spec_path
        .strip_prefix(root)
        .unwrap_or(spec_path)
        .to_string_lossy()
        .to_string();

    let mut result = ValidationResult::new(rel_path);

    let content = match fs::read_to_string(spec_path) {
        Ok(c) => c.replace("\r\n", "\n"),
        Err(e) => {
            result.errors.push(format!("Cannot read spec: {e}"));
            return result;
        }
    };

    let parsed = match parse_frontmatter(&content) {
        Some(p) => p,
        None => {
            result.errors.push(
                "Missing or malformed YAML frontmatter (expected --- delimiters)".to_string(),
            );
            return result;
        }
    };

    let fm = &parsed.frontmatter;
    let body = &parsed.body;

    // ─── Level 1: Structural ──────────────────────────────────────────

    if fm.module.is_none() {
        result
            .errors
            .push("Frontmatter missing required field: module".to_string());
    }
    if fm.version.is_none() {
        result
            .errors
            .push("Frontmatter missing required field: version".to_string());
    }
    if fm.status.is_none() {
        result
            .errors
            .push("Frontmatter missing required field: status".to_string());
    }
    if fm.files.is_empty() {
        result.errors.push(
            "Frontmatter missing required field: files (must be a non-empty list)".to_string(),
        );
    }

    // Check files exist
    for file in &fm.files {
        let full_path = root.join(file);
        if !full_path.exists() {
            result.errors.push(format!("Source file not found: {file}"));
        }
    }

    // Check db_tables exist in schema
    for table in &fm.db_tables {
        if !schema_tables.is_empty() && !schema_tables.contains(table) {
            result
                .errors
                .push(format!("DB table not found in schema: {table}"));
        }
    }

    // Required markdown sections
    let missing = get_missing_sections(body, &config.required_sections);
    for section in &missing {
        result
            .errors
            .push(format!("Missing required section: ## {section}"));
    }

    // ─── Level 2: API Surface ─────────────────────────────────────────

    if !fm.files.is_empty() {
        let mut all_exports: Vec<String> = Vec::new();
        for file in &fm.files {
            let full_path = root.join(file);
            let exports = get_exported_symbols(&full_path);
            all_exports.extend(exports);
        }

        // Deduplicate
        let mut seen = HashSet::new();
        all_exports.retain(|s| seen.insert(s.clone()));

        let spec_symbols = get_spec_symbols(body);
        let spec_set: HashSet<&str> = spec_symbols.iter().map(|s| s.as_str()).collect();
        let export_set: HashSet<&str> = all_exports.iter().map(|s| s.as_str()).collect();

        // Spec documents something that doesn't exist = ERROR
        for sym in &spec_symbols {
            if !export_set.contains(sym.as_str()) {
                result.errors.push(format!(
                    "Spec documents '{sym}' but no matching export found in source"
                ));
            }
        }

        // Code exports something not in spec = WARNING
        for sym in &all_exports {
            if !spec_set.contains(sym.as_str()) {
                result
                    .warnings
                    .push(format!("Export '{sym}' not in spec (undocumented)"));
            }
        }

        let documented = spec_symbols
            .iter()
            .filter(|s| export_set.contains(s.as_str()))
            .count();

        if !all_exports.is_empty() {
            let summary = format!("{documented}/{} exports documented", all_exports.len());
            if documented < all_exports.len() {
                result.warnings.insert(0, summary);
            } else {
                result.export_summary = Some(summary);
            }
        }
    }

    // ─── Level 3: Dependencies ────────────────────────────────────────

    if !fm.depends_on.is_empty() {
        for dep in &fm.depends_on {
            let full_path = root.join(dep);
            if !full_path.exists() {
                result
                    .errors
                    .push(format!("Dependency spec not found: {dep}"));
            }
        }
    }

    // Check Consumed By section references
    let consumed_re = Regex::new(r"(?s)### Consumed By\s*\n(.*?)(?:\n## |\n### |$)").unwrap();
    if let Some(caps) = consumed_re.captures(body) {
        let section = caps.get(1).unwrap().as_str();
        let file_ref_re = Regex::new(r"\|\s*`([^`]+\.\w+)`\s*\|").unwrap();
        for caps in file_ref_re.captures_iter(section) {
            if let Some(file_ref) = caps.get(1) {
                let file_path = root.join(file_ref.as_str());
                if !file_path.exists() {
                    result.warnings.push(format!(
                        "Consumed By references missing file: {}",
                        file_ref.as_str()
                    ));
                }
            }
        }
    }

    result
}

// ─── Coverage ────────────────────────────────────────────────────────────

fn collect_specced_files(spec_files: &[PathBuf]) -> HashSet<String> {
    let mut specced = HashSet::new();
    for spec_file in spec_files {
        if let Ok(content) = fs::read_to_string(spec_file) {
            let content = content.replace("\r\n", "\n");
            if let Some(parsed) = parse_frontmatter(&content) {
                for f in &parsed.frontmatter.files {
                    specced.insert(f.clone());
                }
            }
        }
    }
    specced
}

fn get_module_dirs(dir: &Path, exclude_dirs: &HashSet<String>) -> Vec<String> {
    let mut modules = Vec::new();
    if !dir.exists() {
        return modules;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type()
                && ft.is_dir()
            {
                let name = entry.file_name().to_string_lossy().to_string();
                if !exclude_dirs.contains(&name) {
                    modules.push(name);
                }
            }
        }
    }

    modules.sort();
    modules
}

/// Compute file and module coverage.
pub fn compute_coverage(
    root: &Path,
    spec_files: &[PathBuf],
    config: &SpecSyncConfig,
) -> CoverageReport {
    let specced_files = collect_specced_files(spec_files);
    let exclude_dirs: HashSet<String> = config.exclude_dirs.iter().cloned().collect();

    let mut all_source_files: Vec<String> = Vec::new();
    for src_dir in &config.source_dirs {
        let full_dir = root.join(src_dir);
        let files: Vec<String> = find_source_files(&full_dir, &exclude_dirs, config)
            .into_iter()
            .filter_map(|f| {
                let rel = f.strip_prefix(root).ok()?;
                let rel_str = rel.to_string_lossy().replace('\\', "/");
                // Check exclude patterns (simple glob matching)
                for pattern in &config.exclude_patterns {
                    // **/dir/** — matches path containing dir
                    if pattern.starts_with("**/") && pattern.ends_with("/**") {
                        let dir_part = &pattern[3..pattern.len() - 3];
                        if rel_str.contains(dir_part) {
                            return None;
                        }
                    }
                    // **/*.ext or **/filename — matches suffix/filename
                    else if let Some(suffix) = pattern.strip_prefix("**/") {
                        if let Some(ext) = suffix.strip_prefix('*') {
                            // **/*.test.ts -> .test.ts
                            if rel_str.ends_with(ext) {
                                return None;
                            }
                        } else {
                            // **/index.ts -> matches any path ending in /index.ts or equal to index.ts
                            if rel_str.ends_with(&format!("/{suffix}")) || rel_str == *suffix {
                                return None;
                            }
                        }
                    }
                    // Literal contains match as fallback
                    else if rel_str.contains(pattern.as_str()) {
                        return None;
                    }
                }
                Some(rel_str)
            })
            .collect();
        all_source_files.extend(files);
    }

    let unspecced_files: Vec<String> = all_source_files
        .iter()
        .filter(|f| !specced_files.contains(*f))
        .cloned()
        .collect();

    // Module coverage
    let specs_dir = root.join(&config.specs_dir);
    let spec_modules: HashSet<String> = get_module_dirs(&specs_dir, &HashSet::new())
        .into_iter()
        .collect();

    let mut unspecced_modules = Vec::new();
    for src_dir in &config.source_dirs {
        let full_dir = root.join(src_dir);
        for module in get_module_dirs(&full_dir, &exclude_dirs) {
            if !spec_modules.contains(&module) {
                unspecced_modules.push(module);
            }
        }
    }

    let specced_count = all_source_files.len() - unspecced_files.len();
    let coverage_percent = if all_source_files.is_empty() {
        100
    } else {
        (specced_count * 100) / all_source_files.len()
    };

    CoverageReport {
        total_source_files: all_source_files.len(),
        specced_file_count: specced_count,
        unspecced_files,
        unspecced_modules,
        coverage_percent,
    }
}
