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
/// Every named subroutine declaration ("sub NAME { ... }") counts as exported,
/// regardless of scope/package or leading `our` — Perl has no real sub-level
/// privacy convention, matching the permissive behavior of the regex reference.
pub fn extract_exports(content: &str) -> Vec<String> {
    let tree = match parse_perl(content) {
        Some(t) => t,
        None => return Vec::new(),
    };

    let root = tree.root_node();
    let src = content.as_bytes();
    let mut symbols = Vec::new();

    collect_subs(&root, src, &mut symbols);

    symbols
}

/// Walk the whole tree collecting every subroutine declaration's name.
fn collect_subs(node: &Node, src: &[u8], symbols: &mut Vec<String>) {
    if node.kind() == "function_definition"
        && let Some(name_node) = node.child_by_field_name("name")
        && let Ok(name) = name_node.utf8_text(src)
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
}
