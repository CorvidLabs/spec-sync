use colored::Colorize;
use std::path::Path;
use std::process;

use crate::config::load_config;
use crate::deps;
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
            if report.errors.is_empty() && report.warnings.is_empty() {
                println!("All dependency declarations are valid.");
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
                println!("\n  {} All dependency declarations are valid.", "✓".green());
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
        let enforcement =
            enforcement.unwrap_or_else(|| default_enforcement(&config));
        let code = compute_exit_code(0, 0, strict, enforcement, &coverage, Some(req));
        if format != types::OutputFormat::Json {
            if code == 0 {
                println!(
                    "  {} Coverage {}% meets --require-coverage {req}%",
                    "✓".green(),
                    coverage.coverage_percent
                );
            } else {
                eprintln!(
                    "{} {req}%: actual coverage is {}% ({} file(s) missing specs)",
                    "--require-coverage".red(),
                    coverage.coverage_percent,
                    coverage.unspecced_files.len()
                );
            }
        }
        if code != 0 {
            process::exit(code);
        }
    }
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
