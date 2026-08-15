pub mod ast;
mod bash;
mod c;
mod clojure;
mod cpp;
mod crystal;
mod csharp;
mod d;
mod dart;
mod elixir;
mod erlang;
mod fsharp;
mod go;
mod groovy;
mod haskell;
mod java;
mod kotlin;
mod lisp;
mod lua;
mod nim;
mod objective_c;
mod ocaml;
mod perl;
mod php;
mod powershell;
mod python;
mod r;
mod ruby;
mod rust_lang;
mod scala;
mod swift;
mod typescript;
mod vala;
mod yaml;

use crate::types::{ExportLevel, Language, ParseMode};
use std::path::Path;

/// Extract exported symbol names from a source file, auto-detecting language.
/// Uses `ExportLevel::Member` (all symbols) and regex parsing for backwards compatibility.
///
/// NOT for any command that reports on a project: this pins the surface to
/// member/regex regardless of what the project configured, so `score`, `diff`,
/// `new`, and the generators disagreed with `check` about what a module's API
/// even is — under `export_level = "type"` check said `2/2 exports documented`
/// while score deducted for `id`, `name`, `find` (#474). Those callers now pass
/// `config.export_level` / `config.parse_mode` to `get_exported_symbols_full`.
/// Kept for callers that genuinely want the default surface (tests).
#[allow(dead_code)]
pub fn get_exported_symbols(file_path: &Path) -> Vec<String> {
    get_exported_symbols_full(file_path, ExportLevel::Member, ParseMode::Regex)
}

/// Extract exported symbol names from a source file with configurable granularity.
/// When `level` is `Type`, only top-level type declarations are returned.
/// When `level` is `Member`, all public symbols are returned (default).
/// Uses regex parsing for backwards compatibility.
#[allow(dead_code)]
pub fn get_exported_symbols_with_level(file_path: &Path, level: ExportLevel) -> Vec<String> {
    get_exported_symbols_full(file_path, level, ParseMode::Regex)
}

/// The outcome of attempting to extract exported symbols from a source file,
/// distinguishing a genuine "nothing is exported" from a failure to analyze the
/// file. Callers that gate on coverage or export drift (`diff`, `score`) must not
/// treat an unreadable or unsupported file as export-free — doing so silently drops
/// a file's real API from the comparison and reports a false clean result.
#[derive(Debug, Clone, PartialEq)]
pub enum ExportScan {
    /// The file's language was recognized and it was read and parsed. The vector may
    /// be empty, which means the file genuinely exports nothing.
    Parsed(Vec<String>),
    /// The extension is not a recognized source language, so exports cannot be
    /// extracted (e.g. a `.md`/`.sql` file listed in `files:`). Not a failure.
    UnknownLanguage,
    /// The file could not be read — missing, permission-denied, or not valid UTF-8 —
    /// so its exports are unknown. A gate must not treat this as "no exports".
    Unreadable,
    /// The file carries an unresolved merge conflict and the extractor read
    /// declarations from BOTH sides of the same hunk, so the symbol list is the
    /// union of two alternative trees and describes source that does not exist.
    /// A gate must not compare a spec against this — it would pass a tree that
    /// cannot compile.
    Conflicted(ConflictedExtraction),
}

/// Evidence that extraction unioned both sides of a merge conflict.
///
/// Carries the symbols that exist on exactly one side, which is what makes the
/// union bogus and what a report needs in order to be actionable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictedExtraction {
    /// Label on the `<<<<<<<` marker of the offending hunk.
    pub ours_label: String,
    /// Label on the `>>>>>>>` marker of the offending hunk.
    pub theirs_label: String,
    /// Symbols the extractor read that survive only on the `ours` side.
    pub ours_only: Vec<String>,
    /// Symbols the extractor read that survive only on the `theirs` side.
    pub theirs_only: Vec<String>,
}

impl ConflictedExtraction {
    /// A one-line, human-readable account naming both sides and a sample of the
    /// symbols each contributed.
    pub fn describe(&self) -> String {
        format!(
            "exports were read from both sides of an unresolved merge conflict ({} contributes {}; {} contributes {})",
            self.ours_label,
            sample_symbols(&self.ours_only),
            self.theirs_label,
            sample_symbols(&self.theirs_only),
        )
    }
}

/// Render up to three symbol names for a diagnostic, noting any remainder.
fn sample_symbols(symbols: &[String]) -> String {
    const SHOWN: usize = 3;
    let head: Vec<String> = symbols
        .iter()
        .take(SHOWN)
        .map(|s| format!("'{s}'"))
        .collect();
    match symbols.len().saturating_sub(SHOWN) {
        0 => head.join(", "),
        rest => format!("{} (+{rest} more)", head.join(", ")),
    }
}

/// Extract exported symbol names, distinguishing failure from genuine emptiness.
/// Uses `ExportLevel::Member` and regex parsing (matches `get_exported_symbols`).
///
/// Carries the same warning as `get_exported_symbols`: the surface is pinned to
/// member/regex, so no command that grades or reports against a project may use
/// it — `score` and `diff` did, and graded every project against a surface it
/// had not configured (#474). Use `scan_exported_symbols_full` with the
/// project's `export_level`/`parse_mode` instead.
#[allow(dead_code)]
pub fn scan_exported_symbols(file_path: &Path) -> ExportScan {
    scan_exported_symbols_full(file_path, ExportLevel::Member, ParseMode::Regex)
}

/// Like `get_exported_symbols_full`, but returns an `ExportScan` so a read/parse
/// failure or unsupported language is distinguishable from a file that genuinely
/// exports nothing — required by callers that gate on the result.
pub fn scan_exported_symbols_full(
    file_path: &Path,
    level: ExportLevel,
    parse_mode: ParseMode,
) -> ExportScan {
    let content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(_) => return ExportScan::Unreadable,
    };
    scan_exported_symbols_content_internal(file_path, &content, level, parse_mode, true)
}

/// Extract symbols from caller-supplied source bytes without reopening the source
/// path. TypeScript wildcard imports are intentionally not resolved through the
/// ambient filesystem; snapshot callers must only validate bytes they retained.
///
/// Callers that gate on the result now use `scan_exported_symbols_from_content`
/// instead, because collapsing a conflicted file to an empty vector here is the
/// very fail-open this guard exists to close. Kept as the plain-vector form of
/// the same confined entry point.
#[allow(dead_code)]
pub(super) fn get_exported_symbols_from_content(
    file_path: &Path,
    content: &str,
    level: ExportLevel,
    parse_mode: ParseMode,
) -> Vec<String> {
    match scan_exported_symbols_content_internal(file_path, content, level, parse_mode, false) {
        ExportScan::Parsed(symbols) => symbols,
        ExportScan::UnknownLanguage | ExportScan::Unreadable | ExportScan::Conflicted(_) => {
            Vec::new()
        }
    }
}

/// Like [`get_exported_symbols_from_content`] but preserving the scan outcome,
/// so a snapshot caller can distinguish a conflicted file from an empty one.
pub(super) fn scan_exported_symbols_from_content(
    file_path: &Path,
    content: &str,
    level: ExportLevel,
    parse_mode: ParseMode,
) -> ExportScan {
    scan_exported_symbols_content_internal(file_path, content, level, parse_mode, false)
}

/// Extract, then check whether extraction was fooled by an unresolved conflict.
///
/// The guard is symptom-based on purpose. A content guard that fails on
/// marker-shaped lines is unshippable: this repository alone carries twelve
/// complete, well-formed conflict triples inside raw string literals in test
/// fixtures, and nothing in the bytes distinguishes them from a real conflict.
/// What *does* distinguish them is whether the extractor believed them — a
/// triple inside a string literal contributes no declarations, because the
/// extractors blank string literals before matching. So we ask the extractor,
/// not the bytes: resolve the file to each side in turn, re-extract, and
/// escalate only when the raw parse claims symbols that survive on one side and
/// other symbols that survive only on the other. That union is the defect
/// itself — it is what let a spec documenting both `sub` and `mul` report
/// `3/3 exports documented` against source that cannot compile.
fn scan_exported_symbols_content_internal(
    file_path: &Path,
    content: &str,
    level: ExportLevel,
    parse_mode: ParseMode,
    resolve_typescript_imports: bool,
) -> ExportScan {
    let scan = extract_scan(
        file_path,
        content,
        level,
        parse_mode,
        resolve_typescript_imports,
    );
    let ExportScan::Parsed(symbols) = &scan else {
        return scan;
    };
    match conflicted_union(content, symbols, &|side: &str| match extract_scan(
        file_path,
        side,
        level,
        parse_mode,
        resolve_typescript_imports,
    ) {
        ExportScan::Parsed(side_symbols) => side_symbols,
        _ => Vec::new(),
    }) {
        Some(evidence) => ExportScan::Conflicted(evidence),
        None => scan,
    }
}

/// Decide whether `symbols` is the union of two sides of a conflict hunk.
///
/// Returns evidence only when the raw parse contributes at least one symbol
/// that survives resolving to `ours` but not to `theirs`, AND at least one that
/// survives the reverse. One-sided differences are not enough: a hunk that only
/// adds declarations on one side leaves the other side's parse a strict subset,
/// which is also what a file with a *quoted* triple looks like when the
/// extractor correctly ignores it (no difference at all).
fn conflicted_union(
    content: &str,
    symbols: &[String],
    extract: &dyn Fn(&str) -> Vec<String>,
) -> Option<ConflictedExtraction> {
    let hunks = crate::merge::conflict_hunks(content);
    if hunks.is_empty() {
        return None;
    }

    let raw: std::collections::HashSet<&str> = symbols.iter().map(String::as_str).collect();
    let ours_text = crate::merge::conflict_free_side(content, crate::merge::ConflictSide::Ours);
    let theirs_text = crate::merge::conflict_free_side(content, crate::merge::ConflictSide::Theirs);
    let ours: std::collections::HashSet<String> = extract(&ours_text).into_iter().collect();
    let theirs: std::collections::HashSet<String> = extract(&theirs_text).into_iter().collect();

    let mut ours_only: Vec<String> = raw
        .iter()
        .filter(|s| ours.contains(**s) && !theirs.contains(**s))
        .map(|s| (*s).to_string())
        .collect();
    let mut theirs_only: Vec<String> = raw
        .iter()
        .filter(|s| theirs.contains(**s) && !ours.contains(**s))
        .map(|s| (*s).to_string())
        .collect();

    if ours_only.is_empty() || theirs_only.is_empty() {
        return None;
    }
    ours_only.sort();
    theirs_only.sort();

    let hunk = &hunks[0];
    Some(ConflictedExtraction {
        ours_label: hunk.ours_label.clone(),
        theirs_label: hunk.theirs_label.clone(),
        ours_only,
        theirs_only,
    })
}

/// Pure language dispatch — no conflict analysis, so the guard above can call
/// it on reconstructed variants without re-entering itself.
fn extract_scan(
    file_path: &Path,
    content: &str,
    level: ExportLevel,
    parse_mode: ParseMode,
    resolve_typescript_imports: bool,
) -> ExportScan {
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let lang = match Language::from_extension(ext) {
        Some(l) => l,
        None => return ExportScan::UnknownLanguage,
    };

    let symbols = if parse_mode == ParseMode::Ast {
        match lang {
            Language::TypeScript => {
                let result = if resolve_typescript_imports {
                    let base_dir = file_path.parent().unwrap_or(Path::new(".")).to_path_buf();
                    let resolver =
                        move |import_path: &str| resolve_ts_import(&base_dir, import_path);
                    ast::typescript::extract_exports_with_resolver(content, Some(&resolver))
                } else {
                    ast::typescript::extract_exports_with_resolver(content, None)
                };
                if result.is_empty() {
                    // Fallback to regex if AST returned nothing (parse failure)
                    if resolve_typescript_imports {
                        let base_dir = file_path.parent().unwrap_or(Path::new(".")).to_path_buf();
                        let resolver =
                            move |import_path: &str| resolve_ts_import(&base_dir, import_path);
                        typescript::extract_exports_with_resolver(content, Some(&resolver))
                    } else {
                        typescript::extract_exports_with_resolver(content, None)
                    }
                } else {
                    result
                }
            }
            Language::Python => {
                let result = ast::python::extract_exports(content);
                if result.is_empty() {
                    python::extract_exports(content)
                } else {
                    result
                }
            }
            Language::Rust => {
                let result = ast::rust_lang::extract_exports(content);
                if result.is_empty() {
                    rust_lang::extract_exports(content)
                } else {
                    result
                }
            }
            Language::C => {
                let result = ast::c::extract_exports(content);
                if result.is_empty() {
                    c::extract_exports(content)
                } else {
                    result
                }
            }
            Language::Cpp => {
                let result = ast::cpp::extract_exports(content);
                if result.is_empty() {
                    cpp::extract_exports(content)
                } else {
                    result
                }
            }
            Language::Scala => {
                let result = ast::scala::extract_exports(content);
                if result.is_empty() {
                    scala::extract_exports(content)
                } else {
                    result
                }
            }
            Language::Erlang => {
                let result = ast::erlang::extract_exports(content);
                if result.is_empty() {
                    erlang::extract_exports(content)
                } else {
                    result
                }
            }
            Language::Elixir => {
                let result = ast::elixir::extract_exports(content);
                if result.is_empty() {
                    elixir::extract_exports(content)
                } else {
                    result
                }
            }
            Language::Perl => {
                let result = ast::perl::extract_exports(content);
                if result.is_empty() {
                    perl::extract_exports(content)
                } else {
                    result
                }
            }
            Language::Lisp => {
                let result = ast::lisp::extract_exports(content, ext);
                if result.is_empty() {
                    lisp::extract_exports(content)
                } else {
                    result
                }
            }
            // Nim and Crystal have no published tree-sitter grammar crate: fall back to regex
            _ => extract_with_regex(content, lang, file_path),
        }
    } else if lang == Language::TypeScript && !resolve_typescript_imports {
        typescript::extract_exports_with_resolver(content, None)
    } else {
        extract_with_regex(content, lang, file_path)
    };

    // If type-level granularity, filter to only type declarations
    let symbols = if level == ExportLevel::Type {
        filter_type_level_exports(content, &symbols, lang)
    } else {
        symbols
    };

    // Deduplicate preserving order
    let mut seen = std::collections::HashSet::new();
    ExportScan::Parsed(
        symbols
            .into_iter()
            .filter(|s| seen.insert(s.clone()))
            .collect(),
    )
}

/// Extract exported symbol names with full control over granularity and parse mode.
/// When `parse_mode` is `Ast`, uses tree-sitter for TypeScript, Python, Rust, C, C++,
/// Scala, Erlang, Elixir, Perl, and Lisp/Scheme/Emacs Lisp.
/// Falls back to regex for all other languages or if AST parsing fails. A read/parse
/// failure or unsupported language yields an empty vector — use
/// `scan_exported_symbols_full` when that distinction matters (e.g. for gating).
pub fn get_exported_symbols_full(
    file_path: &Path,
    level: ExportLevel,
    parse_mode: ParseMode,
) -> Vec<String> {
    match scan_exported_symbols_full(file_path, level, parse_mode) {
        ExportScan::Parsed(symbols) => symbols,
        ExportScan::UnknownLanguage | ExportScan::Unreadable | ExportScan::Conflicted(_) => {
            Vec::new()
        }
    }
}

/// Dispatch to the regex-based export extractor for the given language.
fn extract_with_regex(content: &str, lang: Language, file_path: &Path) -> Vec<String> {
    match lang {
        Language::TypeScript => {
            let base_dir = file_path.parent().unwrap_or(Path::new(".")).to_path_buf();
            let resolver = move |import_path: &str| resolve_ts_import(&base_dir, import_path);
            typescript::extract_exports_with_resolver(content, Some(&resolver))
        }
        Language::Rust => rust_lang::extract_exports(content),
        Language::Go => go::extract_exports(content),
        Language::Python => python::extract_exports(content),
        Language::Swift => swift::extract_exports(content),
        Language::Kotlin => kotlin::extract_exports(content),
        Language::Java => java::extract_exports(content),
        Language::CSharp => csharp::extract_exports(content),
        Language::Dart => dart::extract_exports(content),
        Language::Php => php::extract_exports(content),
        Language::Ruby => ruby::extract_exports(content),
        Language::Yaml => yaml::extract_exports(content),
        Language::C => c::extract_exports(content),
        Language::Cpp => cpp::extract_exports(content),
        Language::Scala => scala::extract_exports(content),
        Language::Crystal => crystal::extract_exports(content),
        Language::Nim => nim::extract_exports(content),
        Language::Erlang => erlang::extract_exports(content),
        Language::Elixir => elixir::extract_exports(content),
        Language::Perl => perl::extract_exports(content),
        Language::Lisp => lisp::extract_exports(content),
        Language::Haskell => haskell::extract_exports(content),
        Language::Lua => lua::extract_exports(content),
        Language::R => r::extract_exports(content),
        Language::OCaml => ocaml::extract_exports(content),
        Language::Groovy => groovy::extract_exports(content),
        Language::FSharp => fsharp::extract_exports(content),
        Language::Clojure => clojure::extract_exports(content),
        Language::D => d::extract_exports(content),
        Language::ObjectiveC => objective_c::extract_exports(content),
        Language::Bash => bash::extract_exports(content),
        Language::PowerShell => powershell::extract_exports(content),
        Language::Vala => vala::extract_exports(content),
    }
}

/// Filter symbols to only include type-level declarations (class, struct, enum, etc.).
/// This removes individual functions, variables, constants, and properties.
fn filter_type_level_exports(content: &str, symbols: &[String], lang: Language) -> Vec<String> {
    use regex::Regex;

    let type_pattern = match lang {
        Language::TypeScript => {
            // Exported ESM declarations, local classes later exported through
            // CommonJS, and inline CommonJS class assignments.
            Regex::new(
                r"(?m)(?:\b(?:abstract\s+)?(?:class|interface|type|enum)\s+([A-Za-z_$][A-Za-z0-9_$]*)|(?:module\s*\.\s*)?exports\s*\.\s*([A-Za-z_$][A-Za-z0-9_$]*)\s*=\s*(?:abstract\s+)?class\b)",
            )
            .ok()
        }
        Language::Rust => {
            // Capture optional visibility restriction so narrower-than-crate type names
            // can be discarded before intersecting with the symbol list.
            Regex::new(
                r"(?m)\bpub\s*(\(\s*[^)]*?\s*\))?\s+(?:struct|enum|trait|type|mod)\s+(\w+)",
            )
            .ok()
        }
        Language::Go => {
            // Go: type X struct/interface
            Regex::new(r"(?m)^type\s+([A-Z]\w*)\s+(?:struct|interface)").ok()
        }
        Language::Python => {
            Regex::new(r"(?m)^class\s+(\w+)").ok()
        }
        Language::Swift => {
            Regex::new(
                r"(?m)(?:public|open)\s+(?:final\s+)?(?:class|struct|enum|protocol|actor)\s+(\w+)",
            )
            .ok()
        }
        Language::Kotlin => {
            Regex::new(
                r"(?m)(?:public\s+|open\s+|abstract\s+|sealed\s+)*(?:class|interface|enum\s+class|object|data\s+class)\s+(\w+)",
            )
            .ok()
        }
        Language::Java => {
            Regex::new(
                r"(?m)(?:public\s+)?(?:abstract\s+|final\s+)?(?:class|interface|enum|record)\s+(\w+)",
            )
            .ok()
        }
        Language::CSharp => {
            Regex::new(
                r"(?m)(?:public\s+)?(?:abstract\s+|sealed\s+|static\s+)?(?:class|interface|enum|struct|record)\s+(\w+)",
            )
            .ok()
        }
        Language::Dart => {
            Regex::new(r"(?m)(?:abstract\s+)?class\s+(\w+)|(?m)enum\s+(\w+)").ok()
        }
        Language::Php => {
            Regex::new(
                r"(?m)(?:abstract\s+|final\s+)?(?:readonly\s+)?(?:class|interface|trait|enum)\s+(\w+)",
            )
            .ok()
        }
        Language::Ruby => {
            Regex::new(r"(?m)(?:class|module)\s+([A-Z]\w*)").ok()
        }
        Language::Yaml => {
            // YAML has no type declarations
            return symbols.to_vec();
        }
        Language::C => Regex::new(r"(?m)^[^\S\n]*(?:struct|union|enum)\s+(\w+)").ok(),
        Language::Cpp => Regex::new(r"(?m)^[^\S\n]*(?:class|struct|union|enum|namespace)\s+(\w+)").ok(),
        Language::Scala => Regex::new(r"(?m)^[^\S\n]*(?:class|object|trait)\s+(\w+)").ok(),
        Language::Crystal => Regex::new(r"(?m)^[^\S\n]*(?:class|module|struct|enum)\s+(\w+)").ok(),
        Language::Nim => Regex::new(r"(?m)^[^\S\n]*type\s+(\w+)\*").ok(),
        Language::Elixir => Regex::new(r"(?m)^[^\S\n]*defmodule\s+([\w.!?]+)").ok(),
        Language::Erlang | Language::Perl | Language::Lisp => None,
        Language::Haskell => Regex::new(r"(?m)^[^\S\n]*(?:data|newtype|type|class)\s+(\w+)").ok(),
        Language::Lua | Language::R | Language::Bash | Language::PowerShell => None,
        Language::OCaml => Regex::new(r"(?m)^[^\S\n]*(?:type|module)\s+(\w+)").ok(),
        Language::Groovy => {
            Regex::new(r"(?m)^[^\S\n]*(?:class|interface|trait|enum)\s+(\w+)").ok()
        }
        Language::FSharp => Regex::new(r"(?m)^[^\S\n]*(?:type|module)\s+(\w+)").ok(),
        Language::Clojure => {
            Regex::new(r"(?m)\(\s*(?:defrecord|deftype|defprotocol)\s+(\w+)").ok()
        }
        Language::D => Regex::new(r"(?m)^[^\S\n]*(?:class|struct|interface|enum)\s+(\w+)").ok(),
        Language::ObjectiveC => Regex::new(r"(?m)^[^\S\n]*@(?:interface|protocol)\s+(\w+)").ok(),
        Language::Vala => Regex::new(r"(?m)^[^\S\n]*(?:class|interface|struct|enum)\s+(\w+)").ok(),
    };

    let type_names: std::collections::HashSet<String> = match type_pattern {
        Some(re) => re
            .captures_iter(content)
            .filter_map(|caps| {
                let restriction = caps.get(1).map(|value| value.as_str());
                if lang == Language::Rust
                    && restriction.is_some_and(|value| {
                        value
                            .chars()
                            .filter(|character| !character.is_whitespace())
                            .collect::<String>()
                            != "(crate)"
                    })
                {
                    return None;
                }
                if lang == Language::Rust {
                    caps.get(2).map(|m| m.as_str().to_string())
                } else {
                    (1..caps.len())
                        .find_map(|index| caps.get(index))
                        .map(|name| name.as_str().to_string())
                }
            })
            .collect(),
        None => return symbols.to_vec(),
    };

    symbols
        .iter()
        .filter(|s| type_names.contains(s.as_str()))
        .cloned()
        .collect()
}

/// Resolve a TypeScript/JavaScript relative import to file content.
/// Tries TypeScript-family and JavaScript-family extensions and index files.
fn resolve_ts_import(base_dir: &Path, import_path: &str) -> Option<String> {
    // Only resolve relative imports
    if !import_path.starts_with('.') {
        return None;
    }

    let target = base_dir.join(import_path);

    // Try exact path first (might already have extension)
    if target.is_file() {
        return std::fs::read_to_string(&target).ok();
    }

    // Try common extensions
    for ext in &[".ts", ".tsx", ".js", ".jsx", ".mts", ".cts", ".mjs", ".cjs"] {
        let with_ext = target.with_extension(ext.trim_start_matches('.'));
        if with_ext.is_file() {
            return std::fs::read_to_string(&with_ext).ok();
        }
    }

    // Try as directory with index file
    for index in &[
        "index.ts",
        "index.tsx",
        "index.js",
        "index.jsx",
        "index.mts",
        "index.cts",
        "index.mjs",
        "index.cjs",
    ] {
        let index_path = target.join(index);
        if index_path.is_file() {
            return std::fs::read_to_string(&index_path).ok();
        }
    }

    None
}

/// Well-known test directory names (case-insensitive check).
const TEST_DIR_NAMES: &[&str] = &[
    "tests",
    "test",
    "__tests__",
    "spec",
    "specs",
    "testing",
    "uitests",
    "unittests",
    "integrationtests",
    "testcases",
    "fixtures",
    "mocks",
    "stubs",
    "fakes",
];

/// Check if a file is a test file based on language conventions and path.
pub fn is_test_file(file_path: &Path, root: &Path) -> bool {
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let lang = match Language::from_extension(ext) {
        Some(l) => l,
        None => return false,
    };

    let name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // Check filename patterns
    for pattern in lang.test_patterns() {
        if name.ends_with(pattern) || name.starts_with(pattern) {
            return true;
        }
    }

    // Check whether any directory *inside the project* is a test directory. Bound the
    // walk to components below `root`: when a full/absolute path is passed (as coverage
    // does), an ancestor above the project named `test`/`spec`/etc. must not
    // misclassify an ordinary source file. If the path is not under `root`, we cannot
    // tell which components are project-relative, so we rely on the filename alone.
    if let Ok(relative) = file_path.strip_prefix(root) {
        for component in relative.components() {
            if let std::path::Component::Normal(dir) = component {
                let dir_lower = dir.to_string_lossy().to_lowercase();
                if TEST_DIR_NAMES.contains(&dir_lower.as_str()) {
                    return true;
                }
            }
        }
    }

    false
}

/// Check if a file extension is a supported source file.
pub fn is_source_file(file_path: &Path) -> bool {
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

    Language::from_extension(ext).is_some()
}

/// Check if a file extension matches a specific set of allowed extensions.
pub fn has_extension(file_path: &Path, extensions: &[String]) -> bool {
    if extensions.is_empty() {
        return is_source_file(file_path);
    }
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    extensions.iter().any(|e| e == ext)
}

/// Check whether a file matches configured source discovery, including optional
/// extensionless files in addition to the default or explicit extension set.
pub(super) fn has_configured_extension(
    file_path: &Path,
    extensions: &[String],
    include_extensionless: bool,
) -> bool {
    (include_extensionless && file_path.extension().is_none())
        || has_extension(file_path, extensions)
}

#[cfg(test)]
mod is_test_file_tests {
    use super::is_test_file;
    use std::path::Path;

    #[test]
    fn ignores_test_named_ancestors_above_root() {
        // Regression: a project living under a directory named `spec`/`test`/etc. must
        // not have its ordinary sources mis-classified as tests. The directory check is
        // bounded to components below `root`.
        let root = Path::new("/home/user/spec/myproject");
        let file = Path::new("/home/user/spec/myproject/src/app.ts");
        assert!(
            !is_test_file(file, root),
            "a `spec/` ancestor above the project root must be ignored"
        );

        let root2 = Path::new("/ci/tests/checkout/repo");
        let file2 = Path::new("/ci/tests/checkout/repo/src/lib.rs");
        assert!(
            !is_test_file(file2, root2),
            "`tests/` above root must be ignored"
        );
    }

    #[test]
    fn detects_in_project_test_directory() {
        let root = Path::new("/home/user/myproject");
        assert!(is_test_file(
            Path::new("/home/user/myproject/src/tests/helper.ts"),
            root
        ));
        assert!(is_test_file(
            Path::new("/home/user/myproject/__tests__/util.ts"),
            root
        ));
    }

    #[test]
    fn detects_test_filename_patterns() {
        let root = Path::new("/home/user/myproject");
        assert!(is_test_file(
            Path::new("/home/user/myproject/src/app.test.ts"),
            root
        ));
        assert!(is_test_file(
            Path::new("/home/user/myproject/src/app.spec.ts"),
            root
        ));
    }

    #[test]
    fn plain_source_is_not_a_test() {
        let root = Path::new("/home/user/myproject");
        assert!(!is_test_file(
            Path::new("/home/user/myproject/src/app.ts"),
            root
        ));
        // A non-source extension is never a test file.
        assert!(!is_test_file(
            Path::new("/home/user/myproject/tests/data.json"),
            root
        ));
    }
}

#[cfg(test)]
mod configured_extension_tests {
    use super::has_configured_extension;
    use std::path::Path;

    #[test]
    fn extensionless_is_additive_and_explicit() {
        let extensions = vec!["sh".to_string()];
        assert!(!has_configured_extension(
            Path::new("bin/tool"),
            &extensions,
            false
        ));
        assert!(has_configured_extension(
            Path::new("bin/tool"),
            &extensions,
            true
        ));
        assert!(has_configured_extension(
            Path::new("bin/tool.sh"),
            &extensions,
            true
        ));
        assert!(!has_configured_extension(
            Path::new("bin/tool.py"),
            &extensions,
            true
        ));
    }

    #[test]
    fn empty_extension_list_keeps_supported_language_defaults() {
        assert!(has_configured_extension(
            Path::new("src/lib.rs"),
            &[],
            false
        ));
        assert!(!has_configured_extension(Path::new("bin/tool"), &[], false));
        assert!(has_configured_extension(Path::new("bin/tool"), &[], true));
    }
}

#[cfg(test)]
mod scan_tests {
    use super::{
        ExportScan, get_exported_symbols_from_content, scan_exported_symbols,
        scan_exported_symbols_full,
    };
    use crate::types::{ExportLevel, ParseMode};
    use std::io::Write;

    #[test]
    fn parsed_distinguishes_from_unreadable_and_unknown() {
        let dir = tempfile::tempdir().unwrap();

        // A recognized-language file that parses → Parsed(symbols).
        let ts = dir.path().join("a.ts");
        std::fs::write(&ts, "export function hi() {}\n").unwrap();
        assert_eq!(
            scan_exported_symbols(&ts),
            ExportScan::Parsed(vec!["hi".to_string()])
        );

        // A recognized-language file that genuinely exports nothing → Parsed(empty),
        // NOT Unreadable — the caller can tell "clean but empty" from "could not read".
        let empty = dir.path().join("empty.rs");
        std::fs::write(&empty, "fn private_only() {}\n").unwrap();
        assert_eq!(scan_exported_symbols(&empty), ExportScan::Parsed(vec![]));

        // A non-source extension → UnknownLanguage (a `.md`/`.sql` listed in files:).
        let md = dir.path().join("readme.md");
        std::fs::write(&md, "# doc\n").unwrap();
        assert_eq!(scan_exported_symbols(&md), ExportScan::UnknownLanguage);

        // A missing file → Unreadable.
        assert_eq!(
            scan_exported_symbols(&dir.path().join("missing.ts")),
            ExportScan::Unreadable
        );

        // A non-UTF-8 recognized-language file → Unreadable (not silently empty).
        let bad = dir.path().join("bad.ts");
        let mut f = std::fs::File::create(&bad).unwrap();
        f.write_all(b"export function x() {}\n\xff\xfe").unwrap();
        assert_eq!(scan_exported_symbols(&bad), ExportScan::Unreadable);
    }

    #[test]
    fn module_javascript_resolves_extensionless_file_and_index_barrels() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("values.mjs"),
            "export const fromMjs = true;\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("legacy.cjs"), "exports.fromCjs = true;\n").unwrap();
        std::fs::create_dir_all(dir.path().join("nested")).unwrap();
        std::fs::write(
            dir.path().join("nested/index.mjs"),
            "export const fromIndex = true;\n",
        )
        .unwrap();
        let barrel = dir.path().join("index.mjs");
        std::fs::write(
            &barrel,
            "export * from './values';\nexport * from './legacy';\nexport * from './nested';\n",
        )
        .unwrap();

        assert_eq!(
            scan_exported_symbols(&barrel),
            ExportScan::Parsed(vec![
                "fromMjs".to_string(),
                "fromCjs".to_string(),
                "fromIndex".to_string(),
            ])
        );
    }

    #[test]
    fn commonjs_classes_survive_type_level_filtering_in_both_modes() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("classes.cjs");
        std::fs::write(
            &source,
            "class Widget {}\nexports.Widget = Widget;\nexports.Inline = class {};\nexports.value = true;\n",
        )
        .unwrap();

        for parse_mode in [ParseMode::Regex, ParseMode::Ast] {
            assert_eq!(
                scan_exported_symbols_full(&source, ExportLevel::Type, parse_mode),
                ExportScan::Parsed(vec!["Widget".to_string(), "Inline".to_string()])
            );
        }
    }

    #[test]
    fn supplied_content_extraction_never_resolves_ambient_typescript_imports() {
        let directory = tempfile::tempdir().unwrap();
        let barrel = directory.path().join("index.ts");
        let content = "export * from './outside';\n";
        std::fs::write(&barrel, content).unwrap();
        std::fs::write(
            directory.path().join("outside.ts"),
            "export const outsideSecret = true;\n",
        )
        .unwrap();

        assert_eq!(
            scan_exported_symbols(&barrel),
            ExportScan::Parsed(vec!["outsideSecret".to_string()])
        );
        assert!(
            get_exported_symbols_from_content(
                &barrel,
                content,
                ExportLevel::Member,
                ParseMode::Regex,
            )
            .is_empty()
        );
    }

    /// The confirmed #578 repro, at the level the defect actually lives: the
    /// extractor reading `sub` from one side of a hunk and `mul` from the other
    /// and returning both as though the file exported three functions.
    #[test]
    fn conflicted_source_is_reported_instead_of_the_union_of_both_sides() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("calc.rs");
        std::fs::write(
            &source,
            "pub fn add(a: i32, b: i32) -> i32 { a + b }\n\
             <<<<<<< HEAD\n\
             pub fn sub(a: i32, b: i32) -> i32 { a - b }\n\
             =======\n\
             pub fn mul(a: i32, b: i32) -> i32 { a * b }\n\
             >>>>>>> feature/other\n",
        )
        .unwrap();

        match scan_exported_symbols(&source) {
            ExportScan::Conflicted(evidence) => {
                assert_eq!(evidence.ours_label, "HEAD");
                assert_eq!(evidence.theirs_label, "feature/other");
                assert_eq!(evidence.ours_only, vec!["sub".to_string()]);
                assert_eq!(evidence.theirs_only, vec!["mul".to_string()]);
            }
            other => panic!("a conflicted file must not scan as {other:?}"),
        }
        // The plain-vector entry point must not leak the union either.
        assert!(super::get_exported_symbols(&source).is_empty());
    }

    /// The false positive that makes the naive fix unshippable. This shape is
    /// verbatim `src/exports/ast/rust_lang.rs`'s
    /// `test_merge_conflict_markers_defer_to_regex_fallback` fixture: a
    /// complete, well-formed triple with `pub fn` on BOTH sides, inside a raw
    /// string literal. Nothing in the marker bytes distinguishes it from the
    /// test above — only whether the extractor believed it.
    #[test]
    fn a_conflict_triple_quoted_in_a_string_literal_is_not_a_conflict() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("fixture.rs");
        std::fs::write(
            &source,
            "pub fn real_export() {}\n\
             \n\
             #[test]\n\
             fn fixture() {\n\
             let src = r#\"\n\
             <<<<<<< HEAD\n\
             pub fn feature_a() {}\n\
             =======\n\
             pub fn feature_b() {}\n\
             >>>>>>> feature-branch\n\
             \"#;\n\
             }\n",
        )
        .unwrap();

        assert_eq!(
            scan_exported_symbols(&source),
            ExportScan::Parsed(vec!["real_export".to_string()]),
            "a quoted triple must stay Parsed — the extractor blanks string literals, \
             so neither side contributes a symbol the other lacks"
        );
    }

    /// A hunk that only ADDS declarations on one side is not the union defect:
    /// the other side's parse is a strict subset, so nothing is invented.
    #[test]
    fn a_one_sided_hunk_is_not_reported_as_conflicted() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("one_sided.rs");
        std::fs::write(
            &source,
            "pub fn add() {}\n\
             <<<<<<< HEAD\n\
             pub fn sub() {}\n\
             =======\n\
             >>>>>>> feature/other\n",
        )
        .unwrap();

        assert!(matches!(
            scan_exported_symbols(&source),
            ExportScan::Parsed(_)
        ));
    }

    /// The acceptance requirement as an executable assertion: the guard must not
    /// fire on spec-sync's own tree, which carries twelve complete conflict
    /// triples across three files. A guard that red-lights this repository is a
    /// failed guard however well it handles a fixture.
    #[test]
    fn no_source_file_in_this_repository_scans_as_conflicted() {
        fn walk(dir: &std::path::Path, found: &mut Vec<std::path::PathBuf>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, found);
                } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    found.push(path);
                }
            }
        }

        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut sources = Vec::new();
        walk(&repo.join("src"), &mut sources);
        walk(&repo.join("tests"), &mut sources);
        assert!(
            sources.len() > 20,
            "expected to walk the real tree, found {} files",
            sources.len()
        );

        let mut marker_files = 0usize;
        let mut offenders = Vec::new();
        for source in &sources {
            let content = std::fs::read_to_string(source).unwrap_or_default();
            if !crate::merge::conflict_hunks(&content).is_empty() {
                marker_files += 1;
            }
            for parse_mode in [ParseMode::Regex, ParseMode::Ast] {
                if let ExportScan::Conflicted(evidence) =
                    scan_exported_symbols_full(source, ExportLevel::Member, parse_mode)
                {
                    offenders.push(format!(
                        "{} [{parse_mode:?}] {}",
                        source.display(),
                        evidence.describe()
                    ));
                }
            }
        }

        assert!(
            marker_files >= 3,
            "the false-positive hazard must still be present in the tree, \
             otherwise this test proves nothing; saw {marker_files} file(s) with complete triples"
        );
        assert!(
            offenders.is_empty(),
            "spec-sync must not report its own fixtures as conflicts:\n{}",
            offenders.join("\n")
        );
    }
}
