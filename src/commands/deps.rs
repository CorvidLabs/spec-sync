use colored::Colorize;
use std::path::Path;
use std::process;

use crate::config::load_config;
use crate::deps;
use crate::output::NO_FILES_MEASURED;
use crate::types;

use super::{compute_exit_code, default_enforcement, load_and_discover};

pub fn cmd_deps(
    root: &Path,
    strict: bool,
    enforcement: Option<types::EnforcementMode>,
    require_coverage: Option<usize>,
    format: types::OutputFormat,
    mermaid: bool,
    dot: bool,
) {
    let config = load_config(root);

    // --mermaid or --dot: output graph visualization and exit
    if mermaid || dot {
        let graph = deps::build_dep_graph(root, &config.specs_dir);
        let has_edges = graph.values().any(|n| !n.declared_deps.is_empty());
        if mermaid {
            println!("{}", render_mermaid(&graph));
        } else {
            println!("{}", render_dot(&graph));
        }
        if !has_edges && !graph.is_empty() {
            eprintln!(
                "\n{} No depends_on relationships found. Add `depends_on: [module_name]` to spec frontmatter to see the dependency graph.",
                "Hint:".yellow()
            );
        }
        // A visualization request must still honor the gate flags: without this,
        // `deps --strict --mermaid`/`--dot` silently exited 0 on the same undeclared
        // imports / cycles / missing deps that the non-visual path fails on. The
        // diagram remains the only thing on stdout; the gate note and exit go to
        // stderr / the exit code, consistent with the normal `deps` path.
        let report = deps::validate_deps(root, &config.specs_dir);
        // A rendered graph is as incomplete as the analysis behind it: say which
        // languages contributed no edges because nobody could parse them, and
        // which imports were parsed but could not be attributed (#477).
        for note in disclosures(&report) {
            eprintln!("{} {note}", "⊘".yellow());
        }
        let strict_fail = strict && !report.warnings.is_empty();
        if strict_fail {
            eprintln!(
                "{}: {} dependency warning(s) treated as errors",
                "--strict mode".red(),
                report.warnings.len()
            );
        }
        if !report.errors.is_empty() || strict_fail {
            process::exit(1);
        }
        return;
    }

    let report = deps::validate_deps(root, &config.specs_dir);

    match format {
        types::OutputFormat::Json => {
            let output = serde_json::json!({
                "modules": report.module_count,
                "edges": report.edge_count,
                "errors": report.errors,
                "warnings": report.warnings,
                "cycles": report.cycles,
                "missing_deps": report.missing_deps.iter()
                    .map(|(m, d)| serde_json::json!({"module": m, "dep": d}))
                    .collect::<Vec<_>>(),
                "undeclared_imports": report.undeclared_imports.iter()
                    .map(|(m, i)| serde_json::json!({"module": m, "import": i}))
                    .collect::<Vec<_>>(),
                // Languages whose declared files were never parsed for imports:
                // an empty `undeclared_imports` covering them means "not
                // analysed", and a machine consumer must be able to tell (#477).
                "unanalyzed_languages": report.unanalyzed_languages.iter()
                    .map(|(language, files)| serde_json::json!({"language": language, "files": files}))
                    .collect::<Vec<_>>(),
                // Imports that were parsed but could not be mapped to a spec
                // module. Same distinction one level down: these produced no
                // edge because attribution failed, not because there was none.
                "unresolved_imports": report.unresolved_imports.iter()
                    .map(|(m, i)| serde_json::json!({"module": m, "import": i}))
                    .collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        types::OutputFormat::Markdown | types::OutputFormat::Github => {
            println!("## Dependency Validation\n");
            println!(
                "**Modules:** {}  **Edges:** {}\n",
                report.module_count, report.edge_count
            );
            if !report.errors.is_empty() {
                println!("### Errors\n");
                for e in &report.errors {
                    println!("- {e}");
                }
                println!();
            }
            if !report.warnings.is_empty() {
                println!("### Warnings\n");
                for w in &report.warnings {
                    println!("- {w}");
                }
                println!();
            }
            let disclosures = disclosures(&report);
            if !disclosures.is_empty() {
                println!("### Not Analysed\n");
                for note in disclosures {
                    println!("- {note}");
                }
                println!();
            }
            if report.errors.is_empty() && report.warnings.is_empty() {
                println!("{}", deps::valid_declarations_line(&report));
            }
        }
        types::OutputFormat::Text | types::OutputFormat::Table | types::OutputFormat::Csv => {
            println!(
                "\n--- {} ------------------------------------------------",
                "Dependency Validation".bold()
            );
            println!(
                "\n  Modules: {}  Edges: {}",
                report.module_count, report.edge_count
            );

            if report.errors.is_empty() && report.warnings.is_empty() {
                println!(
                    "\n  {} {}",
                    "✓".green(),
                    deps::valid_declarations_line(&report)
                );
            }

            // Say what went unread and what went unattributed. A category that
            // is empty because nobody parsed it — or because the parse could
            // not be mapped to a module — must not read as a category with no
            // problems.
            for note in disclosures(&report) {
                println!("  {} {note}", "⊘".yellow());
            }

            for e in &report.errors {
                println!("  {} {e}", "✗".red());
            }
            for w in &report.warnings {
                println!("  {} {w}", "⚠".yellow());
            }

            // Show topological order if no cycles
            if report.cycles.is_empty() && report.module_count > 0 {
                let graph = deps::build_dep_graph(root, &config.specs_dir);
                if let Some(order) = deps::topological_sort(&graph) {
                    println!("\n  {} Build order: {}", "→".cyan(), order.join(" -> "));
                }
            }

            println!();
        }
    }

    // `--strict` treats dependency warnings (undeclared imports — imports of a
    // module not listed in `depends_on`) as failures. Without this, `deps --strict`
    // was a silent no-op: undeclared imports were reported but never gated CI.
    let strict_fail = strict && !report.warnings.is_empty();
    // Human diagnostic, not report content. Suppress it in JSON mode so JSON
    // output stays fully machine-readable — no ANSI, nothing on stderr to parse
    // around; a JSON consumer already sees the failing warnings in the
    // `warnings` array and the non-zero exit code. Every other format gets the
    // note on stderr, keeping stdout a clean, parseable body.
    if strict_fail && format != types::OutputFormat::Json {
        eprintln!(
            "{}: {} dependency warning(s) treated as errors",
            "--strict mode".red(),
            report.warnings.len()
        );
    }

    if !report.errors.is_empty() || strict_fail {
        process::exit(1);
    }

    // --require-coverage gate (#419): was completely inert. Evaluate the same
    // coverage computation `specsync coverage` uses and fail below the
    // threshold. Honored in every output format; JSON/machine formats gate
    // silently via the exit code (handled by not printing here when Json).
    if let Some(req) = require_coverage {
        let (_, spec_files) = load_and_discover(root, true);
        let coverage = crate::validator::compute_coverage(root, &spec_files, &config);
        let enforcement = enforcement.unwrap_or_else(|| default_enforcement(&config));
        let code = compute_exit_code(0, 0, strict, enforcement, &coverage, Some(req));
        if format != types::OutputFormat::Json {
            match (code, coverage.file_coverage_percent()) {
                // A gate that passed because nothing was measured must not
                // print a percentage it does not have. `--require-coverage 0`
                // is the only threshold an unmeasured tree can satisfy (#582).
                (0, None) => println!(
                    "  {} --require-coverage {req}% is satisfied by an unmeasured tree — \
                     {NO_FILES_MEASURED}",
                    "⊘".yellow(),
                ),
                (0, Some(pct)) => println!(
                    "  {} Coverage {pct}% meets --require-coverage {req}%",
                    "✓".green(),
                ),
                (_, None) => eprintln!(
                    "{} {req}%: {NO_FILES_MEASURED} — check `source_dirs` and `exclude_patterns`",
                    "--require-coverage".red(),
                ),
                (_, Some(pct)) => eprintln!(
                    "{} {req}%: actual coverage is {pct}% ({} file(s) missing specs)",
                    "--require-coverage".red(),
                    coverage.unspecced_files.len()
                ),
            }
        }
        if code != 0 {
            process::exit(code);
        }
    }
}

/// Everything the analysis could NOT account for, in report order: languages it
/// cannot parse, then imports it parsed but could not attribute. One list for
/// every renderer, so a gap cannot be disclosed in one output format and hidden
/// in another (#477).
fn disclosures(report: &deps::DepsReport) -> Vec<String> {
    [
        deps::unanalyzed_languages_note(report),
        deps::unresolved_imports_note(report),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// Render the dependency graph as a Mermaid flowchart diagram.
fn render_mermaid(graph: &std::collections::HashMap<String, deps::DepNode>) -> String {
    let mut out = String::from("graph LR\n");

    // Sort modules for deterministic output
    let mut modules: Vec<&String> = graph.keys().collect();
    modules.sort();

    for module in &modules {
        out.push_str(&format!("    {module}[{module}]\n"));
    }

    for module in &modules {
        if let Some(node) = graph.get(*module) {
            let mut deps: Vec<&String> = node.declared_deps.iter().collect();
            deps.sort();
            for dep in deps {
                if graph.contains_key(dep) {
                    out.push_str(&format!("    {module} --> {dep}\n"));
                } else {
                    out.push_str(&format!("    {module} -.-> {dep}[\"❌ {dep}\"]\n"));
                }
            }
        }
    }

    out
}

/// Render the dependency graph as a Graphviz DOT diagram.
fn render_dot(graph: &std::collections::HashMap<String, deps::DepNode>) -> String {
    let mut out =
        String::from("digraph specs {\n    rankdir=LR;\n    node [shape=box, style=rounded];\n\n");

    let mut modules: Vec<&String> = graph.keys().collect();
    modules.sort();

    for module in &modules {
        out.push_str(&format!("    \"{module}\";\n"));
    }

    out.push('\n');

    for module in &modules {
        if let Some(node) = graph.get(*module) {
            let mut deps: Vec<&String> = node.declared_deps.iter().collect();
            deps.sort();
            for dep in deps {
                if graph.contains_key(dep) {
                    out.push_str(&format!("    \"{module}\" -> \"{dep}\";\n"));
                } else {
                    out.push_str(&format!(
                        "    \"{dep}\" [style=dashed, color=red];\n    \"{module}\" -> \"{dep}\" [style=dashed, color=red];\n"
                    ));
                }
            }
        }
    }

    out.push_str("}\n");
    out
}
