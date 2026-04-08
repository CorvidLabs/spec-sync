use colored::Colorize;
use std::path::Path;
use std::process;

use crate::config::load_config;
use crate::deps;
use crate::types;

pub fn cmd_deps(root: &Path, format: types::OutputFormat, mermaid: bool, dot: bool) {
    let config = load_config(root);

    // --mermaid or --dot: output graph visualization and exit
    if mermaid || dot {
        let graph = deps::build_dep_graph(root, &config.specs_dir);
        if mermaid {
            println!("{}", render_mermaid(&graph));
        } else {
            println!("{}", render_dot(&graph));
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
        types::OutputFormat::Text => {
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

    if !report.errors.is_empty() {
        process::exit(1);
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
