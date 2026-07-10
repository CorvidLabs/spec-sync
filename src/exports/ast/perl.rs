use tree_sitter::{Node, Parser, Tree};

/// Parse Perl source into a tree-sitter AST.
fn parse_perl(content: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let language = tree_sitter_perl::LANGUAGE.into();
    parser.set_language(&language).ok()?;
    parser.parse(content, None)
}

/// Extract exported symbols from Perl source using tree-sitter AST.
///
/// Mirrors the regex reference's two-step policy (see `src/exports/perl.rs`):
/// if the module declares an `@EXPORT`/`@EXPORT_OK` array (`Exporter`'s
/// public-API list, with or without a leading `our`), those listed names ARE
/// the public API, full stop -- every other `sub`, regardless of how it's
/// declared, is excluded. Otherwise, every named subroutine declaration
/// counts as exported EXCEPT ones using the leading-underscore
/// "private by convention" naming shared with Python/Bash elsewhere in this
/// codebase.
///
/// Before this policy was implemented here, the AST backend captured every
/// `sub` unconditionally and never looked at `@EXPORT`/`@EXPORT_OK` at all.
/// Because that almost never yields an *empty* result for a real Perl file
/// with any subs, the "empty AST result -> fall back to regex" dispatch
/// policy in `src/exports/mod.rs` could never actually reach the regex
/// backend's hardened `@EXPORT_OK`/underscore filtering for Perl -- a
/// wrong-but-nonempty masking bug.
pub fn extract_exports(content: &str) -> Vec<String> {
    let tree = match parse_perl(content) {
        Some(t) => t,
        None => return Vec::new(),
    };

    let root = tree.root_node();
    let src = content.as_bytes();

    let mut export_list = Vec::new();
    collect_export_arrays(&root, src, &mut export_list);
    if !export_list.is_empty() {
        return export_list;
    }

    let mut symbols = Vec::new();
    collect_subs(&root, src, &mut symbols);
    symbols
}

/// Walk the whole tree collecting every subroutine declaration's name,
/// excluding leading-underscore "private by convention" names. Only used
/// when no `@EXPORT`/`@EXPORT_OK` array is present (see `extract_exports`).
fn collect_subs(node: &Node, src: &[u8], symbols: &mut Vec<String>) {
    if node.kind() == "function_definition"
        && let Some(name_node) = node.child_by_field_name("name")
        && let Ok(name) = name_node.utf8_text(src)
        && !name.starts_with('_')
    {
        let n = name.to_string();
        if !symbols.contains(&n) {
            symbols.push(n);
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_subs(&child, src, symbols);
    }
}

/// Walk the whole tree looking for `our @EXPORT = (...)` / `our @EXPORT_OK =
/// (...)` assignments (the `our` is optional), accumulating every listed
/// name across all such statements -- a file may declare both, or grow the
/// list across several statements.
fn collect_export_arrays(node: &Node, src: &[u8], names: &mut Vec<String>) {
    if node.kind() == "binary_expression" {
        let mut cursor = node.walk();
        let vars: Vec<Node> = node
            .children_by_field_name("variable", &mut cursor)
            .collect();
        if let [lhs, rhs] = vars.as_slice()
            && is_export_array_name(lhs, src)
        {
            collect_list_values(rhs, src, names);
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_export_arrays(&child, src, names);
    }
}

/// True if `node` is the LHS of an `@EXPORT`/`@EXPORT_OK` assignment: either
/// a bare `array_variable` (`@EXPORT_OK = (...)`) or a `variable_declaration`
/// wrapping one (`our @EXPORT_OK = (...)`).
fn is_export_array_name(node: &Node, src: &[u8]) -> bool {
    let var = match node.kind() {
        "array_variable" => *node,
        "variable_declaration" => match node.child_by_field_name("variable_name") {
            Some(v) => v,
            None => return false,
        },
        _ => return false,
    };
    matches!(
        var.utf8_text(src).unwrap_or_default(),
        "@EXPORT" | "@EXPORT_OK"
    )
}

/// Extract the string values out of the RHS of an export-array assignment:
/// a `qw(...)`/`qw[...]`/`qw{...}`/`qw/.../` bareword list (uniform
/// `word_list_qw` node regardless of delimiter), or a plain quoted-list
/// literal (`('foo', 'bar')` / `("foo", "bar")`).
fn collect_list_values(node: &Node, src: &[u8], names: &mut Vec<String>) {
    match node.kind() {
        "word_list_qw" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "list_item" {
                    push_name(child.utf8_text(src).unwrap_or_default(), names);
                }
            }
        }
        "array" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if matches!(
                    child.kind(),
                    "string_single_quoted" | "string_double_quoted"
                ) {
                    let text = child.utf8_text(src).unwrap_or_default();
                    push_name(text.trim_matches(['\'', '"']), names);
                }
            }
        }
        _ => {}
    }
}

/// Push a trimmed, non-empty, deduplicated name onto `names`.
fn push_name(raw: &str, names: &mut Vec<String>) {
    let name = raw.trim();
    if !name.is_empty() && !names.contains(&name.to_string()) {
        names.push(name.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perl_exports() {
        let src = r#"
# Perl program
sub my_function {
    my ($a, $b) = @_;
    return $a + $b;
}

our sub exported_function {
    print "Raku";
}

sub another_one is export {
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"my_function".to_string()));
        assert!(symbols.contains(&"exported_function".to_string()));
        assert!(symbols.contains(&"another_one".to_string()));
    }

    #[test]
    fn test_ignores_sub_in_comment_and_string() {
        // AST-native: text that merely looks like a sub declaration inside a
        // comment or a string literal must not be captured.
        let src = r#"
# sub fake_in_comment {
sub real_fn {
    my $s = "sub fake_in_string { }";
    return $s;
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_fn"]);
    }

    #[test]
    fn test_learnxinyminutes_modules_and_objects_excerpt() {
        // Real excerpt from learnxinyminutes.com/perl (the "Modules" and
        // "Objects" sections), unmodified aside from trimming surrounding
        // material. Exercises real-world idioms: back-to-back `package`
        // blocks, regex substitutions inside a sub body (`s/^\s+//`), the
        // `bless` constructor pattern, and prose comments that mention
        // "subroutines" without being sub declarations.
        let src = r#"
package MyModule;
use strict;
use warnings;

sub trim {
  my $string = shift;
  $string =~ s/^\s+//;
  $string =~ s/\s+$//;
  return $string;
}

1;

# From elsewhere:

use MyModule;
MyModule::trim($string);

# The Exporter module can help with making subroutines exportable, so
# they can be used like this:

use MyModule 'trim';
trim($string);

# Many Perl modules can be downloaded from CPAN (http://www.cpan.org/)
# and provide a range of features to help you avoid reinventing the
# wheel.  A number of popular modules like Exporter are included with
# the Perl distribution itself. See perlmod for more details on modules
# in Perl.

#### Objects

# Objects in Perl are just references that know which class (package)
# they belong to, so that methods (subroutines) called on it can be
# found there. The bless function is used in constructors (usually new)
# to set this up. However, you never need to call it yourself if you use
# a module like Moose or Moo (see below).

package MyCounter;
use strict;
use warnings;

sub new {
  my $class = shift;
  my $self = {count => 0};
  return bless $self, $class;
}

sub count {
  my $self = shift;
  return $self->{count};
}

sub increment {
  my $self = shift;
  $self->{count}++;
}

1;
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec![
                "trim".to_string(),
                "new".to_string(),
                "count".to_string(),
                "increment".to_string(),
            ]
        );
    }

    #[test]
    fn test_dedup_and_nested() {
        // Same-named sub declared twice should only appear once; nested subs
        // (still real named declarations) are also collected, matching the
        // regex reference which scans line-by-line without scope filtering.
        let src = r#"
sub outer {
    sub inner {
        return 1;
    }
    return inner();
}

sub outer {
    return 2;
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"outer".to_string()));
        assert!(symbols.contains(&"inner".to_string()));
        assert_eq!(symbols.iter().filter(|s| *s == "outer").count(), 1);
    }

    #[test]
    fn test_export_ok_restricts_to_listed_names() {
        // Regression test for the core fix: before it existed, the AST
        // backend ignored `@EXPORT_OK` entirely and returned every `sub`
        // unconditionally (including `_private_helper` and `not_listed`),
        // permanently masking the regex backend's `@EXPORT_OK`/underscore
        // filtering since a nonempty-but-wrong AST result never triggers the
        // "empty result -> fall back to regex" dispatch policy.
        let src = r#"
package Foo;
our @EXPORT_OK = qw(public_one public_two);

sub public_one { 1 }
sub public_two { 1 }
sub _private_helper { 1 }
sub not_listed { 1 }
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec!["public_one".to_string(), "public_two".to_string()],
            "got {symbols:?}"
        );
    }

    #[test]
    fn test_underscore_convention_only_applies_without_export_list() {
        let src = "sub public_fn { 1 }\nsub _private_fn { 1 }\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["public_fn".to_string()]);
    }

    #[test]
    fn test_export_list_overrides_underscore_convention() {
        // An explicit `@EXPORT_OK` listing is authoritative -- even a
        // leading-underscore name is exported if the module says so.
        let src = "our @EXPORT_OK = qw(_explicit);\nsub _explicit { 1 }\nsub _not_listed { 1 }\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["_explicit".to_string()]);
    }

    #[test]
    fn test_export_and_export_ok_accumulate_together() {
        let src = "our @EXPORT = qw(always);\nour @EXPORT_OK = qw(optional);\nsub always { 1 }\nsub optional { 1 }\nsub neither { 1 }\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["always".to_string(), "optional".to_string()]);
    }

    #[test]
    fn test_export_ok_all_qw_delimiters_and_quoted_list_and_no_our() {
        // `qw(...)`/`qw[...]`/`qw{...}`/`qw/.../` all produce the same
        // `word_list_qw` node regardless of delimiter; a plain quoted list
        // and a declaration without `our` must also work.
        for src in [
            "our @EXPORT_OK = qw(foo bar);\n",
            "our @EXPORT_OK = qw[foo bar];\n",
            "our @EXPORT_OK = qw{foo bar};\n",
            "our @EXPORT_OK = qw/foo bar/;\n",
            "our @EXPORT_OK = ('foo', 'bar');\n",
            "our @EXPORT_OK = (\"foo\", \"bar\");\n",
            "@EXPORT_OK = qw(foo bar);\n",
        ] {
            let symbols = extract_exports(src);
            assert_eq!(
                symbols,
                vec!["foo".to_string(), "bar".to_string()],
                "failed for {src:?}, got {symbols:?}"
            );
        }
    }

    #[test]
    fn test_begin_block_subs_are_captured() {
        let src = "BEGIN {\n  sub setup { 1 }\n}\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["setup".to_string()]);
    }

    #[test]
    fn test_heredoc_body_not_parsed_as_code() {
        let src =
            "sub real {\n    my $doc = <<END;\nsub fake_in_heredoc { }\nEND\n    return $doc;\n}\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real".to_string()]);
    }

    #[test]
    fn test_pod_block_fake_sub_excluded() {
        let src = "=pod\n\nsub fake_in_pod { }\n\n=cut\n\nsub real { 1 }\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real".to_string()]);
    }

    #[test]
    fn test_prototyped_sub_captured() {
        let src = "sub foo ($$;@) {\n    return 1;\n}\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["foo".to_string()]);
    }

    #[test]
    fn test_multi_package_file_collects_all_subs_unqualified() {
        let src = "package A;\nsub a_func { 1 }\npackage B;\nsub b_func { 1 }\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["a_func".to_string(), "b_func".to_string()]);
    }

    #[test]
    fn test_moo_role_requires_is_not_a_sub_export() {
        let src = "package Foo::Role;\nuse Moo::Role;\nrequires 'do_thing';\nsub real { 1 }\n";
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real".to_string()]);
    }

    #[test]
    fn test_typeglob_dynamic_sub_assignment_is_a_known_limitation() {
        // `*{"Pkg::$name"} = sub { ... }` (dynamic sub installation via a
        // typeglob) cannot be statically resolved to a name by any
        // structural tool -- the name only exists at runtime, built from a
        // string. This documents the current, honest limitation (empty
        // result here) rather than claiming coverage that doesn't exist.
        let src = "for my $name (qw(a b)) {\n    *{\"Pkg::$name\"} = sub { 1 };\n}\n";
        let symbols = extract_exports(src);
        assert!(symbols.is_empty(), "got {symbols:?}");
    }
}
