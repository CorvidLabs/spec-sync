use regex::Regex;
use std::sync::LazyLock;

static COMMENT_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"#.*$").unwrap());

/// Elixir public declarations: defmodule, def, defmacro, defprotocol, @callback
static ELIXIR_DECL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[^\S\n]*(?:defmodule|def|defmacro|defprotocol|@callback)\s+([\w.!?]+)")
        .unwrap()
});

/// Exclude private declarations
static ELIXIR_PRIVATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[^\S\n]*(?:defp|defmacrop)\b").unwrap());

/// Extract public symbols from Elixir source code.
pub fn extract_exports(content: &str) -> Vec<String> {
    let stripped = COMMENT_SINGLE.replace_all(content, "");

    let mut symbols = Vec::new();

    for line in stripped.lines() {
        if ELIXIR_PRIVATE.is_match(line) {
            continue;
        }
        if let Some(caps) = ELIXIR_DECL.captures(line) {
            if let Some(name) = caps.get(1) {
                let n = name.as_str().to_string();
                if !symbols.contains(&n) {
                    symbols.push(n);
                }
            }
        }
    }

    symbols
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

    /// Real excerpt from learnxinyminutes.com's Elixir tutorial (the
    /// `PrivateMath`/`Geometry` sections): exercises a private helper
    /// (`defp`) that must be excluded, and a multi-clause guarded public
    /// function (`def area/1` defined twice, once with a `when` guard).
    #[test]
    fn test_real_learnxinyminutes_private_and_guarded_clauses() {
        let src = r#"
defmodule PrivateMath do
  def sum(a, b) do
    do_sum(a, b)
  end

  defp do_sum(a, b) do
    a + b
  end
end

defmodule Geometry do
  def area({:rectangle, w, h}) do
    w * h
  end

  def area({:circle, r}) when is_number(r) do
    3.14 * r * r
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"PrivateMath".to_string()));
        assert!(symbols.contains(&"sum".to_string()));
        assert!(!symbols.contains(&"do_sum".to_string()));
        assert!(symbols.contains(&"Geometry".to_string()));
        assert!(symbols.contains(&"area".to_string()));
    }

    /// Real excerpt from learnxinyminutes.com's Elixir tutorial (the
    /// `receive`-loop example in the Concurrency section): a zero-arity
    /// `def` whose body contains a `receive do ... end` block with string
    /// interpolation (`"Area = #{w * h}"`) and a recursive self-call. The
    /// `#{` inside the string must not be mistaken for the start of a `#`
    /// comment in a way that corrupts extraction.
    #[test]
    fn test_real_learnxinyminutes_receive_loop_with_interpolation() {
        let src = r#"
defmodule Geometry do
  def area_loop do
    receive do
      {:rectangle, w, h} ->
        IO.puts("Area = #{w * h}")
        area_loop()
      {:circle, r} ->
        IO.puts("Area = #{3.14 * r * r}")
        area_loop()
    end
  end
end
"#;
        let symbols = extract_exports(src);
        assert!(symbols.contains(&"Geometry".to_string()));
        assert!(symbols.contains(&"area_loop".to_string()));
        assert_eq!(
            symbols.iter().filter(|s| s.as_str() == "area_loop").count(),
            1
        );
    }
}
