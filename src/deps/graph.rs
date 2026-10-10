//! Dependency graph construction, validation, cycle detection and ordering.

use super::imports::check_undeclared_imports;
use super::{DepNode, DepsReport};
use crate::parser::parse_frontmatter;
use crate::validator::{find_spec_files, is_cross_project_ref};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

// ─── Graph Construction ─────────────────────────────────────────────────

/// Build the dependency graph from all spec files in the project.
pub fn build_dep_graph(root: &Path, specs_dir: &str) -> HashMap<String, DepNode> {
    build_dep_graph_checked(root, specs_dir).0
}

/// Like `build_dep_graph`, but also returns error messages for spec files that
/// EXIST yet could not be read as UTF-8. Silently dropping such a spec removes its
/// node from the graph, which defeats cycle detection and missing-dependency
/// checks for that module — so the validation path (`validate_deps`) surfaces
/// these as hard errors rather than losing them. Kept internal; `build_dep_graph`
/// preserves the original signature for the non-gating callers (visualization,
/// topological order).
fn build_dep_graph_checked(
    root: &Path,
    specs_dir: &str,
) -> (HashMap<String, DepNode>, Vec<String>) {
    let specs_path = root.join(specs_dir);
    let spec_files = find_spec_files(&specs_path);
    let mut graph: HashMap<String, DepNode> = HashMap::new();
    let mut unreadable: Vec<String> = Vec::new();

    for spec_file in &spec_files {
        let content = match fs::read_to_string(spec_file) {
            Ok(c) => c.replace("\r\n", "\n"),
            Err(err) => {
                let rel = spec_file
                    .strip_prefix(root)
                    .unwrap_or(spec_file)
                    .to_string_lossy()
                    .to_string();
                unreadable.push(format!(
                    "{rel}: spec file could not be read as UTF-8; dependency analysis skipped this spec: {err}"
                ));
                continue;
            }
        };

        let rel = spec_file
            .strip_prefix(root)
            .unwrap_or(spec_file)
            .to_string_lossy()
            .to_string();

        // Every `continue` below drops a spec out of the graph. Silently, they
        // made `deps` answer a question it had not asked: a malformed
        // `depends_on` parsed to an empty list, contributed no edges, and the
        // command then affirmatively reported "All dependency declarations are
        // valid" over frontmatter that `check` rejects outright (#550). Absence
        // of input is not absence of problems — say which specs were dropped.
        let parsed = match parse_frontmatter(&content) {
            Some(p) => p,
            None => {
                unreadable.push(format!(
                    "{rel}: frontmatter could not be parsed; dependency analysis skipped this spec"
                ));
                continue;
            }
        };

        // A spec whose frontmatter parsed but whose `depends_on` is malformed
        // would otherwise contribute an empty edge set indistinguishable from a
        // module that genuinely declares nothing.
        for error in &parsed.errors {
            unreadable.push(format!("{rel}: {error}"));
        }

        let module_name = match &parsed.frontmatter.module {
            Some(m) => m.clone(),
            None => {
                unreadable.push(format!(
                    "{rel}: frontmatter declares no `module`; dependency analysis skipped this spec"
                ));
                continue;
            }
        };

        let spec_path = spec_file
            .strip_prefix(root)
            .unwrap_or(spec_file)
            .to_string_lossy()
            .to_string();

        // Extract module names from depends_on paths.
        // Paths like "specs/types/types.spec.md" → module name "types"
        // Cross-project refs are skipped here.
        // Every entry gets the SAME verdict as `check`/`resolve`: escapes,
        // missing targets, and unresolvable shapes are hard errors, not
        // silently dropped edges.
        let mut declared_deps: Vec<String> = Vec::new();
        for dep in &parsed.frontmatter.depends_on {
            if is_cross_project_ref(dep) {
                continue;
            }
            match crate::validator::validate_local_dependency(dep, root, specs_dir) {
                Ok(_) => match extract_module_from_dep_path(dep) {
                    Some(module) => declared_deps.push(module),
                    None => unreadable.push(format!(
                        "{spec_path}: dependency entry `{dep}` does not name a module (expected a bare module name or a path ending in .spec.md)"
                    )),
                },
                Err(message) => {
                    // Keep the edge so missing_deps still names the target.
                    if let Some(module) = extract_module_from_dep_path(dep) {
                        declared_deps.push(module);
                    }
                    unreadable.push(format!("{spec_path}: {message}"));
                }
            }
        }

        graph.insert(
            module_name.clone(),
            DepNode {
                module: module_name,
                spec_path,
                declared_deps,
                files: parsed.frontmatter.files,
            },
        );
    }

    (graph, unreadable)
}

/// Extract a module name from a dependency path.
/// `specs/types/types.spec.md` -> `types`
/// `specs/parser/parser.spec.md` -> `parser`
/// Also handles bare module names like `types`.
fn extract_module_from_dep_path(dep: &str) -> Option<String> {
    let path = Path::new(dep);

    // If it ends with .spec.md, extract the stem
    if let Some(name) = path.file_name().and_then(|n| n.to_str())
        && let Some(stem) = name.strip_suffix(".spec.md")
    {
        return Some(stem.to_string());
    }

    // Bare module name (no path separators, no extension)
    if !dep.contains('/') && !dep.contains('.') {
        return Some(dep.to_string());
    }

    None
}

// ─── Validation ─────────────────────────────────────────────────────────

/// Validate the entire dependency graph.
pub fn validate_deps(root: &Path, specs_dir: &str) -> DepsReport {
    let (graph, unreadable_specs) = build_dep_graph_checked(root, specs_dir);
    let mut report = DepsReport::default();
    // A spec that exists but couldn't be read was dropped from the graph; record it
    // as a hard error so cmd_deps exits 1 instead of silently under-validating.
    report.errors.extend(unreadable_specs);

    let known_modules: HashSet<&str> = graph.keys().map(|k| k.as_str()).collect();
    report.module_count = graph.len();

    // Count edges and check for missing dependencies
    for node in graph.values() {
        for dep in &node.declared_deps {
            report.edge_count += 1;
            if !known_modules.contains(dep.as_str()) {
                report.missing_deps.push((node.module.clone(), dep.clone()));
                report.errors.push(format!(
                    "{}: depends on '{}' but no spec exists for that module",
                    node.spec_path, dep
                ));
            }
        }
    }

    // Detect circular dependencies
    report.cycles = detect_cycles(&graph);
    for cycle in &report.cycles {
        let chain = cycle.join(" -> ");
        report.errors.push(format!("Circular dependency: {chain}"));
    }

    // Cross-reference imports in source code against declared deps
    check_undeclared_imports(root, &graph, &mut report);

    report
}

// ─── Cycle Detection ────────────────────────────────────────────────────

/// Detect all cycles in the dependency graph using DFS with coloring.
fn detect_cycles(graph: &HashMap<String, DepNode>) -> Vec<Vec<String>> {
    #[derive(Clone, Copy, PartialEq)]
    enum Color {
        White,
        Gray,
        Black,
    }

    let mut colors: HashMap<&str, Color> = HashMap::new();
    let mut path: Vec<String> = Vec::new();
    let mut cycles: Vec<Vec<String>> = Vec::new();

    for key in graph.keys() {
        colors.insert(key.as_str(), Color::White);
    }

    fn dfs<'a>(
        node: &'a str,
        graph: &'a HashMap<String, DepNode>,
        colors: &mut HashMap<&'a str, Color>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        colors.insert(node, Color::Gray);
        path.push(node.to_string());

        if let Some(dep_node) = graph.get(node) {
            for dep in &dep_node.declared_deps {
                match colors.get(dep.as_str()) {
                    Some(Color::Gray) => {
                        // Found a cycle — extract the cycle from path
                        if let Some(start) = path.iter().position(|p| p == dep) {
                            let mut cycle: Vec<String> = path[start..].to_vec();
                            cycle.push(dep.clone());
                            cycles.push(cycle);
                        }
                    }
                    Some(Color::White) | None => {
                        if graph.contains_key(dep.as_str()) {
                            dfs(dep, graph, colors, path, cycles);
                        }
                    }
                    Some(Color::Black) => {}
                }
            }
        }

        path.pop();
        colors.insert(node, Color::Black);
    }

    for key in graph.keys() {
        if colors.get(key.as_str()) == Some(&Color::White) {
            dfs(key, graph, &mut colors, &mut path, &mut cycles);
        }
    }

    cycles
}

/// Build a topological ordering of modules (if DAG is valid).
/// Returns None if the graph contains cycles.
pub fn topological_sort(graph: &HashMap<String, DepNode>) -> Option<Vec<String>> {
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    for key in graph.keys() {
        in_degree.entry(key.as_str()).or_insert(0);
    }
    for node in graph.values() {
        for dep in &node.declared_deps {
            if graph.contains_key(dep.as_str()) {
                *in_degree.entry(dep.as_str()).or_insert(0) += 0;
                // dep is depended on by node, so node has incoming from dep perspective
                // Actually: node depends on dep, so node's "depends on" is an edge node -> dep
                // For topological sort we need: dep must come before node
                // in_degree counts how many modules a module depends on (must be built first)
            }
        }
    }

    // in_degree[m] = number of modules that m depends on (that exist in graph)
    for node in graph.values() {
        let count = node
            .declared_deps
            .iter()
            .filter(|d| graph.contains_key(d.as_str()))
            .count();
        in_degree.insert(node.module.as_str(), count);
    }

    let mut queue: Vec<&str> = in_degree
        .iter()
        .filter(|(_, deg)| **deg == 0)
        .map(|(&k, _)| k)
        .collect();
    queue.sort(); // deterministic ordering

    let mut order: Vec<String> = Vec::new();

    while let Some(current) = queue.pop() {
        order.push(current.to_string());

        // Find modules that depend on `current` and decrement their in-degree
        for node in graph.values() {
            if node.declared_deps.iter().any(|d| d == current) {
                let deg = in_degree.get_mut(node.module.as_str()).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push(node.module.as_str());
                    queue.sort(); // keep deterministic
                }
            }
        }
    }

    if order.len() == graph.len() {
        Some(order)
    } else {
        None // cycles exist
    }
}

#[cfg(test)]
mod tests;
