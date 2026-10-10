//! Rust, TypeScript and Python import regressions.

use super::*;
use crate::deps::test_support::{create_source, create_spec};
use crate::deps::validate_deps;
use tempfile::TempDir;

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
