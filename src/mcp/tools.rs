//! Tool catalog, tool handlers, resources, scoring, and issue verification.

use super::confinement::{
    load_confined_config, read_file_bounded, validate_generated_module_names,
    validate_known_config_files, validate_path_or_ancestor, validate_spec_file_mappings,
};
use super::preflight::{validate_autodetection_tree, validate_manifest_inputs};
use super::protocol::validate_tool_content_response_size;
use super::publication::{write_generated_specs_transactionally, write_new_confined};
use crate::config::detect_source_dirs;
use crate::deps::build_dep_graph;
use crate::generator::generate_specs_for_unspecced_modules_paths;
use crate::output::{CoverageFindings, coverage_json};
use crate::scoring;
use crate::types::SpecSyncConfig;
use crate::validator::{
    compute_coverage_checked, find_spec_files, get_schema_table_names, validate_spec,
};
use cap_std::fs::Dir;
use serde_json::{Value, json};
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub(super) const MAX_GENERATED_SPECS: usize = 1_000;

pub(super) const MAX_GENERATED_OUTPUT_BYTES: u64 = 64 * 1024 * 1024;

pub(super) const MAX_ISSUE_DIAGNOSTIC_PATH_CHARS: usize = 240;

// ─── Resource Implementations ───────────────────────────────────────────

pub(super) fn resource_specs_list(
    root: &Path,
    git_freshness_available: bool,
) -> Result<(String, &'static str), String> {
    let (config, spec_files) = load_and_discover(root, true)?;

    let specs: Vec<Value> = spec_files
        .iter()
        .map(|f| -> Result<Value, String> {
            let content = read_file_bounded(root, f, "spec file")?;
            let parsed = crate::parser::parse_frontmatter(&content);
            let score = score_spec_for_mcp(f, root, &config, git_freshness_available);
            let relative = f
                .strip_prefix(root)
                .unwrap_or(f)
                .to_string_lossy()
                .to_string();

            if let Some(parsed) = parsed {
                let fm = parsed.frontmatter;
                Ok(json!({
                    "path": relative,
                    "module": fm.module,
                    "version": fm.version,
                    "status": fm.status,
                    "files": fm.files,
                    "depends_on": fm.depends_on,
                    "score": score.total,
                    "grade": score.grade,
                    "git_freshness_available": git_freshness_available,
                }))
            } else {
                Ok(json!({
                    "path": relative,
                    "module": null,
                    "score": score.total,
                    "grade": score.grade,
                    "git_freshness_available": git_freshness_available,
                }))
            }
        })
        .collect::<Result<_, _>>()?;

    let output = json!({
        "specs": specs,
        "count": specs.len(),
        "git_freshness_available": git_freshness_available,
    });
    Ok((
        serde_json::to_string_pretty(&output).unwrap(),
        "application/json",
    ))
}

pub(super) fn resource_spec_by_module(
    root: &Path,
    module: &str,
) -> Result<(String, &'static str), String> {
    let (_config, spec_files) = load_and_discover(root, true)?;

    // Find the spec file matching this module name
    for f in &spec_files {
        let content = read_file_bounded(root, f, "spec file")?;

        let parsed = crate::parser::parse_frontmatter(&content);
        let matches = parsed
            .as_ref()
            .and_then(|p| p.frontmatter.module.as_deref())
            .map(|m| m == module)
            .unwrap_or(false);

        if matches {
            return Ok((content, "text/markdown"));
        }
    }

    Err(format!("No spec found for module: {module}"))
}

pub(super) fn resource_graph(root: &Path) -> Result<(String, &'static str), String> {
    let config = load_confined_config(root)?;
    let graph = build_dep_graph(root, &config.specs_dir);

    let nodes: Vec<Value> = graph
        .values()
        .map(|node| {
            json!({
                "module": node.module,
                "spec_path": node.spec_path,
                "depends_on": node.declared_deps,
                "files": node.files,
            })
        })
        .collect();

    // Build edges list
    let mut edges: Vec<Value> = Vec::new();
    for node in graph.values() {
        for dep in &node.declared_deps {
            edges.push(json!({
                "from": node.module,
                "to": dep,
            }));
        }
    }

    // Detect cycles
    let cycles = crate::deps::validate_deps(root, &config.specs_dir).cycles;
    let cycle_values: Vec<Value> = cycles.iter().map(|c| json!(c)).collect();

    // Topological order
    let topo = crate::deps::topological_sort(&graph);

    let output = json!({
        "modules": nodes,
        "edges": edges,
        "module_count": graph.len(),
        "edge_count": edges.len(),
        "cycles": cycle_values,
        "topological_order": topo,
    });

    Ok((
        serde_json::to_string_pretty(&output).unwrap(),
        "application/json",
    ))
}

pub(super) fn resource_config(root: &Path) -> Result<(String, &'static str), String> {
    let config = load_confined_config(root)?;
    let output = json!({
        "specs_dir": config.specs_dir,
        "source_dirs": config.source_dirs,
        "required_sections": config.required_sections,
        "exclude_dirs": config.exclude_dirs,
        "exclude_patterns": config.exclude_patterns,
        "schema_dir": config.schema_dir,
    });

    Ok((
        serde_json::to_string_pretty(&output).unwrap(),
        "application/json",
    ))
}

pub(super) fn resource_coverage(root: &Path) -> Result<(String, &'static str), String> {
    // Byte-identical to the `specsync_coverage` tool: one constructor, one
    // validation pass, one payload. This resource is what an agent reads when
    // it asks the project how it is doing, and it used to answer with
    // percentages alone — no findings, no `passed`. An agent has no exit code
    // and no second channel, so "nothing reported" is the only thing it can
    // read as "nothing wrong".
    let output = mcp_coverage_payload(root)?;
    Ok((
        serde_json::to_string_pretty(&output).unwrap(),
        "application/json",
    ))
}

/// Every spec's validation outcome, collected once for whichever MCP payload
/// needs it.
///
/// `tool_check`, `tool_coverage` and `resource_coverage` used to answer three
/// different questions from three different amounts of work: check validated,
/// the other two validated nothing and reported no findings at all. One
/// collector means a finding cannot be present in one payload and absent from
/// another.
#[derive(Default)]
struct McpFindings {
    total_errors: usize,
    total_warnings: usize,
    specs_passed: usize,
    errors: Vec<String>,
    warnings: Vec<String>,
    notices: Vec<String>,
    specs: Vec<Value>,
}

fn validate_all(root: &Path, config: &SpecSyncConfig, spec_files: &[PathBuf]) -> McpFindings {
    let schema_tables = get_schema_table_names(root, config);
    let schema_columns = config
        .schema_dir
        .as_deref()
        .map(|schema_dir| crate::schema::build_schema(&root.join(schema_dir)))
        .unwrap_or_default();

    let mut collected = McpFindings::default();
    for spec_file in spec_files {
        let result = validate_spec(spec_file, root, &schema_tables, &schema_columns, config);
        let spec_passed = result.errors.is_empty();

        collected.specs.push(json!({
            "spec": result.spec_path,
            "passed": spec_passed,
            "errors": result.errors,
            "warnings": result.warnings,
            "export_summary": result.export_summary,
        }));

        for e in &result.errors {
            collected.errors.push(format!("{}: {e}", result.spec_path));
        }
        for w in &result.warnings {
            collected
                .warnings
                .push(format!("{}: {w}", result.spec_path));
        }
        for n in &result.notices {
            collected.notices.push(format!("{}: {n}", result.spec_path));
        }

        collected.total_errors += result.errors.len();
        collected.total_warnings += result.warnings.len();
        if spec_passed {
            collected.specs_passed += 1;
        }
    }
    collected
}

/// The coverage payload both MCP surfaces return, built by the one shared
/// constructor in `output`.
///
/// `passed` is the gate verdict, the same semantics the CLI uses: "would
/// `specsync coverage` exit 0 on this project?" — evaluated with no CLI
/// overrides and the project's configured enforcement, since MCP has neither
/// flags nor an exit code. The counts and arrays beside it answer the factual
/// question independently of that policy.
fn mcp_coverage_payload(root: &Path) -> Result<Value, String> {
    let (config, spec_files) = load_and_discover(root, true)?;
    let coverage = compute_coverage_checked(root, &spec_files, &config)
        .map_err(|error| format!("MCP coverage is inconclusive: {error}"))?;
    let collected = validate_all(root, &config, &spec_files);
    let passed = crate::commands::compute_exit_code(
        collected.total_errors,
        collected.total_warnings,
        false,
        config.enforcement,
        &coverage,
        None,
    ) == 0;

    Ok(coverage_json(
        &coverage,
        &CoverageFindings {
            passed,
            specs_checked: spec_files.len(),
            specs_passed: collected.specs_passed,
            errors: collected.errors,
            warnings: collected.warnings,
            notices: collected.notices,
        },
    ))
}

// ─── Tool Implementations ────────────────────────────────────────────────

pub(super) fn load_and_discover(
    root: &Path,
    allow_empty: bool,
) -> Result<(SpecSyncConfig, Vec<PathBuf>), String> {
    let config = load_confined_config(root)?;
    let specs_dir = root.join(&config.specs_dir);
    let spec_files: Vec<PathBuf> = find_spec_files(&specs_dir)
        .into_iter()
        .filter(|f| {
            f.file_name()
                .and_then(|n| n.to_str())
                .map(|n| !n.starts_with('_'))
                .unwrap_or(true)
        })
        .collect();

    validate_spec_file_mappings(root, &spec_files, &config.exclude_dirs)?;

    if spec_files.is_empty() && !allow_empty {
        return Err(format!(
            "No spec files found in {}/. Run specsync generate to scaffold specs.",
            config.specs_dir
        ));
    }

    Ok((config, spec_files))
}

pub(super) fn tool_check(root: &Path, arguments: &Value) -> Result<Value, String> {
    let (config, spec_files) = load_and_discover(root, false)?;
    let strict = arguments
        .get("strict")
        .and_then(|s| s.as_bool())
        .unwrap_or(false);

    // Classify changes for staleness detection
    let cache = crate::hash_cache::HashCache::load(root);
    let classifications = crate::hash_cache::classify_all_changes(root, &spec_files, &cache);
    let mut stale_entries: Vec<Value> = Vec::new();
    for classification in &classifications {
        let spec_rel = classification
            .spec_path
            .strip_prefix(root)
            .unwrap_or(&classification.spec_path)
            .to_string_lossy()
            .to_string();
        if classification.has(&crate::hash_cache::ChangeKind::Requirements) {
            stale_entries.push(json!({
                "spec": spec_rel,
                "reason": "requirements_changed",
                "message": "requirements changed — spec may need re-validation"
            }));
        }
    }

    // Same collector the coverage payloads use — one validation implementation
    // for every MCP surface, so a finding cannot exist in one and not another.
    let collected = validate_all(root, &config, &spec_files);
    let total_errors = collected.total_errors;
    let total_warnings = collected.total_warnings;
    let passed = collected.specs_passed;
    let all_errors = collected.errors;
    let mut all_warnings = collected.warnings;
    let spec_results = collected.specs;

    // Add staleness warnings into the warnings array for consistency
    for entry in &stale_entries {
        if let Some(msg) = entry["message"].as_str() {
            let spec = entry["spec"].as_str().unwrap_or("unknown");
            all_warnings.push(format!("{spec}: {msg}"));
        }
    }

    let coverage = compute_coverage_checked(root, &spec_files, &config)
        .map_err(|error| format!("MCP coverage is inconclusive: {error}"))?;
    let staleness_warnings = stale_entries.len();
    let effective_warnings = total_warnings + staleness_warnings;
    let overall_passed = total_errors == 0 && (!strict || effective_warnings == 0);

    Ok(json!({
        "passed": overall_passed,
        "specs_checked": spec_files.len(),
        "specs_passed": passed,
        "total_errors": total_errors,
        "total_warnings": effective_warnings,
        "errors": all_errors,
        "warnings": all_warnings,
        "stale": stale_entries,
        "specs": spec_results,
        "coverage": {
            // Null, not 100, when nothing was measured (#582).
            "file_percent": coverage.file_coverage_percent(),
            "loc_percent": coverage.loc_coverage_percent(),
        }
    }))
}

pub(super) fn tool_coverage(root: &Path) -> Result<Value, String> {
    mcp_coverage_payload(root)
}

pub(super) fn tool_generate(
    root: &Path,
    write_directory: &Dir,
    arguments: &Value,
) -> Result<Value, String> {
    const RETIRED_ARGUMENTS: &[&str] = &[
        "ai",
        "provider",
        "aiProvider",
        "ai_provider",
        "model",
        "aiModel",
        "ai_model",
        "apiKey",
        "api_key",
        "aiApiKey",
        "ai_api_key",
        "credential",
        "credentials",
        "baseUrl",
        "base_url",
        "aiBaseUrl",
        "ai_base_url",
        "timeout",
        "timeoutSecs",
        "timeout_secs",
        "aiTimeout",
        "ai_timeout",
        "command",
        "aiCommand",
        "ai_command",
    ];
    if let Some(name) = RETIRED_ARGUMENTS
        .iter()
        .find(|name| arguments.get(**name).is_some())
    {
        return Err(format!(
            "MCP argument `{name}` was removed in spec-sync 5.0. `specsync_generate` is deterministic; use your coding agent to enrich the generated spec."
        ));
    }
    if let Some(name) = arguments
        .as_object()
        .and_then(|object| object.keys().find(|name| name.as_str() != "root"))
    {
        return Err(format!(
            "Unknown MCP generate argument `{name}`. `specsync_generate` accepts only `root`."
        ));
    }

    let (config, spec_files) = load_and_discover(root, true)?;
    let coverage = compute_coverage_checked(root, &spec_files, &config)
        .map_err(|error| format!("MCP coverage is inconclusive: {error}"))?;
    if coverage.unspecced_modules.len() > MAX_GENERATED_SPECS {
        return Err(format!(
            "MCP generation exceeds the {MAX_GENERATED_SPECS}-spec output limit"
        ));
    }
    validate_generated_module_names(&coverage.unspecced_modules)?;
    let mut expected_destinations = Vec::new();
    for module_name in &coverage.unspecced_modules {
        let destination = root
            .join(&config.specs_dir)
            .join(module_name)
            .join(format!("{module_name}.spec.md"));
        validate_path_or_ancestor(root, &destination, "generation destination", None)?;
        match fs::symlink_metadata(&destination) {
            Ok(_) => {
                return Err(format!(
                    "MCP generation destination already exists for uncovered module `{module_name}`: {}",
                    destination.display()
                ));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Cannot inspect MCP generation destination {}: {error}",
                    destination.display()
                ));
            }
        }
        expected_destinations.push((module_name.clone(), destination));
    }
    let outcome = generate_specs_for_unspecced_modules_paths(root, &coverage, &config);

    let generated_specs = validate_generation_outcome(root, &expected_destinations, &outcome)?;
    let generated_paths: Vec<String> = generated_specs
        .iter()
        .map(|(path, _)| path.to_string_lossy().to_string())
        .collect();
    let result = json!({
        "generated": generated_paths,
        "count": outcome.generated,
    });
    validate_tool_content_response_size(&result)?;
    write_generated_specs_transactionally(write_directory, generated_specs)?;

    Ok(result)
}

pub(super) fn validate_generation_outcome(
    root: &Path,
    expected_destinations: &[(String, PathBuf)],
    outcome: &crate::generator::GenerationOutcome,
) -> Result<Vec<(PathBuf, String)>, String> {
    validate_generation_outcome_with_limits(
        root,
        expected_destinations,
        outcome,
        MAX_GENERATED_SPECS,
        MAX_GENERATED_OUTPUT_BYTES,
    )
}

pub(super) fn validate_generation_outcome_with_limits(
    root: &Path,
    expected_destinations: &[(String, PathBuf)],
    outcome: &crate::generator::GenerationOutcome,
    max_generated_specs: usize,
    max_generated_output_bytes: u64,
) -> Result<Vec<(PathBuf, String)>, String> {
    if expected_destinations.len() > max_generated_specs || outcome.generated > max_generated_specs
    {
        return Err(format!(
            "MCP generation exceeds the {max_generated_specs}-spec output limit"
        ));
    }
    if outcome.generated != expected_destinations.len() {
        return Err(format!(
            "MCP generation created {} of {} required specs",
            outcome.generated,
            expected_destinations.len()
        ));
    }
    let mut generated_specs = Vec::with_capacity(expected_destinations.len());
    let mut generated_bytes = 0u64;
    for (module_name, destination) in expected_destinations {
        if !destination.is_file() {
            return Err(format!(
                "MCP generation did not create the required spec for module `{module_name}`: {}",
                destination.display()
            ));
        }
        let content = read_file_bounded(root, destination, "generated spec")?;
        let relative = destination.strip_prefix(root).map_err(|_| {
            format!(
                "Generated MCP spec is not relative to the operation root: {}",
                destination.display()
            )
        })?;
        generated_bytes = generated_bytes.saturating_add(content.len() as u64);
        if generated_bytes > max_generated_output_bytes {
            return Err(format!(
                "MCP generated specs exceed the {} MiB cumulative output limit",
                max_generated_output_bytes / (1024 * 1024)
            ));
        }
        generated_specs.push((relative.to_path_buf(), content));
    }
    let expected_paths: Vec<String> = generated_specs
        .iter()
        .map(|(path, _)| path.to_string_lossy().to_string())
        .collect();
    if outcome.generated_paths != expected_paths {
        return Err("MCP generator reported paths that do not match its confined outputs".into());
    }
    Ok(generated_specs)
}

pub(super) fn tool_list_specs(root: &Path) -> Result<Value, String> {
    let (_config, spec_files) = load_and_discover(root, true)?;

    let specs: Vec<Value> = spec_files
        .iter()
        .map(|f| -> Result<Value, String> {
            let content = read_file_bounded(root, f, "spec file")?;
            let parsed = crate::parser::parse_frontmatter(&content);
            let relative = f
                .strip_prefix(root)
                .unwrap_or(f)
                .to_string_lossy()
                .to_string();

            if let Some(parsed) = parsed {
                let fm = parsed.frontmatter;
                Ok(json!({
                    "path": relative,
                    "module": fm.module,
                    "version": fm.version,
                    "status": fm.status,
                    "files": fm.files,
                }))
            } else {
                Ok(json!({
                    "path": relative,
                    "module": null,
                    "version": null,
                    "status": null,
                    "files": [],
                }))
            }
        })
        .collect::<Result<_, _>>()?;

    Ok(json!({
        "specs": specs,
        "count": specs.len(),
    }))
}

pub(super) fn tool_init(root: &Path, write_directory: &Dir) -> Result<Value, String> {
    validate_known_config_files(root)?;
    if write_directory
        .try_exists("specsync.json")
        .map_err(|error| format!("Cannot inspect MCP initialization destination: {error}"))?
    {
        return Ok(json!({
            "created": false,
            "message": "specsync.json already exists"
        }));
    }

    validate_autodetection_tree(root)?;
    validate_manifest_inputs(root)?;
    let detected_dirs = detect_source_dirs(root);

    let default = json!({
        "specsDir": "specs",
        "sourceDirs": detected_dirs,
        "requiredSections": [
            "Purpose",
            "Public API",
            "Invariants",
            "Behavioral Examples",
            "Error Cases",
            "Dependencies",
            "Change Log"
        ],
        "excludeDirs": ["__tests__"],
        "excludePatterns": ["**/__tests__/**", "**/*.test.ts", "**/*.spec.ts"]
    });

    let content = serde_json::to_string_pretty(&default).unwrap() + "\n";
    write_new_confined(
        write_directory,
        Path::new("specsync.json"),
        content.as_bytes(),
        "initialization destination",
    )?;

    Ok(json!({
        "created": true,
        "source_dirs": detected_dirs,
        "message": "Created specsync.json"
    }))
}

pub(super) fn tool_score(root: &Path, git_freshness_available: bool) -> Result<Value, String> {
    let (config, spec_files) = load_and_discover(root, false)?;

    let scores: Vec<scoring::SpecScore> = spec_files
        .iter()
        .map(|f| score_spec_for_mcp(f, root, &config, git_freshness_available))
        .collect();
    let project = scoring::compute_project_score(scores);

    let specs_json: Vec<Value> = project
        .spec_scores
        .iter()
        .map(|s| {
            json!({
                "spec": s.spec_path,
                "total": s.total,
                "grade": s.grade,
                "frontmatter": s.frontmatter_score,
                "sections": s.sections_score,
                "api": s.api_score,
                "depth": s.depth_score,
                "freshness": s.freshness_score,
                "git_freshness_available": git_freshness_available,
                "suggestions": s.suggestions,
            })
        })
        .collect();

    Ok(json!({
        "average_score": (project.average_score * 10.0).round() / 10.0,
        "grade": project.grade,
        "total_specs": project.total_specs,
        "git_freshness_available": git_freshness_available,
        "distribution": {
            "A": project.grade_distribution[0],
            "B": project.grade_distribution[1],
            "C": project.grade_distribution[2],
            "D": project.grade_distribution[3],
            "F": project.grade_distribution[4],
        },
        "specs": specs_json,
    }))
}

pub(super) fn score_spec_for_mcp(
    spec_path: &Path,
    root: &Path,
    config: &crate::types::SpecSyncConfig,
    git_freshness_available: bool,
) -> scoring::SpecScore {
    let mut score = scoring::score_spec(spec_path, root, config);
    if git_freshness_available {
        return score;
    }

    const UNAVAILABLE_GIT_FRESHNESS_POINTS: u32 = 5;
    // `score_spec` now withholds these points itself whenever there is no
    // history to measure against (#572), which is always true of the confined
    // snapshot — it is a temp copy with no `.git`. Deducting again would charge
    // the spec twice for one absence, so only deduct when the points are still
    // on the board. The relabel below always runs: "intentionally unavailable
    // here" is a more useful thing to read than "not a git repository".
    let penalty = match score.git_freshness {
        scoring::GitFreshness::Withheld => 0,
        _ => score.freshness_score.min(UNAVAILABLE_GIT_FRESHNESS_POINTS),
    };
    score.freshness_score = score.freshness_score.saturating_sub(penalty);
    score.total = score.total.saturating_sub(penalty);
    score.grade = scoring::letter_grade(score.total);
    score
        .suggestions
        .retain(|suggestion| !suggestion.contains("drift is unverifiable"));
    score.suggestions.push(
        "Freshness (-5pts): Git history is intentionally unavailable in the confined MCP snapshot"
            .to_string(),
    );
    if let Some(freshness) = score
        .explain
        .iter_mut()
        .find(|detail| detail.dimension == "Freshness")
    {
        freshness.score = score.freshness_score;
        if let Some(criterion) = freshness
            .criteria
            .iter_mut()
            .find(|criterion| criterion.name == "git_freshness")
        {
            criterion.passed = false;
            criterion.points = 0;
            criterion.detail = Some("unavailable in confined MCP snapshot".to_string());
        }
    }
    score
}

pub(super) fn tool_issues(root: &Path) -> Result<Value, String> {
    use crate::github;

    let (config, references) = discover_issue_references(root)?;

    let mut results: Vec<Value> = Vec::new();
    let mut total_valid = 0usize;
    let mut total_closed = 0usize;
    let mut total_not_found = 0usize;

    if references.is_empty() {
        return Ok(json!({
            "repo": null,
            "total_valid": 0,
            "total_closed": 0,
            "total_not_found": 0,
            "specs": results,
        }));
    }

    let repo = config
        .github
        .as_ref()
        .and_then(|github| github.repo.as_deref())
        .filter(|repo| !repo.trim().is_empty())
        .ok_or_else(|| {
            "MCP specsync_issues requires an explicit `github.repo`; Git metadata auto-detection is disabled by the server security boundary".to_string()
        })?
        .to_string();

    for verification in github::verify_issue_batch(&repo, &references) {
        let rel_path = verification.spec_path.clone();
        ensure_issue_verification_complete(&rel_path, &verification)?;

        total_valid += verification.valid.len();
        total_closed += verification.closed.len();
        total_not_found += verification.not_found.len();

        results.push(json!({
            "spec": rel_path,
            "valid": verification.valid.iter().map(|i| json!({
                "number": i.number,
                "title": i.title,
                "state": i.state,
            })).collect::<Vec<_>>(),
            "closed": verification.closed.iter().map(|i| json!({
                "number": i.number,
                "title": i.title,
            })).collect::<Vec<_>>(),
            "not_found": verification.not_found,
        }));
    }

    Ok(json!({
        "repo": repo,
        "total_valid": total_valid,
        "total_closed": total_closed,
        "total_not_found": total_not_found,
        "specs": results,
    }))
}

type IssueReferences = (String, Vec<u64>, Vec<u64>);

pub(super) fn is_issue_spec_file_name(name: &OsStr) -> Result<bool, ()> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;

        let bytes = name.as_bytes();
        if !bytes.ends_with(b".spec.md") {
            return Ok(false);
        }
        if name.to_str().is_none() {
            return Err(());
        }
        Ok(!bytes.starts_with(b"_"))
    }

    #[cfg(not(unix))]
    {
        Ok(name
            .to_str()
            .is_some_and(|name| name.ends_with(".spec.md") && !name.starts_with('_')))
    }
}

fn find_issue_spec_files_checked(root: &Path, specs_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut spec_files = Vec::new();
    for result in WalkDir::new(specs_dir)
        .follow_links(false)
        .sort_by_file_name()
    {
        let entry = result.map_err(|error| {
            let rel_path = issue_relative_spec_path(root, error.path().unwrap_or(specs_dir));
            format!(
                "MCP specsync_issues issue discovery is inconclusive for {rel_path}: spec directory entry could not be inspected"
            )
        })?;
        let is_spec = is_issue_spec_file_name(entry.file_name()).map_err(|()| {
            let rel_path = issue_relative_spec_path(root, entry.path());
            format!(
                "MCP specsync_issues issue discovery is inconclusive for {rel_path}: spec filename is not valid UTF-8"
            )
        })?;
        if !is_spec {
            continue;
        }
        if !entry.file_type().is_file() {
            let rel_path = issue_relative_spec_path(root, entry.path());
            return Err(format!(
                "MCP specsync_issues issue discovery is inconclusive for {rel_path}: spec path is not a regular file"
            ));
        }
        spec_files.push(entry.into_path());
    }
    spec_files.sort();
    Ok(spec_files)
}

pub(super) fn discover_issue_references(
    root: &Path,
) -> Result<(SpecSyncConfig, Vec<IssueReferences>), String> {
    let config = load_confined_config(root).map_err(|error| {
        format!(
            "MCP specsync_issues issue discovery is inconclusive: {}",
            issue_project_error_reason(&error)
        )
    })?;
    let specs_dir = root.join(&config.specs_dir);
    let spec_files = find_issue_spec_files_checked(root, &specs_dir)?;

    if spec_files.is_empty() {
        return Err(
            "MCP specsync_issues issue discovery is inconclusive: no spec files were found"
                .to_string(),
        );
    }

    let mut references = Vec::new();
    for spec_path in &spec_files {
        let rel_path = issue_relative_spec_path(root, spec_path);
        let content = read_file_bounded(root, spec_path, "spec file").map_err(|error| {
            format!(
                "MCP specsync_issues issue discovery is inconclusive for {rel_path}: {}",
                issue_spec_read_error_reason(&error)
            )
        })?;
        let (implements, tracks) = crate::parser::parse_checked_issue_references(&content)
            .map_err(|reason| {
                format!(
                    "MCP specsync_issues issue discovery is inconclusive for {rel_path}: {reason}"
                )
            })?;

        if !implements.is_empty() || !tracks.is_empty() {
            references.push((rel_path, implements, tracks));
        }
    }

    validate_spec_file_mappings(root, &spec_files, &config.exclude_dirs).map_err(|error| {
        format!(
            "MCP specsync_issues issue discovery is inconclusive: {}",
            issue_project_error_reason(&error)
        )
    })?;

    Ok((config, references))
}

fn is_unsafe_issue_diagnostic_character(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '\u{061c}'
                | '\u{200e}'
                | '\u{200f}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2066}'..='\u{2069}'
        )
}

pub(super) fn issue_relative_spec_path(root: &Path, spec_path: &Path) -> String {
    let Some(relative) = spec_path.strip_prefix(root).ok() else {
        return "<spec>".to_string();
    };
    let relative = relative.to_string_lossy();
    let mut sanitized = String::with_capacity(relative.len());
    for character in relative.chars() {
        if is_unsafe_issue_diagnostic_character(character) {
            sanitized.push_str(&format!("\\u{{{:04X}}}", character as u32));
        } else if cfg!(windows) && character == '\\' {
            sanitized.push('/');
        } else {
            sanitized.push(character);
        }
    }
    if sanitized.chars().count() <= MAX_ISSUE_DIAGNOSTIC_PATH_CHARS {
        return sanitized;
    }

    let mut bounded: String = sanitized
        .chars()
        .take(MAX_ISSUE_DIAGNOSTIC_PATH_CHARS.saturating_sub(3))
        .collect();
    bounded.push_str("...");
    bounded
}

pub(super) fn issue_spec_read_error_reason(error: &str) -> &'static str {
    if error.contains("not valid UTF-8") {
        "spec file is not valid UTF-8"
    } else if error.contains("exceeds") {
        "spec file exceeds the configured size limit"
    } else if error.contains("not a file") {
        "spec path is not a regular file"
    } else if error.contains("Cannot inspect") {
        "spec file metadata could not be inspected"
    } else if error.contains("Cannot read") {
        "spec file could not be read"
    } else {
        "spec path failed safety validation"
    }
}

fn issue_project_error_reason(error: &str) -> &'static str {
    if error.contains("exceeds") || error.contains("limit") {
        "project input exceeds a configured safety limit"
    } else if error.contains("Cannot read") || error.contains("not valid UTF-8") {
        "project metadata could not be read"
    } else if error.contains("Cannot inspect") {
        "project metadata could not be inspected"
    } else if error.contains("escape")
        || error.contains("outside")
        || error.contains("symlink")
        || error.contains("junction")
        || error.contains("relative path")
    {
        "project input failed confinement validation"
    } else {
        "project discovery failed safety validation"
    }
}

pub(super) fn ensure_issue_verification_complete(
    spec_path: &str,
    verification: &crate::github::IssueVerification,
) -> Result<(), String> {
    if verification.errors.is_empty() {
        return Ok(());
    }
    Err(format!(
        "GitHub issue verification was inconclusive for {spec_path}: {}",
        verification.errors.join("; ")
    ))
}

#[cfg(test)]
mod tests;
