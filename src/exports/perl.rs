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

/// An identifier-shaped token within an `EXPORT_ARRAY` body — deliberately ignores
/// surrounding quotes/commas/whitespace so it works uniformly across both the bareword
/// `qw(...)` form and the quoted-list `('foo', 'bar')` form without needing separate
/// per-delimiter tokenizers.
static EXPORT_ARRAY_TOKEN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[A-Za-z_]\w*").unwrap());

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

    let mut export_list: Vec<String> = Vec::new();
    for caps in EXPORT_ARRAY.captures_iter(&stripped) {
        let body = caps
            .iter()
            .skip(1)
            .find_map(|m| m)
            .map(|m| m.as_str())
            .unwrap_or("");
        for tok in EXPORT_ARRAY_TOKEN.find_iter(body) {
            let n = tok.as_str().to_string();
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
}
