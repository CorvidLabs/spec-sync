use super::mappings::{
    directory_mapping_fix, planned_source_path_is_safe, source_within_root, suggest_similar_file,
};
use super::schema_checks::{
    add_missing_db_table_error, add_schema_resolution_error, resolve_schema_value,
    schema_table_exists, schema_table_names_available, validate_db_table_identifier,
};
use super::{SourceSnapshot, is_cross_project_ref};
use crate::parser::{
    find_stub_sections, get_duplicate_spec_symbols, get_missing_sections, get_near_miss_sections,
    get_spec_symbols, parse_frontmatter,
};
use crate::schema::{self, SchemaTable};
use crate::types::{SpecSyncConfig, ValidationResult};
use policy::{apply_custom_rules, validate_companion_scaffold_markers};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

mod policy;

static CONSUMED_BY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)### Consumed By\s*\n(.*?)(?:\n## |\n### |$)").unwrap());
static FILE_REF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\|\s*`([^`]+\.\w+)`\s*\|").unwrap());

#[allow(clippy::too_many_arguments)]
pub(super) fn validate_spec_content_internal(
    spec_path: &Path,
    content: &str,
    root: &Path,
    schema_tables: &HashSet<String>,
    schema_columns: &HashMap<String, SchemaTable>,
    config: &SpecSyncConfig,
    validate_companions: bool,
    source_snapshots: Option<&HashMap<String, SourceSnapshot>>,
) -> ValidationResult {
    let rel_path = spec_path
        .strip_prefix(root)
        .unwrap_or(spec_path)
        .to_string_lossy()
        .to_string();

    let mut result = ValidationResult::new(rel_path);
    let content_size = content.len() as u64;
    let normalized = if content.contains("\r\n") {
        std::borrow::Cow::Owned(content.replace("\r\n", "\n"))
    } else {
        std::borrow::Cow::Borrowed(content)
    };
    let content = normalized.as_ref();

    // ─── Level 0: Is this file a document at all? ─────────────────────
    // A spec carrying an unresolved conflict is not a spec; every section and
    // symbol check below would assert a contract over text nobody has read.
    // Reported before frontmatter parsing so a conflict inside the frontmatter
    // is named as a conflict rather than incidentally as a `duplicate key`.
    //
    // Only complete opener/separator/closer triples outside fenced code blocks
    // count. A bare `=======` line is a setext `<h1>` underline in Markdown, so
    // a guard on marker *shape* would fail ordinary specs.
    if let Some(hunk) = crate::merge::document_conflict_hunks(content)
        .into_iter()
        .next()
    {
        result.errors.push(format!(
            "Unresolved merge conflict in spec body ({} ↔ {}) — resolve it before this file can be validated",
            hunk.ours_label, hunk.theirs_label
        ));
        result.fixes.push(
            "Run `spec-sync merge --all` to attempt auto-resolution, or resolve the conflict by hand"
                .to_string(),
        );
        return result;
    }
    if let Some(unmerged) = crate::merge::cached_unmerged_paths(root) {
        let spec_rel = spec_path
            .strip_prefix(root)
            .unwrap_or(spec_path)
            .to_string_lossy()
            .replace('\\', "/");
        if unmerged.contains(&spec_rel) {
            result.errors.push(format!(
                "git reports {spec_rel} as unmerged — resolve the merge conflict before validating"
            ));
            return result;
        }
    }

    let parsed = match parse_frontmatter(content) {
        Some(p) => p,
        None => {
            result.errors.push(
                "Missing or malformed YAML frontmatter (expected --- delimiters)".to_string(),
            );
            return result;
        }
    };

    // Surface parse-time frontmatter diagnostics before structural checks.
    result.errors.extend(parsed.errors.iter().cloned());
    result.warnings.extend(parsed.warnings.iter().cloned());

    let fm = &parsed.frontmatter;
    let body = &parsed.body;
    result.status = fm.parsed_status();

    // File size guard: warn if the spec exceeds the configurable limit (default 512 KB)
    {
        let limit_kb = config.rules.max_spec_size_kb.unwrap_or(512) as u64;
        let size_kb = content_size / 1024;
        if size_kb > limit_kb {
            result.warnings.push(format!(
                "Spec file is {size_kb} KB — exceeds limit of {limit_kb} KB, consider splitting into smaller specs"
            ));
        }
    }

    // Archived specs: skip all validation with zero diagnostics.
    // Must be invisible to --strict mode.
    if fm.parsed_status() == Some(crate::types::SpecStatus::Archived) {
        return result;
    }

    if validate_companions {
        validate_companion_scaffold_markers(spec_path, root, &mut result);
    }

    let config_hint = config
        .config_path
        .as_deref()
        .map(|p| format!(" (check config: {})", p.display()))
        .unwrap_or_default();

    // ─── Level 1: Structural ──────────────────────────────────────────

    if fm.module.is_none() {
        result
            .errors
            .push("Frontmatter missing required field: module".to_string());
        result
            .fixes
            .push("Add `module: your-module-name` to the YAML frontmatter block".to_string());
    }
    if fm.version.is_none() {
        result
            .errors
            .push("Frontmatter missing required field: version".to_string());
        result
            .fixes
            .push("Add `version: 1` to the YAML frontmatter block".to_string());
    }
    if fm.status.is_none() {
        result
            .errors
            .push("Frontmatter missing required field: status".to_string());
        result.fixes.push(
            "Add `status: active` (or draft/review/stable/deprecated/archived) to the frontmatter"
                .to_string(),
        );
    } else if let Some(status_str) = &fm.status
        && fm.parsed_status().is_none()
    {
        result.warnings.push(format!(
                "Unknown status '{}' — expected one of: draft, review, active, stable, deprecated, archived",
                status_str
            ));
    }

    // Status lifecycle warnings
    let spec_status = fm.parsed_status();
    if spec_status == Some(crate::types::SpecStatus::Deprecated) {
        result
            .warnings
            .push("Spec is deprecated — consider archiving with `specsync lifecycle promote <spec>` or `specsync lifecycle set <spec> archived`".to_string());
    }

    // Validate agent_policy if present
    if let Some(policy) = &fm.agent_policy {
        match policy.as_str() {
            "read-only" | "suggest-only" | "full-access" => {}
            _ => {
                result.warnings.push(format!(
                    "Unknown agent_policy '{}' — expected: read-only, suggest-only, or full-access",
                    policy
                ));
            }
        }
    }
    if fm.files.is_empty() {
        result.errors.push(
            "Frontmatter missing required field: files (must be a non-empty list)".to_string(),
        );
        result.fixes.push(
            "Add a `files:` list with relative paths to source files this spec covers".to_string(),
        );
    }

    // Check files exist and can be read as UTF-8.
    // A file that exists but cannot be decoded must fail loud here: otherwise
    // export extraction silently yields zero symbols (see exports::mod read path),
    // so undocumented exports are never checked and the spec passes — a silent
    // false-PASS of the core API-surface guarantee.
    for file in &fm.files {
        let full_path = root.join(file);
        let safe_project_relative = planned_source_path_is_safe(file);
        let snapshot = source_snapshots.and_then(|sources| sources.get(file));
        let ambient_existing_escape =
            source_snapshots.is_none() && full_path.exists() && !source_within_root(root, file);
        let confined_rejection = matches!(snapshot, Some(SourceSnapshot::Rejected))
            || (source_snapshots.is_none() && !source_within_root(root, file));
        // A `files:` entry that resolves to a directory extracts zero exports, so
        // leaving it unreported lets a spec document nothing and still pass the
        // Public API comparison — the gate goes green while measuring nothing.
        let directory_mapping = matches!(snapshot, Some(SourceSnapshot::Directory))
            || (source_snapshots.is_none()
                && safe_project_relative
                && crate::exports::files_entry_is_directory(&full_path)
                && source_within_root(root, file));
        if ambient_existing_escape || (safe_project_relative && confined_rejection) {
            result.errors.push(format!(
                "Source file `{file}` resolves outside the project root and is ignored for security"
            ));
            result.fixes.push(format!(
                "Use a path inside the project (no absolute paths, `..` escapes, or symlinks that leave the project), or remove `{file}` from the files list"
            ));
        } else if !safe_project_relative {
            result.errors.push(format!(
                "Source mapping is not a safe project-relative path: {file}"
            ));
            result.fixes.push(format!(
                "Use a safe project-relative path for `{file}` (no absolute paths, `..`, drive prefixes, or backslashes)"
            ));
        } else if directory_mapping {
            result.errors.push(format!(
                "Source file `{file}` is a directory — `files:` must list source files, not directories"
            ));
            result
                .fixes
                .push(directory_mapping_fix(root, file, config, source_snapshots));
        } else if matches!(snapshot, Some(SourceSnapshot::Unreadable)) {
            result.errors.push(format!(
                "Source file `{file}` could not be read for validation"
            ));
            result.fixes.push(format!(
                "Remove `{file}` from the files list or fix permissions"
            ));
        } else if (source_snapshots.is_some() && snapshot.is_none())
            || matches!(snapshot, Some(SourceSnapshot::Missing))
            || (source_snapshots.is_none() && !full_path.exists())
        {
            let planned_draft_mapping =
                spec_status == Some(crate::types::SpecStatus::Draft) && !config.require_draft_files;
            if planned_draft_mapping {
                result.notices.push(format!(
                    "Planned source mapping (draft; file not created yet): {file}"
                ));
            } else {
                result.errors.push(format!("Source file not found: {file}"));
                if source_snapshots.is_none()
                    && let Some(suggestion) = suggest_similar_file(root, file)
                {
                    result.fixes.push(format!(
                        "Did you mean `{suggestion}`? Update the path in frontmatter"
                    ));
                } else {
                    result.fixes.push(format!(
                        "Remove `{file}` from files list or create the source file"
                    ));
                }
            }
        } else if let Some(SourceSnapshot::Present(bytes)) = snapshot {
            result.had_present_source = true;
            let ext = full_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let is_supported = crate::types::Language::from_extension(ext).is_some();
            if is_supported && std::str::from_utf8(bytes).is_err() {
                result.errors.push(format!(
                    "Source file `{file}` could not be read as UTF-8 for validation"
                ));
                result.fixes.push(format!(
                    "Re-save `{file}` as UTF-8 (specsync validates UTF-8 source), or remove it from the files list"
                ));
            }
        } else if source_snapshots.is_none() && full_path.is_file() {
            result.had_present_source = true;
            let ext = full_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let is_supported = crate::types::Language::from_extension(ext).is_some();
            if is_supported {
                if let Err(err) = fs::read_to_string(&full_path) {
                    result.errors.push(format!(
                        "Source file `{file}` could not be read as UTF-8 for validation: {err}"
                    ));
                    result.fixes.push(format!(
                        "Re-save `{file}` as UTF-8 (specsync validates UTF-8 source), or remove it from the files list"
                    ));
                }
            } else if let Err(err) = fs::File::open(&full_path) {
                result.errors.push(format!(
                    "Source file `{file}` could not be read for validation: {err}"
                ));
                result.fixes.push(format!(
                    "Remove `{file}` from the files list or fix permissions"
                ));
            }
        }
    }

    // Check db_tables exist in the canonical replayed schema. An omitted `schema_dir` stays
    // VISIBLE rather than silently disabling every declaration — but it is a NOTICE, not a
    // warning, because it describes what this run did not do rather than a defect in the spec.
    //
    // Warnings escalate to errors under `--strict` (`compute_exit_code`). That made this
    // unactionable for any project whose schema lives in application code rather than `.sql`
    // migrations: there is nothing to point `schema_dir` at, `schema_dir` is not a spec
    // frontmatter field, and the only ways to clear it were deleting the `db_tables`
    // documentation or abandoning `strict`. Gating on advice the reader cannot take is not
    // drift detection.
    if !fm.db_tables.is_empty() && config.schema_dir.is_none() {
        result.notices.push(
            "DB table validation skipped: `db_tables` is declared but `schema_dir` is not configured"
                .to_string(),
        );
        result.fixes.push(
            "Configure `schema_dir` to validate declared database tables against migrations, or leave it unset if the schema is defined in application code"
                .to_string(),
        );
    }

    let mut schema_tables_known: Option<bool> = None;
    for table in &fm.db_tables {
        if validate_db_table_identifier(table, &mut result).is_err() {
            continue;
        }
        if config.schema_dir.is_some() {
            match schema_table_exists(table, schema_tables) {
                Ok(true) => {}
                // Only claim a table is MISSING when the schema was actually readable. An
                // unparseable migration already reports its own error; adding "every declared
                // table is absent" on top buries the real cause and suggests writing a
                // `CREATE TABLE` that is already present and correct. Resolved lazily so the
                // happy path never re-replays the schema.
                Ok(false) => {
                    let known = *schema_tables_known
                        .get_or_insert_with(|| schema_table_names_available(root, config));
                    if known {
                        add_missing_db_table_error(table, &mut result);
                    }
                }
                Err(error) => add_schema_resolution_error(
                    format!("Invalid DB table identifier `{table}`: {error}"),
                    &mut result,
                ),
            }
        }
    }

    // ─── Level 1.5: Schema Columns ──────────────────────────────────────
    if !schema_columns.is_empty() {
        let spec_schema = schema::parse_spec_schema(body);
        for table_name in &fm.db_tables {
            let actual_table = match resolve_schema_value(table_name, schema_columns) {
                Ok(Some((_, table))) => table,
                Ok(None) => continue,
                Err(error) => {
                    add_schema_resolution_error(error, &mut result);
                    continue;
                }
            };
            let spec_cols = match resolve_schema_value(table_name, &spec_schema) {
                Ok(Some((_, columns))) => columns,
                Ok(None) => continue,
                Err(error) => {
                    add_schema_resolution_error(error, &mut result);
                    continue;
                }
            };

            let actual_names: HashSet<&str> = actual_table
                .columns
                .iter()
                .map(|c| c.name.as_str())
                .collect();
            let spec_names: HashSet<&str> = spec_cols.iter().map(|c| c.name.as_str()).collect();

            // Spec documents a column that doesn't exist = ERROR
            for sc in spec_cols {
                if !actual_names.contains(sc.name.as_str()) {
                    result.errors.push(format!(
                        "Schema column `{}.{}` documented in spec but not found in migrations",
                        table_name, sc.name
                    ));
                    result.fixes.push(format!(
                        "Remove `{}` from the ### Schema section or add it via ALTER TABLE",
                        sc.name
                    ));
                }
            }

            // Column exists in schema but not in spec = WARNING
            for ac in &actual_table.columns {
                if !spec_names.contains(ac.name.as_str()) {
                    result.warnings.push(format!(
                        "Schema column `{}.{}` exists in migrations but not documented in spec",
                        table_name, ac.name
                    ));
                }
            }

            // Type mismatch = WARNING
            for sc in spec_cols {
                if let Some(ac) = actual_table.columns.iter().find(|c| c.name == sc.name) {
                    // Normalise both to uppercase for comparison
                    let spec_type = sc.col_type.to_uppercase();
                    let actual_type = ac.col_type.to_uppercase();
                    if spec_type != actual_type {
                        result.warnings.push(format!(
                                "Schema column `{}.{}` type mismatch: spec says {} but migrations say {}",
                                table_name, sc.name, spec_type, actual_type
                            ));
                    }
                }
            }
            // If spec has db_tables but no ### Schema section, that's fine —
            // column-level docs are optional. Only validate when present.
        }
    }

    // Required markdown sections
    // - draft: structure only — skip all required section checks
    // - review: structure + sections, but skip "Public API"
    // - active/stable/deprecated: all sections required
    let is_draft = spec_status == Some(crate::types::SpecStatus::Draft);
    let is_review = spec_status == Some(crate::types::SpecStatus::Review);

    if !is_draft {
        let missing = get_missing_sections(body, &config.required_sections);
        let near_misses = get_near_miss_sections(body, &config.required_sections);
        for section in &missing {
            if is_review && section == "Public API" {
                continue; // review specs can skip Public API
            }
            // Check if a near-miss heading exists — give a targeted hint
            if let Some((_, found)) = near_misses.iter().find(|(req, _)| req == section) {
                result.errors.push(format!(
                    "Missing required section: ## {section} (found '## {found}' — typo? Run --fix to rename)"
                ));
                result.fixes.push(format!(
                    "Run `spec-sync check --fix` to rename `## {found}` → `## {section}`"
                ));
            } else {
                result.errors.push(format!(
                    "Missing required section: ## {section}{config_hint}"
                ));
                result
                    .fixes
                    .push(format!("Add `## {section}` heading to the spec body"));
            }
        }
    }

    // Recorded even when export validation is skipped: a draft that documents
    // a contract over source that exists is opting out of a check it could
    // have passed, which is what separates it from an honest stub.
    result.documents_contract = !crate::parser::get_spec_symbols(body).is_empty();

    // ─── Level 1.7: Empty-Draft Section Detection ───────────────────
    let stub_sections = find_stub_sections(body, &config.required_sections);
    if !stub_sections.is_empty() {
        for section in &stub_sections {
            result.warnings.push(format!(
                "Section ## {section} contains only unfinished draft text"
            ));
            result.fixes.push(format!(
                "Replace draft content in ## {section} with real documentation"
            ));
        }
    }

    // ─── Level 2: API Surface ─────────────────────────────────────────
    // Draft and review specs skip API surface validation — exports may not exist yet.
    let skip_api = matches!(
        spec_status,
        Some(crate::types::SpecStatus::Draft) | Some(crate::types::SpecStatus::Review)
    );

    if !skip_api {
        for symbol in get_duplicate_spec_symbols(body) {
            result.errors.push(format!(
                "Public API lists `{symbol}` more than once — remove the duplicate row"
            ));
        }
    }

    if !fm.files.is_empty() && !skip_api {
        // Track exports with their source file for attribution
        let mut exports_by_file: Vec<(String, String)> = Vec::new(); // (symbol, file)
        let mut all_exports: Vec<String> = Vec::new();
        // Source files whose symbol list is fiction. Collected rather than acted
        // on inline: once ANY mapped file is conflicted, the spec-versus-source
        // comparison below is meaningless, so it is skipped entirely instead of
        // emitting a page of "documents X but no export found" noise derived
        // from a tree that does not exist.
        let mut conflicted_sources: Vec<String> = Vec::new();
        let unmerged = crate::merge::cached_unmerged_paths(root);
        for file in &fm.files {
            // Never extract exports from a path that escapes the project root — it
            // would leak arbitrary host-file identifiers. The files-exist check
            // above already reports such entries as errors.
            let full_path = root.join(file);
            // git's own unmerged list is authoritative and costs no heuristic: a
            // path in it IS a conflict, whatever the extractor made of the bytes.
            if unmerged
                .as_ref()
                .is_some_and(|paths| paths.contains(&file.replace('\\', "/")))
            {
                conflicted_sources.push(format!("{file} — git reports this path as unmerged"));
                continue;
            }
            let scan = if let Some(sources) = source_snapshots {
                let Some(SourceSnapshot::Present(bytes)) = sources.get(file) else {
                    continue;
                };
                let Ok(content) = std::str::from_utf8(bytes) else {
                    continue;
                };
                crate::exports::scan_exported_symbols_from_content(
                    &full_path,
                    content,
                    config.export_level,
                    config.parse_mode,
                )
            } else {
                if !source_within_root(root, file) {
                    continue;
                }
                crate::exports::scan_exported_symbols_full(
                    &full_path,
                    config.export_level,
                    config.parse_mode,
                )
            };
            let exports = match scan {
                crate::exports::ExportScan::Conflicted(evidence) => {
                    conflicted_sources.push(format!("{file} — {}", evidence.describe()));
                    continue;
                }
                crate::exports::ExportScan::Parsed(symbols) => symbols,
                crate::exports::ExportScan::UnknownLanguage
                | crate::exports::ExportScan::Unreadable
                | crate::exports::ExportScan::Directory => Vec::new(),
            };
            for sym in &exports {
                exports_by_file.push((sym.clone(), file.clone()));
            }
            all_exports.extend(exports);
        }

        if !conflicted_sources.is_empty() {
            for detail in &conflicted_sources {
                result
                    .errors
                    .push(format!("Unresolved merge conflict in source: {detail}"));
            }
            result.fixes.push(
                "Resolve the merge conflict in the listed source file(s) before re-running check"
                    .to_string(),
            );
        }

        // Deduplicate (keep first occurrence for file attribution)
        let mut seen = HashSet::new();
        all_exports.retain(|s| seen.insert(s.clone()));

        // A conflicted mapping makes the whole API surface unknown, so the
        // comparison is not attempted. Reporting `n/n exports documented` over a
        // partial set would be the same fail-open in a different costume.
        if conflicted_sources.is_empty() {
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

            // Code exports something not in spec = WARNING (with source file attribution)
            for sym in &all_exports {
                if !spec_set.contains(sym.as_str()) {
                    // Find the source file for this export
                    let source_file = exports_by_file
                        .iter()
                        .find(|(s, _)| s == sym)
                        .map(|(_, f)| f.as_str());
                    match source_file {
                        Some(file) => {
                            result
                                .warnings
                                .push(format!("Undocumented export '{sym}' from {file}"));
                        }
                        None => {
                            result
                                .warnings
                                .push(format!("Export '{sym}' not in spec (undocumented)"));
                        }
                    }
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
    }

    // ─── Level 3: Dependencies ────────────────────────────────────────

    if !fm.depends_on.is_empty() {
        for dep in &fm.depends_on {
            if is_cross_project_ref(dep) {
                // Cross-project refs (e.g. "owner/repo@module") are validated
                // by `specsync resolve`, not during local checks.
                continue;
            }
            // Bare module names (no path separators or extension) resolve
            // against the specs directory, not the project root.
            let full_path = if !dep.contains('/') && !dep.contains('.') {
                root.join(&config.specs_dir).join(dep)
            } else {
                root.join(dep)
            };
            if !full_path.exists() {
                result
                    .errors
                    .push(format!("Dependency spec not found: {dep}"));
            }
        }
    }

    // Check Consumed By section references
    if let Some(caps) = CONSUMED_BY_RE.captures(body) {
        let section = caps.get(1).unwrap().as_str();
        for caps in FILE_REF_RE.captures_iter(section) {
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

    // ─── Level 4: Requirements Companion File ─────────────────────────
    // spec-sync expects requirements in a separate companion file (requirements.md),
    // not inline in the spec body. Warn if requirements appear inline.
    // Drafts and review specs skip this check.
    if !is_draft && spec_status != Some(crate::types::SpecStatus::Review) {
        let has_inline_requirements = {
            let lower = body.to_ascii_lowercase();
            lower.contains("## requirements") || lower.contains("## acceptance criteria")
        };
        if has_inline_requirements {
            result.warnings.push(
                "Inline requirements detected — specs are technical contracts; user stories and acceptance criteria belong in a companion requirements.md file".to_string()
            );
            result.fixes.push(
                "Run `specsync add-spec <name>` to scaffold requirements.md, then move ## Requirements / ## Acceptance Criteria content there".to_string()
            );
        }

        // Canonical companions are adaptive in the 5.0 SDD model. Technical
        // and internal-only modules do not need an empty requirements file.
    }

    // ─── Custom Validation Rules ─────────────────────────────────────
    apply_custom_rules(spec_path, body, fm, config, &config_hint, &mut result);

    result
}

#[cfg(test)]
mod tests;
