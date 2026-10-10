//! Gradle executable-directive and control-flow guards.

use super::syntax::{find_unquoted_gradle_fragment, gradle_parenthesized, gradle_string_arguments};
use super::{is_gradle_simple_assignment, normalize_gradle_project_relative_path};

pub(super) fn reject_non_leading_gradle_includes(content: &str) -> Result<(), String> {
    let mut search_start = 0usize;
    while let Some(index) = find_unquoted_gradle_fragment(content, "include", search_start) {
        let before_is_boundary = content[..index]
            .chars()
            .next_back()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_');
        if !before_is_boundary {
            search_start = index + "include".len();
            continue;
        }
        let token_end = content[index..]
            .char_indices()
            .take_while(|(_, character)| character.is_alphanumeric() || *character == '_')
            .last()
            .map_or(index, |(offset, character)| {
                index + offset + character.len_utf8()
            });
        let token = &content[index..token_end];
        if token == "include" {
            let line_start = content[..index].rfind('\n').map_or(0, |line| line + 1);
            if !content[line_start..index].trim().is_empty() {
                return Err("Unsupported qualified or conditional Gradle include".to_string());
            }
            if gradle_brace_depth(&content[..index])? != 0 {
                return Err("Unsupported block-scoped Gradle include".to_string());
            }
            if gradle_follows_detached_control_header(content, index)? {
                return Err("Unsupported conditional Gradle include".to_string());
            }
        } else if token == "includeBuild"
            && gradle_include_prefixed_token_is_executable(content, index, token_end)
        {
            gradle_include_build_target(content, token_end)?;
        } else if token.starts_with("include")
            && gradle_include_prefixed_token_is_executable(content, index, token_end)
        {
            return Err(format!("Unsupported Gradle workspace mutator {token}"));
        }
        search_start = token_end.max(index + "include".len());
    }
    Ok(())
}

/// Validate a composite-build declaration by its ARGUMENT rather than its name.
///
/// `includeBuild` was refused on the token prefix alone, so
/// `includeBuild("vendor/podo-shared")` — an ordinary in-repo composite build,
/// the common and correct usage — failed identically to
/// `includeBuild("../outside")`, which escapes the repository. Every fixture the
/// guard was written against used `../outside`, so no test ever distinguished
/// the two, and a valid Gradle project could not be measured at all (#723).
///
/// A single complete string literal that normalizes to a path beneath the
/// project root is accepted and then ignored: it names a separate build inside
/// the tree, and the root build's own `include(...)` list — the only thing this
/// parser derives modules from — is unaffected by it. Everything else keeps
/// failing closed, in line with the rest of this parser's literal-only subset:
/// a `../` escape, an interpolated or otherwise dynamic expression, more than
/// one argument, and any trailing expression other than a balanced configuration
/// block all name something that cannot be resolved from the settings text
/// alone.
///
/// A trailing `{ dependencySubstitution … }` block is skipped rather than
/// refused (#725). That block is the NORMAL spelling — it is how a composite
/// build substitutes a local project for a published coordinate — so refusing it
/// refused the common form while accepting the bare minority one. It carries
/// substitution rules, not project declarations, so it contributes no module and
/// no source directory and nothing in it needs to be resolved.
///
/// Sibling mutators are deliberately untouched. `includeFlat` resolves against
/// the PARENT of the root, so its argument is outside the project by
/// construction, and `includeWorkspace` is not a form this parser models at
/// all; judging either by its argument would not make it supportable.
///
/// Position is not judged either, and that asymmetry with `include` is
/// deliberate. A conditional or block-scoped `include` is refused because it
/// makes the module set unknowable from the text. A composite build contributes
/// no module whether or not its branch runs, so where it sits cannot change what
/// is discovered — only its argument can.
fn gradle_include_build_target(content: &str, token_end: usize) -> Result<String, String> {
    let arguments = content[token_end..].trim_start();
    let values = if arguments.starts_with('(') {
        let (inside, remainder) = gradle_parenthesized(arguments)?;
        // The configuration block is skipped BEFORE the statement-end check and
        // never before the path check below, so it can only ever remove trailing
        // text from the verdict — never an argument from it.
        let remainder = skip_gradle_include_build_configuration_block(remainder)?;
        // A parenthesized call may span lines, so the balanced scan above runs
        // over the rest of the file. Only the tail of the CLOSING line belongs
        // to this statement; the lines after it are other directives.
        let statement_remainder = remainder
            .split_once('\n')
            .map_or(remainder, |(line, _)| line);
        require_gradle_include_build_statement_end(statement_remainder)?;
        gradle_string_arguments(inside)?
    } else {
        let statement = arguments
            .split_once('\n')
            .map_or(arguments, |(line, _)| line);
        gradle_string_arguments(gradle_include_build_statement_body(statement))?
    };
    if values.len() != 1 {
        return Err("Gradle includeBuild declaration must name one literal path".to_string());
    }
    normalize_gradle_project_relative_path(&values[0], "includeBuild path", false)
}

/// Characters that can follow a composite-build declaration without being part
/// of it: statement terminators, and the closing braces of an enclosing block.
fn gradle_include_build_statement_tail(character: char) -> bool {
    matches!(character, '}' | ';') || character.is_whitespace()
}

/// Everything on the line that belongs to the declaration itself.
fn gradle_include_build_statement_body(statement: &str) -> &str {
    statement.trim_end_matches(gradle_include_build_statement_tail)
}

/// Skip a balanced trailing configuration block on a composite-build declaration.
///
/// Returns the input untouched when no block opens on the declaration's own
/// line, so nothing about the bare form changes, and the closing `}` of an
/// ENCLOSING block is left for `require_gradle_include_build_statement_end` —
/// `if (x) { includeBuild("vendor/shared") }` opens no block of its own.
///
/// Skipping is safe because it removes nothing from any other guard. The path
/// argument is parsed from inside the parentheses, which this never touches, so
/// a block cannot smuggle `includeBuild("../outside") { … }` past the
/// confinement check. And the block's own text stays in `content`: the scan in
/// `reject_non_leading_gradle_includes` resumes from the declaration token, and
/// `reject_unsupported_gradle_project_dir_mutations` runs over the whole file
/// afterwards — so a block-scoped `include`, a `projectDir` mutation, or an
/// unrecognized `project(...)` mutation written inside the block still fails
/// closed exactly as it does anywhere else.
///
/// The scan is quote- and escape-aware, so a brace inside a string literal moves
/// the depth in neither direction. Braces inside comments are already gone:
/// `strip_gradle_comments` runs before any of this. An unbalanced block reaches
/// the end of the file without returning to depth zero and is refused.
fn skip_gradle_include_build_configuration_block(remainder: &str) -> Result<&str, String> {
    let opened = remainder.trim_start_matches([' ', '\t', '\r']);
    if !opened.starts_with('{') {
        return Ok(remainder);
    }

    let mut depth = 0i32;
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in opened.char_indices() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth <= 0 {
                    return Ok(&opened[index + character.len_utf8()..]);
                }
            }
            _ => {}
        }
    }

    Err("Gradle includeBuild configuration block has unbalanced braces".to_string())
}

/// Require that nothing but statement punctuation follows the declaration.
///
/// The check has to refuse a trailing EXPRESSION this parser cannot resolve —
/// anything but the configuration block skipped above — without refusing the
/// closing brace of an enclosing block, which belongs to the `if` around the
/// declaration rather than to the declaration.
///
/// Without that distinction the same conditional composite build was accepted
/// written across three lines and refused written on one, because only in the
/// second case did the `}` land on the declaration's own line. A verdict that
/// turns on where the author pressed Enter is a bug whichever way it is settled.
fn require_gradle_include_build_statement_end(remainder: &str) -> Result<(), String> {
    if remainder.chars().all(gradle_include_build_statement_tail) {
        return Ok(());
    }
    Err("Unsupported trailing Gradle includeBuild declaration expression".to_string())
}

fn gradle_include_prefixed_token_is_executable(
    content: &str,
    token_start: usize,
    token_end: usize,
) -> bool {
    let line_start = content[..token_start]
        .rfind('\n')
        .map_or(0, |line| line + 1);
    let leading = content[line_start..token_start].trim();
    let trailing = content[token_end..]
        .split_once('\n')
        .map_or(&content[token_end..], |(line, _)| line)
        .trim_start();
    if trailing.is_empty() {
        return false;
    }
    if trailing.starts_with("==")
        || trailing.starts_with("!=")
        || trailing.starts_with("<=")
        || trailing.starts_with(">=")
        || trailing.starts_with('.')
        || trailing.starts_with("?.")
    {
        return false;
    }
    leading.is_empty()
        || trailing.starts_with('(')
        || trailing.starts_with('\'')
        || trailing.starts_with('"')
        || trailing.starts_with('{')
        || trailing
            .chars()
            .next()
            .is_some_and(|character| character.is_alphanumeric() || character == '_')
}

pub(super) fn gradle_follows_detached_control_header(
    content: &str,
    directive_start: usize,
) -> Result<bool, String> {
    let preceding = content[..directive_start].trim_end();
    if preceding.is_empty() {
        return Ok(false);
    }
    for keyword in ["else", "do", "try", "finally"] {
        if preceding.ends_with(keyword) {
            let keyword_start = preceding.len() - keyword.len();
            let boundary = preceding[..keyword_start]
                .chars()
                .next_back()
                .is_none_or(|character| !character.is_alphanumeric() && character != '_');
            if boundary {
                return Ok(true);
            }
        }
    }
    if !preceding.ends_with(')') {
        return Ok(false);
    }

    let mut quote = None;
    let mut escaped = false;
    let mut open_parens = Vec::new();
    let mut final_open = None;
    for (index, character) in preceding.char_indices() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }
        if matches!(character, '\'' | '"') {
            quote = Some(character);
            continue;
        }
        if character == '(' {
            open_parens.push(index);
        } else if character == ')' {
            let Some(open) = open_parens.pop() else {
                return Err("Gradle settings contain unmatched parentheses".to_string());
            };
            if index + character.len_utf8() == preceding.len() {
                final_open = Some(open);
            }
        }
    }
    let Some(final_open) = final_open else {
        return Ok(false);
    };
    let before_open = preceding[..final_open].trim_end();
    let keyword_start = before_open
        .char_indices()
        .rev()
        .take_while(|(_, character)| character.is_alphanumeric() || *character == '_')
        .last()
        .map_or(before_open.len(), |(index, _)| index);
    let keyword = &before_open[keyword_start..];
    Ok(matches!(
        keyword,
        "if" | "when" | "for" | "while" | "switch" | "catch"
    ))
}

pub(super) fn gradle_brace_depth(content: &str) -> Result<i32, String> {
    let mut depth = 0i32;
    let mut quote = None;
    let mut escaped = false;
    for character in content.chars() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }
        if matches!(character, '\'' | '"') {
            quote = Some(character);
        } else if character == '{' {
            depth += 1;
        } else if character == '}' {
            depth -= 1;
            if depth < 0 {
                return Err("Gradle settings contain an unmatched closing brace".to_string());
            }
        }
    }
    Ok(depth)
}

pub(super) fn reject_unsupported_gradle_project_dir_mutations(content: &str) -> Result<(), String> {
    reject_unrecognized_gradle_project_mutations(content)?;

    let mut search_start = 0usize;
    while let Some(index) = find_unquoted_gradle_fragment(content, "projectDir", search_start) {
        let before_is_boundary = content[..index]
            .chars()
            .next_back()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_');
        let after_index = index + "projectDir".len();
        let after_is_boundary = content[after_index..]
            .chars()
            .next()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_');
        let directly_dotted = content[..index].ends_with('.');
        if before_is_boundary && after_is_boundary && !directly_dotted {
            let after = content[after_index..].trim_start();
            if is_gradle_simple_assignment(after)?.then_some(()).is_some() {
                return Err("Unsupported indirect Gradle projectDir mutation".to_string());
            }
        }
        search_start = after_index;
    }

    search_start = 0;
    while let Some(index) = find_unquoted_gradle_fragment(content, "setProjectDir", search_start) {
        let before_is_boundary = content[..index]
            .chars()
            .next_back()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_');
        let after_index = index + "setProjectDir".len();
        let after_is_boundary = content[after_index..]
            .chars()
            .next()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_');
        let directly_dotted = content[..index].ends_with('.');
        if before_is_boundary
            && after_is_boundary
            && !directly_dotted
            && content[after_index..].trim_start().starts_with('(')
        {
            return Err("Unsupported indirect Gradle setProjectDir mutation".to_string());
        }
        search_start = after_index;
    }
    Ok(())
}

fn reject_unrecognized_gradle_project_mutations(content: &str) -> Result<(), String> {
    let mut search_start = 0usize;
    while let Some(project_index) = find_unquoted_gradle_fragment(content, "project", search_start)
    {
        let before_is_boundary = content[..project_index]
            .chars()
            .next_back()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_');
        let after_keyword = &content[project_index + "project".len()..];
        let after_is_boundary = after_keyword
            .chars()
            .next()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_');
        let project_call = after_keyword.trim_start();
        if !before_is_boundary || !after_is_boundary || !project_call.starts_with('(') {
            search_start = project_index + "project".len();
            continue;
        }

        let (_, remainder) = gradle_parenthesized(project_call)?;
        let call_end = content.len() - remainder.len();
        let suffix = gradle_project_chain_suffix(remainder);
        if suffix.is_empty() {
            search_start = call_end;
            continue;
        }
        let statement = gradle_project_chain_statement(suffix).trim();

        if suffix.starts_with(".setProjectDir") {
            search_start = call_end;
            continue;
        }
        if let Some(after_project_dir) = suffix.strip_prefix(".projectDir")
            && is_gradle_simple_assignment(after_project_dir.trim_start())?
        {
            search_start = call_end;
            continue;
        }
        if suffix.starts_with(".setProperty") {
            return Err("Unsupported dynamic Gradle project mutation".to_string());
        }

        if !gradle_project_statement_is_read_only(statement) {
            return Err("Unsupported executable Gradle project mutation".to_string());
        }
        search_start = call_end;
    }
    Ok(())
}

fn gradle_project_chain_suffix(remainder: &str) -> &str {
    let horizontal = remainder.trim_start_matches([' ', '\t', '\r']);
    if horizontal.starts_with('.') || horizontal.starts_with('[') {
        return horizontal;
    }
    if horizontal.starts_with('\n') {
        let continued = horizontal.trim_start();
        if continued.starts_with('.') || continued.starts_with('[') {
            return continued;
        }
    }
    ""
}

fn gradle_project_chain_statement(suffix: &str) -> &str {
    let mut quote = None;
    let mut escaped = false;
    let mut parentheses = 0i32;
    let mut brackets = 0i32;
    let mut braces = 0i32;
    for (index, character) in suffix.char_indices() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }
        if matches!(character, '\'' | '"') {
            quote = Some(character);
            continue;
        }
        match character {
            '(' => parentheses += 1,
            ')' => parentheses -= 1,
            '[' => brackets += 1,
            ']' => brackets -= 1,
            '{' => braces += 1,
            '}' => braces -= 1,
            ';' if parentheses <= 0 && brackets <= 0 && braces <= 0 => {
                return &suffix[..index];
            }
            '\n' if parentheses <= 0 && brackets <= 0 && braces <= 0 => {
                let before = suffix[..index].trim_end();
                let after = suffix[index + character.len_utf8()..].trim_start();
                if !gradle_statement_continues(before, after) {
                    return &suffix[..index];
                }
            }
            _ => {}
        }
    }
    suffix
}

fn gradle_statement_continues(before: &str, after: &str) -> bool {
    before.chars().next_back().is_some_and(|character| {
        matches!(
            character,
            '.' | '?' | ':' | '=' | '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^'
        )
    }) || after.starts_with('.')
        || after.starts_with('[')
        || after.starts_with('=')
        || after.starts_with("+=")
        || after.starts_with("-=")
        || after.starts_with("*=")
        || after.starts_with("/=")
}

fn gradle_project_statement_is_read_only(statement: &str) -> bool {
    if statement.is_empty() || statement == ";" {
        return true;
    }
    if !statement.starts_with('.') && !statement.starts_with('[') {
        return false;
    }
    if gradle_contains_mutating_assignment(statement) || statement.contains('{') {
        return false;
    }

    let comparison = ["==", "!=", "<=", ">=", "=~", "!~"]
        .into_iter()
        .filter_map(|operator| statement.find(operator))
        .min();
    let invocation = find_unquoted_gradle_fragment(statement, "(", 0);
    match (comparison, invocation) {
        (Some(comparison), Some(invocation)) => comparison < invocation,
        (Some(_), None) | (None, None) => true,
        (None, Some(_)) => false,
    }
}

fn gradle_contains_mutating_assignment(statement: &str) -> bool {
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in statement.char_indices() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }
        if matches!(character, '\'' | '"') {
            quote = Some(character);
            continue;
        }
        if character != '=' {
            continue;
        }
        let before = statement[..index].chars().next_back();
        let after = statement[index + character.len_utf8()..].chars().next();
        if after.is_some_and(|next| matches!(next, '=' | '~'))
            || before.is_some_and(|previous| matches!(previous, '=' | '!' | '<' | '>'))
        {
            continue;
        }
        return true;
    }
    false
}
