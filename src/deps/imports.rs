//! Import dialect dispatch, lexical scanners and source-module ownership.

use super::{DepNode, DepsReport};
use crate::types::Language;
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
pub(super) fn check_undeclared_imports(
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

#[cfg(test)]
mod basic_tests;
#[cfg(test)]
mod jvm_tests;
