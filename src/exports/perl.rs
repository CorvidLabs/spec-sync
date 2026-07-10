use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"#.*$").unwrap());

/// Perl subroutines, including optional our/is export
static PERL_SUB: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[^\S\n]*(?:our\s+)?sub\s+(\w+)(?:\s+is\s+export)?").unwrap()
});

/// `Exporter`'s public-API arrays: `our @EXPORT = qw(foo bar);` (always exported when the
/// module is `use`d) or `our @EXPORT_OK = qw(...);` (exportable on request). Supports the
/// common `qw(...)`/`qw[...]`/`qw{...}`/`qw/.../` bareword-list delimiters as well as a
/// plain quoted list (`('foo', 'bar')`). When either array is present, only the names
/// actually listed make up the module's public surface, mirroring how an explicit export
/// list is already the authoritative source of truth for Haskell and Erlang elsewhere in
/// this codebase (rather than "every sub found, minus a naming convention").
static EXPORT_ARRAY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?s)@EXPORT(?:_OK)?\s*=\s*(?:qw\s*\(([^)]*)\)|qw\s*\[([^\]]*)\]|qw\s*\{([^}]*)\}|qw\s*/([^/]*)/|\(([^)]*)\))",
    )
    .unwrap()
});

/// `push @EXPORT, ...` / `push @EXPORT_OK, ...`: the common idiom for appending to an
/// export array after its initial declaration (e.g. conditionally, or split across
/// multiple statements). `EXPORT_ARRAY` above requires `@EXPORT(?:_OK)?\s*=` immediately
/// after the array name, which a `push` statement never satisfies (a comma follows the
/// array name there, not `=`), so without this second regex any names appended via
/// `push` are silently dropped from the tracked public API. Mirrors the same delimiter
/// alternatives as `EXPORT_ARRAY`, plus a final catch-all up to the statement-ending
/// `;` for a bare comma-separated quoted list (`push @EXPORT_OK, 'bar', 'baz';`).
static EXPORT_PUSH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?s)push\s+@EXPORT(?:_OK)?\s*,\s*(?:qw\s*\(([^)]*)\)|qw\s*\[([^\]]*)\]|qw\s*\{([^}]*)\}|qw\s*/([^/]*)/|\(([^)]*)\)|([^;]*));",
    )
    .unwrap()
});

/// An identifier-shaped token within an `EXPORT_ARRAY`/`EXPORT_PUSH` body — deliberately
/// ignores surrounding quotes/commas/whitespace so it works uniformly across both the
/// bareword `qw(...)` form and the quoted-list `('foo', 'bar')` form without needing
/// separate per-delimiter tokenizers.
static EXPORT_ARRAY_TOKEN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[A-Za-z_]\w*").unwrap());

/// Extract identifier tokens from an export-list body, but only when the body is a
/// literal list the regex can statically resolve. A body containing a `$` or `@`
/// sigil (e.g. `@{$EXPORT_TAGS{all}}`, dereferencing a `%EXPORT_TAGS` hash entry) is a
/// runtime Perl expression, not a literal list -- the regex has no way to evaluate it.
/// Blindly tokenizing it would pick up the referenced variable's own name/hash-keys
/// (e.g. `EXPORT_TAGS`, `all`) as if they were export names. Returning `None` here lets
/// the caller correctly skip the match and fall through to the underscore-naming-
/// convention scan instead of returning that garbage.
fn extract_export_names(body: &str) -> Option<Vec<String>> {
    if body.contains('$') || body.contains('@') {
        return None;
    }
    Some(
        EXPORT_ARRAY_TOKEN
            .find_iter(body)
            .map(|m| m.as_str().to_string())
            .collect(),
    )
}

/// Strip Perl POD documentation blocks (`=head1` ... `=cut`, `=pod` ... `=cut`,
/// `=item`, `=over`/`=back`, etc.) before scanning for code.
///
/// Per perlpod, a POD block starts with a line beginning with `=` followed by
/// an identifier (in column 0) and runs until a line beginning with `=cut`
/// (inclusive of both boundary lines). Example code embedded in SYNOPSIS/
/// documentation sections is prose, not real subroutine declarations, and
/// must not be scanned as source.
fn strip_pod(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut in_pod = false;

    for line in content.split_inclusive('\n') {
        let bare = line.trim_end_matches(['\n', '\r']);

        if in_pod {
            if bare.starts_with("=cut") {
                in_pod = false;
            }
            continue;
        }

        let starts_pod_directive =
            bare.starts_with('=') && bare[1..].chars().next().is_some_and(char::is_alphabetic);

        if starts_pod_directive {
            in_pod = true;
            continue;
        }

        out.push_str(line);
    }

    out
}

/// Extract public symbols from Perl source code.
///
/// Perl has no native visibility keyword: without an `Exporter` `@EXPORT`/`@EXPORT_OK`
/// array, every `sub` is technically callable via full package qualification. This
/// mirrors Perl's own real-world convention (used throughout CPAN and this project's
/// sibling backends) in two steps: if the module declares `@EXPORT`/`@EXPORT_OK`, those
/// listed names ARE the public API, full stop; otherwise, fall back to the leading-
/// underscore-is-private naming convention shared with Python/Bash.
pub fn extract_exports(content: &str) -> Vec<String> {
    let no_pod = strip_pod(content);
    let stripped = COMMENT_SINGLE.replace_all(&no_pod, "");

    // Collect matches from both the initial-assignment (`@EXPORT(_OK) = ...`) and
    // `push @EXPORT(_OK), ...` regexes, then process them in the order they occur in
    // the source so a `push` that appends to an earlier declaration accumulates onto
    // it rather than being processed out of order.
    let mut export_matches: Vec<regex::Captures> = EXPORT_ARRAY.captures_iter(&stripped).collect();
    export_matches.extend(EXPORT_PUSH.captures_iter(&stripped));
    export_matches.sort_by_key(|caps| caps.get(0).map(|m| m.start()).unwrap_or(0));

    let mut export_list: Vec<String> = Vec::new();
    for caps in export_matches {
        let body = caps
            .iter()
            .skip(1)
            .find_map(|m| m)
            .map(|m| m.as_str())
            .unwrap_or("");
        let Some(names) = extract_export_names(body) else {
            continue;
        };
        for n in names {
            if !export_list.contains(&n) {
                export_list.push(n);
            }
        }
    }

    if !export_list.is_empty() {
        return export_list;
    }

    let mut symbols = Vec::new();

    for caps in PERL_SUB.captures_iter(&stripped) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str();
            if !n.starts_with('_') && !symbols.contains(&n.to_string()) {
                symbols.push(n.to_string());
            }
        }
    }

    symbols
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
    fn test_export_ok_array_restricts_to_listed_names() {
        // Regression test: Perl previously had no visibility filtering at all -- every
        // `sub` was captured unconditionally, regardless of `@EXPORT_OK` or a
        // leading-underscore "private by convention" name. When `@EXPORT`/`@EXPORT_OK`
        // is present, only the names it actually lists make up the public surface.
        let src = r#"
package MyModule;
our @EXPORT_OK = qw(public_api another_public);

sub public_api {
    return 1;
}

sub another_public {
    return 2;
}

sub _helper {
    return 3;
}

sub undocumented_internal {
    return 4;
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec!["public_api".to_string(), "another_public".to_string()]
        );
        assert!(!symbols.contains(&"_helper".to_string()));
        assert!(!symbols.contains(&"undocumented_internal".to_string()));
    }

    #[test]
    fn test_leading_underscore_sub_excluded_without_export_array() {
        // Regression test: without any `@EXPORT`/`@EXPORT_OK` declaration, Perl's
        // real-world naming convention (shared with Python/Bash) treats a leading
        // underscore as "private" -- previously every `sub` leaked unfiltered.
        let src = r#"
package MyModule;

sub public_helper {
    return 1;
}

sub _private_helper {
    return 2;
}
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"public_helper".to_string()));
        assert!(!symbols.contains(&"_private_helper".to_string()));
    }

    #[test]
    fn test_export_array_quoted_list_form() {
        // `@EXPORT` can also be written as a plain quoted list rather than `qw(...)`.
        let src = r#"
package MyModule;
our @EXPORT = ('foo', 'bar');

sub foo { return 1; }
sub bar { return 2; }
sub baz { return 3; }
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["foo".to_string(), "bar".to_string()]);
    }

    #[test]
    fn test_pod_synopsis_example_sub_is_not_exported() {
        // SYNOPSIS/documentation sections routinely embed example `sub`
        // definitions to illustrate usage; those are prose, not real
        // subroutine declarations in this file, and must not leak into the
        // tracked public API.
        let src = r#"
package MyModule;

=head1 SYNOPSIS

    use MyModule;
    my $obj = MyModule->new;
    sub example_usage {
        return $obj->do_thing;
    }

=head1 METHODS

=cut

sub real_method {
    return 42;
}

1;
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["real_method".to_string()]);
    }

    #[test]
    fn test_pod_block_without_leading_blank_line_and_multiple_sections() {
        // =pod ... =cut and =item entries inside =over/=back should also be
        // stripped, even when POD blocks are back-to-back with no blank line
        // separating them from surrounding code.
        let src = r#"
package Widget;
=pod

=over

=item sub not_a_real_sub { }

=back

=cut
sub build {
    return {};
}
=head1 AUTHOR

Some Author

=cut
sub destroy {
    return 1;
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["build".to_string(), "destroy".to_string()]);
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
    fn test_moo_role_requires_is_not_a_sub_declaration() {
        // Moo::Role's `requires 'NAME'` declares a contract method but is not
        // itself a sub declaration, and must not be captured as one.
        let src = r#"
package Authenticatable;
use Moo::Role;

requires 'validate';

sub helper {
    return 1;
}

sub another_helper {
    my ($self) = @_;
    return $self->helper;
}
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"validate".to_string()));
        assert!(symbols.contains(&"helper".to_string()));
        assert!(symbols.contains(&"another_helper".to_string()));
    }

    #[test]
    fn test_export_ok_hash_tag_deref_falls_back_to_underscore_convention() {
        // Regression test for a real bug: `our @EXPORT_OK = (@{$EXPORT_TAGS{all}});`
        // dereferences a `%EXPORT_TAGS` hash entry -- a runtime expression the regex
        // cannot statically resolve. The plain-paren fallback branch of EXPORT_ARRAY
        // used to blindly tokenize the captured body, previously returning the hash
        // variable's own name/key (`["EXPORT_TAGS", "all"]`) as if they were real
        // export names. It must instead detect the `$`/`@` sigils, treat the match as
        // unresolvable, and fall through to the underscore-naming-convention scan.
        let src = r#"
package MyModule;
our %EXPORT_TAGS = (all => [qw(foo bar)]);
our @EXPORT_OK = (@{$EXPORT_TAGS{all}});

sub foo { return 1; }
sub bar { return 2; }
sub _hidden { return 3; }
"#;
        let symbols = extract_exports(src);
        assert!(!symbols.contains(&"EXPORT_TAGS".to_string()));
        assert!(!symbols.contains(&"all".to_string()));
        assert!(symbols.contains(&"foo".to_string()));
        assert!(symbols.contains(&"bar".to_string()));
        assert!(!symbols.contains(&"_hidden".to_string()));
    }

    #[test]
    fn test_push_export_ok_accumulates_after_initial_assignment() {
        // Regression test for a real bug: `push @EXPORT_OK, qw(bar baz);` following an
        // initial `our @EXPORT_OK = qw(foo);` was silently dropped -- EXPORT_ARRAY only
        // matches `@EXPORT_OK\s*=`, which a `push` statement's `@EXPORT_OK,` never
        // satisfies. Names appended via `push` must be accumulated onto the initial
        // declaration, not discarded.
        let src = r#"
package MyModule;
our @EXPORT_OK = qw(foo);
push @EXPORT_OK, qw(bar baz);

sub foo { return 1; }
sub bar { return 2; }
sub baz { return 3; }
sub _hidden { return 4; }
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec!["foo".to_string(), "bar".to_string(), "baz".to_string()]
        );
    }

    #[test]
    fn test_push_export_ok_plain_quoted_list() {
        // `push` can also append a bare comma-separated quoted list rather than
        // `qw(...)`; the catch-all `([^;]*)` branch of EXPORT_PUSH must handle it too.
        let src = r#"
package MyModule;
our @EXPORT_OK = qw(foo);
push @EXPORT_OK, 'bar', 'baz';

sub foo { return 1; }
sub bar { return 2; }
sub baz { return 3; }
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec!["foo".to_string(), "bar".to_string(), "baz".to_string()]
        );
    }

    #[test]
    fn test_isa_exporter_base_with_export_ok_still_works() {
        // `use base 'Exporter'; our @ISA = ('Exporter');` is the classic pre-`use
        // parent` idiom for declaring a module as an Exporter subclass. Its `@ISA`
        // array must not be confused with `@EXPORT_OK`, and the latter must still be
        // parsed correctly alongside it.
        let src = r#"
package MyModule;
use base 'Exporter';
our @ISA = ('Exporter');
our @EXPORT_OK = qw(public_api);

sub public_api {
    return 1;
}

sub _helper {
    return 2;
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["public_api".to_string()]);
        assert!(!symbols.contains(&"Exporter".to_string()));
        assert!(!symbols.contains(&"_helper".to_string()));
    }

    #[test]
    fn test_export_tags_hash_does_not_confuse_plain_export_ok_parsing() {
        // A `%EXPORT_TAGS` hash declared alongside a plain (non-dereferencing)
        // `@EXPORT_OK` list must not interfere with parsing the latter: `%EXPORT_TAGS`
        // has no literal `@EXPORT` substring (it's prefixed with `%`, not `@`), so the
        // EXPORT_ARRAY/EXPORT_PUSH regexes must only match the real `@EXPORT_OK` line.
        let src = r#"
package MyModule;
our %EXPORT_TAGS = (all => [qw(foo bar)]);
our @EXPORT_OK = qw(foo bar);

sub foo { return 1; }
sub bar { return 2; }
sub _hidden { return 3; }
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["foo".to_string(), "bar".to_string()]);
        assert!(!symbols.contains(&"EXPORT_TAGS".to_string()));
    }

    #[test]
    fn test_export_array_spanning_multiple_physical_lines() {
        // `@EXPORT`/`@EXPORT_OK` bareword lists are frequently wrapped across multiple
        // lines for readability; the export-array body must still be captured in full.
        let src = r#"
package MyModule;
our @EXPORT_OK = qw(
    foo
    bar
    baz
);

sub foo { return 1; }
sub bar { return 2; }
sub baz { return 3; }
sub _hidden { return 4; }
"#;
        let symbols = extract_exports(src);
        assert_eq!(
            symbols,
            vec!["foo".to_string(), "bar".to_string(), "baz".to_string()]
        );
    }

    #[test]
    fn test_no_exporter_usage_falls_back_to_underscore_convention() {
        // A module that never mentions Exporter, @EXPORT, or @EXPORT_OK at all (not
        // even @ISA) must still fall back cleanly to the leading-underscore naming
        // convention, with no export-array parsing accidentally triggering on
        // unrelated code.
        let src = r#"
package PlainModule;
use strict;
use warnings;

sub public_thing {
    return 1;
}

sub _private_thing {
    return 2;
}
"#;
        let symbols = extract_exports(src);
        assert_eq!(symbols, vec!["public_thing".to_string()]);
        assert!(!symbols.contains(&"_private_thing".to_string()));
    }
}
