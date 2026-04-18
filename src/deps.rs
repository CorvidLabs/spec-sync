//! Cross-module dependency validation.
//!
//! Parses `depends_on` declarations from spec frontmatter, builds a dependency
//! graph, validates that declared dependencies actually exist, detects circular
//! dependency chains, and cross-references declared dependencies against actual
//! import statements in source code (Rust, TypeScript, Python).

use crate::parser::parse_frontmatter;
use crate::types::Language;
use crate::validator::{find_spec_files, is_cross_project_ref};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

// ─── Import Extraction Regexes ──────────────────────────────────────────

/// Rust `use crate::module` or `mod module;`
static RUST_USE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(?:pub\s+)?use\s+(?:crate::)?(\w+)").unwrap());
static RUST_MOD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(?:pub\s+)?mod\s+(\w+)\s*[;{]").unwrap());

/// TypeScript/JavaScript `import ... from './module'` or `require('./module')`
static TS_IMPORT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?m)(?:import\s+.*?\s+from\s+|require\s*\(\s*)['"]\.?\.?/?([^'"./][^'"]*)['"]"#)
        .unwrap()
});

/// Python `import module` or `from module import ...` (relative: `from .module`)
static PY_IMPORT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(?:from\s+\.?(\w+)|import\s+(\w+))").unwrap());

// ─── Types ──────────────────────────────────────────────────────────────

/// A node in the dependency graph, representing one spec module.
#[derive(Debug, Clone)]
pub struct DepNode {
    /// Module name from frontmatter.
    pub module: String,
    /// Relative path to the spec file.
    pub spec_path: String,
    /// Declared dependencies (module names extracted from `depends_on` paths).
    pub declared_deps: Vec<String>,
    /// Source files listed in frontmatter.
    pub files: Vec<String>,
}

/// Result of cross-module dependency validation.
#[derive(Debug, Default)]
pub struct DepsReport {
    /// Errors: declared dep not found, circular deps, etc.
    pub errors: Vec<String>,
    /// Warnings: undeclared imports, etc.
    pub warnings: Vec<String>,
    /// Informational: total modules, edges, etc.
    pub module_count: usize,
    pub edge_count: usize,
    /// Circular dependency chains found.
    pub cycles: Vec<Vec<String>>,
    /// Dependencies declared in spec but the target module doesn't exist.
    pub missing_deps: Vec<(String, String)>,
    /// Imports found in source code but not declared in spec depends_on.
    pub undeclared_imports: Vec<(String, String)>,
}

// ─── Graph Construction ─────────────────────────────────────────────────

/// Build the dependency graph from all spec files in the project.
pub fn build_dep_graph(root: &Path, specs_dir: &str) -> HashMap<String, DepNode> {
    let specs_path = root.join(specs_dir);
    let spec_files = find_spec_files(&specs_path);
    let mut graph: HashMap<String, DepNode> = HashMap::new();

    for spec_file in &spec_files {
        let content = match fs::read_to_string(spec_file) {
            Ok(c) => c.replace("\r\n", "\n"),
            Err(_) => continue,
        };

        let parsed = match parse_frontmatter(&content) {
            Some(p) => p,
            None => continue,
        };

        let module_name = match &parsed.frontmatter.module {
            Some(m) => m.clone(),
            None => continue,
        };

        let spec_path = spec_file
            .strip_prefix(root)
            .unwrap_or(spec_file)
            .to_string_lossy()
            .to_string();

        // Extract module names from depends_on paths.
        // Paths like "specs/types/types.spec.md" → module name "types"
        // Cross-project refs are skipped here.
        let declared_deps: Vec<String> = parsed
            .frontmatter
            .depends_on
            .iter()
            .filter(|d| !is_cross_project_ref(d))
            .filter_map(|d| extract_module_from_dep_path(d))
            .collect();

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

    graph
}

/// Extract a module name from a dependency path.
/// `specs/types/types.spec.md` -> `types`
/// `specs/parser/parser.spec.md` -> `parser`
/// Also handles bare module names like `types`.
fn extract_module_from_dep_path(dep: &str) -> Option<String> {
    let path = Path::new(dep);

    // If it ends with .spec.md, extract the stem
    if let Some(name) = path.file_name().and_then(|n| n.to_str())
        && let Some(stem) = name.strip_suffix(".spec.md") {
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
    let graph = build_dep_graph(root, specs_dir);
    let mut report = DepsReport::default();

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

// ─── Import Analysis ────────────────────────────────────────────────────

/// Extract imported module names from a source file based on language.
pub fn extract_imports(file_path: &Path, content: &str) -> HashSet<String> {
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let lang = match Language::from_extension(ext) {
        Some(l) => l,
        None => return HashSet::new(),
    };

    match lang {
        Language::Rust => extract_rust_imports(content),
        Language::TypeScript => extract_ts_imports(content),
        Language::Python => extract_python_imports(content),
        _ => HashSet::new(),
    }
}

fn extract_rust_imports(content: &str) -> HashSet<String> {
    let mut modules = HashSet::new();

    for caps in RUST_USE.captures_iter(content) {
        if let Some(m) = caps.get(1) {
            modules.insert(m.as_str().to_string());
        }
    }
    for caps in RUST_MOD.captures_iter(content) {
        if let Some(m) = caps.get(1) {
            modules.insert(m.as_str().to_string());
        }
    }

    modules
}

fn extract_ts_imports(content: &str) -> HashSet<String> {
    let mut modules = HashSet::new();

    for caps in TS_IMPORT.captures_iter(content) {
        if let Some(m) = caps.get(1) {
            // Extract just the module name (first path segment)
            let module = m.as_str().split('/').next().unwrap_or(m.as_str());
            modules.insert(module.to_string());
        }
    }

    modules
}

fn extract_python_imports(content: &str) -> HashSet<String> {
    let mut modules = HashSet::new();

    for caps in PY_IMPORT.captures_iter(content) {
        if let Some(m) = caps.get(1) {
            modules.insert(m.as_str().to_string());
        } else if let Some(m) = caps.get(2) {
            modules.insert(m.as_str().to_string());
        }
    }

    modules
}

/// Check that imports in source files match declared dependencies.
fn check_undeclared_imports(
    root: &Path,
    graph: &HashMap<String, DepNode>,
    report: &mut DepsReport,
) {
    let known_modules: HashSet<&str> = graph.keys().map(|k| k.as_str()).collect();

    for node in graph.values() {
        let declared: HashSet<&str> = node.declared_deps.iter().map(|d| d.as_str()).collect();
        let mut actual_imports: HashSet<String> = HashSet::new();

        for file in &node.files {
            let full_path = root.join(file);
            let content = match fs::read_to_string(&full_path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let file_imports = extract_imports(&full_path, &content);
            actual_imports.extend(file_imports);
        }

        // Only flag imports that correspond to known spec modules
        // and are not already declared.
        for import in &actual_imports {
            if known_modules.contains(import.as_str())
                && !declared.contains(import.as_str())
                && import != &node.module
            {
                report
                    .undeclared_imports
                    .push((node.module.clone(), import.clone()));
                report.warnings.push(format!(
                    "{}: source imports '{}' but it is not in depends_on",
                    node.spec_path, import
                ));
            }
        }
    }
}

/// Format the dependency report as a printable summary.
#[allow(dead_code)]
pub fn format_report(report: &DepsReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Modules: {}  Edges: {}\n",
        report.module_count, report.edge_count
    ));

    if report.errors.is_empty() && report.warnings.is_empty() {
        out.push_str("All dependency declarations are valid.\n");
        return out;
    }

    if !report.errors.is_empty() {
        out.push_str(&format!("\nErrors ({}):\n", report.errors.len()));
        for e in &report.errors {
            out.push_str(&format!("  - {e}\n"));
        }
    }

    if !report.warnings.is_empty() {
        out.push_str(&format!("\nWarnings ({}):\n", report.warnings.len()));
        for w in &report.warnings {
            out.push_str(&format!("  - {w}\n"));
        }
    }

    out
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

// ─── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper: create a spec file in the temp dir.
    fn create_spec(tmp: &Path, module: &str, depends_on: &[&str], files: &[&str]) {
        let spec_dir = tmp.join("specs").join(module);
        fs::create_dir_all(&spec_dir).unwrap();

        let deps_yaml = if depends_on.is_empty() {
            "depends_on: []".to_string()
        } else {
            let items: String = depends_on.iter().map(|d| format!("  - {d}\n")).collect();
            format!("depends_on:\n{items}")
        };

        let files_yaml = if files.is_empty() {
            "files: []".to_string()
        } else {
            let items: String = files.iter().map(|f| format!("  - {f}\n")).collect();
            format!("files:\n{items}")
        };

        let content = format!(
            "---\nmodule: {module}\nversion: 1\nstatus: active\n{files_yaml}\ndb_tables: []\n{deps_yaml}\n---\n\n# {module}\n\n## Purpose\nTest\n## Public API\n## Invariants\n## Behavioral Examples\n## Error Cases\n## Dependencies\n## Change Log\n"
        );

        fs::write(spec_dir.join(format!("{module}.spec.md")), content).unwrap();
    }

    fn create_source(tmp: &Path, path: &str, content: &str) {
        let full = tmp.join(path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(full, content).unwrap();
    }

    #[test]
    fn test_extract_module_from_dep_path() {
        assert_eq!(
            extract_module_from_dep_path("specs/types/types.spec.md"),
            Some("types".to_string())
        );
        assert_eq!(
            extract_module_from_dep_path("specs/parser/parser.spec.md"),
            Some("parser".to_string())
        );
        assert_eq!(
            extract_module_from_dep_path("types"),
            Some("types".to_string())
        );
        assert_eq!(extract_module_from_dep_path("foo/bar.txt"), None);
    }

    #[test]
    fn test_build_dep_graph_empty() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("specs")).unwrap();
        let graph = build_dep_graph(tmp.path(), "specs");
        assert!(graph.is_empty());
    }

    #[test]
    fn test_build_dep_graph_basic() {
        let tmp = TempDir::new().unwrap();
        create_spec(tmp.path(), "auth", &[], &[]);
        create_spec(tmp.path(), "api", &["specs/auth/auth.spec.md"], &[]);

        let graph = build_dep_graph(tmp.path(), "specs");
        assert_eq!(graph.len(), 2);
        assert!(graph.contains_key("auth"));
        assert!(graph.contains_key("api"));
        assert_eq!(graph["api"].declared_deps, vec!["auth".to_string()]);
        assert!(graph["auth"].declared_deps.is_empty());
    }

    #[test]
    fn test_validate_no_errors() {
        let tmp = TempDir::new().unwrap();
        create_spec(tmp.path(), "types", &[], &[]);
        create_spec(tmp.path(), "parser", &["specs/types/types.spec.md"], &[]);
        create_spec(
            tmp.path(),
            "validator",
            &["specs/types/types.spec.md", "specs/parser/parser.spec.md"],
            &[],
        );

        let report = validate_deps(tmp.path(), "specs");
        assert_eq!(report.module_count, 3);
        assert!(report.errors.is_empty(), "errors: {:?}", report.errors);
        assert!(report.cycles.is_empty());
        assert!(report.missing_deps.is_empty());
    }

    #[test]
    fn test_validate_missing_dep() {
        let tmp = TempDir::new().unwrap();
        create_spec(
            tmp.path(),
            "api",
            &["specs/nonexistent/nonexistent.spec.md"],
            &[],
        );

        let report = validate_deps(tmp.path(), "specs");
        assert_eq!(report.missing_deps.len(), 1);
        assert_eq!(report.missing_deps[0].0, "api");
        assert_eq!(report.missing_deps[0].1, "nonexistent");
        assert!(!report.errors.is_empty());
    }

    #[test]
    fn test_detect_circular_deps() {
        let tmp = TempDir::new().unwrap();
        create_spec(tmp.path(), "a", &["specs/b/b.spec.md"], &[]);
        create_spec(tmp.path(), "b", &["specs/a/a.spec.md"], &[]);

        let report = validate_deps(tmp.path(), "specs");
        assert!(
            !report.cycles.is_empty(),
            "Expected circular dependency, got none"
        );
        assert!(!report.errors.is_empty());
        assert!(
            report.errors.iter().any(|e| e.contains("Circular")),
            "errors: {:?}",
            report.errors
        );
    }

    #[test]
    fn test_detect_three_node_cycle() {
        let tmp = TempDir::new().unwrap();
        create_spec(tmp.path(), "a", &["specs/b/b.spec.md"], &[]);
        create_spec(tmp.path(), "b", &["specs/c/c.spec.md"], &[]);
        create_spec(tmp.path(), "c", &["specs/a/a.spec.md"], &[]);

        let report = validate_deps(tmp.path(), "specs");
        assert!(!report.cycles.is_empty());
    }

    #[test]
    fn test_cross_project_refs_skipped() {
        let tmp = TempDir::new().unwrap();
        create_spec(tmp.path(), "api", &["corvid-labs/algochat@auth"], &[]);

        let report = validate_deps(tmp.path(), "specs");
        // Cross-project refs should not be treated as missing
        assert!(report.missing_deps.is_empty());
        assert!(report.errors.is_empty());
    }

    #[test]
    fn test_undeclared_rust_import() {
        let tmp = TempDir::new().unwrap();
        create_source(
            tmp.path(),
            "src/validator.rs",
            "use crate::parser;\nuse crate::types;\n\npub fn validate() {}\n",
        );
        create_spec(
            tmp.path(),
            "validator",
            &["specs/types/types.spec.md"],
            &["src/validator.rs"],
        );
        create_spec(tmp.path(), "parser", &[], &[]);
        create_spec(tmp.path(), "types", &[], &[]);

        let report = validate_deps(tmp.path(), "specs");
        // validator imports parser but doesn't declare it in depends_on
        assert!(
            report
                .undeclared_imports
                .iter()
                .any(|(m, imp)| m == "validator" && imp == "parser"),
            "Expected undeclared import of parser, got: {:?}",
            report.undeclared_imports
        );
    }

    #[test]
    fn test_undeclared_ts_import() {
        let tmp = TempDir::new().unwrap();
        create_source(
            tmp.path(),
            "src/api.ts",
            "import { Auth } from './auth';\nimport { Types } from './types';\n",
        );
        create_spec(
            tmp.path(),
            "api",
            &["specs/types/types.spec.md"],
            &["src/api.ts"],
        );
        create_spec(tmp.path(), "auth", &[], &[]);
        create_spec(tmp.path(), "types", &[], &[]);

        let report = validate_deps(tmp.path(), "specs");
        assert!(
            report
                .undeclared_imports
                .iter()
                .any(|(m, imp)| m == "api" && imp == "auth"),
            "Expected undeclared import of auth, got: {:?}",
            report.undeclared_imports
        );
    }

    #[test]
    fn test_undeclared_python_import() {
        let tmp = TempDir::new().unwrap();
        create_source(
            tmp.path(),
            "src/api.py",
            "from .auth import login\nimport types\n",
        );
        create_spec(
            tmp.path(),
            "api",
            &["specs/types/types.spec.md"],
            &["src/api.py"],
        );
        create_spec(tmp.path(), "auth", &[], &[]);
        create_spec(tmp.path(), "types", &[], &[]);

        let report = validate_deps(tmp.path(), "specs");
        assert!(
            report
                .undeclared_imports
                .iter()
                .any(|(m, imp)| m == "api" && imp == "auth"),
            "Expected undeclared import of auth, got: {:?}",
            report.undeclared_imports
        );
    }

    #[test]
    fn test_extract_rust_imports() {
        let imports = extract_rust_imports(
            "use crate::parser;\nuse crate::types::Frontmatter;\nmod config;\npub mod exports;\n",
        );
        assert!(imports.contains("parser"));
        assert!(imports.contains("types"));
        assert!(imports.contains("config"));
        assert!(imports.contains("exports"));
    }

    #[test]
    fn test_extract_ts_imports() {
        let imports = extract_ts_imports(
            "import { foo } from './auth';\nimport bar from '../utils';\nconst x = require('config');\n",
        );
        assert!(imports.contains("auth"));
        assert!(imports.contains("config"));
    }

    #[test]
    fn test_extract_python_imports() {
        let imports =
            extract_python_imports("from .auth import login\nimport config\nfrom os import path\n");
        assert!(imports.contains("auth"));
        assert!(imports.contains("config"));
        assert!(imports.contains("os"));
    }

    #[test]
    fn test_topological_sort_valid() {
        let tmp = TempDir::new().unwrap();
        create_spec(tmp.path(), "types", &[], &[]);
        create_spec(tmp.path(), "parser", &["specs/types/types.spec.md"], &[]);
        create_spec(
            tmp.path(),
            "validator",
            &["specs/types/types.spec.md", "specs/parser/parser.spec.md"],
            &[],
        );

        let graph = build_dep_graph(tmp.path(), "specs");
        let order = topological_sort(&graph);
        assert!(order.is_some(), "Expected valid topological sort");
        let order = order.unwrap();

        // types must come before parser and validator
        let types_pos = order.iter().position(|m| m == "types").unwrap();
        let parser_pos = order.iter().position(|m| m == "parser").unwrap();
        let validator_pos = order.iter().position(|m| m == "validator").unwrap();
        assert!(types_pos < parser_pos);
        assert!(types_pos < validator_pos);
        assert!(parser_pos < validator_pos);
    }

    #[test]
    fn test_topological_sort_cycle() {
        let tmp = TempDir::new().unwrap();
        create_spec(tmp.path(), "a", &["specs/b/b.spec.md"], &[]);
        create_spec(tmp.path(), "b", &["specs/a/a.spec.md"], &[]);

        let graph = build_dep_graph(tmp.path(), "specs");
        let order = topological_sort(&graph);
        assert!(order.is_none(), "Expected None for cyclic graph");
    }

    #[test]
    fn test_format_report_clean() {
        let report = DepsReport {
            module_count: 3,
            edge_count: 2,
            ..DepsReport::default()
        };
        let out = format_report(&report);
        assert!(out.contains("Modules: 3"));
        assert!(out.contains("Edges: 2"));
        assert!(out.contains("valid"));
    }

    #[test]
    fn test_format_report_with_errors() {
        let report = DepsReport {
            module_count: 2,
            edge_count: 1,
            errors: vec!["missing dep".to_string()],
            warnings: vec!["undeclared import".to_string()],
            ..DepsReport::default()
        };
        let out = format_report(&report);
        assert!(out.contains("Errors (1)"));
        assert!(out.contains("Warnings (1)"));
    }

    #[test]
    fn test_self_import_not_flagged() {
        // A module importing its own submodules should not flag itself
        let tmp = TempDir::new().unwrap();
        create_source(
            tmp.path(),
            "src/parser.rs",
            "use crate::parser;\n\npub fn parse() {}\n",
        );
        create_spec(tmp.path(), "parser", &[], &["src/parser.rs"]);

        let report = validate_deps(tmp.path(), "specs");
        // Should not warn about parser importing itself
        assert!(
            !report
                .undeclared_imports
                .iter()
                .any(|(m, imp)| m == "parser" && imp == "parser"),
            "Self-import should not be flagged: {:?}",
            report.undeclared_imports
        );
    }

    #[test]
    fn test_declared_import_not_flagged() {
        let tmp = TempDir::new().unwrap();
        create_source(
            tmp.path(),
            "src/validator.rs",
            "use crate::types;\n\npub fn validate() {}\n",
        );
        create_spec(
            tmp.path(),
            "validator",
            &["specs/types/types.spec.md"],
            &["src/validator.rs"],
        );
        create_spec(tmp.path(), "types", &[], &[]);

        let report = validate_deps(tmp.path(), "specs");
        // types is declared, so no warning
        assert!(
            report.undeclared_imports.is_empty(),
            "Declared import should not be flagged: {:?}",
            report.undeclared_imports
        );
    }
}
