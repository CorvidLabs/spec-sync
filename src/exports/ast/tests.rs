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
    fn rs_pub_crate() {
        let src = r#"
pub(crate) fn internal_fn() {}
pub(crate) struct InternalStruct {}
"#;
        let regex_result = rust_lang::extract_exports(src);
        let ast_result = ast::rust_lang::extract_exports(src);
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
