use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process;

use crate::parser::{parse_checked_issue_references, parse_frontmatter};
use crate::validator::find_spec_files;

/// Result of merging a single spec file.
pub struct MergeResult {
    pub spec_path: String,
    pub status: MergeStatus,
    pub details: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeStatus {
    /// File had conflicts and they were resolved automatically.
    Resolved,
    /// File had conflicts that require manual intervention.
    Manual,
    /// File had no conflicts.
    Clean,
}

/// Detect and resolve git merge conflicts in spec files.
/// Returns a list of results — one per conflicted spec file.
pub fn merge_specs(
    root: &Path,
    specs_dir: &Path,
    dry_run: bool,
    all_files: bool,
) -> Vec<MergeResult> {
    let conflicted = if all_files {
        // Scan all spec files for conflict markers
        let spec_files = find_spec_files(specs_dir);
        spec_files
            .into_iter()
            .filter(|path| match fs::read_to_string(path) {
                Ok(content) => has_conflict_markers(&content),
                // Keep unreadable candidates so the main loop emits an explicit
                // Manual result instead of silently dropping them.
                Err(_) => true,
            })
            .collect::<Vec<_>>()
    } else {
        // Use git to find conflicted spec files
        detect_conflicted_specs(root, specs_dir)
    };

    let mut results = Vec::new();

    for spec_path in &conflicted {
        let content = match fs::read_to_string(spec_path) {
            Ok(c) => c,
            Err(e) => {
                results.push(MergeResult {
                    spec_path: rel_path(root, spec_path),
                    status: MergeStatus::Manual,
                    details: vec![format!("Cannot read file: {e}")],
                });
                continue;
            }
        };

        let (resolved, mut result, should_write) =
            resolve_spec_conflicts(&content, &rel_path(root, spec_path));

        // Preserve all-or-nothing writes: an ambiguous hunk leaves the complete
        // original file untouched, even when other hunks are auto-resolvable.
        if !dry_run && should_write {
            if let Err(e) = fs::write(spec_path, &resolved) {
                results.push(MergeResult {
                    spec_path: rel_path(root, spec_path),
                    status: MergeStatus::Manual,
                    details: vec![format!("Cannot write file: {e}")],
                });
                continue;
            }
            for detail in &mut result.details {
                if let Some(suffix) = detail.strip_prefix("Auto-resolvable") {
                    *detail = format!("Auto-resolved{suffix}");
                }
            }
        }

        results.push(result);
    }

    results
}

/// Check whether content contains git conflict markers.
pub fn has_conflict_markers(content: &str) -> bool {
    content.lines().any(is_conflict_marker_like)
}

/// Use `git status` to find spec files with merge conflicts.
fn detect_conflicted_specs(root: &Path, specs_dir: &Path) -> Vec<std::path::PathBuf> {
    let output = process::Command::new("git")
        .args(["diff", "--name-only", "--diff-filter=U"])
        .current_dir(root)
        .output();

    let output = match output {
        Ok(output) if output.status.success() => output,
        Err(_) => return Vec::new(),
        Ok(_) => return Vec::new(),
    };

    let specs_rel = specs_dir
        .strip_prefix(root)
        .unwrap_or(specs_dir)
        .to_string_lossy();

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| l.starts_with(specs_rel.as_ref()) && l.ends_with(".md"))
        .map(|l| root.join(l))
        .collect()
}

/// Resolve conflicts in a single spec file.
/// Returns (resolved_content, merge_result, should_write).
///
/// `should_write` is true only when every hunk was resolved and the output
/// passed the frontmatter safety net. Ambiguous files remain byte-for-byte
/// untouched.
fn resolve_spec_conflicts(content: &str, path: &str) -> (String, MergeResult, bool) {
    let mut details = Vec::new();
    let mut all_resolved = true;
    let mut auto_count = 0usize;
    let mut manual_count = 0usize;

    // Split the file into regions: clean text and conflict blocks
    let regions = parse_conflict_regions(content);

    let mut output = String::new();

    for (index, region) in regions.iter().enumerate() {
        match region {
            Region::Clean(text) => output.push_str(text),
            Region::Conflict {
                ours,
                theirs,
                marker_label,
                theirs_label,
                raw,
                well_formed,
            } => {
                // Determine what section this conflict is in. We are still inside
                // the leading frontmatter block until its closing `---` — i.e.
                // fewer than two fence lines have been emitted so far.
                let section = detect_section(&output);
                let in_frontmatter = output.lines().filter(|l| l.trim() == "---").count() < 2;

                let next_clean_starts_with_separator = regions
                    .get(index + 1)
                    .and_then(|region| match region {
                        Region::Clean(text) => Some(text),
                        Region::Conflict { .. } => None,
                    })
                    .is_some_and(|text| clean_region_starts_with_table_separator(text));

                let resolution = if !*well_formed {
                    Resolution::Manual("malformed or incomplete conflict markers")
                } else if !in_frontmatter && next_clean_starts_with_separator {
                    Resolution::Manual("conflict includes a table header")
                } else {
                    resolve_conflict(ours, theirs, &section, in_frontmatter)
                };

                match resolution {
                    Resolution::Auto(merged, strategy) => {
                        auto_count += 1;
                        details.push(format!(
                            "Auto-resolvable in {} ({} ↔ {}): {}",
                            section.as_deref().unwrap_or("unknown section"),
                            marker_label,
                            if theirs_label.is_empty() {
                                "incoming"
                            } else {
                                theirs_label
                            },
                            strategy
                        ));
                        output.push_str(&merged);
                    }
                    Resolution::Manual(reason) => {
                        manual_count += 1;
                        details.push(format!(
                            "Manual resolution needed in {} ({} ↔ {}): {}",
                            section.as_deref().unwrap_or("unknown section"),
                            marker_label,
                            if theirs_label.is_empty() {
                                "incoming"
                            } else {
                                theirs_label
                            },
                            reason
                        ));
                        all_resolved = false;
                        // Preserve the original conflict block verbatim (markers,
                        // diff3 base section and all) — nothing is lost.
                        output.push_str(raw);
                    }
                }
            }
        }
    }

    // Safety net for frontmatter validity: if the assembled output does not parse
    // as a complete spec frontmatter block, refuse to persist ANYTHING so the
    // caller leaves the ORIGINAL
    // file untouched. (This guards frontmatter structure; body/row preservation is
    // handled upstream by only auto-merging pure field/table hunks, never prose- or
    // section-carrying ones.)
    let resolved_frontmatter_ok = parse_frontmatter(&output)
        .map(|parsed| {
            parsed.frontmatter.module.is_some()
                && parsed.frontmatter.version.is_some()
                && parsed.frontmatter.status.is_some()
                && parsed.frontmatter.parsed_status().is_some()
                && !parsed.frontmatter.files.is_empty()
        })
        .unwrap_or(false)
        && parse_checked_issue_references(&output).is_ok();
    let mut should_write = all_resolved && auto_count > 0;
    if !output.is_empty() && !resolved_frontmatter_ok {
        all_resolved = false;
        should_write = false;
        details.push(
            "Resolved content would have invalid or empty frontmatter — left for \
             manual resolution; the original file was not modified"
                .to_string(),
        );
    }

    if manual_count > 0 && auto_count > 0 {
        details.push(format!(
            "{auto_count} hunk(s) are auto-resolvable, but {manual_count} hunk(s) need \
             manual resolution; the file was left unchanged (all-or-nothing)"
        ));
    }

    let status = if !all_resolved {
        MergeStatus::Manual
    } else if details.is_empty() {
        MergeStatus::Clean
    } else {
        MergeStatus::Resolved
    };

    let output = restore_input_line_endings(content, output);

    (
        output,
        MergeResult {
            spec_path: path.to_string(),
            status,
            details,
        },
        should_write,
    )
}

fn restore_input_line_endings(input: &str, mut output: String) -> String {
    if !input.ends_with('\n') && output.ends_with('\n') {
        output.pop();
    }

    let newline_count = input.bytes().filter(|byte| *byte == b'\n').count();
    let crlf_count = input.match_indices("\r\n").count();
    if newline_count > 0 && newline_count == crlf_count {
        output.replace('\n', "\r\n")
    } else {
        output
    }
}

enum Region {
    Clean(String),
    Conflict {
        ours: String,
        theirs: String,
        /// Label on the `<<<<<<<` marker (ours side, usually `HEAD`).
        marker_label: String,
        /// Label on the `>>>>>>>` marker (the incoming side).
        theirs_label: String,
        /// The complete original conflict block, markers included — re-emitted
        /// verbatim when the hunk needs manual resolution so nothing (including
        /// diff3 base sections) is lost or rewritten.
        raw: String,
        /// Whether the block contained exactly one separator and a closing marker.
        well_formed: bool,
    },
}

/// Parse content into clean regions and conflict blocks.
///
/// Handles `merge.conflictStyle diff3` hunks: the `||||||| base` section is
/// captured (in `raw`) but excluded from `ours`/`theirs`, so auto-resolution
/// never leaks base content or the `|||||||` marker into the output.
fn parse_conflict_regions(content: &str) -> Vec<Region> {
    let mut regions = Vec::new();
    let mut clean_buf = String::new();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        if let Some(label) = conflict_opener_label(line) {
            // Flush clean buffer
            if !clean_buf.is_empty() {
                regions.push(Region::Clean(clean_buf.clone()));
                clean_buf.clear();
            }

            let marker_label = label.to_string();
            let mut raw = format!("{line}\n");
            let mut ours = String::new();
            let mut theirs = String::new();
            let mut theirs_label = String::new();
            let mut in_base = false;
            let mut in_theirs = false;
            let mut saw_base = false;
            let mut saw_separator = false;
            let mut saw_end = false;
            let mut malformed = false;
            let mut nested_depth = 0usize;

            for inner_line in lines.by_ref() {
                raw.push_str(inner_line);
                raw.push('\n');

                if conflict_opener_label(inner_line).is_some() {
                    malformed = true;
                    nested_depth += 1;
                    continue;
                }
                if nested_depth > 0 {
                    if conflict_closer_label(inner_line).is_some() {
                        nested_depth -= 1;
                    }
                    continue;
                }

                if is_diff3_base_marker(inner_line) {
                    if saw_base || saw_separator || in_theirs {
                        malformed = true;
                    }
                    saw_base = true;
                    in_base = true;
                } else if inner_line == "=======" {
                    if saw_separator {
                        malformed = true;
                    }
                    saw_separator = true;
                    in_base = false;
                    in_theirs = true;
                } else if let Some(label) = conflict_closer_label(inner_line) {
                    if !saw_separator {
                        malformed = true;
                    }
                    theirs_label = label.to_string();
                    saw_end = true;
                    break;
                } else if is_conflict_marker_like(inner_line) {
                    malformed = true;
                } else if in_theirs {
                    theirs.push_str(inner_line);
                    theirs.push('\n');
                } else if !in_base {
                    ours.push_str(inner_line);
                    ours.push('\n');
                }
            }

            regions.push(Region::Conflict {
                ours,
                theirs,
                marker_label,
                theirs_label,
                raw,
                well_formed: saw_separator && saw_end && !malformed,
            });
        } else if is_conflict_marker_like(line) {
            if !clean_buf.is_empty() {
                regions.push(Region::Clean(clean_buf.clone()));
                clean_buf.clear();
            }
            regions.push(Region::Conflict {
                ours: String::new(),
                theirs: String::new(),
                marker_label: "unavailable".to_string(),
                theirs_label: "unavailable".to_string(),
                raw: format!("{line}\n"),
                well_formed: false,
            });
        } else {
            clean_buf.push_str(line);
            clean_buf.push('\n');
        }
    }

    if !clean_buf.is_empty() {
        regions.push(Region::Clean(clean_buf));
    }

    regions
}

fn conflict_opener_label(line: &str) -> Option<&str> {
    line.strip_prefix("<<<<<<< ")
        .filter(|label| !label.is_empty() && !label.starts_with('<'))
}

fn conflict_closer_label(line: &str) -> Option<&str> {
    line.strip_prefix(">>>>>>> ")
        .filter(|label| !label.is_empty() && !label.starts_with('>'))
}

fn is_diff3_base_marker(line: &str) -> bool {
    line.strip_prefix("||||||| ")
        .is_some_and(|label| !label.is_empty() && !label.starts_with('|'))
}

fn is_conflict_marker_like(line: &str) -> bool {
    line.starts_with("<<<<<<<")
        || line.starts_with("|||||||")
        || line.starts_with("=======")
        || line.starts_with(">>>>>>>")
}

/// Detect which markdown section the cursor is currently in,
/// based on the content already emitted.
fn detect_section(content_so_far: &str) -> Option<String> {
    // Find the last ## heading
    content_so_far
        .lines()
        .rev()
        .find(|l| l.starts_with("## "))
        .map(|l| l.trim_start_matches("## ").trim().to_string())
}

enum Resolution {
    /// (merged content, human-readable description of WHICH side/strategy won
    /// — reported so the log never claims HEAD won when it didn't, #427)
    Auto(String, &'static str),
    Manual(&'static str),
}

/// Try to auto-resolve a conflict based on section context.
///
/// `in_frontmatter` is true only while the cursor is still inside the leading
/// `---…---` frontmatter block. A `None` section (no `## ` heading seen yet) can
/// mean either the frontmatter OR a pre-first-heading body region (a `# Title` /
/// intro); only the former may use the field resolver — routing intro prose to it
/// would silently drop bullet lists and non-winning text.
fn resolve_conflict(
    ours: &str,
    theirs: &str,
    section: &Option<String>,
    in_frontmatter: bool,
) -> Resolution {
    let section_name = section.as_deref().unwrap_or("");

    // Frontmatter fields — only when genuinely inside the frontmatter block.
    if section_name.is_empty() && in_frontmatter {
        return resolve_frontmatter_conflict(ours, theirs);
    }

    // Every remaining auto-merge requires the hunk to be PURE table rows. A hunk
    // that also carries prose/headings (e.g. a Change Log conflict that swallowed
    // the following section) is NOT pure, so the row merge — which keeps only
    // `|`-rows and would delete everything else — is refused and left for manual
    // resolution instead.
    let pure_rows = is_pure_table_rows(ours) && is_pure_table_rows(theirs);
    match section_name {
        _ if !pure_rows => Resolution::Manual("conflict contains prose or non-table content"),
        _ if contains_table_separator(ours) || contains_table_separator(theirs) => {
            Resolution::Manual("conflict includes a table header separator")
        }
        "Change Log" => resolve_changelog_conflict(ours, theirs),
        _ => resolve_table_conflict(ours, theirs),
    }
}

/// Check if text consists only of markdown table rows (lines starting with |).
fn is_pure_table_rows(text: &str) -> bool {
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .all(|l| l.trim_start().starts_with('|'))
}

fn contains_table_separator(text: &str) -> bool {
    text.lines().any(|line| is_table_separator_row(line.trim()))
}

fn clean_region_starts_with_table_separator(text: &str) -> bool {
    text.lines()
        .find(|line| !line.trim().is_empty())
        .is_some_and(|line| is_table_separator_row(line.trim()))
}

/// Merge changelog table rows by date (union, sorted chronologically).
fn resolve_changelog_conflict(ours: &str, theirs: &str) -> Resolution {
    let our_rows = parse_table_rows(ours);
    let their_rows = parse_table_rows(theirs);

    if our_rows.is_empty() && their_rows.is_empty() {
        return Resolution::Manual("no changelog rows could be parsed");
    }
    if our_rows.is_empty() || their_rows.is_empty() {
        return Resolution::Manual("one side deleted all changelog rows");
    }

    // Deduplicate by full row content, preserve chronological order
    let mut seen = HashSet::new();
    let mut all_rows: Vec<&str> = Vec::new();

    for row in our_rows.iter().chain(their_rows.iter()) {
        let normalized = row.trim();
        if seen.insert(normalized) {
            all_rows.push(row);
        }
    }

    // Sort by date (first cell) — dates in ISO format sort lexicographically
    all_rows.sort_by_key(|a| extract_first_cell(a));

    let merged = all_rows
        .iter()
        .map(|r| r.trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    Resolution::Auto(
        format!("{merged}\n"),
        "union of both sides, sorted chronologically",
    )
}

/// Merge generic table rows (union, deduplicated by first cell / key).
fn resolve_table_conflict(ours: &str, theirs: &str) -> Resolution {
    let our_rows = parse_table_rows(ours);
    let their_rows = parse_table_rows(theirs);

    if our_rows.is_empty() && their_rows.is_empty() {
        return Resolution::Manual("no table rows could be parsed");
    }
    if our_rows.is_empty() || their_rows.is_empty() {
        return Resolution::Manual("one side deleted all table rows");
    }

    // Union rows by first cell (e.g., symbol name), but never choose a winner
    // when both sides changed the same key differently. Such a choice would
    // silently discard one branch's API contract.
    let mut seen: HashMap<String, String> = HashMap::new();
    let mut order = Vec::new();

    for row in our_rows.iter().chain(their_rows.iter()) {
        let key = extract_first_cell(row);
        let normalized = row.trim_end();
        if let Some(existing) = seen.get(&key) {
            if existing != normalized {
                return Resolution::Manual("same table key has divergent row content");
            }
            continue;
        }
        order.push(key.clone());
        seen.insert(key, normalized.to_string());
    }

    let merged = order
        .iter()
        .filter_map(|k| seen.get(k))
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");

    Resolution::Auto(
        format!("{merged}\n"),
        "union of both sides; identical duplicate rows deduplicated",
    )
}

/// Whether a conflict-hunk side is ONLY interior frontmatter fields that
/// [`parse_yaml_fields`] can round-trip WITHOUT dropping anything.
///
/// It must mirror the parser's keep/drop rules exactly, or a hunk it accepts could
/// still lose content on merge:
/// - a `---` fence, `##` heading, table row, or prose line ⇒ reject (not a field);
/// - a `- item` list line is kept by the parser only while it follows a key with an
///   empty/`[]` value (a list key). A `- item` before any such key — its owning key
///   sits in the surrounding clean region — is silently dropped by the parser, so
///   reject it here and defer to manual resolution instead of losing list items.
///
/// The upshot: we only auto-merge conflicts whose fences stay in the clean regions
/// (the common "two branches bumped `version`" / "diverging `files:` list" case);
/// anything else is left for the human.
fn is_frontmatter_only(text: &str) -> bool {
    let mut under_list_key = false;
    text.lines().all(|line| {
        let t = line.trim();
        if t.is_empty() {
            return true;
        }
        if t.starts_with("- ") {
            // Only safe when an owning list key was opened earlier in THIS hunk.
            return under_list_key;
        }
        // Indented mappings are valid YAML extensions, but this deliberately
        // small resolver would flatten them into top-level fields. Leave them
        // untouched instead of changing their meaning.
        if line.len() != line.trim_start().len() {
            return false;
        }
        // `key: value` with a spaceless key (a `---` fence has no colon → rejected).
        match line.find(':') {
            Some(c) => {
                let key = line[..c].trim();
                if key.is_empty() || key.contains(' ') {
                    return false;
                }
                let value = line[c + 1..].trim();
                // Only a block-style empty value can own following `- item`
                // lines. An inline `[]` is already complete.
                under_list_key = value.is_empty();
                true
            }
            None => false,
        }
    })
}

/// Merge frontmatter YAML fields.
/// Lists (files, depends_on, db_tables) are unioned.
/// Numeric versions take the maximum value. Other scalar conflicts are
/// ambiguous and require manual resolution.
fn resolve_frontmatter_conflict(ours: &str, theirs: &str) -> Resolution {
    // Only auto-merge when BOTH sides are pure interior frontmatter fields (no
    // `---` fence, heading, or body in the hunk). A plain `git merge` of specs
    // whose content differs right after the frontmatter swallows the closing `---`
    // and body into the hunk; since this resolver rebuilds from parsed `key: value`
    // fields alone, reconstructing those fences is error-prone (dropped body,
    // doubled fences). So we don't try — such hunks are left for manual resolution.
    if !is_frontmatter_only(ours) || !is_frontmatter_only(theirs) {
        return Resolution::Manual("frontmatter hunk contains unsupported content");
    }

    // Parse both sides as YAML-like key-value pairs
    let our_fields = parse_yaml_fields(ours);
    let their_fields = parse_yaml_fields(theirs);

    if our_fields.is_empty() && their_fields.is_empty() {
        return Resolution::Manual("no frontmatter fields could be parsed");
    }
    if has_duplicate_yaml_keys(&our_fields) || has_duplicate_yaml_keys(&their_fields) {
        return Resolution::Manual("frontmatter contains duplicate keys");
    }

    let list_keys: HashSet<&str> = ["files", "db_tables", "depends_on"].into_iter().collect();

    let mut merged_lines = Vec::new();
    let mut handled = HashSet::new();

    // Process in order of our fields first, then any new fields from theirs
    let all_keys: Vec<String> = {
        let mut keys = Vec::new();
        for (k, _) in &our_fields {
            if !keys.contains(k) {
                keys.push(k.clone());
            }
        }
        for (k, _) in &their_fields {
            if !keys.contains(k) {
                keys.push(k.clone());
            }
        }
        keys
    };

    for key in &all_keys {
        if handled.contains(key.as_str()) {
            continue;
        }
        handled.insert(key.as_str());

        let our_val = our_fields.iter().find(|(k, _)| k == key).map(|(_, v)| v);
        let their_val = their_fields.iter().find(|(k, _)| k == key).map(|(_, v)| v);

        match (our_val, their_val) {
            (Some(YamlValue::List(a)), Some(YamlValue::List(b)))
                if list_keys.contains(key.as_str()) =>
            {
                // Union the lists
                let mut combined = Vec::new();
                for item in a.iter().chain(b.iter()) {
                    if !combined.contains(item) {
                        combined.push(item.clone());
                    }
                }
                combined.sort();
                if combined.is_empty() {
                    merged_lines.push(format!("{key}: []"));
                } else {
                    merged_lines.push(format!("{key}:"));
                    for item in &combined {
                        merged_lines.push(format!("  - {item}"));
                    }
                }
            }
            (Some(YamlValue::Scalar(a)), Some(YamlValue::Scalar(b))) if key == "version" => {
                // #427: version must never regress on merge — take max(), not
                // blindly "theirs" (which could be the older number).
                match (
                    parse_numeric_version_scalar(a),
                    parse_numeric_version_scalar(b),
                ) {
                    (Ok(x), Ok(y)) => {
                        let selected = match x.cmp(&y) {
                            std::cmp::Ordering::Greater => a,
                            std::cmp::Ordering::Less => b,
                            std::cmp::Ordering::Equal if a == b => a,
                            std::cmp::Ordering::Equal => {
                                return Resolution::Manual(
                                    "equal version values use different scalar syntax",
                                );
                            }
                        };
                        merged_lines.push(format!("{key}: {selected}"));
                    }
                    _ => {
                        return Resolution::Manual("version values are not both unsigned integers");
                    }
                }
            }
            (Some(YamlValue::Scalar(a)), Some(YamlValue::Scalar(b))) => {
                if a != b {
                    return Resolution::Manual("frontmatter scalar values differ");
                }
                merged_lines.push(format_yaml_field(key, &YamlValue::Scalar(a.clone())));
            }
            (Some(YamlValue::List(a)), Some(YamlValue::List(b))) => {
                if a != b {
                    return Resolution::Manual("unsupported frontmatter list values differ");
                }
                merged_lines.push(format_yaml_field(key, &YamlValue::List(a.clone())));
            }
            (Some(YamlValue::Null), Some(YamlValue::Null)) => {
                merged_lines.push(format_yaml_field(key, &YamlValue::Null));
            }
            (Some(_), Some(_)) => {
                return Resolution::Manual("frontmatter field types differ");
            }
            (None, Some(val @ YamlValue::List(_))) if list_keys.contains(key.as_str()) => {
                merged_lines.push(format_yaml_field(key, val));
            }
            (Some(val @ YamlValue::List(_)), None) if list_keys.contains(key.as_str()) => {
                merged_lines.push(format_yaml_field(key, val));
            }
            (None, Some(_)) | (Some(_), None) => {
                return Resolution::Manual("frontmatter field exists on only one side");
            }
            (None, None) => {}
        }
    }

    // Emit only the merged interior fields. The `---` fences come from the
    // surrounding clean regions (guaranteed: `is_frontmatter_only` rejects any hunk
    // that contains a fence), so we never reconstruct or double a delimiter.
    let body = merged_lines.join("\n");
    Resolution::Auto(
        format!("{body}\n"),
        "known lists unioned; version = max(both sides); equal scalars preserved",
    )
}

fn parse_numeric_version_scalar(raw: &str) -> Result<u64, ()> {
    let raw = raw.trim();
    let value = if let Some(quote) = raw
        .chars()
        .next()
        .filter(|character| *character == '\'' || *character == '"')
    {
        let remainder = &raw[quote.len_utf8()..];
        let close = remainder.find(quote).ok_or(())?;
        let suffix = &remainder[close + quote.len_utf8()..];
        if !suffix.trim().is_empty() && !suffix.trim_start().starts_with('#') {
            return Err(());
        }
        &remainder[..close]
    } else {
        let comment = raw.find(" #").unwrap_or(raw.len());
        &raw[..comment]
    };

    value.trim().parse::<u64>().map_err(|_| ())
}

#[derive(Clone, Debug)]
enum YamlValue {
    Scalar(String),
    List(Vec<String>),
    Null,
}

fn has_duplicate_yaml_keys(fields: &[(String, YamlValue)]) -> bool {
    let mut seen = HashSet::new();
    fields.iter().any(|(key, _)| !seen.insert(key))
}

/// Simple YAML field parser (handles our zero-dep YAML subset).
fn parse_yaml_fields(text: &str) -> Vec<(String, YamlValue)> {
    let mut fields = Vec::new();
    let mut current_key: Option<String> = None;
    let mut current_list: Vec<String> = Vec::new();

    for line in text.lines() {
        if let Some(stripped) = line.trim_start().strip_prefix("- ") {
            if current_key.is_some() {
                current_list.push(stripped.trim().to_string());
            }
            continue;
        }

        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim();
            if key.is_empty() || key.contains(' ') {
                continue;
            }

            // Flush previous
            if let Some(prev_key) = current_key.take() {
                let value = if current_list.is_empty() {
                    YamlValue::Null
                } else {
                    YamlValue::List(current_list.clone())
                };
                fields.push((prev_key, value));
                current_list.clear();
            }

            let value = line[colon_pos + 1..].trim();
            if value.is_empty() {
                current_key = Some(key.to_string());
                current_list.clear();
            } else if value == "[]" {
                fields.push((key.to_string(), YamlValue::List(Vec::new())));
            } else {
                fields.push((key.to_string(), YamlValue::Scalar(value.to_string())));
            }
        }
    }

    if let Some(prev_key) = current_key.take() {
        let value = if current_list.is_empty() {
            YamlValue::Null
        } else {
            YamlValue::List(current_list)
        };
        fields.push((prev_key, value));
    }

    fields
}

fn format_yaml_field(key: &str, value: &YamlValue) -> String {
    match value {
        YamlValue::Scalar(s) => format!("{key}: {s}"),
        YamlValue::List(items) if items.is_empty() => format!("{key}: []"),
        YamlValue::List(items) => {
            let mut lines = vec![format!("{key}:")];
            for item in items {
                lines.push(format!("  - {item}"));
            }
            lines.join("\n")
        }
        YamlValue::Null => format!("{key}:"),
    }
}

/// Parse markdown table data rows from text (skip header/separator).
fn parse_table_rows(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|l| {
            let t = l.trim();
            // Keep every `|`-row EXCEPT the GFM separator row. Match the separator
            // structurally (all cells are dashes/colons), not by a `| -` prefix —
            // otherwise a data row whose first cell legitimately starts with `-`
            // (a CLI flag like `| --debug |`, a negative number) is dropped.
            t.starts_with('|') && !is_table_separator_row(t)
        })
        .collect()
}

/// Whether a `|`-delimited row is a GFM header/body separator (`|---|:--:|`),
/// i.e. every cell contains only dashes, colons, and spaces (with at least one
/// dash). Data rows — even ones whose first cell starts with `-` — are not.
fn is_table_separator_row(row: &str) -> bool {
    let inner = row.trim();
    if !inner.contains('-') {
        return false;
    }
    inner.trim_matches('|').split('|').all(|cell| {
        let c = cell.trim();
        !c.is_empty() && c.chars().all(|ch| ch == '-' || ch == ':')
    })
}

/// Extract the first cell value from a markdown table row.
fn extract_first_cell(row: &str) -> String {
    let parts: Vec<&str> = row.split('|').collect();
    if parts.len() >= 2 {
        parts[1].trim().to_string()
    } else {
        String::new()
    }
}

fn rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

/// Print merge results to stdout (text format).
pub fn print_results(results: &[MergeResult], dry_run: bool) {
    if results.is_empty() {
        println!("{}", "No spec files with merge conflicts found.".green());
        return;
    }

    let mut resolved_count = 0;
    let mut manual_count = 0;

    for r in results {
        match r.status {
            MergeStatus::Resolved => {
                resolved_count += 1;
                let verb = if dry_run { "would resolve" } else { "resolved" };
                println!("  {} {} {}", "✓".green(), verb.green(), r.spec_path.bold());
            }
            MergeStatus::Manual => {
                manual_count += 1;
                println!(
                    "  {} {} {}",
                    "✗".red(),
                    "needs manual merge:".red(),
                    r.spec_path.bold()
                );
            }
            MergeStatus::Clean => {}
        }

        for detail in &r.details {
            println!("    {detail}");
        }
    }

    println!();
    if resolved_count > 0 {
        let verb = if dry_run {
            "can be auto-resolved"
        } else {
            "auto-resolved"
        };
        println!(
            "{} {} spec file(s) {verb}.",
            "Summary:".bold(),
            resolved_count
        );
    }
    if manual_count > 0 {
        println!(
            "{} {} spec file(s) need manual resolution.",
            "Summary:".bold(),
            manual_count
        );
    }
}

/// Format results as JSON.
pub fn results_to_json(results: &[MergeResult]) -> String {
    let items: Vec<String> = results
        .iter()
        .map(|r| {
            let status = match r.status {
                MergeStatus::Resolved => "resolved",
                MergeStatus::Manual => "manual",
                MergeStatus::Clean => "clean",
            };
            let details_json: Vec<String> = r
                .details
                .iter()
                .map(|d| format!("\"{}\"", d.replace('\"', "\\\"")))
                .collect();
            format!(
                "    {{\"path\": \"{}\", \"status\": \"{}\", \"details\": [{}]}}",
                r.spec_path.replace('\"', "\\\""),
                status,
                details_json.join(", ")
            )
        })
        .collect();

    format!("{{\n  \"results\": [\n{}\n  ]\n}}", items.join(",\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_conflict_markers() {
        assert!(has_conflict_markers(
            "some text\n<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>> branch\n"
        ));
        assert!(!has_conflict_markers("clean file\nno conflicts\n"));
    }

    #[test]
    fn test_parse_conflict_regions() {
        let content =
            "before\n<<<<<<< HEAD\nours line\n=======\ntheirs line\n>>>>>>> branch\nafter\n";
        let regions = parse_conflict_regions(content);
        assert_eq!(regions.len(), 3);
        match &regions[0] {
            Region::Clean(s) => assert_eq!(s, "before\n"),
            _ => panic!("expected Clean"),
        }
        match &regions[1] {
            Region::Conflict {
                ours,
                theirs,
                marker_label,
                theirs_label,
                ..
            } => {
                assert_eq!(ours, "ours line\n");
                assert_eq!(theirs, "theirs line\n");
                assert_eq!(marker_label, "HEAD");
                assert_eq!(theirs_label, "branch");
            }
            _ => panic!("expected Conflict"),
        }
        match &regions[2] {
            Region::Clean(s) => assert_eq!(s, "after\n"),
            _ => panic!("expected Clean"),
        }
    }

    #[test]
    fn test_resolve_changelog_conflict() {
        let ours = "| 2026-01-01 | Added auth |\n| 2026-01-15 | Fixed login |\n";
        let theirs = "| 2026-01-01 | Added auth |\n| 2026-01-10 | Added signup |\n";

        match resolve_changelog_conflict(ours, theirs) {
            Resolution::Auto(merged, _) => {
                assert!(merged.contains("Added auth"));
                assert!(merged.contains("Fixed login"));
                assert!(merged.contains("Added signup"));
                // Check chronological order
                let lines: Vec<&str> = merged.lines().collect();
                assert_eq!(lines.len(), 3);
                assert!(lines[0].contains("2026-01-01"));
                assert!(lines[1].contains("2026-01-10"));
                assert!(lines[2].contains("2026-01-15"));
            }
            Resolution::Manual(reason) => panic!("expected auto resolution: {reason}"),
        }
    }

    #[test]
    fn test_resolve_table_conflict() {
        let ours = "| `createAuth` | config: Config | Auth | Creates auth |\n";
        let theirs = "| `validateToken` | token: string | bool | Validates |\n";

        match resolve_table_conflict(ours, theirs) {
            Resolution::Auto(merged, _) => {
                assert!(merged.contains("createAuth"));
                assert!(merged.contains("validateToken"));
            }
            Resolution::Manual(reason) => panic!("expected auto resolution: {reason}"),
        }
    }

    #[test]
    fn divergent_table_rows_require_manual_resolution() {
        let ours = "| `createAuth` | config: Config | Auth | Creates auth |\n";
        let theirs = "| `createAuth` | config: Config | Auth | Updated desc |\n";

        match resolve_table_conflict(ours, theirs) {
            Resolution::Manual(reason) => {
                assert!(reason.contains("divergent row content"), "{reason}");
            }
            Resolution::Auto(merged, _) => panic!("must not discard a row: {merged}"),
        }
    }

    #[test]
    fn test_resolve_frontmatter_conflict() {
        let ours =
            "module: auth\nversion: 2\nfiles:\n  - src/auth.ts\n  - src/login.ts\ndepends_on: []\n";
        let theirs = "module: auth\nversion: 3\nfiles:\n  - src/auth.ts\n  - src/signup.ts\ndepends_on: []\n";

        match resolve_frontmatter_conflict(ours, theirs) {
            Resolution::Auto(merged, _) => {
                // Theirs wins for scalar (version)
                assert!(merged.contains("version: 3"));
                // Lists are unioned
                assert!(merged.contains("src/auth.ts"));
                assert!(merged.contains("src/login.ts"));
                assert!(merged.contains("src/signup.ts"));
            }
            Resolution::Manual(reason) => panic!("expected auto resolution: {reason}"),
        }
    }

    #[test]
    fn test_full_spec_conflict_resolution() {
        let content = r#"---
<<<<<<< HEAD
module: auth
version: 2
status: active
files:
  - src/auth.ts
  - src/login.ts
db_tables: []
depends_on: []
=======
module: auth
version: 3
status: active
files:
  - src/auth.ts
  - src/signup.ts
db_tables: []
depends_on: []
>>>>>>> feature-branch
---

## Purpose

Auth module.

## Change Log

| Date | Change |
|------|--------|
<<<<<<< HEAD
| 2026-01-01 | Initial spec |
| 2026-01-15 | Added login |
=======
| 2026-01-01 | Initial spec |
| 2026-01-10 | Added signup |
>>>>>>> feature-branch
"#;

        let (resolved, result, _) = resolve_spec_conflicts(content, "specs/auth/auth.spec.md");
        assert!(matches!(result.status, MergeStatus::Resolved));
        assert!(!has_conflict_markers(&resolved));
        // Frontmatter: maximum version is 3, files are unioned
        assert!(resolved.contains("version: 3"));
        assert!(resolved.contains("src/login.ts"));
        assert!(resolved.contains("src/signup.ts"));
        // Changelog: all entries merged chronologically
        assert!(resolved.contains("Added login"));
        assert!(resolved.contains("Added signup"));
    }

    #[test]
    fn frontmatter_conflict_with_fences_in_hunk_falls_to_manual() {
        // Regression (CRITICAL): a conflict hunk that swallows the `---` fences was
        // the original corruption source (it resolved to loose `key: value` lines
        // with the delimiters dropped, or a doubled/empty fence, written as
        // "✓ resolved"). We no longer try to reconstruct fences — any hunk carrying
        // a `---` fence falls to Manual with the file left UNTOUCHED (no data loss).
        let content = "\
<<<<<<< HEAD
---
module: minimal
version: 2
status: active
files:
  - src/minimal.ts
depends_on: []
---
=======
---
module: minimal
version: 1
status: review
files:
  - src/minimal.ts
depends_on: []
---
>>>>>>> branch
# Minimal
";
        let (resolved, result, _) =
            resolve_spec_conflicts(content, "specs/minimal/minimal.spec.md");
        assert_eq!(
            result.status,
            MergeStatus::Manual,
            "a fence-carrying frontmatter hunk must not auto-resolve: {:?}",
            result.details
        );
        assert!(
            has_conflict_markers(&resolved),
            "conflict markers must be preserved for manual resolution"
        );
        // No data loss: both sides' fields survive verbatim in the preserved hunk.
        assert!(resolved.contains("version: 2"));
        assert!(resolved.contains("version: 1"));
        assert!(resolved.contains("# Minimal"));
    }

    #[test]
    fn frontmatter_conflict_with_separator_before_fence_falls_to_manual() {
        // Regression: a blank/whitespace line between the clean opening `---` and a
        // fence-swallowing hunk previously defeated the re-wrap + backstop and wrote
        // an empty-frontmatter spec as "✓ resolved". The hunk carries a `---`, so it
        // must fall to Manual — file untouched, fields preserved.
        let content = "\
---

<<<<<<< HEAD
module: a
version: 2
---
=======
module: a
version: 3
---
>>>>>>> feature
# Body
";
        let (resolved, result, _) = resolve_spec_conflicts(content, "specs/a/a.spec.md");
        assert_eq!(
            result.status,
            MergeStatus::Manual,
            "separator-before-fence hunk must fall to Manual: {:?}",
            result.details
        );
        assert!(has_conflict_markers(&resolved));
        assert!(resolved.contains("version: 2"));
        assert!(resolved.contains("version: 3"));
        assert!(resolved.contains("# Body"));
    }

    #[test]
    fn frontmatter_conflict_swallowing_body_falls_to_manual() {
        // Regression (CRITICAL): a plain `git merge` of two specs whose content
        // differs right after the frontmatter yields a hunk that spans the closing
        // `---` AND the body. The field-only resolver would silently DELETE that
        // body; it must instead fall to Manual — markers preserved, body intact.
        let content = "\
---
<<<<<<< HEAD
module: alpha
version: 2
status: active
files:
  - src/a.ts
db_tables: []
depends_on: []
---
# Alpha

## Purpose

Ours purpose.
=======
module: alpha
version: 3
status: active
files:
  - src/a.ts
db_tables: []
depends_on: []
---
# Alpha

## Purpose

Theirs purpose.
>>>>>>> feature
";
        let (resolved, result, _) = resolve_spec_conflicts(content, "specs/alpha/alpha.spec.md");
        assert_eq!(
            result.status,
            MergeStatus::Manual,
            "a body-swallowing frontmatter conflict must NOT auto-resolve: {:?}",
            result.details
        );
        assert!(
            has_conflict_markers(&resolved),
            "conflict markers must be preserved for manual resolution"
        );
        assert!(
            resolved.contains("## Purpose"),
            "spec body must not be deleted, got:\n{resolved}"
        );
        assert!(resolved.contains("Ours purpose."));
        assert!(resolved.contains("Theirs purpose."));
    }

    #[test]
    fn pre_heading_body_field_hunk_falls_to_manual() {
        // Regression: a conflict in a pre-first-`##` intro (a `# Notes` region,
        // AFTER the frontmatter) has no `## ` heading, so it used to route to the
        // frontmatter field resolver, which drops bullet lists and non-winning
        // prose. It must fall to Manual — bullets and both sides preserved.
        let content = "\
---
module: m
version: 1
status: active
files:
  - src/m.ts
db_tables: []
depends_on: []
---
# Notes

<<<<<<< HEAD
TODO: fix the auth flow
- step one
- step two
=======
TODO: rewrite the auth flow
- step three
>>>>>>> feature

## Purpose

The module.
";
        let (resolved, result, _) = resolve_spec_conflicts(content, "specs/m/m.spec.md");
        assert_eq!(
            result.status,
            MergeStatus::Manual,
            "pre-heading body hunk must not be field-merged: {:?}",
            result.details
        );
        assert!(has_conflict_markers(&resolved));
        assert!(resolved.contains("- step one"), "bullets must survive");
        assert!(resolved.contains("- step two"));
        assert!(resolved.contains("- step three"));
    }

    #[test]
    fn frontmatter_orphan_list_items_hunk_falls_to_manual() {
        // Regression: a real `git merge` of two branches that both extended `files:`
        // (and another field) yields a hunk that STARTS with orphan `- item` lines
        // (their `files:` key is in the clean region) then continues into
        // `db_tables:`. The parser drops the orphan items while the trailing field
        // keeps the result non-empty — silently losing both branches' new files.
        // Must fall to Manual with the items preserved.
        let content = "\
---
module: m
version: 1
status: active
files:
  - src/a.ts
<<<<<<< HEAD
  - src/b.ts
db_tables:
  - users
=======
  - src/c.ts
db_tables:
  - accounts
>>>>>>> theirs
depends_on: []
---
# M
";
        let (resolved, result, _) = resolve_spec_conflicts(content, "specs/m/m.spec.md");
        assert_eq!(
            result.status,
            MergeStatus::Manual,
            "orphan leading list items must not be field-merged: {:?}",
            result.details
        );
        assert!(has_conflict_markers(&resolved));
        assert!(
            resolved.contains("- src/b.ts") && resolved.contains("- src/c.ts"),
            "list items from both sides must survive:\n{resolved}"
        );
    }

    #[test]
    fn changelog_hunk_swallowing_next_section_falls_to_manual() {
        // Regression: a Change Log conflict whose hunk runs past the table into the
        // following section previously deleted that section (the row resolver keeps
        // only `|`-rows). Impure changelog hunks now fall to Manual.
        let content = "\
---
module: m
version: 1
status: active
files:
  - src/m.ts
db_tables: []
depends_on: []
---
# M

## Change Log

| Date | Change |
|------|--------|
<<<<<<< HEAD
| 2026-01-01 | a |

## Migration Notes

Run the migration script before deploying.
=======
| 2026-01-02 | b |

## Migration Notes

Run it after.
>>>>>>> feature
";
        let (resolved, result, _) = resolve_spec_conflicts(content, "specs/m/m.spec.md");
        assert_eq!(
            result.status,
            MergeStatus::Manual,
            "impure changelog hunk must not be row-merged: {:?}",
            result.details
        );
        assert!(has_conflict_markers(&resolved));
        assert!(
            resolved.contains("## Migration Notes") && resolved.contains("migration script"),
            "the swallowed following section must not be deleted:\n{resolved}"
        );
    }

    #[test]
    fn separator_row_detection_excludes_dash_leading_data_rows() {
        assert!(is_table_separator_row("|------|------|"));
        assert!(is_table_separator_row("| :--- | ---: |"));
        assert!(!is_table_separator_row("| --debug | Enable debug |"));
        assert!(!is_table_separator_row("| Flag | Description |"));
        assert!(!is_table_separator_row("| -5 | a negative number |"));
    }

    #[test]
    fn table_conflict_preserves_dash_leading_row() {
        // Regression: a generic table hunk mixing a normal row with a `--flag` row
        // used to DROP the flag row (misread as a `|---|` separator) and write the
        // result as Resolved — content loss. The flag rows must survive the merge.
        let content = "\
---
module: m
version: 1
status: active
files:
  - src/m.ts
db_tables: []
depends_on: []
---
# M

## Flags

| Flag | Description |
|------|-------------|
<<<<<<< HEAD
| newFlag | A new flag |
| --debug | Enable debug |
=======
| newFlag | A new flag |
| --quiet | Quiet mode |
>>>>>>> feature
";
        let (resolved, _result, _) = resolve_spec_conflicts(content, "specs/m/m.spec.md");
        assert!(
            !has_conflict_markers(&resolved),
            "should auto-resolve:\n{resolved}"
        );
        assert!(
            resolved.contains("--debug") && resolved.contains("--quiet"),
            "dash-leading rows must not be dropped:\n{resolved}"
        );
        assert!(resolved.contains("newFlag"));
    }

    #[test]
    fn test_manual_fallback_for_prose() {
        let content = "## Purpose\n\n<<<<<<< HEAD\nThis is our purpose description.\n=======\nThis is their different purpose.\n>>>>>>> branch\n";
        let (resolved, result, _) = resolve_spec_conflicts(content, "test.spec.md");
        // Prose conflicts should remain for manual resolution
        assert!(matches!(result.status, MergeStatus::Manual));
        assert!(has_conflict_markers(&resolved));
    }

    #[test]
    fn test_parse_yaml_fields() {
        let yaml = "module: auth\nversion: 1\nfiles:\n  - src/a.ts\n  - src/b.ts\ndb_tables: []\n";
        let fields = parse_yaml_fields(yaml);
        assert_eq!(fields.len(), 4);
        assert!(matches!(&fields[0], (k, YamlValue::Scalar(v)) if k == "module" && v == "auth"));
        assert!(matches!(&fields[2], (k, YamlValue::List(v)) if k == "files" && v.len() == 2));
    }

    #[test]
    fn test_is_pure_table_rows() {
        assert!(is_pure_table_rows("| a | b |\n| c | d |\n"));
        assert!(!is_pure_table_rows("some text\n| a | b |\n"));
        assert!(is_pure_table_rows("| a | b |\n\n| c | d |\n"));
    }

    // ── #427 regressions ────────────────────────────────────────────

    const OPEN_HEAD: &str = concat!("<<<<", "<<< HEAD\n");
    const OPEN_OUTER: &str = concat!("<<<<", "<<< outer-ours\n");
    const OPEN_INNER: &str = concat!("<<<<", "<<< inner-ours\n");
    const BASE_MARKER: &str = concat!("||||", "||| base\n");
    const SEPARATOR: &str = concat!("===", "====\n");
    const CLOSE_SIDE: &str = concat!(">>>>", ">>> side\n");
    const CLOSE_OUTER: &str = concat!(">>>>", ">>> outer-incoming\n");
    const CLOSE_INNER: &str = concat!(">>>>", ">>> inner-incoming\n");

    #[test]
    fn divergent_api_row_detail_names_both_sides_and_requires_manual_resolution() {
        // #427: never claim HEAD won while selecting incoming content. Divergent
        // rows are ambiguous, so neither side may be selected.
        let content = format!(
            "---\nmodule: m\nversion: 1\n---\n# M\n\n## Public API\n\n\
             | Name | Description |\n|------|-------------|\n\
             {OPEN_HEAD}| `a` | MAIN description. |\n\
             {SEPARATOR}| `a` | SIDE description. |\n{CLOSE_SIDE}"
        );
        let (resolved, result, should_write) =
            resolve_spec_conflicts(&content, "specs/m/m.spec.md");
        assert_eq!(result.status, MergeStatus::Manual);
        assert!(!should_write);
        let detail = &result.details[0];
        assert!(detail.contains("HEAD"), "{detail}");
        assert!(detail.contains("side"), "{detail}");
        assert!(detail.contains("divergent row content"), "{detail}");
        assert!(resolved.contains("MAIN description."), "{resolved}");
        assert!(resolved.contains("SIDE description."), "{resolved}");
        assert!(has_conflict_markers(&resolved));
    }

    #[test]
    fn lossless_table_union_detail_names_both_sides_and_strategy() {
        let content = concat!(
            "---\nmodule: m\nversion: 1\nstatus: stable\nfiles:\n  - src/m.rs\n",
            "---\n# M\n\n## Public API\n\n",
            "| Name | Description |\n|------|-------------|\n",
            "<<<<",
            "<<< HEAD\n| `a` | MAIN description. |\n",
            "===",
            "====\n| `b` | SIDE description. |\n",
            ">>>>",
            ">>> side\n",
        );
        let (resolved, result, should_write) = resolve_spec_conflicts(content, "specs/m/m.spec.md");
        assert_eq!(result.status, MergeStatus::Resolved);
        assert!(should_write);
        assert!(resolved.contains("MAIN description."), "{resolved}");
        assert!(resolved.contains("SIDE description."), "{resolved}");
        let detail = &result.details[0];
        assert!(detail.contains("HEAD"), "{detail}");
        assert!(detail.contains("side"), "{detail}");
        assert!(detail.contains("union of both sides"), "{detail}");
        assert!(!detail.contains("wins"), "{detail}");
    }

    #[test]
    fn frontmatter_version_takes_max_not_incoming() {
        // #427: version 3 on HEAD regressing to 2 from the incoming side.
        let ours = "module: auth\nversion: 3\n";
        let theirs = "module: auth\nversion: 2\n";
        match resolve_frontmatter_conflict(ours, theirs) {
            Resolution::Auto(merged, _) => {
                assert!(merged.contains("version: 3"), "{merged}");
                assert!(!merged.contains("version: 2"));
            }
            Resolution::Manual(reason) => panic!("expected auto resolution: {reason}"),
        }
    }

    #[test]
    fn nonnumeric_version_requires_manual_resolution() {
        let ours = "module: auth\nversion: next\n";
        let theirs = "module: auth\nversion: 3\n";
        match resolve_frontmatter_conflict(ours, theirs) {
            Resolution::Manual(reason) => {
                assert!(reason.contains("unsigned integers"), "{reason}");
            }
            Resolution::Auto(merged, _) => panic!("must not choose a version: {merged}"),
        }
    }

    #[test]
    fn numeric_versions_accept_supported_yaml_scalar_syntax_and_preserve_the_max_side() {
        let ours = "module: auth\nversion: '3' # current\nstatus: stable\n";
        let theirs = "module: auth\nversion: \"2\" # incoming\nstatus: stable\n";
        match resolve_frontmatter_conflict(ours, theirs) {
            Resolution::Auto(merged, _) => {
                assert!(merged.contains("version: '3' # current"), "{merged}");
                assert!(!merged.contains("\"2\""), "{merged}");
            }
            Resolution::Manual(reason) => panic!("expected auto resolution: {reason}"),
        }

        assert_eq!(parse_numeric_version_scalar("0"), Ok(0));
        assert_eq!(
            parse_numeric_version_scalar(&u64::MAX.to_string()),
            Ok(u64::MAX)
        );
        assert!(parse_numeric_version_scalar("-1").is_err());
        assert!(parse_numeric_version_scalar("1.2.3").is_err());
        assert!(parse_numeric_version_scalar("18446744073709551616").is_err());
    }

    #[test]
    fn equal_versions_with_different_scalar_syntax_require_manual_resolution() {
        let ours = "module: auth\nversion: 3\n";
        let theirs = "module: auth\nversion: '3'\n";
        assert!(matches!(
            resolve_frontmatter_conflict(ours, theirs),
            Resolution::Manual("equal version values use different scalar syntax")
        ));
    }

    #[test]
    fn divergent_frontmatter_scalar_requires_manual_resolution() {
        let ours = "module: auth\nversion: 3\nstatus: stable\n";
        let theirs = "module: auth\nversion: 3\nstatus: deprecated\n";
        match resolve_frontmatter_conflict(ours, theirs) {
            Resolution::Manual(reason) => {
                assert!(reason.contains("scalar values differ"), "{reason}");
            }
            Resolution::Auto(merged, _) => panic!("must not choose a scalar: {merged}"),
        }
    }

    #[test]
    fn one_sided_scalar_fields_require_manual_resolution() {
        for (ours, theirs) in [
            (
                "module: auth\nversion: 3\nstatus: stable\n",
                "module: auth\nversion: 3\n",
            ),
            (
                "module: auth\nversion: 3\n",
                "module: auth\nversion: 3\nstatus: stable\n",
            ),
            (
                "module: auth\nversion: 3\nstatus: stable\n",
                "module: auth\nstatus: stable\n",
            ),
            (
                "module: auth\nversion: 3\nowner: team\n",
                "module: auth\nversion: 3\n",
            ),
        ] {
            assert!(matches!(
                resolve_frontmatter_conflict(ours, theirs),
                Resolution::Manual("frontmatter field exists on only one side")
            ));
        }
    }

    #[test]
    fn supported_frontmatter_lists_union_and_sort_every_key() {
        let ours = "\
module: auth
version: 3
status: stable
files:
  - src/z.rs
db_tables:
  - z_table
depends_on:
  - specs/z/z.spec.md
";
        let theirs = "\
module: auth
version: 3
status: stable
files:
  - src/a.rs
db_tables:
  - a_table
depends_on:
  - specs/a/a.spec.md
";
        match resolve_frontmatter_conflict(ours, theirs) {
            Resolution::Auto(merged, _) => {
                assert!(
                    merged.contains("files:\n  - src/a.rs\n  - src/z.rs"),
                    "{merged}"
                );
                assert!(
                    merged.contains("db_tables:\n  - a_table\n  - z_table"),
                    "{merged}"
                );
                assert!(
                    merged.contains("depends_on:\n  - specs/a/a.spec.md\n  - specs/z/z.spec.md"),
                    "{merged}"
                );
            }
            Resolution::Manual(reason) => panic!("expected auto resolution: {reason}"),
        }
    }

    #[test]
    fn diff3_base_markers_do_not_leak_into_output() {
        // #427: with merge.conflictStyle diff3, `|||||||` base content must not
        // survive in an auto-resolved file.
        let content = format!(
            "---\nmodule: m\nversion: 1\nstatus: stable\nfiles:\n  - src/m.rs\n\
             ---\n# M\n\n## Public API\n\n\
             | Name | Description |\n|------|-------------|\n\
             {OPEN_HEAD}| `a` | MAIN description. |\n\
             {BASE_MARKER}| `base` | BASE description. |\n\
             {SEPARATOR}| `b` | SIDE description. |\n{CLOSE_SIDE}"
        );
        let (resolved, result, _) = resolve_spec_conflicts(&content, "specs/m/m.spec.md");
        assert_eq!(result.status, MergeStatus::Resolved);
        assert!(!resolved.contains("|||||||"), "{resolved}");
        assert!(!resolved.contains("BASE description"), "{resolved}");
        assert!(resolved.contains("MAIN description"), "{resolved}");
        assert!(resolved.contains("SIDE description"), "{resolved}");
    }

    #[test]
    fn manual_hunk_preserves_diff3_block_verbatim() {
        let content = format!(
            "## Purpose\n\n{OPEN_HEAD}Our purpose.\n{BASE_MARKER}Base purpose.\n\
             {SEPARATOR}Their purpose.\n{CLOSE_SIDE}"
        );
        let (resolved, result, should_write) = resolve_spec_conflicts(&content, "test.spec.md");
        assert_eq!(result.status, MergeStatus::Manual);
        assert!(!should_write, "nothing auto-resolved — do not write");
        assert!(resolved.contains("||||||| base"), "{resolved}");
        assert!(resolved.contains("Base purpose."), "{resolved}");
    }

    #[test]
    fn malformed_conflict_block_is_never_auto_resolved() {
        let content = concat!(
            "---\nmodule: m\nversion: 1\n---\n# M\n\n## Public API\n\n",
            "<<<<",
            "<<< HEAD\n| `a` | MAIN description. |\n",
            "===",
            "====\n| `b` | SIDE description. |\n",
        );
        let (resolved, result, should_write) = resolve_spec_conflicts(content, "specs/m/m.spec.md");
        assert_eq!(result.status, MergeStatus::Manual);
        assert!(!should_write);
        assert!(resolved.contains("<<<<<<< HEAD"), "{resolved}");
        assert!(
            result
                .details
                .iter()
                .any(|detail| detail.contains("malformed or incomplete")),
            "{:?}",
            result.details
        );
    }

    #[test]
    fn diff3_marker_requires_exact_seven_pipes_and_label_separator() {
        let malformed_base = concat!("||||", "|||| base\n");
        let content = format!(
            "---\nmodule: m\nversion: 1\nstatus: stable\nfiles:\n  - src/m.rs\n---\n\
             # M\n\n## Public API\n\n| Name | Description |\n|------|-------------|\n\
             {OPEN_HEAD}| `a` | ours |\n{malformed_base}| discarded | must survive |\n\
             {SEPARATOR}| `b` | incoming |\n{CLOSE_SIDE}"
        );
        let (resolved, result, should_write) =
            resolve_spec_conflicts(&content, "specs/m/m.spec.md");
        assert_eq!(result.status, MergeStatus::Manual);
        assert!(!should_write);
        assert_eq!(resolved, content);
    }

    #[test]
    fn orphan_marker_families_after_a_valid_hunk_block_every_write() {
        for orphan in [
            SEPARATOR,
            BASE_MARKER,
            CLOSE_SIDE,
            concat!("||||", "|||| base\n"),
        ] {
            let content = format!(
                "---\nmodule: m\nversion: 1\nstatus: stable\nfiles:\n  - src/m.rs\n---\n\
                 # M\n\n## Public API\n\n| Name | Description |\n|------|-------------|\n\
                 {OPEN_HEAD}| `a` | ours |\n\
                 {SEPARATOR}| `b` | incoming |\n{CLOSE_SIDE}{orphan}"
            );
            let (resolved, result, should_write) =
                resolve_spec_conflicts(&content, "specs/m/m.spec.md");
            assert_eq!(result.status, MergeStatus::Manual, "orphan {orphan:?}");
            assert!(!should_write, "orphan {orphan:?}");
            assert!(resolved.contains(orphan.trim_end()), "orphan {orphan:?}");

            let tmp = tempfile::tempdir().unwrap();
            let specs = tmp.path().join("specs");
            fs::create_dir_all(&specs).unwrap();
            let spec = specs.join("m.spec.md");
            fs::write(&spec, &content).unwrap();
            let results = merge_specs(tmp.path(), &specs, false, true);
            assert_eq!(results[0].status, MergeStatus::Manual);
            assert_eq!(fs::read_to_string(&spec).unwrap(), content);
        }
    }

    #[test]
    fn nested_marker_diagnostics_keep_the_outer_side_labels() {
        let content = format!(
            "## Purpose\n\n{OPEN_OUTER}ours\n{OPEN_INNER}inner ours\n\
             {SEPARATOR}inner incoming\n{CLOSE_INNER}outer ours tail\n\
             {SEPARATOR}outer incoming\n{CLOSE_OUTER}"
        );
        let (_resolved, result, should_write) = resolve_spec_conflicts(&content, "test.spec.md");
        assert_eq!(result.status, MergeStatus::Manual);
        assert!(!should_write);
        let detail = &result.details[0];
        assert!(detail.contains("outer-ours ↔ outer-incoming"), "{detail}");
        assert!(!detail.contains("inner-incoming"), "{detail}");
    }

    #[test]
    fn table_hunks_containing_headers_or_separators_remain_manual() {
        for section in ["Public API", "Change Log"] {
            let content = format!(
                "---\nmodule: m\nversion: 1\nstatus: stable\nfiles:\n  - src/m.rs\n---\n\
                 # M\n\n## {section}\n\n{OPEN_HEAD}| Name | Description |\n\
                 |------|-------------|\n| `a` | ours |\n\
                 {SEPARATOR}| Name | Description |\n|------|-------------|\n\
                 | `b` | incoming |\n{CLOSE_SIDE}"
            );
            let (resolved, result, should_write) =
                resolve_spec_conflicts(&content, "specs/m/m.spec.md");
            assert_eq!(result.status, MergeStatus::Manual, "{section}");
            assert!(!should_write, "{section}");
            assert_eq!(resolved, content, "{section}");
            assert!(
                result
                    .details
                    .iter()
                    .any(|detail| detail.contains("table header separator")),
                "{:?}",
                result.details
            );
        }
    }

    #[test]
    fn table_header_only_hunks_remain_manual_and_byte_identical() {
        for section in ["Public API", "Change Log"] {
            let tmp = tempfile::tempdir().unwrap();
            let specs = tmp.path().join("specs/m");
            fs::create_dir_all(&specs).unwrap();
            let spec = specs.join("m.spec.md");
            let content = format!(
                "---\nmodule: m\nversion: 1\nstatus: stable\nfiles:\n  - src/m.rs\n---\n\
                 # M\n\n## {section}\n\n\
                 {OPEN_HEAD}| Name | Description |\n\
                 {SEPARATOR}| Symbol | Summary |\n\
                 {CLOSE_SIDE}|------|-------------|\n| `a` | body row |\n"
            );
            fs::write(&spec, &content).unwrap();

            let results = merge_specs(tmp.path(), &tmp.path().join("specs"), false, true);
            assert_eq!(results.len(), 1, "{section}");
            assert_eq!(results[0].status, MergeStatus::Manual, "{section}");
            assert!(
                results[0]
                    .details
                    .iter()
                    .any(|detail| detail.contains("table header")),
                "{:?}",
                results[0].details
            );
            assert_eq!(fs::read_to_string(&spec).unwrap(), content, "{section}");
        }
    }

    #[test]
    fn unknown_empty_scalar_and_empty_list_are_not_equivalent() {
        for (ours, theirs) in [("metadata:", "metadata: []"), ("metadata: []", "metadata:")] {
            let tmp = tempfile::tempdir().unwrap();
            let specs = tmp.path().join("specs/m");
            fs::create_dir_all(&specs).unwrap();
            let spec = specs.join("m.spec.md");
            let content = format!(
                "---\nmodule: m\nversion: 1\nstatus: stable\nfiles:\n  - src/m.rs\n\
                 {OPEN_HEAD}{ours}\n\
                 {SEPARATOR}{theirs}\n\
                 {CLOSE_SIDE}---\n# M\n"
            );
            fs::write(&spec, &content).unwrap();

            let results = merge_specs(tmp.path(), &tmp.path().join("specs"), false, true);
            assert_eq!(results.len(), 1, "{ours:?} vs {theirs:?}");
            assert_eq!(
                results[0].status,
                MergeStatus::Manual,
                "{ours:?} vs {theirs:?}"
            );
            assert_eq!(
                fs::read_to_string(&spec).unwrap(),
                content,
                "{ours:?} vs {theirs:?}"
            );
        }
    }

    #[test]
    fn missing_frontmatter_prevents_resolvable_body_write() {
        let tmp = tempfile::tempdir().unwrap();
        let specs = tmp.path().join("specs/m");
        fs::create_dir_all(&specs).unwrap();
        let spec = specs.join("m.spec.md");
        let content = format!(
            "# M\n\n## Public API\n\n| Name | Description |\n|------|-------------|\n\
             {OPEN_HEAD}| `a` | ours |\n\
             {SEPARATOR}| `b` | incoming |\n{CLOSE_SIDE}"
        );
        fs::write(&spec, &content).unwrap();

        let results = merge_specs(tmp.path(), &tmp.path().join("specs"), false, true);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, MergeStatus::Manual);
        assert!(
            results[0]
                .details
                .iter()
                .any(|detail| detail.contains("invalid or empty frontmatter")),
            "{:?}",
            results[0].details
        );
        assert_eq!(fs::read_to_string(&spec).unwrap(), content);
    }

    #[test]
    fn nested_frontmatter_mapping_is_not_flattened() {
        let content = format!(
            "---\nmodule: m\nversion: 1\nstatus: stable\nfiles:\n  - src/m.rs\n\
             {OPEN_HEAD}metadata:\n  owner: team-a\n\
             {SEPARATOR}metadata:\n  owner: team-b\n{CLOSE_SIDE}---\n# M\n"
        );
        let (resolved, result, should_write) =
            resolve_spec_conflicts(&content, "specs/m/m.spec.md");
        assert_eq!(result.status, MergeStatus::Manual);
        assert!(!should_write);
        assert_eq!(resolved, content);
    }

    #[test]
    fn duplicate_or_invalid_frontmatter_suppresses_otherwise_resolvable_writes() {
        for frontmatter in [
            "module: m\nmodule: duplicate\nversion: 1\nstatus: stable\nfiles:\n  - src/m.rs\n",
            "module: m\nstatus: stable\nfiles:\n  - src/m.rs\n",
            "module: m\nversion: 1\nfiles:\n  - src/m.rs\n",
            "module: m\nversion: 1\nstatus: impossible\nfiles:\n  - src/m.rs\n",
            "module: m\nversion: 1\nstatus: stable\nfiles: []\n",
        ] {
            let content = format!(
                "---\n{frontmatter}---\n# M\n\n## Public API\n\n\
                 | Name | Description |\n|------|-------------|\n\
                 {OPEN_HEAD}| `a` | ours |\n\
                 {SEPARATOR}| `b` | incoming |\n{CLOSE_SIDE}"
            );
            let (_resolved, result, should_write) =
                resolve_spec_conflicts(&content, "specs/m/m.spec.md");
            assert_eq!(result.status, MergeStatus::Manual, "{frontmatter}");
            assert!(!should_write, "{frontmatter}");
            assert!(
                result
                    .details
                    .iter()
                    .any(|detail| detail.contains("invalid or empty frontmatter")),
                "{:?}",
                result.details
            );
        }
    }

    #[test]
    fn resolved_diff3_file_preserves_crlf_line_endings() {
        let content = concat!(
            "---\r\nmodule: m\r\nversion: 1\r\nstatus: stable\r\n",
            "files:\r\n  - src/m.rs\r\n---\r\n# M\r\n\r\n",
            "## Public API\r\n\r\n| Name | Description |\r\n|------|-------------|\r\n",
            "<<<<",
            "<<< HEAD\r\n| `a` | MAIN description. |\r\n",
            "||||",
            "||| base\r\n| `base` | BASE description. |\r\n",
            "===",
            "====\r\n| `b` | SIDE description. |\r\n",
            ">>>>",
            ">>> side\r\n",
        );
        let (resolved, result, should_write) = resolve_spec_conflicts(content, "specs/m/m.spec.md");
        assert_eq!(result.status, MergeStatus::Resolved);
        assert!(should_write);
        assert!(!resolved.replace("\r\n", "").contains('\n'), "{resolved:?}");
        assert!(resolved.ends_with("\r\n"));
    }

    #[test]
    fn fully_resolved_file_preserves_missing_final_newline() {
        let content = format!(
            "---\nmodule: m\nversion: 1\nstatus: stable\nfiles:\n  - src/m.rs\n---\n\
             # M\n\n## Public API\n\n| Name | Description |\n|------|-------------|\n\
             {OPEN_HEAD}| `a` | ours |\n\
             {SEPARATOR}| `b` | incoming |\n{}",
            CLOSE_SIDE.trim_end()
        );
        let (resolved, result, should_write) =
            resolve_spec_conflicts(&content, "specs/m/m.spec.md");
        assert_eq!(result.status, MergeStatus::Resolved);
        assert!(should_write);
        assert!(!resolved.ends_with('\n'));
    }

    #[test]
    fn fully_resolvable_dry_run_leaves_disk_bytes_unchanged() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let specs = root.join("specs");
        fs::create_dir_all(&specs).unwrap();
        let spec = specs.join("m.spec.md");
        let content = format!(
            "---\nmodule: m\nversion: 1\nstatus: stable\nfiles:\n  - src/m.rs\n---\n\
             # M\n\n## Public API\n\n| Name | Description |\n|------|-------------|\n\
             {OPEN_HEAD}| `a` | ours |\n\
             {SEPARATOR}| `b` | incoming |\n{CLOSE_SIDE}"
        );
        fs::write(&spec, &content).unwrap();

        let results = merge_specs(root, &specs, true, true);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, MergeStatus::Resolved);
        assert_eq!(fs::read_to_string(&spec).unwrap(), content);
    }

    #[test]
    fn git_discovery_failure_is_a_safe_noop() {
        let tmp = tempfile::tempdir().unwrap();
        let specs = tmp.path().join("specs");
        fs::create_dir_all(&specs).unwrap();
        assert!(detect_conflicted_specs(tmp.path(), &specs).is_empty());
    }

    #[test]
    fn partial_resolution_preview_keeps_all_or_nothing_write_boundary() {
        let content = format!(
            "---\nmodule: m\nversion: 1\n---\n# M\n\n## Public API\n\n\
             | Name | Description |\n|------|-------------|\n\
             {OPEN_HEAD}| `a` | MAIN description. |\n\
             {SEPARATOR}| `b` | SIDE description. |\n{CLOSE_SIDE}\n\
             ## Purpose\n\n{OPEN_HEAD}Our purpose.\n\
             {SEPARATOR}Their purpose.\n{CLOSE_SIDE}"
        );
        let (resolved, result, should_write) =
            resolve_spec_conflicts(&content, "specs/m/m.spec.md");
        assert_eq!(result.status, MergeStatus::Manual);
        assert!(!should_write, "ambiguous files must remain untouched");
        assert!(resolved.contains("MAIN description"), "{resolved}");
        assert!(resolved.contains("SIDE description"), "{resolved}");
        assert!(has_conflict_markers(&resolved), "manual hunk keeps markers");
        assert!(resolved.contains("Our purpose."), "{resolved}");
        assert!(
            result
                .details
                .iter()
                .any(|d| d.contains("left unchanged (all-or-nothing)")),
            "{:?}",
            result.details
        );
        assert!(
            result
                .details
                .iter()
                .all(|detail| !detail.contains("Auto-resolved")),
            "{:?}",
            result.details
        );
    }

    #[test]
    fn merge_specs_leaves_partially_resolvable_file_untouched() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let specs = root.join("specs");
        fs::create_dir_all(&specs).unwrap();
        let spec = specs.join("m.spec.md");
        let content = format!(
            "---\nmodule: m\nversion: 1\n---\n# M\n\n## Public API\n\n\
             | Name | Description |\n|------|-------------|\n\
             {OPEN_HEAD}| `a` | MAIN description. |\n\
             {SEPARATOR}| `b` | SIDE description. |\n{CLOSE_SIDE}\n\
             ## Purpose\n\n{OPEN_HEAD}Our purpose.\n\
             {SEPARATOR}Their purpose.\n{CLOSE_SIDE}"
        );
        fs::write(&spec, &content).unwrap();
        let results = merge_specs(root, &specs, false, true);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, MergeStatus::Manual);
        let on_disk = fs::read_to_string(&spec).unwrap();
        assert_eq!(on_disk, content);
    }

    #[cfg(unix)]
    #[test]
    fn all_files_reports_unreadable_specs_as_manual() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let specs = root.join("specs");
        fs::create_dir_all(&specs).unwrap();
        let spec = specs.join("m.spec.md");
        fs::write(
            &spec,
            format!(
                "---\nmodule: m\nversion: 1\nstatus: stable\nfiles:\n  - src/m.rs\n---\n\
                 # M\n\n## Public API\n\n{OPEN_HEAD}| `a` | ours |\n\
                 {SEPARATOR}| `b` | incoming |\n{CLOSE_SIDE}"
            ),
        )
        .unwrap();
        fs::set_permissions(&spec, fs::Permissions::from_mode(0o000)).unwrap();

        let results = merge_specs(root, &specs, false, true);

        fs::set_permissions(&spec, fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, MergeStatus::Manual);
        assert!(
            results[0]
                .details
                .iter()
                .any(|detail| detail.contains("Cannot read file")),
            "{:?}",
            results[0].details
        );
    }
}
