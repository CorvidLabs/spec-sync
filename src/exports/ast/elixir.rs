use tree_sitter::{Node, Parser, Tree};

/// Parse Elixir source into a tree-sitter AST.
fn parse_elixir(content: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let language = tree_sitter_elixir::LANGUAGE.into();
    parser.set_language(&language).ok()?;
    parser.parse(content, None)
}

/// Extract public symbols from Elixir source using tree-sitter AST.
///
/// Elixir has no dedicated syntax for module/function definitions --
/// `defmodule`, `def`, `defp`, `defmacro`, and `defmacrop` are all ordinary
/// macro calls. This walks every `call` node in the tree (not just top-level)
/// looking for those macro names, mirroring the regex reference which matches
/// per source line regardless of nesting depth.
///
/// Handles:
/// - `defmodule Name do ... end`, including dotted aliases like `Foo.Bar.Baz`
/// - `def name(args) do ... end` / `def name do ... end` (no-parens form)
/// - `def name(args), do: expr` (keyword-list form, no `do_block`)
/// - `def name(args) when guard do ... end` (guard clauses)
/// - `defmacro` (exported) vs `defp` / `defmacrop` (excluded)
/// - Correctly ignores `def`/`defmodule`-looking text inside strings or
///   comments (AST-native)
pub fn extract_exports(content: &str) -> Vec<String> {
    let tree = match parse_elixir(content) {
        Some(t) => t,
        None => return Vec::new(),
    };

    let root = tree.root_node();
    let src = content.as_bytes();
    let mut symbols = Vec::new();

    walk(&root, src, &mut symbols);

    symbols
}

/// Recursively visit every node, handling `call` nodes as we go.
fn walk(node: &Node, src: &[u8], symbols: &mut Vec<String>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "call" {
            handle_call(&child, src, symbols);
        }
        walk(&child, src, symbols);
    }
}

/// Inspect a `call` node; if it's a defmodule/def/defmacro/defprotocol (and
/// not the private defp/defmacrop variants), or an `@callback` module
/// attribute declaring a behaviour's function contract, record the defined
/// name.
fn handle_call(call: &Node, src: &[u8], symbols: &mut Vec<String>) {
    let Some(target) = call.child_by_field_name("target") else {
        return;
    };
    let macro_name = target.utf8_text(src).unwrap_or_default();

    let exported = match macro_name {
        "defmodule" | "def" | "defmacro" | "defprotocol" => true,
        "defp" | "defmacrop" => false,
        "callback" if is_attribute_call(call) => true,
        _ => return,
    };

    if !exported {
        return;
    }

    if let Some(name) = extract_defined_name(call, src)
        && !symbols.contains(&name)
    {
        symbols.push(name);
    }
}

/// True if `call` is the operand of a unary `@` module-attribute expression
/// (e.g. the `callback(...)` call inside `@callback validate(...) :: ...`),
/// as opposed to an ordinary function call that happens to be named
/// `callback`.
fn is_attribute_call(call: &Node) -> bool {
    call.parent()
        .filter(|p| p.kind() == "unary_operator")
        .and_then(|p| p.child(0))
        .is_some_and(|op| op.kind() == "@")
}

/// Find the `arguments` child of a call node. This isn't exposed as a
/// tree-sitter field on this grammar -- it's just a plain named child of
/// kind "arguments" wrapping the actual argument nodes.
fn find_arguments<'tree>(call: &Node<'tree>) -> Option<Node<'tree>> {
    let mut cursor = call.walk();
    call.named_children(&mut cursor)
        .find(|c| c.kind() == "arguments")
}

/// Given a `defmodule`/`def`/`defmacro` call node, dig into its arguments to
/// find the defined name.
fn extract_defined_name(call: &Node, src: &[u8]) -> Option<String> {
    let arguments = find_arguments(call)?;
    let first = arguments.named_child(0)?;
    extract_name_from_signature(&first, src)
}

/// Given the first argument node of a def-like call, resolve the actual
/// defined name:
/// - `alias` (defmodule's `MyModule` / `Foo.Bar.Baz`) -> text directly
/// - `dot` (defmodule's `__MODULE__.Sub` -- a dotted path where a segment is
///   an identifier like `__MODULE__` rather than a literal Alias token, so
///   the grammar can't fold it into a single `alias` node) -> text directly
/// - `call` (`name(args)`) -> its target identifier
/// - `identifier` (bare `name`, no parens, e.g. `def active?`) -> text directly
/// - `binary_operator` (`name(args) when guard`) -> recurse into the `left` side
fn extract_name_from_signature(node: &Node, src: &[u8]) -> Option<String> {
    match node.kind() {
        "alias" | "identifier" | "dot" => Some(node.utf8_text(src).unwrap_or_default().to_string()),
        "call" => node
            .child_by_field_name("target")
            .map(|n| n.utf8_text(src).unwrap_or_default().to_string()),
        "binary_operator" => {
            let left = node.child_by_field_name("left")?;
            extract_name_from_signature(&left, src)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elixir_exports() {
        let src = r#"
defmodule MyModule do
  def public_func(a) do
    a
  end
  defp private_func do
    :ok
  end
  defmacro assert_something(expr) do
    expr
  end
  def active? do
    true
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"MyModule".to_string()));
        assert!(symbols.contains(&"public_func".to_string()));
        assert!(symbols.contains(&"assert_something".to_string()));
        assert!(symbols.contains(&"active?".to_string()));
        assert!(!symbols.contains(&"private_func".to_string()));
    }

    #[test]
    fn test_dotted_module_alias() {
        let src = r#"
defmodule Foo.Bar.Baz do
  def hello do
    :ok
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Foo.Bar.Baz".to_string()));
        assert!(symbols.contains(&"hello".to_string()));
    }

    #[test]
    fn test_guard_clause_and_keyword_form() {
        let src = r#"
defmodule Guarded do
  def guarded(x) when x > 0 do
    x
  end

  def no_do_block(x), do: x

  defp priv_no_do(x), do: x

  def zero_arity, do: :ok
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"guarded".to_string()));
        assert!(symbols.contains(&"no_do_block".to_string()));
        assert!(symbols.contains(&"zero_arity".to_string()));
        assert!(!symbols.contains(&"priv_no_do".to_string()));
    }

    #[test]
    fn test_ignores_def_in_string() {
        // AST inherently doesn't parse string contents as code, unlike the
        // regex reference which strips only `#` comments before matching.
        let src = r#"
defmodule Real do
  def real_func do
    "def fake_func do end"
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Real".to_string()));
        assert!(symbols.contains(&"real_func".to_string()));
        assert!(!symbols.contains(&"fake_func".to_string()));
    }

    #[test]
    fn test_module_dunder_concat() {
        // `defmodule __MODULE__.Sub do` is a common idiom for defining a
        // submodule relative to the enclosing module. Because `__MODULE__`
        // is an identifier (not a literal Alias token), the grammar
        // represents the whole thing as a `dot` node rather than a single
        // `alias` node, unlike plain `Foo.Bar.Baz`. The regex reference
        // captures the full dotted text (`[\w.!?]+` includes `.`), so the
        // AST extractor must too.
        let src = r#"
defmodule __MODULE__.Sub do
  def hi do
    :ok
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"hi".to_string()));
        assert!(
            symbols.contains(&"__MODULE__.Sub".to_string()),
            "got {symbols:?}"
        );
    }

    #[test]
    fn test_ignores_def_in_comment() {
        let src = r#"
defmodule Real do
  # def commented_out do end
  def real_func do
    :ok
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"real_func".to_string()));
        assert!(!symbols.contains(&"commented_out".to_string()));
    }

    #[test]
    fn test_behaviour_callbacks() {
        let src = r#"
defmodule MyApp.Authenticator do
  @moduledoc "Behaviour for pluggable auth strategies."
  @callback validate(credentials :: map()) :: {:ok, term()} | {:error, atom()}
  @callback refresh_token(token :: String.t()) :: {:ok, String.t()} | :error
  @optional_callbacks refresh_token: 1

  defmacro __using__(_opts) do
    quote do
      @behaviour MyApp.Authenticator
    end
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"MyApp.Authenticator".to_string()));
        assert!(symbols.contains(&"__using__".to_string()));
        assert!(symbols.contains(&"validate".to_string()), "got {symbols:?}");
        assert!(
            symbols.contains(&"refresh_token".to_string()),
            "got {symbols:?}"
        );
    }

    #[test]
    fn test_defprotocol_is_exported() {
        let src = r#"
defprotocol Formattable do
  @moduledoc "Protocol for types that can render themselves as strings."
  @doc "Formats the value for display."
  @spec format(t) :: String.t()
  def format(value)

  @doc "Returns a short label."
  def label(value)
end

defimpl Formattable, for: Integer do
  def format(value), do: Integer.to_string(value)
  def label(_value), do: "integer"
end
"#;
        let symbols = extract_exports(src);
        assert!(
            symbols.contains(&"Formattable".to_string()),
            "got {symbols:?}"
        );
        assert!(symbols.contains(&"format".to_string()));
        assert!(symbols.contains(&"label".to_string()));
    }

    #[test]
    fn test_plain_callback_call_not_treated_as_attribute() {
        // A plain function invocation named `callback` (no leading `@`) is an
        // ordinary call, not a behaviour contract -- it must not be mistaken
        // for `@callback` and must not leak its argument as an export.
        let src = r#"
defmodule Runner do
  def run(fun) do
    callback(fun)
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Runner".to_string()));
        assert!(symbols.contains(&"run".to_string()));
        assert!(!symbols.contains(&"fun".to_string()));
        assert!(!symbols.contains(&"callback".to_string()));
    }
}

#[cfg(test)]
mod throwaway_real_world {
    use super::*;

    #[test]
    fn test_real_learnxinyminutes_elixir_ast() {
        let src = include_str!(
            "/private/tmp/claude-501/-Users-leif-Development--CorvidLabs-spec-sync/1429498c-236f-41e9-839d-cd71a8ca63b8/scratchpad/elixir_extracted.ex"
        );
        let symbols = extract_exports(src);
        eprintln!("AST SYMBOLS ({}): {:#?}", symbols.len(), symbols);
    }
}
