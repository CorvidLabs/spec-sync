//! Cross-module dependency validation.
//!
//! Parses `depends_on` declarations from spec frontmatter, builds a dependency
//! graph, validates that declared dependencies actually exist, detects circular
//! dependency chains, and cross-references declared dependencies against actual
//! import statements in source code (Rust, TypeScript, Python, Kotlin).

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

/// Kotlin `import com.example.core.Core`, `import com.example.core.*`,
/// `import com.example.core.Core as Alias`. Only a statement that starts its own
/// line (leading whitespace aside) is an import; the alias tail is not captured.
static KOTLIN_IMPORT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[ \t]*import[ \t]+([A-Za-z_`][^\s;]*)").unwrap());

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

// ─── Import Analysis ────────────────────────────────────────────────────

/// The import dialects this module can actually read.
///
/// This enum is the single source of truth for "can `deps` analyse this file?".
/// Before it existed, the dispatch in `extract_imports` fell through to an empty
/// set for every other language, and `check_undeclared_imports` could not tell
/// "this file declares no cross-module imports" from "nobody parsed this file"
/// — the second read as the first, and `deps --strict` called an unexamined
/// graph valid (#477). Any new language extractor belongs here, so the
/// disclosure in `DepsReport::unanalyzed_languages` stays correct by
/// construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportDialect {
    Rust,
    TypeScript,
    Python,
    Kotlin,
}

impl ImportDialect {
    /// The dialect for a file, or `None` when no extractor exists for it.
    fn for_path(file_path: &Path) -> Option<Self> {
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        Self::for_language(Language::from_extension(ext)?)
    }

    fn for_language(language: Language) -> Option<Self> {
        match language {
            Language::Rust => Some(Self::Rust),
            Language::TypeScript => Some(Self::TypeScript),
            Language::Python => Some(Self::Python),
            Language::Kotlin => Some(Self::Kotlin),
            _ => None,
        }
    }

    fn extract(self, content: &str) -> HashSet<String> {
        match self {
            Self::Rust => extract_rust_imports(content),
            Self::TypeScript => extract_ts_imports(content),
            Self::Python => extract_python_imports(content),
            Self::Kotlin => extract_kotlin_imports(content),
        }
    }
}

/// Does this language name other units of code from source, so that an
/// unparsed file could be hiding an undeclared dependency?
///
/// Only such a language belongs in `DepsReport::unanalyzed_languages`. A YAML
/// file has no imports to miss, and disclosing it as "not analysed" turns a
/// real gap ("your Go imports were never read") into noise that a pure-Rust
/// project sees on every run just for listing `ci.yml` in its spec. Shell is
/// excluded on the same ground: `source`/`.` splices a file that is named by
/// path and, being a source file, is already visible to spec `files:` — there
/// is no module namespace to cross-reference against `depends_on`.
///
/// The match is exhaustive on purpose: a new `Language` variant must not
/// silently inherit either answer.
fn language_has_import_concept(language: Language) -> bool {
    match language {
        // Each of these names another unit of code from source — `use`,
        // `import`, `require`, `using`, `open`, `#include`, `#import`,
        // `library`, `Import-Module` — so an unparsed file may hide an
        // undeclared dependency. The four dialects with extractors are listed
        // too; they never reach the tally, but leaving them out would make this
        // read as "not analysable".
        Language::Rust
        | Language::TypeScript
        | Language::Python
        | Language::Kotlin
        | Language::Go
        | Language::Swift
        | Language::Java
        | Language::CSharp
        | Language::Dart
        | Language::Php
        | Language::Ruby
        | Language::C
        | Language::Cpp
        | Language::Scala
        | Language::Crystal
        | Language::Nim
        | Language::Erlang
        | Language::Elixir
        | Language::Perl
        | Language::Lisp
        | Language::Haskell
        | Language::Lua
        | Language::R
        | Language::OCaml
        | Language::Groovy
        | Language::FSharp
        | Language::Clojure
        | Language::D
        | Language::ObjectiveC
        | Language::PowerShell
        | Language::Vala => true,
        // No import construct at all: nothing was missed by not parsing these.
        Language::Yaml | Language::Bash => false,
    }
}

/// Extract what a source file imports, in the terms its language uses: a module
/// token for Rust, TypeScript and Python, whose spec module IS the token; a
/// package path (`com.example.core`) for Kotlin, which `validate_deps` must
/// still resolve to an owning spec module via `jvm_package_owners`.
///
/// Returns an empty set for a language with no extractor; callers that gate on
/// the result must consult `ImportDialect::for_path` to tell that case apart
/// from a file that genuinely imports nothing.
pub fn extract_imports(file_path: &Path, content: &str) -> HashSet<String> {
    match ImportDialect::for_path(file_path) {
        Some(dialect) => dialect.extract(content),
        None => HashSet::new(),
    }
}

fn extract_rust_imports(content: &str) -> HashSet<String> {
    let stripped = strip_rust_non_code(content);
    let mut modules = HashSet::new();

    for caps in RUST_USE.captures_iter(&stripped) {
        if let Some(m) = caps.get(1) {
            modules.insert(m.as_str().to_string());
        }
    }
    for caps in RUST_MOD.captures_iter(&stripped) {
        if let Some(m) = caps.get(1) {
            modules.insert(m.as_str().to_string());
        }
    }

    modules
}

/// Replace Rust comments and string/byte-string literals with spaces while
/// preserving line breaks and byte offsets. Rust block comments can nest, and
/// raw strings can use any number of `#` delimiters, so regex stripping would
/// either miss valid forms or consume adjacent code.
fn strip_rust_non_code(content: &str) -> String {
    let bytes = content.as_bytes();
    let mut output = bytes.to_vec();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index..].starts_with(b"//") {
            let start = index;
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            blank_span(&mut output, start, index);
            continue;
        }
        if bytes[index..].starts_with(b"/*") {
            let start = index;
            index += 2;
            let mut depth = 1_usize;
            while index < bytes.len() && depth > 0 {
                if bytes[index..].starts_with(b"/*") {
                    depth += 1;
                    index += 2;
                } else if bytes[index..].starts_with(b"*/") {
                    depth -= 1;
                    index += 2;
                } else {
                    index += 1;
                }
            }
            blank_span(&mut output, start, index);
            continue;
        }

        if rust_token_boundary(bytes, index)
            && let Some((quote, hashes)) = rust_raw_string_start(bytes, index)
        {
            let start = index;
            index = quote + 1;
            while index < bytes.len() {
                if bytes[index] == b'"'
                    && index + 1 + hashes <= bytes.len()
                    && bytes[index + 1..index + 1 + hashes]
                        .iter()
                        .all(|byte| *byte == b'#')
                {
                    index += 1 + hashes;
                    break;
                }
                index += 1;
            }
            blank_span(&mut output, start, index);
            continue;
        }

        if rust_token_boundary(bytes, index) {
            let quote = if bytes[index] == b'"' {
                Some(index)
            } else if bytes[index..].starts_with(b"b\"") {
                Some(index + 1)
            } else {
                None
            };
            if let Some(quote) = quote {
                let start = index;
                index = quote + 1;
                while index < bytes.len() {
                    if bytes[index] == b'\\' {
                        index = (index + 2).min(bytes.len());
                    } else if bytes[index] == b'"' {
                        index += 1;
                        break;
                    } else {
                        index += 1;
                    }
                }
                blank_span(&mut output, start, index);
                continue;
            }
        }

        index += 1;
    }

    // Only ASCII bytes are replaced, so the original UTF-8 boundaries remain.
    String::from_utf8(output).unwrap_or_default()
}

fn rust_token_boundary(bytes: &[u8], index: usize) -> bool {
    index == 0 || !bytes[index - 1].is_ascii_alphanumeric() && bytes[index - 1] != b'_'
}

/// Return `(opening_quote_index, delimiter_hash_count)` for `r"..."`,
/// `r###"..."###`, `br#"..."#`, and the accepted `rb#"..."#` spelling.
fn rust_raw_string_start(bytes: &[u8], index: usize) -> Option<(usize, usize)> {
    let mut cursor = index;
    if bytes
        .get(cursor..cursor + 2)
        .is_some_and(|prefix| prefix == b"br" || prefix == b"rb")
    {
        cursor += 2;
    } else if bytes.get(cursor) == Some(&b'r') {
        cursor += 1;
    } else {
        return None;
    }
    let hashes_start = cursor;
    while bytes.get(cursor) == Some(&b'#') {
        cursor += 1;
    }
    (bytes.get(cursor) == Some(&b'"')).then_some((cursor, cursor - hashes_start))
}

fn blank_span(output: &mut [u8], start: usize, end: usize) {
    for byte in &mut output[start..end] {
        if *byte != b'\n' && *byte != b'\r' {
            *byte = b' ';
        }
    }
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

/// Extract the PACKAGES a Kotlin file imports from — `import com.example.core.Core`
/// yields `com.example.core`. Package paths (not bare tokens) are returned because
/// a Kotlin module's spec name is not a segment of the import on its own:
/// `check_undeclared_imports` maps the package to the spec that owns those files
/// (see `jvm_package_owners`).
fn extract_kotlin_imports(content: &str) -> HashSet<String> {
    let stripped = strip_jvm_non_code(content);
    let mut packages = HashSet::new();

    for caps in KOTLIN_IMPORT.captures_iter(&stripped) {
        if let Some(path) = caps.get(1)
            && let Some(package) = kotlin_import_package(path.as_str())
        {
            packages.insert(package);
        }
    }

    packages
}

/// The package an `import` path names, with the imported declaration — and any
/// enclosing type names — removed. A Kotlin import always ends in a declaration
/// or `*`, never in the package itself.
///
/// `com.example.core.Core` → `com.example.core`
/// `com.example.core.Core.Nested` → `com.example.core`
/// `com.example.core.*` → `com.example.core`
/// `com.example.core.helper` (top-level function) → `com.example.core`
/// `Foo` → `None` (no package to attribute)
fn kotlin_import_package(path: &str) -> Option<String> {
    let segments: Vec<&str> = path
        .trim_end_matches(';')
        .split('.')
        .map(|segment| segment.trim_matches('`'))
        .filter(|segment| !segment.is_empty())
        .collect();

    // The final segment is the imported declaration (or `*`) — never a package.
    let mut end = segments.len().checked_sub(1)?;
    if segments.get(end) != Some(&"*") {
        // Nested types: walk back over capitalized owners (`...core.Core.Nested`).
        while end > 0
            && segments[end - 1]
                .chars()
                .next()
                .is_some_and(char::is_uppercase)
        {
            end -= 1;
        }
    }

    (end > 0).then(|| segments[..end].join("."))
}

/// Replace JVM-family comments and string literals with spaces, preserving line
/// breaks and byte offsets, so a commented-out or quoted `import`/`package` line
/// cannot be mistaken for a real declaration. Kotlin and Scala block comments
/// nest, and raw strings (`"""…"""`) span lines, so regex stripping would either
/// miss valid forms or consume adjacent code. Java and Groovy are a subset of
/// the same syntax, which is why one stripper serves all four.
fn strip_jvm_non_code(content: &str) -> String {
    let bytes = content.as_bytes();
    let mut output = bytes.to_vec();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index..].starts_with(b"//") {
            let start = index;
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            blank_span(&mut output, start, index);
            continue;
        }
        if bytes[index..].starts_with(b"/*") {
            let start = index;
            index += 2;
            let mut depth = 1_usize;
            while index < bytes.len() && depth > 0 {
                if bytes[index..].starts_with(b"/*") {
                    depth += 1;
                    index += 2;
                } else if bytes[index..].starts_with(b"*/") {
                    depth -= 1;
                    index += 2;
                } else {
                    index += 1;
                }
            }
            blank_span(&mut output, start, index);
            continue;
        }
        if bytes[index..].starts_with(b"\"\"\"") {
            let start = index;
            index += 3;
            while index < bytes.len() {
                if bytes[index..].starts_with(b"\"\"\"") {
                    index += 3;
                    break;
                }
                index += 1;
            }
            blank_span(&mut output, start, index);
            continue;
        }
        if bytes[index] == b'"' {
            let start = index;
            index += 1;
            while index < bytes.len() {
                if bytes[index] == b'\\' {
                    index = (index + 2).min(bytes.len());
                } else if bytes[index] == b'"' {
                    index += 1;
                    break;
                } else if bytes[index] == b'\n' {
                    // An unterminated single-quoted string cannot cross a line;
                    // stop rather than blanking the rest of the file.
                    break;
                } else {
                    index += 1;
                }
            }
            blank_span(&mut output, start, index);
            continue;
        }

        index += 1;
    }

    // Only ASCII bytes are replaced, so the original UTF-8 boundaries remain.
    String::from_utf8(output).unwrap_or_default()
}

/// Does this file belong to a JVM language, whose sources declare a package and
/// share one package namespace? A Kotlin file may well import a package whose
/// spec owns `.java` sources, so all four count.
fn is_jvm_source(normalized_path: &str) -> bool {
    matches!(
        Path::new(normalized_path)
            .extension()
            .and_then(|extension| extension.to_str())
            .and_then(Language::from_extension),
        Some(Language::Kotlin | Language::Java | Language::Scala | Language::Groovy)
    )
}

/// The package a JVM source file declares in its own header, or `None` when it
/// declares none (default package, or a file whose header we cannot read).
///
/// This is what makes resolution independent of where the file sits on disk:
/// a directory only *usually* ends with the package path, and when it does not,
/// guessing from the directory silently yields no owner at all. Comments and
/// strings are stripped first, blank and annotation lines (`@file:JvmName`,
/// which Kotlin requires before `package`) are skipped, and Scala's chained
/// clauses (`package com.example` then `package core`) nest into one path.
fn jvm_declared_package(content: &str) -> Option<String> {
    let stripped = strip_jvm_non_code(content);
    let mut segments: Vec<String> = Vec::new();

    for line in stripped.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('@') {
            continue;
        }
        // The first line that is not a package clause ends the header.
        let Some(rest) = line.strip_prefix("package") else {
            break;
        };
        if !rest.starts_with([' ', '\t']) {
            break;
        }
        let Some(token) = rest.split_whitespace().next() else {
            break;
        };
        // Scala's `package object foo` declares a member of the enclosing
        // package rather than another clause.
        if token == "object" {
            break;
        }
        for segment in token.trim_end_matches([';', '{']).split('.') {
            let segment = segment.trim_matches('`');
            if !segment.is_empty() {
                segments.push(segment.to_string());
            }
        }
    }

    (!segments.is_empty()).then(|| segments.join("."))
}

/// The JVM package topology of the project as the specs describe it.
struct JvmPackages {
    /// Package path → the one spec module that owns it.
    owners: HashMap<String, String>,
    /// Every package path the project is known to occupy, and every prefix of
    /// one. Used to tell a package we failed to attribute from a package that
    /// was never ours (`java.util`, `kotlinx.coroutines`).
    namespace: HashSet<String>,
}

/// How an imported package relates to the project's spec modules.
enum PackageOwner {
    /// One spec module owns this package, or the nearest enclosing one.
    Module(String),
    /// The package lies outside every namespace this project occupies — a
    /// standard-library or third-party dependency. It maps to no spec module,
    /// and that is the right answer, not a failure.
    Foreign,
    /// The package looks like this project's own code, but no spec module could
    /// be shown to own it. The analysis ran and came up short, which is NOT the
    /// same as an import that maps to nothing (#477); the caller discloses it.
    Unattributed,
}

/// How many leading segments must match for a package to count as this
/// project's own. One is too few — `com.google.common` and `com.example.core`
/// share `com`, and `java.com` appears in every `src/main/java/com/...` layout.
/// Two separates an organisation's namespace from a foreign one.
const PROJECT_NAMESPACE_SEGMENTS: usize = 2;

/// Map JVM package paths to the spec modules that own them. Mirrors
/// `rust_module_owners`: source topology (`com.example.core`) stays distinct
/// from spec naming (`core`) with no hard-coded aliases.
///
/// Two sources feed the map, in order of authority:
///
/// 1. the `package` statement each declared source file makes about itself —
///    correct regardless of layout, including source roots where the directory
///    does not end with the package path;
/// 2. the directory the file sits in, every suffix of it, which still answers
///    for files that are listed in a spec but missing, unreadable, or written
///    in the default package.
///
/// A package claimed by two modules is ambiguous and is left unowned rather
/// than guessed — an import of it then resolves to `Unattributed` and is
/// disclosed, instead of vanishing.
fn jvm_package_owners(root: &Path, graph: &HashMap<String, DepNode>) -> JvmPackages {
    let mut declared: HashMap<String, HashSet<String>> = HashMap::new();
    let mut layout: HashMap<String, HashSet<String>> = HashMap::new();

    for node in graph.values() {
        for file in &node.files {
            let normalized = file.replace('\\', "/");
            if !is_jvm_source(&normalized) {
                continue;
            }

            if let Some((directory, _)) = normalized.rsplit_once('/') {
                let segments: Vec<&str> = directory
                    .split('/')
                    .filter(|segment| !segment.is_empty() && *segment != ".")
                    .collect();
                for start in 0..segments.len() {
                    layout
                        .entry(segments[start..].join("."))
                        .or_default()
                        .insert(node.module.clone());
                }
            }

            // Reading is skipped for paths that escape the project root (see
            // `validator::source_within_root`); an unreadable file is left to
            // the hard error `check_undeclared_imports` already raises for it.
            if crate::validator::source_within_root(root, file)
                && let Ok(content) = fs::read_to_string(root.join(file))
                && let Some(package) = jvm_declared_package(&content)
            {
                declared
                    .entry(package)
                    .or_default()
                    .insert(node.module.clone());
            }
        }
    }

    fn sole(modules: &HashSet<String>) -> Option<&String> {
        (modules.len() == 1)
            .then(|| modules.iter().next())
            .flatten()
    }

    let mut owners: HashMap<String, String> = HashMap::new();
    for (package, modules) in &declared {
        if let Some(module) = sole(modules) {
            owners.insert(package.clone(), module.clone());
        }
    }
    for (package, modules) in &layout {
        // What a file says about itself outranks the directory it sits in.
        if declared.contains_key(package) {
            continue;
        }
        if let Some(module) = sole(modules) {
            owners.insert(package.clone(), module.clone());
        }
    }

    let mut namespace: HashSet<String> = HashSet::new();
    for package in declared.keys().chain(layout.keys()) {
        let segments: Vec<&str> = package.split('.').collect();
        for end in 1..=segments.len() {
            namespace.insert(segments[..end].join("."));
        }
    }

    JvmPackages { owners, namespace }
}

/// Resolve an imported JVM package against the project's package topology,
/// longest package prefix first, so `com.example.core.internal` still resolves
/// to the spec owning `com.example.core`.
///
/// The three outcomes are deliberately distinct. Collapsing `Foreign` and
/// `Unattributed` into "no owner" is what let an unresolved import disappear
/// into a `filter_map` and leave `deps --strict` calling the remaining graph
/// valid (#477): an import the tool could not map to a module is not the same
/// as an import that correctly maps to none.
fn resolve_jvm_package(package: &str, packages: &JvmPackages) -> PackageOwner {
    let segments: Vec<&str> = package
        .split('.')
        .filter(|segment| !segment.is_empty())
        .collect();
    if segments.is_empty() {
        return PackageOwner::Foreign;
    }

    for end in (1..=segments.len()).rev() {
        if let Some(owner) = packages.owners.get(&segments[..end].join(".")) {
            return PackageOwner::Module(owner.clone());
        }
    }

    if packages.namespace.is_empty() {
        // Nothing at all is known about this project's packages, so nothing can
        // honestly be called foreign. Disclose rather than discard.
        return PackageOwner::Unattributed;
    }

    let probe = segments[..segments.len().min(PROJECT_NAMESPACE_SEGMENTS)].join(".");
    if packages.namespace.contains(&probe) {
        PackageOwner::Unattributed
    } else {
        PackageOwner::Foreign
    }
}

/// Check that imports in source files match declared dependencies.
fn check_undeclared_imports(
    root: &Path,
    graph: &HashMap<String, DepNode>,
    report: &mut DepsReport,
) {
    let known_modules: HashSet<&str> = graph.keys().map(|k| k.as_str()).collect();
    let rust_owners = rust_module_owners(graph);
    let jvm_packages = jvm_package_owners(root, graph);
    // Languages whose declared source files were read but never parsed for
    // imports. Counted, not discarded: their absence from `undeclared_imports`
    // is "not analysed", and the caller has to say so.
    let mut unanalyzed: HashMap<Language, usize> = HashMap::new();
    // Imports that were collected and then could not be attributed to a module.
    // Same rule, one level down: the edge they might have implied is missing
    // from the graph, so the caller has to say so rather than filter them out.
    let mut unresolved: Vec<(String, String)> = Vec::new();

    for node in graph.values() {
        let declared: HashSet<&str> = node.declared_deps.iter().map(|d| d.as_str()).collect();
        let mut actual_imports: HashSet<String> = HashSet::new();

        for file in &node.files {
            // Skip `files:` entries that escape the project root — reading their
            // imports would probe arbitrary host files (see validator::source_within_root).
            if !crate::validator::source_within_root(root, file) {
                continue;
            }
            let full_path = root.join(file);
            let content = match fs::read_to_string(&full_path) {
                Ok(c) => c,
                // A declared source file that can't be read as UTF-8 silently
                // contributed no imports, so `deps --strict` could pass while
                // hiding real undeclared-import violations. Fail loud instead —
                // mirrors the validator's source-read policy (validator.rs) and,
                // because cmd_deps exits 1 on any error, gates CI consistently.
                Err(err) => {
                    report.errors.push(format!(
                        "{}: source file `{}` could not be read as UTF-8 for dependency analysis: {}",
                        node.spec_path, file, err
                    ));
                    continue;
                }
            };
            let file_imports = extract_imports(&full_path, &content);
            match ImportDialect::for_path(&full_path) {
                // Source topology → spec naming, per dialect.
                Some(ImportDialect::Rust) => actual_imports.extend(
                    file_imports
                        .into_iter()
                        .map(|import| rust_owners.get(&import).cloned().unwrap_or(import)),
                ),
                Some(ImportDialect::Kotlin) => {
                    for package in file_imports {
                        match resolve_jvm_package(&package, &jvm_packages) {
                            PackageOwner::Module(owner) => {
                                actual_imports.insert(owner);
                            }
                            // Correctly maps to no spec module: a standard
                            // library or third-party package. Nothing to say.
                            PackageOwner::Foreign => {}
                            // Maps to nothing only because attribution failed.
                            // Disclosed instead of dropped (#477).
                            PackageOwner::Unattributed => {
                                unresolved.push((node.module.clone(), package));
                            }
                        }
                    }
                }
                Some(ImportDialect::TypeScript | ImportDialect::Python) => {
                    actual_imports.extend(file_imports)
                }
                // No extractor: this file contributed nothing because nobody
                // parsed it. Record the language so the verdict can say so
                // instead of counting silence as cleanliness (#477) — but only
                // for a language that has imports to miss in the first place.
                None => {
                    if let Some(language) = full_path
                        .extension()
                        .and_then(|extension| extension.to_str())
                        .and_then(Language::from_extension)
                        .filter(|language| language_has_import_concept(*language))
                    {
                        *unanalyzed.entry(language).or_insert(0) += 1;
                    }
                }
            }
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

    let mut unanalyzed: Vec<(String, usize)> = unanalyzed
        .into_iter()
        .map(|(language, files)| (format!("{language:?}"), files))
        .collect();
    unanalyzed.sort();
    report.unanalyzed_languages = unanalyzed;

    unresolved.sort();
    unresolved.dedup();
    report.unresolved_imports = unresolved;
}

/// Map a top-level Rust module token to the spec that owns its canonical module
/// entry file. This keeps source topology (`crate::cli`) distinct from spec
/// naming (`cli_args`) without hard-coded aliases.
fn rust_module_owners(graph: &HashMap<String, DepNode>) -> HashMap<String, String> {
    let mut candidates: HashMap<String, HashSet<String>> = HashMap::new();
    for node in graph.values() {
        for file in &node.files {
            let normalized = file.replace('\\', "/");
            let Some(relative) = normalized.strip_prefix("src/") else {
                continue;
            };
            let module = if let Some(stem) = relative.strip_suffix(".rs")
                && !stem.contains('/')
                && !matches!(stem, "main" | "lib" | "mod")
            {
                Some(stem)
            } else if let Some(module) = relative.strip_suffix("/mod.rs")
                && !module.contains('/')
            {
                Some(module)
            } else {
                None
            };
            if let Some(module) = module {
                candidates
                    .entry(module.to_string())
                    .or_default()
                    .insert(node.module.clone());
            }
        }
    }
    candidates
        .into_iter()
        .filter_map(|(module, owners)| {
            if owners.len() == 1 {
                owners.into_iter().next().map(|owner| (module, owner))
            } else {
                None
            }
        })
        .collect()
}

/// One sentence naming the languages whose imports could not be read, or `None`
/// when every declared source file that could have imports was analysable.
/// Single formatting site for every renderer, so a language cannot be disclosed
/// in one output format and hidden in another.
///
/// Only languages that HAVE imports appear here (see
/// `language_has_import_concept`): a note that fires for YAML and shell files
/// says nothing true and drowns the cases where something really was missed.
pub fn unanalyzed_languages_note(report: &DepsReport) -> Option<String> {
    if report.unanalyzed_languages.is_empty() {
        return None;
    }
    let listed = report
        .unanalyzed_languages
        .iter()
        .map(|(language, files)| format!("{language} ({files} file(s))"))
        .collect::<Vec<_>>()
        .join(", ");
    Some(format!(
        "Import analysis is not implemented for {listed}; \
         undeclared imports in those files were not checked"
    ))
}

/// One sentence naming the imports that were read but could not be attributed
/// to a spec module, or `None` when every collected import was either owned or
/// recognisably foreign. Single formatting site, like
/// `unanalyzed_languages_note`.
///
/// The count is exact; the listing is capped so a large project cannot bury the
/// rest of the report. Machine consumers get the full list in the JSON
/// `unresolved_imports` array.
pub fn unresolved_imports_note(report: &DepsReport) -> Option<String> {
    const LISTED: usize = 5;
    if report.unresolved_imports.is_empty() {
        return None;
    }
    let total = report.unresolved_imports.len();
    let listed = report
        .unresolved_imports
        .iter()
        .take(LISTED)
        .map(|(module, import)| format!("{module} imports {import}"))
        .collect::<Vec<_>>()
        .join(", ");
    let remainder = total.saturating_sub(LISTED);
    let tail = if remainder > 0 {
        format!(", +{remainder} more")
    } else {
        String::new()
    };
    Some(format!(
        "{total} import(s) could not be mapped to a spec module, so they were \
         not checked against depends_on: {listed}{tail}"
    ))
}

/// The success sentence, qualified when part of the tree went unread or
/// unattributed. "Valid" over a graph the command could not fully build is a
/// claim it has not earned (#477).
pub fn valid_declarations_line(report: &DepsReport) -> &'static str {
    match (
        report.unanalyzed_languages.is_empty(),
        report.unresolved_imports.is_empty(),
    ) {
        (true, true) => "All dependency declarations are valid.",
        (false, true) => "All dependency declarations are valid for the languages analysed.",
        (true, false) => "All dependency declarations are valid for the imports that resolved.",
        (false, false) => {
            "All dependency declarations are valid for the languages analysed \
             and the imports that resolved."
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
        out.push_str(valid_declarations_line(report));
        out.push('\n');
        for note in [
            unanalyzed_languages_note(report),
            unresolved_imports_note(report),
        ]
        .into_iter()
        .flatten()
        {
            out.push_str(&format!("{note}\n"));
        }
        return out;
    }

    for note in [
        unanalyzed_languages_note(report),
        unresolved_imports_note(report),
    ]
    .into_iter()
    .flatten()
    {
        out.push_str(&format!("{note}\n"));
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
    fn test_flow_style_depends_on_counts_edges() {
        // `depends_on: [b]` (the flow style `specsync new` scaffolds) must
        // produce a real edge, not silently parse as zero dependencies.
        let tmp = TempDir::new().unwrap();
        create_spec(tmp.path(), "b", &[], &[]);
        let spec_dir = tmp.path().join("specs").join("a");
        fs::create_dir_all(&spec_dir).unwrap();
        fs::write(
            spec_dir.join("a.spec.md"),
            "---\nmodule: a\nversion: 1\nstatus: active\nfiles: []\ndepends_on: [b]\n---\n\n# A\n\n## Purpose\nTest\n",
        )
        .unwrap();

        let report = validate_deps(tmp.path(), "specs");
        assert_eq!(report.edge_count, 1, "report: {report:?}");
        assert!(report.errors.is_empty(), "errors: {:?}", report.errors);
    }

    #[test]
    fn test_deps_rejects_escapes_and_missing_with_same_verdict_as_check() {
        let tmp = TempDir::new().unwrap();
        let spec_dir = tmp.path().join("specs").join("a");
        fs::create_dir_all(&spec_dir).unwrap();
        fs::write(
            spec_dir.join("a.spec.md"),
            "---\nmodule: a\nversion: 1\nstatus: active\nfiles: []\ndepends_on: [/etc/passwd, nosuchmod]\n---\n\n# A\n\n## Purpose\nTest\n",
        )
        .unwrap();

        let report = validate_deps(tmp.path(), "specs");
        assert!(
            report
                .errors
                .iter()
                .any(|e| e.contains("escapes the project root") && e.contains("/etc/passwd")),
            "errors: {:?}",
            report.errors
        );
        assert!(
            report
                .errors
                .iter()
                .any(|e| e.contains("Dependency spec not found: nosuchmod")),
            "errors: {:?}",
            report.errors
        );
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
    fn test_extract_rust_imports_ignores_comments_and_all_string_forms() {
        let imports = extract_rust_imports(
            r#####"
// use crate::line_comment;
/* use crate::block_comment;
   /* mod nested_comment; */
*/
const ORDINARY: &str = "use crate::ordinary; mod ordinary_mod; \"escaped\"";
const BYTE: &[u8] = b"use crate::byte_string; mod byte_mod; \\";
const RAW: &str = r"use crate::raw_string; mod raw_mod;";
const RAW_HASH: &str = r###"use crate::raw_hash; "## mod raw_hash_mod;"###;
const BYTE_RAW: &[u8] = br##"use crate::byte_raw; mod byte_raw_mod;"##;
const RAW_BYTE: &[u8] = rb#"use crate::raw_byte; mod raw_byte_mod;"#;

use crate::real_import::Value;
pub mod real_module;
"#####,
        );

        assert_eq!(
            imports,
            HashSet::from(["real_import".to_string(), "real_module".to_string()])
        );
    }

    #[test]
    fn test_extract_rust_imports_preserves_code_after_nested_raw_hashes_and_escapes() {
        let imports = extract_rust_imports(
            r####"
const FIRST: &str = r##"embedded "# and // and /* markers"##;
const SECOND: &[u8] = b"escaped quote: \"; use crate::not_code;";
use crate::after_literals;
/* outer /* use crate::nested; */ still comment */
mod after_comment;
"####,
        );

        assert!(imports.contains("after_literals"));
        assert!(imports.contains("after_comment"));
        assert_eq!(imports.len(), 2);
    }

    #[test]
    fn test_rust_imports_map_to_source_owning_spec_module() {
        let tmp = TempDir::new().unwrap();
        create_source(
            tmp.path(),
            "src/commands.rs",
            "use crate::cli::Command;\npub fn run() {}\n",
        );
        create_source(tmp.path(), "src/cli.rs", "pub struct Command;\n");
        create_spec(tmp.path(), "commands", &[], &["src/commands.rs"]);
        create_spec(tmp.path(), "cli_args", &[], &["src/cli.rs"]);
        create_spec(tmp.path(), "cli", &[], &["src/main.rs"]);
        create_source(tmp.path(), "src/main.rs", "fn main() {}\n");

        let report = validate_deps(tmp.path(), "specs");
        assert!(
            report
                .undeclared_imports
                .contains(&("commands".to_string(), "cli_args".to_string())),
            "expected crate::cli to resolve to cli_args: {:?}",
            report.undeclared_imports
        );
        assert!(
            !report
                .undeclared_imports
                .contains(&("commands".to_string(), "cli".to_string()))
        );
    }

    #[test]
    fn test_rust_mod_rs_import_maps_to_source_owning_spec_module() {
        let tmp = TempDir::new().unwrap();
        create_source(
            tmp.path(),
            "src/caller.rs",
            "use crate::config::Settings;\n",
        );
        create_source(tmp.path(), "src/config/mod.rs", "pub struct Settings;\n");
        create_spec(tmp.path(), "caller", &[], &["src/caller.rs"]);
        create_spec(tmp.path(), "settings", &[], &["src/config/mod.rs"]);

        let report = validate_deps(tmp.path(), "specs");
        assert!(
            report
                .undeclared_imports
                .contains(&("caller".to_string(), "settings".to_string()))
        );
    }

    #[test]
    fn test_extract_kotlin_imports_packages() {
        let imports = extract_kotlin_imports(
            "package com.example.feature\n\n\
             import com.example.core.Core\n\
             import com.example.util.*\n\
             import com.example.nested.Outer.Inner\n\
             import com.example.top.helper\n\
             import com.example.alias.Thing as Renamed\n\
             import Bare\n",
        );

        assert_eq!(
            imports,
            HashSet::from([
                "com.example.core".to_string(),
                "com.example.util".to_string(),
                "com.example.nested".to_string(),
                "com.example.top".to_string(),
                "com.example.alias".to_string(),
            ]),
            "a Kotlin import names a declaration, so the package is everything before it"
        );
    }

    #[test]
    fn test_extract_kotlin_imports_ignores_comments_and_strings() {
        let imports = extract_kotlin_imports(
            "package com.example.feature\n\
             // import com.example.linecomment.Thing\n\
             /* import com.example.block.Thing\n\
                /* import com.example.nestedblock.Thing */\n\
             */\n\
             val doc = \"\"\"\nimport com.example.raw.Thing\n\"\"\"\n\
             val line = \"import com.example.quoted.Thing\"\n\
             import com.example.real.Thing\n",
        );

        assert_eq!(imports, HashSet::from(["com.example.real".to_string()]));
    }

    #[test]
    fn test_undeclared_kotlin_import() {
        // #477: Kotlin imports were never collected, so `deps` called an
        // unexamined graph valid.
        let tmp = TempDir::new().unwrap();
        create_source(
            tmp.path(),
            "src/main/kotlin/com/example/core/Core.kt",
            "package com.example.core\n\nclass Core\n",
        );
        create_source(
            tmp.path(),
            "src/main/kotlin/com/example/feature/Feature.kt",
            "package com.example.feature\n\nimport com.example.core.Core\n\nclass Feature(val core: Core)\n",
        );
        create_spec(
            tmp.path(),
            "core",
            &[],
            &["src/main/kotlin/com/example/core/Core.kt"],
        );
        create_spec(
            tmp.path(),
            "feature",
            &[],
            &["src/main/kotlin/com/example/feature/Feature.kt"],
        );

        let report = validate_deps(tmp.path(), "specs");
        assert!(
            report
                .undeclared_imports
                .contains(&("feature".to_string(), "core".to_string())),
            "expected com.example.core to resolve to the spec owning it: {:?}",
            report.undeclared_imports
        );
        assert!(
            report.unanalyzed_languages.is_empty(),
            "Kotlin is analysable now: {:?}",
            report.unanalyzed_languages
        );
    }

    #[test]
    fn test_declared_kotlin_import_not_flagged() {
        let tmp = TempDir::new().unwrap();
        create_source(
            tmp.path(),
            "src/main/kotlin/com/example/core/Core.kt",
            "package com.example.core\n\nclass Core\n",
        );
        create_source(
            tmp.path(),
            "src/main/kotlin/com/example/feature/Feature.kt",
            "package com.example.feature\n\nimport com.example.core.Core\n",
        );
        create_spec(
            tmp.path(),
            "core",
            &[],
            &["src/main/kotlin/com/example/core/Core.kt"],
        );
        create_spec(
            tmp.path(),
            "feature",
            &["specs/core/core.spec.md"],
            &["src/main/kotlin/com/example/feature/Feature.kt"],
        );

        let report = validate_deps(tmp.path(), "specs");
        assert!(
            report.undeclared_imports.is_empty(),
            "a declared dependency must not be flagged: {:?}",
            report.undeclared_imports
        );
    }

    #[test]
    fn test_kotlin_import_of_subpackage_maps_to_owning_spec() {
        // `com.example.core.internal` has no spec of its own; the longest
        // package prefix that does own files wins.
        let tmp = TempDir::new().unwrap();
        create_source(
            tmp.path(),
            "src/main/kotlin/com/example/core/Core.kt",
            "package com.example.core\n\nclass Core\n",
        );
        create_source(
            tmp.path(),
            "src/main/kotlin/com/example/feature/Feature.kt",
            "package com.example.feature\n\nimport com.example.core.internal.Detail\n",
        );
        create_spec(
            tmp.path(),
            "core",
            &[],
            &["src/main/kotlin/com/example/core/Core.kt"],
        );
        create_spec(
            tmp.path(),
            "feature",
            &[],
            &["src/main/kotlin/com/example/feature/Feature.kt"],
        );

        let report = validate_deps(tmp.path(), "specs");
        assert!(
            report
                .undeclared_imports
                .contains(&("feature".to_string(), "core".to_string())),
            "expected the subpackage import to resolve to core: {:?}",
            report.undeclared_imports
        );
    }

    #[test]
    fn test_kotlin_third_party_package_is_not_mistaken_for_a_spec_module() {
        // `java.util` must not resolve to the spec module named `util` just
        // because they share a last segment: the project's package topology is
        // known and `java.util` is not part of it.
        let tmp = TempDir::new().unwrap();
        create_source(
            tmp.path(),
            "src/main/kotlin/com/example/util/Util.kt",
            "package com.example.util\n\nclass Util\n",
        );
        create_source(
            tmp.path(),
            "src/main/kotlin/com/example/feature/Feature.kt",
            "package com.example.feature\n\nimport java.util.UUID\n",
        );
        create_spec(
            tmp.path(),
            "util",
            &[],
            &["src/main/kotlin/com/example/util/Util.kt"],
        );
        create_spec(
            tmp.path(),
            "feature",
            &[],
            &["src/main/kotlin/com/example/feature/Feature.kt"],
        );

        let report = validate_deps(tmp.path(), "specs");
        assert!(
            report.undeclared_imports.is_empty(),
            "java.util is not the spec module `util`: {:?}",
            report.undeclared_imports
        );
        // And it is NOT an attribution failure either: a third-party package
        // correctly maps to no spec module, so disclosing it would be noise.
        assert!(
            report.unresolved_imports.is_empty(),
            "java.util is foreign, not unattributed: {:?}",
            report.unresolved_imports
        );
    }

    #[test]
    fn test_kotlin_flat_layout_resolves_by_declared_package() {
        // No file sits in a package directory, so the directory tells us nothing
        // about package ownership. The `package` statement each file declares
        // does, and that is what resolution uses.
        let tmp = TempDir::new().unwrap();
        create_source(
            tmp.path(),
            "Core.kt",
            "package com.example.core\n\nclass Core\n",
        );
        create_source(
            tmp.path(),
            "Feature.kt",
            "package com.example.feature\n\nimport com.example.core.Core\n",
        );
        create_spec(tmp.path(), "core", &[], &["Core.kt"]);
        create_spec(tmp.path(), "feature", &[], &["Feature.kt"]);

        let report = validate_deps(tmp.path(), "specs");
        assert!(
            report
                .undeclared_imports
                .contains(&("feature".to_string(), "core".to_string())),
            "expected the flat-layout import to still be attributed: {:?}",
            report.undeclared_imports
        );
        assert!(
            report.unresolved_imports.is_empty(),
            "nothing was left unattributed: {:?}",
            report.unresolved_imports
        );
    }

    #[test]
    fn test_kotlin_import_resolves_when_directory_does_not_match_package() {
        // #477, one level down: the source root is `src/kt`, which does not end
        // with either file's package path. Matching a package against directory
        // suffixes finds no owner here — and a resolution that finds no owner
        // used to be dropped, leaving an empty report and exit 0.
        let tmp = TempDir::new().unwrap();
        create_source(
            tmp.path(),
            "src/kt/Core.kt",
            "package com.example.core\n\nclass Core\n",
        );
        create_source(
            tmp.path(),
            "src/kt/Feature.kt",
            "package com.example.feature\n\nimport com.example.core.Core\n",
        );
        create_spec(tmp.path(), "core", &[], &["src/kt/Core.kt"]);
        create_spec(tmp.path(), "feature", &[], &["src/kt/Feature.kt"]);

        let report = validate_deps(tmp.path(), "specs");
        assert!(
            report
                .undeclared_imports
                .contains(&("feature".to_string(), "core".to_string())),
            "the declared package must resolve regardless of directory: {:?}",
            report.undeclared_imports
        );
        assert!(
            report.unresolved_imports.is_empty(),
            "nothing was left unattributed: {:?}",
            report.unresolved_imports
        );
    }

    #[test]
    fn test_kotlin_import_of_project_package_with_no_owner_is_disclosed() {
        // The import was collected and is plainly this project's own namespace,
        // but no spec owns `com.example.internal`. Silently dropping it is the
        // #477 defect: no edge, no finding, exit 0. It must be disclosed.
        let tmp = TempDir::new().unwrap();
        create_source(
            tmp.path(),
            "src/main/kotlin/com/example/feature/Feature.kt",
            "package com.example.feature\n\nimport com.example.internal.Detail\n",
        );
        create_spec(
            tmp.path(),
            "feature",
            &[],
            &["src/main/kotlin/com/example/feature/Feature.kt"],
        );

        let report = validate_deps(tmp.path(), "specs");
        assert_eq!(
            report.unresolved_imports,
            vec![("feature".to_string(), "com.example.internal".to_string())],
            "an import the tool could not map must be reported, not filtered away"
        );
        assert_eq!(
            valid_declarations_line(&report),
            "All dependency declarations are valid for the imports that resolved.",
            "the success sentence must not claim more than was checked"
        );
        let note = unresolved_imports_note(&report).expect("the gap must be disclosed");
        assert!(
            note.contains("feature imports com.example.internal"),
            "got: {note}"
        );
        // Disclosure is advisory: it is not an error and does not gate.
        assert!(report.errors.is_empty() && report.warnings.is_empty());
    }

    #[test]
    fn test_ambiguous_kotlin_package_is_disclosed_not_guessed() {
        // Two specs own files in `com.example.core`, so no single module owns
        // the package. Guessing one would be wrong; dropping the import in
        // silence is the bug. Say it could not be mapped.
        let tmp = TempDir::new().unwrap();
        create_source(
            tmp.path(),
            "src/main/kotlin/com/example/core/One.kt",
            "package com.example.core\n\nclass One\n",
        );
        create_source(
            tmp.path(),
            "src/main/kotlin/com/example/core/Two.kt",
            "package com.example.core\n\nclass Two\n",
        );
        create_source(
            tmp.path(),
            "src/main/kotlin/com/example/feature/Feature.kt",
            "package com.example.feature\n\nimport com.example.core.One\n",
        );
        create_spec(
            tmp.path(),
            "core_one",
            &[],
            &["src/main/kotlin/com/example/core/One.kt"],
        );
        create_spec(
            tmp.path(),
            "core_two",
            &[],
            &["src/main/kotlin/com/example/core/Two.kt"],
        );
        create_spec(
            tmp.path(),
            "feature",
            &[],
            &["src/main/kotlin/com/example/feature/Feature.kt"],
        );

        let report = validate_deps(tmp.path(), "specs");
        assert_eq!(
            report.unresolved_imports,
            vec![("feature".to_string(), "com.example.core".to_string())],
            "an ambiguous package is unattributed, and unattributed is disclosed"
        );
    }

    #[test]
    fn test_kotlin_import_resolves_to_spec_owning_java_sources() {
        // A Kotlin file importing a package whose spec owns `.java` sources still
        // produces the edge — JVM siblings share one package namespace.
        let tmp = TempDir::new().unwrap();
        create_source(
            tmp.path(),
            "src/main/java/com/example/core/Core.java",
            "package com.example.core;\n\npublic class Core {}\n",
        );
        create_source(
            tmp.path(),
            "src/main/kotlin/com/example/feature/Feature.kt",
            "package com.example.feature\n\nimport com.example.core.Core\n",
        );
        create_spec(
            tmp.path(),
            "core",
            &[],
            &["src/main/java/com/example/core/Core.java"],
        );
        create_spec(
            tmp.path(),
            "feature",
            &[],
            &["src/main/kotlin/com/example/feature/Feature.kt"],
        );

        let report = validate_deps(tmp.path(), "specs");
        assert!(
            report
                .undeclared_imports
                .contains(&("feature".to_string(), "core".to_string())),
            "expected the Java-owned package to be attributed: {:?}",
            report.undeclared_imports
        );
        // Java itself still has no import extractor, and that is disclosed.
        assert_eq!(report.unanalyzed_languages, vec![("Java".to_string(), 1)]);
    }

    #[test]
    fn test_kotlin_self_package_import_not_flagged() {
        let tmp = TempDir::new().unwrap();
        create_source(
            tmp.path(),
            "src/main/kotlin/com/example/core/Core.kt",
            "package com.example.core\n\nclass Core\n",
        );
        create_source(
            tmp.path(),
            "src/main/kotlin/com/example/core/Helper.kt",
            "package com.example.core\n\nimport com.example.core.Core\n",
        );
        create_spec(
            tmp.path(),
            "core",
            &[],
            &[
                "src/main/kotlin/com/example/core/Core.kt",
                "src/main/kotlin/com/example/core/Helper.kt",
            ],
        );

        let report = validate_deps(tmp.path(), "specs");
        assert!(
            report.undeclared_imports.is_empty(),
            "a module must not depend on itself: {:?}",
            report.undeclared_imports
        );
    }

    #[test]
    fn test_unanalyzed_language_is_disclosed_not_counted_as_clean() {
        // A language with no import extractor contributes no edges; the report
        // must say so instead of letting the silence read as "no problems".
        let tmp = TempDir::new().unwrap();
        create_source(tmp.path(), "src/core/core.go", "package core\n");
        create_source(
            tmp.path(),
            "src/feature/feature.go",
            "package feature\n\nimport \"example.com/src/core\"\n",
        );
        create_spec(tmp.path(), "core", &[], &["src/core/core.go"]);
        create_spec(tmp.path(), "feature", &[], &["src/feature/feature.go"]);

        let report = validate_deps(tmp.path(), "specs");
        assert_eq!(report.unanalyzed_languages, vec![("Go".to_string(), 2)]);
        assert_eq!(
            valid_declarations_line(&report),
            "All dependency declarations are valid for the languages analysed."
        );
        let note = unanalyzed_languages_note(&report).expect("Go must be disclosed");
        assert!(note.contains("Go (2 file(s))"), "got: {note}");
        // Disclosure is not a failure: the exit code is untouched.
        assert!(report.errors.is_empty() && report.warnings.is_empty());
    }

    #[test]
    fn test_both_disclosures_qualify_the_verdict_together() {
        // Unread languages and unattributable imports are separate gaps, and a
        // tree with both must own up to both rather than pick one sentence.
        let tmp = TempDir::new().unwrap();
        create_source(tmp.path(), "src/core/core.go", "package core\n");
        create_source(
            tmp.path(),
            "src/kt/Feature.kt",
            "package com.example.feature\n\nimport com.example.missing.Gone\n",
        );
        create_spec(tmp.path(), "core", &[], &["src/core/core.go"]);
        create_spec(tmp.path(), "feature", &[], &["src/kt/Feature.kt"]);

        let report = validate_deps(tmp.path(), "specs");
        assert_eq!(report.unanalyzed_languages, vec![("Go".to_string(), 1)]);
        assert_eq!(
            report.unresolved_imports,
            vec![("feature".to_string(), "com.example.missing".to_string())]
        );
        assert_eq!(
            valid_declarations_line(&report),
            "All dependency declarations are valid for the languages analysed \
             and the imports that resolved."
        );
        assert!(unanalyzed_languages_note(&report).is_some());
        assert!(unresolved_imports_note(&report).is_some());
        assert!(report.errors.is_empty() && report.warnings.is_empty());
    }

    #[test]
    fn test_analyzable_tree_reports_no_unanalyzed_languages() {
        let tmp = TempDir::new().unwrap();
        create_source(tmp.path(), "src/api.py", "from .auth import login\n");
        create_spec(tmp.path(), "api", &[], &["src/api.py"]);
        create_spec(tmp.path(), "auth", &[], &[]);

        let report = validate_deps(tmp.path(), "specs");
        assert!(report.unanalyzed_languages.is_empty());
        assert_eq!(
            valid_declarations_line(&report),
            "All dependency declarations are valid."
        );
        assert!(unanalyzed_languages_note(&report).is_none());
    }

    #[test]
    fn test_language_without_an_import_concept_is_not_disclosed() {
        // A YAML file has no imports to miss, and a shell script names a path
        // rather than a module. Listing them as "not analysed" would bury the
        // real disclosure under noise every pure-Rust project sees.
        let tmp = TempDir::new().unwrap();
        create_source(tmp.path(), "src/lib.rs", "pub fn run() {}\n");
        create_source(tmp.path(), "src/ci.yml", "jobs: {}\n");
        create_source(tmp.path(), "src/tool.sh", "#!/usr/bin/env bash\ntrue\n");
        create_spec(
            tmp.path(),
            "tooling",
            &[],
            &["src/lib.rs", "src/ci.yml", "src/tool.sh"],
        );

        let report = validate_deps(tmp.path(), "specs");
        assert!(
            report.unanalyzed_languages.is_empty(),
            "neither YAML nor shell has an import to miss: {:?}",
            report.unanalyzed_languages
        );
        assert!(unanalyzed_languages_note(&report).is_none());
        assert_eq!(
            valid_declarations_line(&report),
            "All dependency declarations are valid.",
            "nothing was skipped, so the verdict must not be hedged"
        );
    }

    #[test]
    fn test_jvm_declared_package_reads_the_header() {
        assert_eq!(
            jvm_declared_package("package com.example.core\n\nclass Core\n"),
            Some("com.example.core".to_string())
        );
        assert_eq!(
            jvm_declared_package("package com.example.core;\n\npublic class Core {}\n"),
            Some("com.example.core".to_string()),
            "Java terminates the clause with a semicolon"
        );
        assert_eq!(
            jvm_declared_package("@file:JvmName(\"CoreKt\")\n\npackage com.example.core\n"),
            Some("com.example.core".to_string()),
            "Kotlin file annotations precede the package clause"
        );
        assert_eq!(
            jvm_declared_package("// package com.example.wrong\npackage com.example.core\n"),
            Some("com.example.core".to_string()),
            "a commented-out clause is not a declaration"
        );
        assert_eq!(
            jvm_declared_package("package com.example\npackage core\n\nclass Core\n"),
            Some("com.example.core".to_string()),
            "Scala chains clauses, and they nest"
        );
        assert_eq!(
            jvm_declared_package("class Core\n\npackage com.example.core\n"),
            None,
            "only the header declares the package"
        );
        assert_eq!(jvm_declared_package("class Core\n"), None);
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
