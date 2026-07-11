---
spec: exports.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/exports/mod.rs` | cargo test exports:: | No inline tests in `mod.rs` itself; covered indirectly via the per-language backends and `tests/integration.rs`. Add focused coverage for `get_exported_symbols`, `get_exported_symbols_full`, `is_test_file`, `is_source_file`, `has_extension` before risky changes |
| `src/exports/typescript.rs` | cargo test exports::typescript:: | `test_basic_exports`, `test_comments_stripped`, `test_re_exports`, `test_wildcard_namespace_export`, `test_wildcard_export_with_resolver`, `test_wildcard_export_without_resolver` |
| `src/exports/python.rs` | cargo test exports::python:: | `test_python_all`, `test_python_no_all`, `test_python_all_single_quotes`, `test_python_all_overrides_conventions`, `test_python_decorators_ignored`, `test_python_nested_not_captured` |
| `src/exports/rust_lang.rs` | cargo test rust_lang | `test_rust_exports`, `test_pub_crate_included`, restricted-visibility exclusions, crate-visible re-exports, string/comment stripping, `test_real_registry_rs` |
| `src/exports/go.rs` | cargo test exports::go:: | `test_go_exports`, `test_go_methods`, `test_go_comments_stripped`, `test_go_interface_declarations`, `test_go_const_var_groups`, `test_go_value_receiver` |
| `src/exports/java.rs` | cargo test exports::java:: | `test_java_exports`, `test_java_abstract`, `test_java_comments_stripped`, `test_java_generics`, `test_java_annotation_type`, `test_java_private_and_protected_excluded` |
| `src/exports/kotlin.rs` | cargo test exports::kotlin:: | `test_kotlin_exports`, `test_kotlin_visibility`, `test_kotlin_suspend` |
| `src/exports/swift.rs` | cargo test exports::swift:: | `test_swift_exports`, `test_swift_init`, `test_swift_open` |
| `src/exports/dart.rs` | cargo test exports::dart:: | `test_dart_exports`, `test_dart_private`, `test_dart_comments_stripped`, `test_dart_abstract_class`, `test_dart_future_stream_return`, `test_dart_const_vs_final` |
| `src/exports/csharp.rs` | cargo test exports::csharp:: | `test_csharp_exports`, `test_csharp_async`, `test_csharp_comments_stripped`, `test_csharp_static_class`, `test_csharp_private_internal_excluded`, `test_csharp_sealed_partial` |
| `src/exports/php.rs` | cargo test exports::php:: | `test_php_class_and_methods`, `test_php_final_readonly`, `test_php_skips_magic_methods` |
| `src/exports/ruby.rs` | cargo test exports::ruby:: | `test_ruby_class_and_methods`, `test_ruby_top_level_functions`, `test_ruby_visibility_toggle`, `test_ruby_skips_initialize` |
| `src/exports/yaml.rs` | cargo test exports::yaml:: | `test_github_actions_workflow`, `test_docker_compose`, `test_anchors`, `test_top_level_only`, `test_four_space_indentation`, `test_four_space_nested_not_extracted` |
| `src/exports/ast/mod.rs` | cargo test exports::ast::tests:: | Parity tests in `ast/tests.rs` cross-check AST vs regex output: `ts_basic_parity`, `ts_re_exports_with_alias`, `ts_wildcard_with_resolver`, `py_basic_parity`, `py_all_takes_precedence`, `rs_basic_parity`, `rs_pub_crate`, `rs_feature_gated`, `rs_pub_mod` |
| `src/exports/ast/typescript.rs` | cargo test exports::ast::typescript:: | `test_basic_exports`, `test_re_exports_with_alias`, `test_wildcard_namespace`, `test_wildcard_with_resolver`, `test_default_export`, `test_async_abstract`, `test_conditional_export`, `test_export_type_clause`, `test_comments_not_exported` |
| `src/exports/ast/python.rs` | cargo test exports::ast::python:: | `test_python_all`, `test_python_no_all`, `test_python_nested_not_captured`, `test_python_dunder_excluded`, `test_python_all_overrides`, `test_decorated_functions`, `test_conditional_import_init` |
| `src/exports/ast/rust_lang.rs` | cargo test exports::ast::rust_lang:: | `test_rust_exports`, `test_pub_crate`, `test_async_unsafe`, `test_ignores_pub_in_strings`, `test_feature_gated`, `test_pub_mod` |
| `tests/integration.rs` | cargo test --test integration fix_adds_undocumented_exports_to_spec | End-to-end fixture: `fix_adds_undocumented_exports_to_spec` |
| `tests/integration.rs` | cargo test --test integration fix_does_not_duplicate_already_documented_exports | End-to-end fixture: `fix_does_not_duplicate_already_documented_exports` |
| `tests/integration.rs` | cargo test --test integration diff_detects_removed_exports | End-to-end fixture: `diff_detects_removed_exports` |
| `tests/integration/languages.rs` | cargo test --test integration rust_multi_file_pub_crate_exports_pass_strict_in_regex_and_ast_modes | Strict two-file regression proves `pub` plus `pub(crate)` coverage in regex and AST modes |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Extract TypeScript exports | a `.ts` file containing `export function authenticate(token: string): User` | `get_exported_symbols(path)` is called | includes "authenticate" in the returned vector |
| Extract Rust pub items | a `.rs` file containing `pub fn validate_spec(...)` | `get_exported_symbols(path)` is called | includes "validate_spec" in the returned vector |
| Unsupported file type | an unsupported file (e.g., `.txt`) | `get_exported_symbols(path)` is called | returns an empty vector |
| Extract PHP exports with visibility | a `.php` file with a `class AuthService` containing `public function validate()`, `private function internalCheck()`, and `public const DEFAULT_TTL` | `get_exported_symbols(path)` is called | includes "AuthService", "validate", "DEFAULT_TTL" but not "internalCheck" |
| Ruby visibility toggles | a `.rb` file with `class Foo` containing `def public_method` then `private` then `def secret_method` | `get_exported_symbols(path)` is called | includes "Foo" and "public_method" but not "secret_method" |
| Python __all__ takes precedence | a `.py` file with `__all__ = ["create_auth", "AuthService"]` and additional top-level functions | `get_exported_symbols(path)` is called | returns only the symbols listed in `__all__`, not all top-level definitions |
| Go uppercase convention | a `.go` file with `func CreateAuth()` and `func privateHelper()` | `get_exported_symbols(path)` is called | includes "CreateAuth" but not "privateHelper" |
| Kotlin default visibility | a `.kt` file with `fun publicFun()` and `private fun privateFun()` | `get_exported_symbols(path)` is called | includes "publicFun" (public by default) but not "privateFun" |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| File cannot be read | Returns empty vector | Keep or add a focused assertion before changing this behavior |
| Unknown file extension | Returns empty vector | Keep or add a focused assertion before changing this behavior |
| File has no exports | Returns empty vector | Keep or add a focused assertion before changing this behavior |
| Binary or non-text file | Returns empty vector (read_to_string fails gracefully) | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/exports/mod.rs`, `src/exports/typescript.rs`, `src/exports/python.rs`, `src/exports/rust_lang.rs`, `src/exports/go.rs`, `src/exports/java.rs`, `src/exports/kotlin.rs`, `src/exports/swift.rs`, `src/exports/dart.rs`, `src/exports/csharp.rs`, `src/exports/php.rs`, `src/exports/ruby.rs`, `src/exports/yaml.rs`, `src/exports/ast/mod.rs`, `src/exports/ast/typescript.rs`, `src/exports/ast/python.rs`, `src/exports/ast/rust_lang.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `./target/release/specsync score --all`.
