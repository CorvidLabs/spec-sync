//! Cross-module dependency validation.
//!
//! Parses `depends_on` declarations from spec frontmatter, builds a dependency
//! graph, validates that declared dependencies actually exist, detects circular
//! dependency chains, and cross-references declared dependencies against actual
//! import statements in source code (Rust, TypeScript, Python, Kotlin).

mod graph;
mod imports;
mod report;

pub use graph::{build_dep_graph, topological_sort, validate_deps};
#[allow(unused_imports)] // Preserve the existing facade API for non-command consumers.
pub use imports::extract_imports;
#[allow(unused_imports)] // Preserve the existing facade API for non-command consumers.
pub use report::format_report;
pub use report::{unanalyzed_languages_note, unresolved_imports_note, valid_declarations_line};

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
    /// Declared source files whose language has an import concept this tool
    /// cannot yet parse, as `(language, file count)` sorted by language. Their
    /// imports were never collected, so an empty `undeclared_imports` for those
    /// files means "nobody looked", not "nothing to report" — callers must
    /// disclose this rather than let it read as a clean bill of health.
    ///
    /// A language with no import construct at all (YAML, shell) is NOT listed:
    /// there is nothing there to miss, and naming it would bury the real
    /// disclosure in noise.
    pub unanalyzed_languages: Vec<(String, usize)>,
    /// Imports that WERE collected but could not be mapped to a spec module, as
    /// `(module, imported package)` sorted and deduplicated. Distinct from an
    /// import that maps to nothing: the analysis ran, produced a package, and
    /// then failed to attribute it — so the edge it might have implied is
    /// missing from the graph. Dropping these silently is the same defect as
    /// never collecting them (#477), so they are reported rather than filtered
    /// away. Third-party packages that correctly belong to no spec module
    /// (`java.util`, `kotlinx.coroutines`) are not listed here.
    pub unresolved_imports: Vec<(String, String)>,
}

#[cfg(test)]
mod test_support;
