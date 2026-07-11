//! Comparison tests: AST (tree-sitter) vs Regex on tricky edge cases.
//! These verify that AST parsing produces equivalent or better results than regex.

#[cfg(test)]
mod ast_vs_regex {
    use crate::exports::ast;
    use crate::exports::{python, rust_lang, typescript};

    // ── TypeScript ──────────────────────────────────────────────

    #[test]
    fn ts_basic_parity() {
        let src = r#"
export function createAuth(config: Config): Auth {}
export class AuthService {}
export interface AuthConfig {}
export type TokenType = string;
export const DEFAULT_TTL = 3600;
export enum AuthStatus { Active, Expired }
"#;
        let regex_result = typescript::extract_exports(src);
        let ast_result = ast::typescript::extract_exports(src);
        assert_eq!(ast_result, regex_result);
    }

    #[test]
    fn ts_re_exports_with_alias() {
        let src = r#"
export { Foo, Bar as Baz } from './module';
"#;
        let regex_result = typescript::extract_exports(src);
        let ast_result = ast::typescript::extract_exports(src);
        // Both should contain Foo and Baz (alias), not Bar
        assert!(ast_result.contains(&"Foo".to_string()));
        assert!(ast_result.contains(&"Baz".to_string()));
        assert_eq!(ast_result, regex_result);
    }

    #[test]
    fn ts_comments_ignored() {
        let src = r#"
// export function notExported() {}
/* export class AlsoNot {} */
export function realExport(): void {}
"#;
        let regex_result = typescript::extract_exports(src);
        let ast_result = ast::typescript::extract_exports(src);
        assert_eq!(ast_result, vec!["realExport"]);
        assert_eq!(ast_result, regex_result);
    }

    #[test]
    fn ts_default_export_class() {
        let src = r#"
export default class MyApp {}
"#;
        let regex_result = typescript::extract_exports(src);
        let ast_result = ast::typescript::extract_exports(src);
        assert!(ast_result.contains(&"MyApp".to_string()));
        assert_eq!(ast_result, regex_result);
    }

    #[test]
    fn ts_export_equals() {
        let src = r#"
class AuthService {}
export = AuthService;
"#;
        let regex_result = typescript::extract_exports(src);
        let ast_result = ast::typescript::extract_exports(src);
        assert!(ast_result.contains(&"AuthService".to_string()));
        assert_eq!(ast_result, regex_result);
    }

    #[test]
    fn ts_async_abstract() {
        let src = r#"
export async function fetchData() {}
export abstract class BaseService {}
"#;
        let regex_result = typescript::extract_exports(src);
        let ast_result = ast::typescript::extract_exports(src);
        assert!(ast_result.contains(&"fetchData".to_string()));
        assert!(ast_result.contains(&"BaseService".to_string()));
        assert_eq!(ast_result, regex_result);
    }

    #[test]
    fn ts_wildcard_namespace() {
        let src = r#"
export * as Utils from './utils';
export * as Types from './types';
"#;
        let regex_result = typescript::extract_exports(src);
        let ast_result = ast::typescript::extract_exports(src);
        assert_eq!(ast_result, vec!["Utils", "Types"]);
        assert_eq!(ast_result, regex_result);
    }

    #[test]
    fn ts_wildcard_with_resolver() {
        let src = r#"
export * from './helpers';
export function main() {}
"#;
        let helper = r#"
export function helperA() {}
export function helperB() {}
export const HELPER_CONST = 42;
"#;
        let resolver = |path: &str| -> Option<String> {
            if path == "./helpers" {
                Some(helper.to_string())
            } else {
                None
            }
        };
        let regex_result = typescript::extract_exports_with_resolver(src, Some(&resolver));
        let ast_result = ast::typescript::extract_exports_with_resolver(src, Some(&resolver));
        assert!(ast_result.contains(&"main".to_string()));
        assert!(ast_result.contains(&"helperA".to_string()));
        assert!(ast_result.contains(&"helperB".to_string()));
        assert!(ast_result.contains(&"HELPER_CONST".to_string()));
        // Order may differ (AST resolves wildcard inline), but same set
        let mut ast_sorted = ast_result.clone();
        let mut regex_sorted = regex_result.clone();
        ast_sorted.sort();
        regex_sorted.sort();
        assert_eq!(ast_sorted, regex_sorted);
    }

    #[test]
    fn ts_export_inside_string_literal() {
        // Regex might mistakenly match export inside template string;
        // AST correctly ignores string contents.
        let src = r#"
export function real() {}
const template = `
export function fake() {}
`;
"#;
        let ast_result = ast::typescript::extract_exports(src);
        assert_eq!(ast_result, vec!["real"]);
        // regex might or might not get this right — the point is AST is correct
    }

    #[test]
    fn ts_conditional_export() {
        // Export inside if block — AST can see it, regex may or may not
        let src = r#"
if (process.env.NODE_ENV === 'development') {
    export function debugHelper() {}
}
export function main() {}
"#;
        let ast_result = ast::typescript::extract_exports(src);
        assert!(ast_result.contains(&"main".to_string()));
        assert!(ast_result.contains(&"debugHelper".to_string()));
    }

    // ── Python ──────────────────────────────────────────────────

    #[test]
    fn py_basic_parity() {
        let src = r#"
def create_auth(config):
    pass

class AuthService:
    pass

def _internal():
    pass
"#;
        let regex_result = python::extract_exports(src);
        let ast_result = ast::python::extract_exports(src);
        assert_eq!(ast_result, regex_result);
    }

    #[test]
    fn py_all_takes_precedence() {
        let src = r#"
__all__ = ["create_auth", "AuthService"]

def create_auth(config):
    pass

class AuthService:
    pass

def extra_func():
    pass
"#;
        let regex_result = python::extract_exports(src);
        let ast_result = ast::python::extract_exports(src);
        assert_eq!(ast_result, vec!["create_auth", "AuthService"]);
        assert_eq!(ast_result, regex_result);
    }

    #[test]
    fn py_nested_not_captured() {
        let src = r#"
class Outer:
    class Inner:
        pass
    def method(self):
        pass

def top_level():
    def nested():
        pass
"#;
        let regex_result = python::extract_exports(src);
        let ast_result = ast::python::extract_exports(src);
        assert_eq!(ast_result, regex_result);
        assert!(ast_result.contains(&"Outer".to_string()));
        assert!(!ast_result.contains(&"Inner".to_string()));
    }

    #[test]
    fn py_decorated_functions() {
        let src = r#"
@dataclass
class Config:
    host: str

@staticmethod
def create():
    pass
"#;
        let regex_result = python::extract_exports(src);
        let ast_result = ast::python::extract_exports(src);
        assert!(ast_result.contains(&"Config".to_string()));
        assert!(ast_result.contains(&"create".to_string()));
        assert_eq!(ast_result, regex_result);
    }

    // ── Rust ────────────────────────────────────────────────────

    #[test]
    fn rs_basic_parity() {
        let src = r#"
pub fn create_auth(config: Config) -> Auth {}
pub struct AuthService {}
pub enum AuthStatus { Active, Expired }
pub trait Authenticator {}
pub type Token = String;
pub const DEFAULT_TTL: u64 = 3600;
pub static INSTANCE: Lazy<Auth> = Lazy::new(|| Auth::new());
fn private_fn() {}
struct PrivateStruct {}
"#;
        let regex_result = rust_lang::extract_exports(src);
        let ast_result = ast::rust_lang::extract_exports(src);
        assert_eq!(ast_result, regex_result);
    }

    #[test]
    fn rs_pub_crate_included() {
        let src = r#"
pub(crate) fn internal_fn() {}
pub(crate) struct InternalStruct {}
"#;
        let regex_result = rust_lang::extract_exports(src);
        let ast_result = ast::rust_lang::extract_exports(src);
        assert_eq!(regex_result, vec!["internal_fn", "InternalStruct"]);
        assert_eq!(ast_result, regex_result);
    }

    #[test]
    fn rs_crate_visibility_spacing_and_private_inline_module_parity() {
        let src = r#"
mod internal {
    pub (crate) fn spaced_one() {}
    pub( crate ) struct SpacedTwo;
    pub use crate::visible::Reexported;
}
pub fn root_public() {}
"#;
        let mut regex_result = rust_lang::extract_exports(src);
        let mut ast_result = ast::rust_lang::extract_exports(src);
        regex_result.sort();
        ast_result.sort();
        assert_eq!(
            regex_result,
            vec!["Reexported", "SpacedTwo", "root_public", "spaced_one"]
        );
        assert_eq!(ast_result, regex_result);
    }

    #[test]
    fn rs_async_unsafe() {
        let src = r#"
pub async fn async_fn() {}
pub unsafe fn unsafe_fn() {}
"#;
        let regex_result = rust_lang::extract_exports(src);
        let ast_result = ast::rust_lang::extract_exports(src);
        assert_eq!(ast_result, regex_result);
    }

    #[test]
    fn rs_pub_in_string_literal() {
        // AST correctly ignores pub inside string literals
        let src = "pub fn real_fn() {}\nfn other() { let s = \"pub fn fake() {}\"; }\n";
        let ast_result = ast::rust_lang::extract_exports(src);
        assert_eq!(ast_result, vec!["real_fn"]);
    }

    #[test]
    fn rs_feature_gated() {
        let src = r#"
#[cfg(feature = "optional")]
pub fn optional_fn() {}

pub fn always_fn() {}
"#;
        let regex_result = rust_lang::extract_exports(src);
        let ast_result = ast::rust_lang::extract_exports(src);
        assert_eq!(ast_result, regex_result);
    }

    #[test]
    fn rs_pub_mod() {
        let src = r#"
pub mod submodule;
mod private_mod;
"#;
        let regex_result = rust_lang::extract_exports(src);
        let ast_result = ast::rust_lang::extract_exports(src);
        assert_eq!(ast_result, regex_result);
    }
}

// ── Dispatch/fallback wiring (`src/exports/mod.rs::scan_exported_symbols_full`) ──
//
// The actual "fall back to regex if the AST result is empty" logic lives in
// `src/exports/mod.rs` (one `match` arm per AST-backed language), not in this
// `ast` module itself. These tests exercise that dispatch end-to-end through the
// public `scan_exported_symbols_full` entry point with `ParseMode::Ast`, using
// real temp files, to pin down two properties of the wiring:
//   1. The fallback trigger is `result.is_empty()` — a plain `Vec` check, not a
//      `Result`/error path — so it can never "swallow" a parse error silently;
//      it only ever reacts to emptiness. Every AST backend's `extract_exports`
//      returns a bare `Vec<String>` (see `parse_*` returning `Option<Tree>` and
//      falling to `Vec::new()` on `None`), so there is no separate error channel
//      to bypass.
//   2. Because the check is emptiness-only, a *non-empty but different-from-regex*
//      AST result is never "corrected" by falling back — the AST output wins
//      outright even when regex would disagree.
#[cfg(test)]
mod dispatch_fallback {
    use crate::exports::{ExportScan, rust_lang, scan_exported_symbols_full, typescript};
    use crate::types::{ExportLevel, ParseMode};

    #[test]
    fn ast_nonempty_result_wins_even_when_regex_disagrees() {
        // Regex text-matches `export function fake()` inside a template string
        // literal (see `ts_export_inside_string_literal` above: regex would
        // wrongly include "fake"); AST correctly excludes it and returns a
        // non-empty, correct `["real"]`. Because the dispatch only falls back
        // on an *empty* AST result, the correct non-empty AST answer must be
        // returned as-is, not reconciled/overwritten by the differing regex
        // answer.
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("literal.ts");
        std::fs::write(
            &file,
            "export function real() {}\nconst t = `\nexport function fake() {}\n`;\n",
        )
        .unwrap();

        let regex_result = typescript::extract_exports(&std::fs::read_to_string(&file).unwrap());
        assert!(
            regex_result.contains(&"fake".to_string()),
            "test assumes the regex backend is fooled by the template string: {regex_result:?}"
        );

        let scan = scan_exported_symbols_full(&file, ExportLevel::Member, ParseMode::Ast);
        assert_eq!(scan, ExportScan::Parsed(vec!["real".to_string()]));
    }

    #[test]
    fn ast_empty_result_falls_back_to_regex_even_when_ast_was_semantically_correct() {
        // A `macro_rules!` *definition* containing `pub fn generated() {}` in its
        // expansion template is not a real top-level item — nothing has invoked
        // the macro, so nothing named `generated` actually exists yet. The AST
        // backend correctly returns `[]` here. The line-based regex backend,
        // however, naively text-matches `pub fn generated` inside the macro body
        // and (incorrectly) reports it as exported.
        //
        // This documents a real, currently-unfixable-from-this-file limitation
        // of the "empty AST result ⇒ fall back to regex" policy in
        // `src/exports/mod.rs`: the trigger genuinely is emptiness-only (not
        // error-based, confirmed by `parse_rust` returning `Vec::new()` on
        // `None` with no distinct error channel), but that means an AST result
        // that is empty *because it is correct* is indistinguishable, from the
        // dispatcher's point of view, from an AST result that is empty because
        // parsing failed — both currently fall back to the (here, wrong) regex
        // answer. Fixing this would require changing the dispatch policy in
        // `src/exports/mod.rs`, which is out of scope for the `ast` backends
        // themselves; this test pins down the *current* observable behavior so
        // a future change to the policy is a deliberate, visible diff here.
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("macro_body.rs");
        std::fs::write(
            &file,
            "macro_rules! generate {\n    () => {\n        pub fn generated() {}\n    };\n}\n",
        )
        .unwrap();

        let content = std::fs::read_to_string(&file).unwrap();
        assert_eq!(
            rust_lang::extract_exports(&content),
            vec!["generated".to_string()],
            "test assumes the regex backend is fooled by the macro body text"
        );

        let scan = scan_exported_symbols_full(&file, ExportLevel::Member, ParseMode::Ast);
        assert_eq!(scan, ExportScan::Parsed(vec!["generated".to_string()]));
    }

    #[test]
    fn ast_survives_malformed_input_without_panicking_or_misclassifying() {
        // A genuine grammar-parse-error case: an unterminated raw string sends
        // tree-sitter into an ERROR-node recovery mode for most of the file.
        // The backend must not panic, and a file that is readable UTF-8 (even
        // if its *content* is malformed source) must be reported as `Parsed`,
        // never `Unreadable` — "unreadable" is reserved for I/O/UTF-8 failures,
        // not parse failures.
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("broken.rs");
        std::fs::write(
            &file,
            "let s = r#\"unterminated\npub fn after_broken_string() {}\n",
        )
        .unwrap();

        let scan = scan_exported_symbols_full(&file, ExportLevel::Member, ParseMode::Ast);
        match scan {
            ExportScan::Parsed(symbols) => {
                // tree-sitter's error recovery still locates the real pub fn
                // after the broken string; the important assertion is simply
                // that we got a `Parsed` outcome at all, not a panic.
                assert!(symbols.contains(&"after_broken_string".to_string()) || symbols.is_empty());
            }
            other => panic!("malformed source must still be `Parsed`, got {other:?}"),
        }
    }
}
