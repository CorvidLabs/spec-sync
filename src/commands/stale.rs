use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::git_utils::{
    MissingHistory, SpecBaseline, StaleInfo, UnmeasurableSpec, git_commits_since, missing_history,
    spec_baseline,
};
use crate::parser;
use crate::types;

use super::{filter_by_status, load_and_discover};

/// Report that staleness cannot be determined here and exit non-zero.
///
/// `stale` has exactly one job, so an unanswerable question is a hard failure
/// rather than a degraded answer. Never returns.
fn refuse_without_history(missing: MissingHistory, format: types::OutputFormat) -> ! {
    match format {
        types::OutputFormat::Json => {
            let output = serde_json::json!({
                "error": missing.reason(),
                "stale_specs": [],
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        _ => {
            eprintln!(
                "{} {} — staleness detection requires git history.",
                "Error:".red().bold(),
                missing.sentence(),
            );
        }
    }
    std::process::exit(1);
}

pub fn cmd_stale(
    root: &Path,
    format: types::OutputFormat,
    threshold: usize,
    exclude_status: &[String],
    only_status: &[String],
    enforcement: Option<types::EnforcementMode>,
) {
    // An unborn HEAD is a git repository by every other test, but nothing can
    // be newer or older than a history that does not exist. Reporting every spec
    // "up to date" there was an answer to a question that could not be asked —
    // and it is the state `git init` leaves behind (#558). Handled alongside the
    // no-repository case, which was already correct.
    if let Some(missing) = missing_history(root) {
        refuse_without_history(missing, format);
    }

    let (config, all_spec_files) = load_and_discover(root, false);
    let spec_files = filter_by_status(&all_spec_files, exclude_status, only_status);

    let mut stale_specs: Vec<StaleInfo> = Vec::new();
    let mut unmeasurable_specs: Vec<UnmeasurableSpec> = Vec::new();
    let mut partial_unmeasurable_specs: Vec<(String, Vec<(String, &'static str)>)> = Vec::new();
    let mut fresh_count = 0;

    for spec_file in &spec_files {
        let content = match fs::read_to_string(spec_file) {
            Ok(c) => c.replace("\r\n", "\n"),
            Err(_) => continue,
        };
        let parsed = match parser::parse_frontmatter(&content) {
            Some(p) => p,
            None => continue,
        };

        let fm = &parsed.frontmatter;
        let module_name = fm.module.clone().unwrap_or_else(|| {
            spec_file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .strip_suffix(".spec")
                .unwrap_or("unknown")
                .to_string()
        });

        let rel_spec = spec_file
            .strip_prefix(root)
            .unwrap_or(spec_file)
            .to_string_lossy()
            .to_string();

        if fm.files.is_empty() {
            fresh_count += 1;
            continue;
        }

        let spec_commit = match spec_baseline(root, &rel_spec) {
            SpecBaseline::Commit(commit) => commit,
            // History exists; this spec is simply not in it yet. There is
            // nothing for it to be behind.
            SpecBaseline::Untracked => {
                fresh_count += 1;
                continue;
            }
            // Guarded above, so only reachable if history vanished mid-run.
            // Fail the same way rather than counting the spec fresh.
            SpecBaseline::Missing(missing) => refuse_without_history(missing, format),
        };

        let mut max_behind: usize = 0;
        let mut source_details: Vec<(String, usize)> = Vec::new();
        let mut unmeasurable: Vec<(String, &'static str)> = Vec::new();
        let mut source_measured: Vec<String> = Vec::new();
        let mut deleted_files: Vec<String> = Vec::new();

        for source_file in &fm.files {
            let absolute = root.join(source_file);
            // A cited file that is absent, or that names a directory, has
            // nothing for git to compare. Skipping it silently left
            // `max_behind` at 0, so a spec whose sources had all been deleted
            // was counted fresh and the run printed an unqualified
            // "All specs are up to date" — the drift was not absent, it was
            // unmeasurable, and the two were reported identically.
            if !absolute.exists() {
                // A committed deletion is not an absence of evidence — git can
                // name the commit that did it. Treating it as unmeasurable
                // throws away the stronger claim and lets a threshold hide it:
                // the deletion typically measures 1 commit, under the default
                // threshold of 5, so counting it as ordinary drift buries it too.
                if crate::git_utils::source_was_deleted(root, &spec_commit, source_file) {
                    deleted_files.push(source_file.clone());
                } else {
                    unmeasurable.push((source_file.clone(), "never tracked by git"));
                }
                continue;
            }
            if crate::exports::files_entry_is_directory(&absolute) {
                unmeasurable.push((source_file.clone(), "is a directory, not a source file"));
                continue;
            }
            source_measured.push(source_file.clone());
            let behind = git_commits_since(root, &spec_commit, source_file);
            if behind > 0 {
                source_details.push((source_file.clone(), behind));
            }
            max_behind = max_behind.max(behind);
        }
        // `files:` may legitimately repeat a path, so compare against what was
        // actually measured rather than against the declared length.
        let measured_any = !source_measured.is_empty();
        // Kept whatever the verdict: a spec that measured SOME of its files
        // still owes the reader the ones it could not, or the per-file
        // breakdown reads as exhaustive while quietly omitting a deleted file.
        let partial_unmeasurable = if measured_any && !unmeasurable.is_empty() {
            Some(unmeasurable.clone())
        } else {
            None
        };

        // A fully in-sync spec (0 commits behind) is never stale, even at
        // `--threshold 0`: the threshold gates how much drift is tolerated,
        // not whether drift exists at all.
        if let Some(partial) = partial_unmeasurable {
            partial_unmeasurable_specs.push((module_name.clone(), partial));
        }

        // A spec citing a file that no longer exists is out of date with its
        // sources by definition, whatever the drift threshold says.
        let is_stale = !deleted_files.is_empty() || (max_behind > 0 && max_behind >= threshold);
        if is_stale {
            stale_specs.push(StaleInfo {
                spec_path: rel_spec,
                module_name,
                max_commits_behind: max_behind,
                source_details,
                deleted_files: deleted_files.clone(),
            });
        } else if measured_any {
            fresh_count += 1;
        } else {
            // Nothing was measurable. Counting this fresh is the defect: it
            // spends the spec against the "up to date" total using evidence
            // that was never gathered.
            unmeasurable_specs.push(UnmeasurableSpec {
                spec_path: rel_spec,
                module_name,
                files: unmeasurable,
            });
        }
    }

    // Sort by most stale first
    stale_specs.sort_by_key(|b| std::cmp::Reverse(b.max_commits_behind));

    let total = spec_files.len();
    let stale_count = stale_specs.len();

    match format {
        types::OutputFormat::Json => {
            let specs_json: Vec<serde_json::Value> = stale_specs
                .iter()
                .map(|s| {
                    let details: Vec<serde_json::Value> = s
                        .source_details
                        .iter()
                        .map(|(file, behind)| {
                            serde_json::json!({
                                "file": file,
                                "commits_behind": behind,
                            })
                        })
                        .collect();
                    serde_json::json!({
                        "spec_path": s.spec_path,
                        "module": s.module_name,
                        "commits_behind": s.max_commits_behind,
                        "source_files": details,
                        "deleted_files": s.deleted_files,
                    })
                })
                .collect();

            // The machine-readable form must carry the same distinctions as the
            // text form, or a consumer subtracting `stale + fresh` from `total`
            // silently absorbs the unmeasurable specs into whichever number it
            // happens to trust.
            let unmeasurable_json: Vec<serde_json::Value> = unmeasurable_specs
                .iter()
                .map(|u| {
                    serde_json::json!({
                        "spec_path": u.spec_path,
                        "module": u.module_name,
                        "files": u.files.iter().map(|(f, why)| serde_json::json!({
                            "file": f, "reason": why,
                        })).collect::<Vec<_>>(),
                    })
                })
                .collect();
            // Derived from `stale_specs`, not from a parallel vec: one source of
            // truth, so a renderer cannot disagree with the verdict it renders.
            let deleted_json: Vec<serde_json::Value> = stale_specs
                .iter()
                .filter(|s| !s.deleted_files.is_empty())
                .map(|s| serde_json::json!({ "module": s.module_name, "files": s.deleted_files }))
                .collect();
            let output = serde_json::json!({
                "total_specs": total,
                "stale_count": stale_count,
                "fresh_count": fresh_count,
                "unmeasurable_count": unmeasurable_specs.len(),
                "threshold": threshold,
                "stale_specs": specs_json,
                "unmeasurable_specs": unmeasurable_json,
                "deleted_source_specs": deleted_json,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        types::OutputFormat::Markdown | types::OutputFormat::Github => {
            println!("## Stale Spec Report\n");
            println!(
                "**{stale_count}** of **{total}** specs are stale (>{threshold} commits behind)\n"
            );

            // Same rule as the text renderer: the all-clear must not appear
            // while any spec went unmeasured. Fixing one renderer and leaving
            // another is how the claim survives in the format a CI comment
            // actually posts.
            if !unmeasurable_specs.is_empty() {
                println!(
                    "**{}** spec(s) could not be measured for staleness :warning:\n",
                    unmeasurable_specs.len()
                );
                println!("| Module | Spec | Unmeasurable Files |");
                println!("|--------|------|--------------------|");
                for u in &unmeasurable_specs {
                    let files: Vec<String> = u
                        .files
                        .iter()
                        .map(|(f, why)| format!("`{f}` ({why})"))
                        .collect();
                    println!(
                        "| {} | {} | {} |",
                        u.module_name,
                        u.spec_path,
                        files.join(", ")
                    );
                }
                println!();
            }

            if stale_specs.is_empty() && unmeasurable_specs.is_empty() {
                println!("All specs are up to date! :white_check_mark:");
            } else if !stale_specs.is_empty() {
                println!("| Module | Spec | Commits Behind | Drifted Files | Deleted Files |");
                println!("|--------|------|---------------|---------------|---------------|");
                for s in &stale_specs {
                    let drifted: Vec<String> = s
                        .source_details
                        .iter()
                        .map(|(f, n)| format!("`{f}` ({n})"))
                        .collect();
                    let deleted: Vec<String> =
                        s.deleted_files.iter().map(|f| format!("`{f}`")).collect();
                    println!(
                        "| {} | {} | {} | {} | {} |",
                        s.module_name,
                        s.spec_path,
                        s.max_commits_behind,
                        drifted.join(", "),
                        deleted.join(", "),
                    );
                }
                println!(
                    "\n> Run `specsync check` to validate these specs, or update them to match current source."
                );
            }
        }
        types::OutputFormat::Text | types::OutputFormat::Table | types::OutputFormat::Csv => {
            println!(
                "\n--- {} ------------------------------------------------",
                "Stale Spec Detection".bold()
            );
            println!(
                "\n  Threshold: {} commit(s) behind source files",
                threshold.to_string().cyan()
            );
            println!(
                "  Result:    {}/{} specs are stale{}\n",
                if stale_count > 0 {
                    stale_count.to_string().yellow().bold().to_string()
                } else {
                    stale_count.to_string().green().to_string()
                },
                total,
                if unmeasurable_specs.is_empty() {
                    String::new()
                } else {
                    format!(", {} unmeasurable", unmeasurable_specs.len())
                }
            );

            for (module, files) in &partial_unmeasurable_specs {
                println!(
                    "  {} {} — measured, but drift is unknown for some cited files",
                    "?".yellow(),
                    module.bold()
                );
                for (file, reason) in files {
                    println!("      {file} ({reason})");
                }
            }

            for u in &unmeasurable_specs {
                println!(
                    "  {} {} — staleness could not be measured",
                    "?".yellow(),
                    u.module_name.bold()
                );
                for (file, reason) in &u.files {
                    println!("      {file} ({reason})");
                }
            }

            // The tick asserts every spec was compared against its source
            // files and found current. It must not appear while ANY cited file
            // went unmeasured — including a spec that measured its other files,
            // because the sentence claims the whole `files:` list, not the part
            // of it that happened to be readable.
            if stale_specs.is_empty()
                && unmeasurable_specs.is_empty()
                && partial_unmeasurable_specs.is_empty()
            {
                println!(
                    "  {} All specs are up to date with their source files.",
                    "✓".green()
                );
            } else if !stale_specs.is_empty() {
                for s in &stale_specs {
                    if !s.deleted_files.is_empty() {
                        println!(
                            "  {} {} — cites {}",
                            "✗".red(),
                            s.module_name.bold(),
                            if s.deleted_files.len() == 1 {
                                "a source file that no longer exists".to_string()
                            } else {
                                format!(
                                    "{} source files that no longer exist",
                                    s.deleted_files.len()
                                )
                            }
                        );
                        for f in &s.deleted_files {
                            println!("      {f} (deleted)");
                        }
                        continue;
                    }
                    println!(
                        "  {} {} — {} commits behind",
                        "⚠".yellow(),
                        s.module_name.bold(),
                        s.max_commits_behind.to_string().yellow(),
                    );
                    println!("    spec: {}", s.spec_path.dimmed());
                    for (file, behind) in &s.source_details {
                        println!(
                            "      {} {file} ({behind} commit{})",
                            "→".dimmed(),
                            if *behind == 1 { "" } else { "s" },
                        );
                    }
                }

                println!(
                    "\n  {} Run {} to validate, or update specs to match source.",
                    "Tip:".cyan(),
                    "specsync check".bold(),
                );
            }
            println!();
        }
    }

    // Warn mode is non-blocking whether selected explicitly or inherited from
    // project configuration.
    let warn_only = enforcement.unwrap_or(config.enforcement) == types::EnforcementMode::Warn;
    // A spec whose staleness could not be determined is the case this command's
    // own header calls a hard failure, and `refuse_without_history` already
    // exits 1 when the MISSING INPUT is the history. A missing cited file is the
    // same question one level down and must fail the same way — reporting it in
    // the text and still exiting 0 fixes only the half a human reads.
    // Partial counts too: a spec that measured three files and could not
    // measure the fourth has an unanswered question in it, and the rule is the
    // same one `refuse_without_history` applies to a whole run. Exiting 0 here
    // while exiting 1 for a fully-unmeasurable spec is a distinction the caller
    // cannot act on.
    if (stale_count > 0 || !unmeasurable_specs.is_empty() || !partial_unmeasurable_specs.is_empty())
        && !warn_only
    {
        std::process::exit(1);
    }
}
