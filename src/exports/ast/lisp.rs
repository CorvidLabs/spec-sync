use tree_sitter::{Node, Parser, Tree};

/// Head symbols the shared regex spec recognizes as "exported": `defun`,
/// `defmacro`, `defvar`, `defparameter`. Deliberately NOT Scheme's `define`
/// — this keeps parity with the regex extractor these AST walkers replace.
fn is_target_keyword(text: &str) -> bool {
    matches!(text, "defun" | "defmacro" | "defvar" | "defparameter")
}

fn parse_commonlisp(content: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let language = tree_sitter_commonlisp::LANGUAGE_COMMONLISP.into();
    parser.set_language(&language).ok()?;
    parser.parse(content, None)
}

fn parse_scheme(content: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let language = tree_sitter_scheme::LANGUAGE.into();
    parser.set_language(&language).ok()?;
    parser.parse(content, None)
}

fn parse_elisp(content: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let language = tree_sitter_elisp::LANGUAGE.into();
    parser.set_language(&language).ok()?;
    parser.parse(content, None)
}

/// Extract `defun`/`defmacro`/`defvar`/`defparameter` names from Lisp-family
/// source, using the tree-sitter grammar that matches `ext`:
/// - `"el"` -> Emacs Lisp
/// - `"scm"` -> Scheme
/// - `"lisp"` | `"lsp"` | anything else -> Common Lisp
///
/// Mirrors the original regex extractor's semantics: it scans the whole
/// tree (any nesting depth, not just top-level forms) and recognizes
/// exactly those four head symbols across all three dialects.
pub fn extract_exports(content: &str, ext: &str) -> Vec<String> {
    let tree = match ext {
        "el" => parse_elisp(content),
        "scm" => parse_scheme(content),
        _ => parse_commonlisp(content),
    };
    let tree = match tree {
        Some(t) => t,
        None => return Vec::new(),
    };

    let src = content.as_bytes();
    let mut symbols = Vec::new();

    match ext {
        "el" => walk_elisp(tree.root_node(), src, &mut symbols),
        "scm" => walk_symbol_list(tree.root_node(), src, &mut symbols),
        _ => walk_commonlisp(tree.root_node(), src, &mut symbols),
    }

    symbols
}

fn push_unique(symbols: &mut Vec<String>, name: String) {
    if !symbols.contains(&name) {
        symbols.push(name);
    }
}

// ---- Common Lisp (tree-sitter-commonlisp) ----
//
// `defun`/`defmacro` (and other defining forms like `defgeneric`) get a
// dedicated `defun` node with a `defun_header` field child exposing
// `keyword`/`function_name` fields. `defvar`/`defparameter` have no special
// node — they parse as a plain `list_lit` with `value`-field children.
fn walk_commonlisp(node: Node, src: &[u8], symbols: &mut Vec<String>) {
    if node.kind() == "quoting_lit" || node.kind() == "syn_quoting_lit" {
        // Quoted (`'(...)`) and quasiquoted (`` `(...) ``) forms are
        // data/templates, not executable code: a `(defun fake ...)` written
        // inside one never actually defines anything at runtime, so don't
        // recurse into it hunting for definitions — doing so previously
        // produced a false-positive export from otherwise-dead/template
        // code (verified via `parse_commonlisp(..).root_node().to_sexp()`:
        // the quoted list is a real nested `defun` node in the grammar,
        // indistinguishable from a live one without this check).
        return;
    }
    if node.kind() == "defun" {
        if let Some(header) = find_child_kind(node, "defun_header") {
            let keyword_text = header
                .child_by_field_name("keyword")
                .and_then(|k| k.utf8_text(src).ok())
                .unwrap_or("");
            if is_target_keyword(keyword_text) {
                // `function_name` can also be a `list_lit` for SETF-function
                // forms like `(defun (setf foo) ...)`. The regex reference
                // only ever captures a bare `[\w\-*!?<>]+` symbol there, so
                // it never matches SETF-function definitions at all — only
                // accept a plain `sym_lit` name to stay in parity.
                if let Some(name_node) = header.child_by_field_name("function_name")
                    && name_node.kind() == "sym_lit"
                    && let Ok(name) = name_node.utf8_text(src)
                {
                    push_unique(symbols, name.to_string());
                }
            }
        }
    } else if node.kind() == "list_lit" {
        capture_value_field_pair(node, src, symbols);
        capture_defstruct(node, src, symbols);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_commonlisp(child, src, symbols);
    }
}

/// Collect all `value`-field children of a `list_lit` node, in order.
fn value_fields<'tree>(node: Node<'tree>) -> Vec<Node<'tree>> {
    let mut values = Vec::new();
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            if cursor.field_name() == Some("value") {
                values.push(cursor.node());
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    values
}

/// `(defstruct name slot...)` / `(defstruct (name option...) slot...)` —
/// Common Lisp's struct idiom. The grammar never gives `defstruct` a
/// dedicated `defun` node (it's just a plain `list_lit`, see the
/// `walk_commonlisp` doc comment above), and none of its generated
/// constructor/predicate/accessor functions have a `defun`/`defvar` of
/// their own — the whole implicit public surface is synthesized purely as
/// a side effect of this one form. Same class of gap as Swift protocol
/// members that inherit the protocol's visibility without repeating it.
fn capture_defstruct(node: Node, src: &[u8], symbols: &mut Vec<String>) {
    let values = value_fields(node);
    let Some(head) = values.first() else {
        return;
    };
    if head.utf8_text(src) != Ok("defstruct") {
        return;
    }
    let Some(name_or_opts) = values.get(1) else {
        return;
    };

    let (struct_name, option_lists): (Option<&str>, Vec<Node>) = if name_or_opts.kind() == "sym_lit"
    {
        (name_or_opts.utf8_text(src).ok(), Vec::new())
    } else if name_or_opts.kind() == "list_lit" {
        let inner = value_fields(*name_or_opts);
        let name = inner.first().and_then(|n| n.utf8_text(src).ok());
        (name, inner.into_iter().skip(1).collect())
    } else {
        (None, Vec::new())
    };
    let Some(struct_name) = struct_name else {
        return;
    };
    push_unique(symbols, struct_name.to_string());

    // `:constructor` / `:predicate` / `:conc-name` options, e.g.
    // `(point (:constructor make-point (x y)) (:predicate point-instance-p))`.
    let mut constructor_names: Vec<String> = Vec::new();
    let mut constructor_seen = false;
    let mut predicate_name: Option<String> = None;
    let mut predicate_seen = false;
    let mut conc_name: Option<String> = None;
    let mut conc_name_seen = false;

    for opt in &option_lists {
        if opt.kind() != "list_lit" {
            continue;
        }
        let opt_values = value_fields(*opt);
        let Some(kwd) = opt_values.first() else {
            continue;
        };
        let Ok(kwd_text) = kwd.utf8_text(src) else {
            continue;
        };
        let arg_text = opt_values.get(1).and_then(|n| n.utf8_text(src).ok());
        match kwd_text {
            ":constructor" => {
                constructor_seen = true;
                if let Some(name) = arg_text
                    && name != "nil"
                {
                    constructor_names.push(name.to_string());
                }
            }
            ":predicate" => {
                predicate_seen = true;
                predicate_name = arg_text.filter(|n| *n != "nil").map(|n| n.to_string());
            }
            ":conc-name" => {
                conc_name_seen = true;
                conc_name = arg_text.filter(|n| *n != "nil").map(|n| n.to_string());
            }
            _ => {}
        }
    }

    if !constructor_seen {
        constructor_names.push(format!("make-{struct_name}"));
    }
    for name in constructor_names {
        push_unique(symbols, name);
    }

    let predicate = if predicate_seen {
        predicate_name
    } else {
        Some(format!("{struct_name}-p"))
    };
    if let Some(predicate) = predicate {
        push_unique(symbols, predicate);
    }

    let prefix = if conc_name_seen {
        conc_name.unwrap_or_default()
    } else {
        format!("{struct_name}-")
    };

    // Remaining `value` fields are an optional docstring followed by slot
    // specs (`slot-name` or `(slot-name default &key ...)`).
    for slot in values.iter().skip(2) {
        let slot_name = match slot.kind() {
            "sym_lit" => slot.utf8_text(src).ok(),
            "list_lit" => value_fields(*slot)
                .first()
                .and_then(|n| n.utf8_text(src).ok()),
            _ => None,
        };
        if let Some(slot_name) = slot_name {
            push_unique(symbols, format!("{prefix}{slot_name}"));
        }
    }
}

fn find_child_kind<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    node.children(&mut cursor).find(|c| c.kind() == kind)
}

/// `(defvar name ...)` / `(defparameter name ...)`: head + name are the
/// first two `value`-field children of a plain list node.
fn capture_value_field_pair(node: Node, src: &[u8], symbols: &mut Vec<String>) {
    let mut values = Vec::new();
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            if cursor.field_name() == Some("value") {
                values.push(cursor.node());
                if values.len() == 2 {
                    break;
                }
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    if values.len() == 2
        && let (Ok(head), Ok(name)) = (values[0].utf8_text(src), values[1].utf8_text(src))
        && is_target_keyword(head)
    {
        push_unique(symbols, name.to_string());
    }
}

// ---- Scheme (tree-sitter-scheme) ----
//
// Everything is a plain `list` node of `symbol` children with no field
// names: `(defun name ...)` -> symbol("defun"), symbol("name"), ...
fn walk_symbol_list(node: Node, src: &[u8], symbols: &mut Vec<String>) {
    if node.kind() == "quote" || node.kind() == "quasiquote" {
        // Same rationale as Common Lisp's `quoting_lit`/`syn_quoting_lit`
        // check above: quoted/quasiquoted data is never executed, so a
        // `(defun fake ...)` written inside `'(...)` or `` `(...) `` must
        // not be treated as a real definition.
        return;
    }
    if node.kind() == "list" {
        capture_symbol_pair(node, src, symbols);
        capture_define_record_type(node, src, symbols);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_symbol_list(child, src, symbols);
    }
}

/// `(define-record-type <type-name> (constructor field ...) predicate
///   (field accessor [modifier]) ...)` — R7RS Scheme's struct-like idiom.
/// None of the constructor/predicate/accessor/modifier names have their own
/// top-level `define`; they're synthesized entirely as a side effect of
/// this one form — the same class of gap as Common Lisp's `defstruct`. The
/// `<type-name>` binding itself is implementation-defined in R7RS (not
/// reliably a callable public symbol), so it's deliberately not extracted.
fn capture_define_record_type(node: Node, src: &[u8], symbols: &mut Vec<String>) {
    let mut cursor = node.walk();
    let children: Vec<Node> = node
        .children(&mut cursor)
        .filter(|c| matches!(c.kind(), "symbol" | "list"))
        .collect();
    let mut iter = children.into_iter();
    let Some(head) = iter.next() else {
        return;
    };
    if head.utf8_text(src) != Ok("define-record-type") {
        return;
    }
    let Some(_type_name) = iter.next() else {
        return;
    };

    if let Some(ctor) = iter.next() {
        let ctor_name_node = if ctor.kind() == "list" {
            let mut c = ctor.walk();
            ctor.children(&mut c).find(|n| n.kind() == "symbol")
        } else {
            Some(ctor)
        };
        if let Some(name_node) = ctor_name_node
            && let Ok(name) = name_node.utf8_text(src)
        {
            push_unique(symbols, name.to_string());
        }
    }

    if let Some(pred) = iter.next()
        && pred.kind() == "symbol"
        && let Ok(name) = pred.utf8_text(src)
    {
        push_unique(symbols, name.to_string());
    }

    for field_spec in iter {
        if field_spec.kind() != "list" {
            continue;
        }
        let mut c = field_spec.walk();
        let syms: Vec<Node> = field_spec
            .children(&mut c)
            .filter(|n| n.kind() == "symbol")
            .collect();
        if let Some(accessor) = syms.get(1)
            && let Ok(name) = accessor.utf8_text(src)
        {
            push_unique(symbols, name.to_string());
        }
        if let Some(modifier) = syms.get(2)
            && let Ok(name) = modifier.utf8_text(src)
        {
            push_unique(symbols, name.to_string());
        }
    }
}

/// First two `symbol`-kind children of a list node: head keyword + name.
/// Also used as the Emacs Lisp `list` fallback (same shape).
fn capture_symbol_pair(node: Node, src: &[u8], symbols: &mut Vec<String>) {
    let mut cursor = node.walk();
    let syms: Vec<Node> = node
        .children(&mut cursor)
        .filter(|c| c.kind() == "symbol")
        .take(2)
        .collect();
    if syms.len() == 2
        && let (Ok(head), Ok(name)) = (syms[0].utf8_text(src), syms[1].utf8_text(src))
        && is_target_keyword(head)
    {
        push_unique(symbols, name.to_string());
    }
}

// ---- Emacs Lisp (tree-sitter-elisp) ----
//
// `defun`/`defmacro` get dedicated `function_definition`/`macro_definition`
// nodes with a `name` field. `defvar` (but not `defparameter`, which isn't a
// recognized Elisp special form) gets a `special_form` node whose keyword
// child's *kind* equals the literal keyword text. Anything else
// defining-shaped (including a stray `defparameter`) falls back to a plain
// `list`, same shape as Scheme.
fn walk_elisp(node: Node, src: &[u8], symbols: &mut Vec<String>) {
    if node.kind() == "quote" {
        // The Elisp grammar uses the same `quote` node kind for both `'` and
        // `` ` `` (backquote/quasiquote) — see the Common Lisp/Scheme
        // walkers above for the same rationale: quoted data is never
        // executed, so a `(defun fake ...)` written inside one must not be
        // treated as a real definition.
        return;
    }
    match node.kind() {
        "function_definition" => {
            // The grammar's `function_definition` node covers BOTH `defun`
            // and `defsubst` (`choice("defun", "defsubst")`) with no field
            // to distinguish them — only the literal keyword child's kind
            // does. The regex reference only ever recognizes `defun`, so
            // `defsubst` forms must be rejected to stay in parity.
            if find_child_kind(node, "defun").is_some()
                && let Some(name_node) = node.child_by_field_name("name")
                && let Ok(name) = name_node.utf8_text(src)
            {
                push_unique(symbols, name.to_string());
            }
        }
        "macro_definition" => {
            if let Some(name_node) = node.child_by_field_name("name")
                && let Ok(name) = name_node.utf8_text(src)
            {
                push_unique(symbols, name.to_string());
            }
        }
        "special_form" => {
            let mut cursor = node.walk();
            let relevant: Vec<Node> = node
                .children(&mut cursor)
                .filter(|c| is_target_keyword(c.kind()) || c.kind() == "symbol")
                .take(2)
                .collect();
            if relevant.len() == 2
                && is_target_keyword(relevant[0].kind())
                && let Ok(name) = relevant[1].utf8_text(src)
            {
                push_unique(symbols, name.to_string());
            }
        }
        "list" => {
            capture_symbol_pair(node, src, symbols);
            capture_cl_defstruct(node, src, symbols);
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_elisp(child, src, symbols);
    }
}

/// `(cl-defstruct name slot...)` / `(cl-defstruct (name option...) slot...)`
/// — Emacs Lisp's `cl-lib` struct idiom, modeled directly on Common Lisp's
/// `defstruct` (same default constructor/predicate/conc-name conventions).
/// `cl-defstruct` isn't a grammar-recognized special form — it parses as a
/// plain `list`, same shape as Scheme — and its accessor/constructor/
/// predicate functions have no `defun` of their own, the same class of gap
/// as CL's `defstruct` (see `capture_defstruct` in the Common Lisp walker).
fn capture_cl_defstruct(node: Node, src: &[u8], symbols: &mut Vec<String>) {
    let mut cursor = node.walk();
    let children: Vec<Node> = node
        .children(&mut cursor)
        .filter(|c| matches!(c.kind(), "symbol" | "list" | "string"))
        .collect();
    let mut iter = children.into_iter();
    let Some(head) = iter.next() else {
        return;
    };
    if head.kind() != "symbol" || head.utf8_text(src) != Ok("cl-defstruct") {
        return;
    }
    let Some(name_or_opts) = iter.next() else {
        return;
    };

    let (struct_name, option_lists): (Option<String>, Vec<Node>) =
        if name_or_opts.kind() == "symbol" {
            (
                name_or_opts.utf8_text(src).ok().map(str::to_string),
                Vec::new(),
            )
        } else if name_or_opts.kind() == "list" {
            let mut c = name_or_opts.walk();
            let mut inner_iter = name_or_opts
                .children(&mut c)
                .filter(|c| matches!(c.kind(), "symbol" | "list"))
                .collect::<Vec<_>>()
                .into_iter();
            let name = inner_iter
                .next()
                .and_then(|n| n.utf8_text(src).ok())
                .map(str::to_string);
            (name, inner_iter.collect())
        } else {
            (None, Vec::new())
        };
    let Some(struct_name) = struct_name else {
        return;
    };
    push_unique(symbols, struct_name.clone());

    let mut constructor_names: Vec<String> = Vec::new();
    let mut constructor_seen = false;
    let mut predicate_name: Option<String> = None;
    let mut predicate_seen = false;
    let mut conc_name: Option<String> = None;
    let mut conc_name_seen = false;

    for opt in &option_lists {
        if opt.kind() != "list" {
            continue;
        }
        let mut c = opt.walk();
        let opt_values: Vec<Node> = opt
            .children(&mut c)
            .filter(|c| matches!(c.kind(), "symbol" | "list"))
            .collect();
        let Some(kwd) = opt_values.first() else {
            continue;
        };
        let Ok(kwd_text) = kwd.utf8_text(src) else {
            continue;
        };
        let arg_text = opt_values.get(1).and_then(|n| n.utf8_text(src).ok());
        match kwd_text {
            ":constructor" => {
                constructor_seen = true;
                if let Some(name) = arg_text
                    && name != "nil"
                {
                    constructor_names.push(name.to_string());
                }
            }
            ":predicate" => {
                predicate_seen = true;
                predicate_name = arg_text.filter(|n| *n != "nil").map(|n| n.to_string());
            }
            ":conc-name" => {
                conc_name_seen = true;
                conc_name = arg_text.filter(|n| *n != "nil").map(|n| n.to_string());
            }
            _ => {}
        }
    }

    if !constructor_seen {
        constructor_names.push(format!("make-{struct_name}"));
    }
    for name in constructor_names {
        push_unique(symbols, name);
    }

    let predicate = if predicate_seen {
        predicate_name
    } else {
        Some(format!("{struct_name}-p"))
    };
    if let Some(predicate) = predicate {
        push_unique(symbols, predicate);
    }

    let prefix = if conc_name_seen {
        conc_name.unwrap_or_default()
    } else {
        format!("{struct_name}-")
    };

    // Remaining items are an optional docstring followed by slot specs
    // (`slot-name` or `(slot-name default &key ...)`).
    for slot in iter {
        let slot_name = match slot.kind() {
            "symbol" => slot.utf8_text(src).ok(),
            "list" => {
                let mut c = slot.walk();
                slot.children(&mut c)
                    .find(|n| n.kind() == "symbol")
                    .and_then(|n| n.utf8_text(src).ok())
            }
            _ => None,
        };
        if let Some(slot_name) = slot_name {
            push_unique(symbols, format!("{prefix}{slot_name}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lisp_exports() {
        // Ported verbatim from the regex extractor's test — exercised
        // against the Common Lisp grammar via the "lisp" extension, same as
        // the original single-dialect test.
        let src = r#"
;; Lisp program
(defun add-numbers (a b)
  (+ a b))

(defparameter *global-var* 42)

(defmacro with-something (&body body)
  `(progn ,@body))
"#;
        let symbols = extract_exports(src, "lisp");
        assert!(symbols.contains(&"add-numbers".to_string()));
        assert!(symbols.contains(&"*global-var*".to_string()));
        assert!(symbols.contains(&"with-something".to_string()));
    }

    #[test]
    fn test_commonlisp_defvar() {
        let src = "(defvar *counter* 0)\n(defparameter *limit* 100)\n";
        let symbols = extract_exports(src, "lsp");
        assert_eq!(symbols, vec!["*counter*", "*limit*"]);
    }

    #[test]
    fn test_scheme_exports() {
        let src = r#"
;; Scheme program
(defun add-numbers (a b)
  (+ a b))

(defparameter *global-var* 42)

(defmacro with-something (body)
  body)
"#;
        let symbols = extract_exports(src, "scm");
        assert!(symbols.contains(&"add-numbers".to_string()));
        assert!(symbols.contains(&"*global-var*".to_string()));
        assert!(symbols.contains(&"with-something".to_string()));
    }

    #[test]
    fn test_scheme_ignores_comments_and_define() {
        // The regex spec never recognized Scheme's idiomatic `define` — the
        // AST walker must not add that coverage either, to stay in parity
        // with checked-in specs generated from the regex behavior.
        let src = r#"
;; (defun fake-in-comment (x) x)
(define regular-define 1)
(defun real-fn (x) x)
"#;
        let symbols = extract_exports(src, "scm");
        assert_eq!(symbols, vec!["real-fn"]);
    }

    #[test]
    fn test_elisp_exports() {
        let src = r#"
;; Elisp program
(defun add-numbers (a b)
  (+ a b))

(defvar global-var 42)

(defmacro with-something (body)
  body)
"#;
        let symbols = extract_exports(src, "el");
        assert!(symbols.contains(&"add-numbers".to_string()));
        assert!(symbols.contains(&"global-var".to_string()));
        assert!(symbols.contains(&"with-something".to_string()));
    }

    #[test]
    fn test_elisp_ignores_comments() {
        let src = r#"
;; (defun fake-in-comment (x) x)
(defun real-fn (x) x)
"#;
        let symbols = extract_exports(src, "el");
        assert_eq!(symbols, vec!["real-fn"]);
    }

    #[test]
    fn test_elisp_defsubst_not_exported() {
        // `defsubst` shares the grammar's `function_definition` node with
        // `defun` — the regex reference never matches `defsubst`, so
        // neither should the AST walker.
        let src = "(defsubst helper (x) x)\n(defun real-fn (x) x)\n";
        let symbols = extract_exports(src, "el");
        assert_eq!(symbols, vec!["real-fn"]);
    }

    #[test]
    fn test_commonlisp_setf_function_name_not_exported() {
        // `(defun (setf foo) ...)` has a list, not a symbol, as its
        // `function_name` — the regex reference requires a bare
        // `[\w\-*!?<>]+` token after `defun`, so it never matches this
        // form either.
        let src = "(defun (setf foo) (val obj) val)\n(defun real-fn (x) x)\n";
        let symbols = extract_exports(src, "lisp");
        assert_eq!(symbols, vec!["real-fn"]);
    }

    #[test]
    fn test_malformed_and_empty_input_no_panic() {
        assert_eq!(extract_exports("", "lisp"), Vec::<String>::new());
        assert_eq!(extract_exports("", "el"), Vec::<String>::new());
        assert_eq!(extract_exports("", "scm"), Vec::<String>::new());
        // Unbalanced / truncated forms.
        let _ = extract_exports("(defun broken (a b", "lisp");
        let _ = extract_exports("(defun", "lisp");
        let _ = extract_exports("(defvar", "lisp");
        let _ = extract_exports("(defun broken (a b", "el");
        let _ = extract_exports("(defvar", "el");
        let _ = extract_exports("(defun broken (a b", "scm");
        let _ = extract_exports(")))))(((((", "lisp");
        let _ = extract_exports("not lisp at all just text", "lisp");
    }

    #[test]
    fn test_commonlisp_defmethod_defgeneric_excluded() {
        // defmethod/defgeneric share the CL `defun` node kind but must not
        // be treated as targets (regex reference never matches them).
        let src =
            "(defgeneric area (shape))\n(defmethod area ((s circle)) 1)\n(defun real-fn () 1)\n";
        let symbols = extract_exports(src, "lisp");
        assert_eq!(symbols, vec!["real-fn"]);
    }

    #[test]
    fn test_commonlisp_defstruct_implicit_members() {
        // `defstruct` never gets its own `defun` node from the grammar, and
        // its constructor/predicate/accessor functions are synthesized as a
        // side effect of the form with no `defun` of their own — the true
        // CL analog of the Swift protocol-member "inherited visibility" gap.
        let src = r#"
(defstruct (point (:constructor make-point (x y)))
  "A 2D point."
  (x 0.0 :type single-float)
  (y 0.0 :type single-float))

(defun distance (p1 p2)
  (sqrt (+ (expt (- (point-x p1) (point-x p2)) 2)
           (expt (- (point-y p1) (point-y p2)) 2))))
"#;
        let symbols = extract_exports(src, "lisp");
        assert!(symbols.contains(&"point".to_string()));
        assert!(symbols.contains(&"make-point".to_string()));
        assert!(symbols.contains(&"point-p".to_string()));
        assert!(symbols.contains(&"point-x".to_string()));
        assert!(symbols.contains(&"point-y".to_string()));
        assert!(symbols.contains(&"distance".to_string()));
    }

    #[test]
    fn test_commonlisp_defstruct_bare_form_defaults() {
        // `(defstruct name slot...)` with no options list: struct name is a
        // bare symbol, and every accessor/constructor/predicate name must
        // fall back to CL's default naming conventions.
        let src = "(defstruct point x y)\n";
        let symbols = extract_exports(src, "lisp");
        assert_eq!(
            symbols,
            vec!["point", "make-point", "point-p", "point-x", "point-y"]
        );
    }

    #[test]
    fn test_scheme_define_record_type_implicit_members() {
        // R7RS `define-record-type` is Scheme's struct-like idiom: the
        // constructor, predicate, and accessor/modifier names are all
        // synthesized as a side effect of the one form and never get their
        // own top-level `define` — same gap as CL's `defstruct`.
        let src = r#"
;; point.scm
(define-record-type point
  (make-point x y)
  point?
  (x point-x set-point-x!)
  (y point-y set-point-y!))

(defun distance (p1 p2) (+ (point-x p1) (point-y p2)))
"#;
        let symbols = extract_exports(src, "scm");
        assert!(symbols.contains(&"make-point".to_string()));
        assert!(symbols.contains(&"point?".to_string()));
        assert!(symbols.contains(&"point-x".to_string()));
        assert!(symbols.contains(&"point-y".to_string()));
        assert!(symbols.contains(&"set-point-x!".to_string()));
        assert!(symbols.contains(&"set-point-y!".to_string()));
        assert!(symbols.contains(&"distance".to_string()));
    }

    #[test]
    fn test_elisp_cl_defstruct_implicit_members() {
        // `cl-lib`'s `cl-defstruct` is extremely common in real MELPA
        // packages and is modeled directly on CL's `defstruct` — same
        // implicit-member gap, different dialect/grammar. `cl-defgeneric`/
        // `cl-defmethod` are deliberately NOT extracted here: they're the
        // Elisp cl-lib equivalent of CL's `defgeneric`/`defmethod`, which
        // are out of this extractor's documented scope (see
        // `test_commonlisp_defmethod_defgeneric_excluded`).
        let src = r#"
(cl-defstruct my-package-item name value)

(cl-defgeneric my-package-render (item stream)
  "Render ITEM to STREAM.")

(cl-defmethod my-package-render ((item my-package-item) stream)
  (princ (my-package-item-name item) stream))

(defun my-package-entry-point (item)
  (my-package-render item t))
"#;
        let symbols = extract_exports(src, "el");
        assert!(symbols.contains(&"my-package-item".to_string()));
        assert!(symbols.contains(&"make-my-package-item".to_string()));
        assert!(symbols.contains(&"my-package-item-p".to_string()));
        assert!(symbols.contains(&"my-package-item-name".to_string()));
        assert!(symbols.contains(&"my-package-item-value".to_string()));
        assert!(symbols.contains(&"my-package-entry-point".to_string()));
        assert!(!symbols.contains(&"my-package-render".to_string()));
    }

    #[test]
    fn test_nested_defun_inside_let_captured_all_dialects() {
        // Deliberate parity with the reference regex, per `extract_exports`'s
        // doc comment: it scans the whole tree at any nesting depth, so a
        // `defun` nested inside a `let`/`progn` body (not just top-level
        // forms) must still be captured, in all three dialects.
        let src = "(let ((x 1)) (defun nested-fn (a) a))\n(defun top-fn (b) b)\n";
        for ext in ["lisp", "scm", "el"] {
            let symbols = extract_exports(src, ext);
            assert!(
                symbols.contains(&"nested-fn".to_string()),
                "ext={ext}: {symbols:?}"
            );
            assert!(
                symbols.contains(&"top-fn".to_string()),
                "ext={ext}: {symbols:?}"
            );
        }
    }

    #[test]
    fn test_commonlisp_quoted_and_quasiquoted_defun_not_exported() {
        // Regression test: before the `quoting_lit`/`syn_quoting_lit` guard
        // was added to `walk_commonlisp`, a `(defun fake-fn ...)` written
        // inside a quoted (`'(...)`) or quasiquoted (`` `(...) ``) form was
        // captured as a real export — confirmed via
        // `parse_commonlisp(..).root_node().to_sexp()`, which shows the
        // grammar still emits a real nested `defun` node inside the
        // `quoting_lit`/`syn_quoting_lit` wrapper. Quoted code is data, not
        // an executable definition, and must not be captured; the macro's
        // own name (`make-it`) must still be captured since that's read from
        // the macro's own header, not from recursing into its templated
        // body.
        let quoted = "(defun real-fn (x) x)\n(setf tmpl '(defun fake-fn (x) x))\n";
        let symbols = extract_exports(quoted, "lisp");
        assert_eq!(symbols, vec!["real-fn"]);
        assert!(!symbols.contains(&"fake-fn".to_string()));

        let quasiquoted = "(defmacro make-it () `(defun fake-fn2 (x) x))\n(defun real-fn2 (x) x)\n";
        let symbols = extract_exports(quasiquoted, "lisp");
        assert_eq!(symbols, vec!["make-it", "real-fn2"]);
        assert!(!symbols.contains(&"fake-fn2".to_string()));
    }

    #[test]
    fn test_scheme_quoted_and_quasiquoted_defun_not_exported() {
        // Same bug/fix as `test_commonlisp_quoted_and_quasiquoted_defun_not_exported`,
        // Scheme dialect: the grammar wraps quoted/quasiquoted lists in
        // dedicated `quote`/`quasiquote` node kinds (confirmed via
        // `parse_scheme(..).root_node().to_sexp()`), which `walk_symbol_list`
        // must skip recursing into.
        let quoted = "(define x '(defun fake-fn (a) a))\n(defun real-fn (b) b)\n";
        let symbols = extract_exports(quoted, "scm");
        assert_eq!(symbols, vec!["real-fn"]);

        let quasiquoted = "(define x `(defun fake-fn (a) a))\n(defun real-fn (b) b)\n";
        let symbols = extract_exports(quasiquoted, "scm");
        assert_eq!(symbols, vec!["real-fn"]);
    }

    #[test]
    fn test_elisp_quoted_and_quasiquoted_defun_not_exported() {
        // Same bug/fix as `test_commonlisp_quoted_and_quasiquoted_defun_not_exported`,
        // Elisp dialect: the grammar uses one `quote` node kind for both `'`
        // and `` ` `` (confirmed via
        // `parse_elisp(..).root_node().to_sexp()`), which `walk_elisp` must
        // skip recursing into.
        let quoted = "(setq x '(defun fake-fn (a) a))\n(defun real-fn (b) b)\n";
        let symbols = extract_exports(quoted, "el");
        assert_eq!(symbols, vec!["real-fn"]);

        let quasiquoted = "(setq x `(defun fake-fn (a) a))\n(defun real-fn (b) b)\n";
        let symbols = extract_exports(quasiquoted, "el");
        assert_eq!(symbols, vec!["real-fn"]);
    }

    #[test]
    fn test_elisp_defparameter_falls_back_to_plain_list_capture() {
        // `defparameter` isn't a recognized Elisp special form (only
        // `defvar` gets a dedicated `special_form` node), so it parses as a
        // plain `list`, same shape as Scheme. The regex reference matches
        // `defparameter` unconditionally regardless of dialect, so the
        // Elisp AST walker must still capture it via the `capture_symbol_pair`
        // plain-`list` fallback to stay in parity.
        let src = "(defparameter my-var 42)\n(defun real-fn (x) x)\n";
        let symbols = extract_exports(src, "el");
        assert!(symbols.contains(&"my-var".to_string()), "{symbols:?}");
        assert!(symbols.contains(&"real-fn".to_string()), "{symbols:?}");
    }

    #[test]
    fn test_elisp_defvar_local_excluded() {
        // `defvar-local` is a real, common Elisp idiom for buffer-local
        // variables, but the regex reference's `defvar\s+` requires
        // whitespace immediately after the literal `defvar` token, so
        // `defvar-local` (which has `-local` immediately after `defvar`,
        // not whitespace) never matches it. Verified the Elisp grammar
        // doesn't give `defvar-local` a `special_form` node with a `defvar`-
        // kind keyword child either, so `is_target_keyword` correctly never
        // sees a match — this documents intentional parity, not a bug.
        let src = "(defvar-local my-local-var nil)\n(defun real-fn (x) x)\n";
        let symbols = extract_exports(src, "el");
        assert_eq!(symbols, vec!["real-fn"]);
        assert!(!symbols.contains(&"my-local-var".to_string()));
    }

    #[test]
    fn test_commonlisp_multiline_lambda_list_with_optional_rest_key() {
        // Real-world multi-line `defun` signatures using `&optional`,
        // `&rest`, and `&key` markers (common in idiomatic Common Lisp) must
        // not confuse the `defun_header`/`function_name` field lookup, which
        // only cares about the header shape, not how the lambda list is
        // formatted or how many lines it spans.
        let src = "(defun complex-fn (a b\n                    &optional (c 1) (d 2)\n                    &rest more\n                    &key (e 3) (f 4))\n  (list a b c d e f more))\n";
        let symbols = extract_exports(src, "lisp");
        assert_eq!(symbols, vec!["complex-fn"]);
    }

    #[test]
    fn test_ignores_defun_text_in_string_literal_elisp() {
        // Extends `test_ignores_defun_text_in_string_literal` (Common Lisp)
        // coverage to the Elisp dialect: text that merely *looks* like a
        // definition inside a string literal must not be captured there
        // either.
        let src = "(defun real-fn (x) x)\n(defvar my-doc \"(defun fake-fn (x) x)\")\n";
        let symbols = extract_exports(src, "el");
        assert!(symbols.contains(&"real-fn".to_string()));
        assert!(symbols.contains(&"my-doc".to_string()));
        assert!(!symbols.contains(&"fake-fn".to_string()));
    }

    #[test]
    fn test_ignores_defun_text_in_string_literal() {
        // AST-native win over the regex: text that merely *looks* like a
        // definition inside a string literal must not be captured.
        let src = r#"
(defun real-fn (x) x)
(defparameter *doc* "(defun fake-fn (x) x)")
"#;
        let symbols = extract_exports(src, "lisp");
        assert_eq!(symbols, vec!["real-fn", "*doc*"]);
        assert!(!symbols.contains(&"fake-fn".to_string()));
    }
}
