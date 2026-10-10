//! Gradle comments, quoted literals, and balanced argument syntax.

pub(super) fn find_gradle_project_call(content: &str) -> Option<usize> {
    let mut search_start = 0usize;
    let mut found = None;
    while let Some(index) = find_unquoted_gradle_fragment(content, "project", search_start) {
        let before_is_boundary = content[..index]
            .chars()
            .next_back()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_');
        let after = content[index + "project".len()..].trim_start();
        let after_is_boundary = content[index + "project".len()..]
            .chars()
            .next()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_');
        if before_is_boundary && after_is_boundary && after.starts_with('(') {
            found = Some(index);
        }
        search_start = index + "project".len();
    }
    found
}

pub(super) fn gradle_parenthesized(value: &str) -> Result<(&str, &str), String> {
    let Some(value) = value.strip_prefix('(') else {
        return Err("Gradle declaration is missing an opening parenthesis".to_string());
    };
    let mut depth = 1usize;
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in value.char_indices() {
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
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Ok((&value[..index], &value[index + character.len_utf8()..]));
                }
            }
            _ => {}
        }
    }
    Err("Gradle declaration has unbalanced parentheses".to_string())
}

pub(super) fn parse_gradle_include_statement(statement: &str) -> Result<Vec<String>, String> {
    let Some(arguments) = statement.strip_prefix("include") else {
        return Err("Gradle include declaration is missing 'include'".to_string());
    };
    let arguments = arguments.trim_start();
    let values = if arguments.starts_with('(') {
        let (arguments, remainder) = gradle_parenthesized(arguments)?;
        require_gradle_complete_remainder(remainder, "include declaration")?;
        gradle_string_arguments(arguments)
    } else {
        let arguments = strip_gradle_statement_terminator(arguments, "include declaration")?;
        gradle_string_arguments(arguments)
    }?;
    if values.is_empty() {
        return Err("Gradle include declaration must contain a literal module".to_string());
    }
    Ok(values)
}

pub(super) fn parse_gradle_root_dir_file_arguments(arguments: &str) -> Result<String, String> {
    let Some(arguments) = arguments.trim_start().strip_prefix("rootDir") else {
        return Err("Gradle new File projectDir base must be rootDir".to_string());
    };
    let Some(arguments) = arguments.trim_start().strip_prefix(',') else {
        return Err("Gradle new File projectDir base must be exactly rootDir".to_string());
    };
    let path_values = gradle_string_arguments(arguments)?;
    if path_values.len() != 1 {
        return Err("Gradle new File projectDir assignment must contain one path".to_string());
    }
    Ok(path_values[0].clone())
}

pub(super) fn require_gradle_expression_end(remainder: &str) -> Result<(), String> {
    let statement_remainder = remainder
        .split_once('\n')
        .map_or(remainder, |(line, _)| line);
    require_gradle_complete_remainder(statement_remainder, "projectDir assignment")
}

fn require_gradle_complete_remainder(remainder: &str, context: &str) -> Result<(), String> {
    let remainder = remainder.trim();
    if remainder.is_empty() || remainder == ";" {
        Ok(())
    } else {
        Err(format!("Unsupported trailing Gradle {context} expression"))
    }
}

fn strip_gradle_statement_terminator<'a>(
    statement: &'a str,
    context: &str,
) -> Result<&'a str, String> {
    let statement = statement.trim_end();
    let Some(statement) = statement.strip_suffix(';') else {
        return Ok(statement);
    };
    if statement.trim_end().ends_with(';') {
        return Err(format!("Unsupported trailing Gradle {context} expression"));
    }
    Ok(statement.trim_end())
}

pub(super) fn strip_gradle_comments(content: &str) -> Result<String, String> {
    let mut cleaned = String::with_capacity(content.len());
    let characters = content.chars().collect::<Vec<_>>();
    let mut index = 0usize;

    while index < characters.len() {
        let character = characters[index];
        if matches!(character, '\'' | '"')
            && characters.get(index + 1) == Some(&character)
            && characters.get(index + 2) == Some(&character)
        {
            cleaned.extend([' ', ' ', ' ']);
            index += 3;
            let mut terminated = false;
            while index < characters.len() {
                if characters.get(index) == Some(&character)
                    && characters.get(index + 1) == Some(&character)
                    && characters.get(index + 2) == Some(&character)
                {
                    cleaned.extend([' ', ' ', ' ']);
                    index += 3;
                    terminated = true;
                    break;
                }
                cleaned.push(if characters[index] == '\n' { '\n' } else { ' ' });
                index += 1;
            }
            if !terminated {
                return Err("Gradle settings contain an unterminated multiline string".to_string());
            }
            continue;
        }
        if matches!(character, '\'' | '"') {
            let delimiter = character;
            let mut escaped = false;
            let mut terminated = false;
            cleaned.push(character);
            index += 1;
            while index < characters.len() {
                let quoted = characters[index];
                cleaned.push(quoted);
                index += 1;
                if escaped {
                    escaped = false;
                } else if quoted == '\\' {
                    escaped = true;
                } else if quoted == delimiter {
                    terminated = true;
                    break;
                }
            }
            if escaped {
                return Err("Gradle settings contain a dangling string escape".to_string());
            }
            if !terminated {
                return Err("Gradle settings contain an unterminated quoted string".to_string());
            }
            continue;
        }
        if character == '/' && characters.get(index + 1) == Some(&'/') {
            index += 2;
            while index < characters.len() {
                let comment_character = characters[index];
                index += 1;
                if comment_character == '\n' {
                    cleaned.push('\n');
                    break;
                }
            }
            continue;
        }
        if character == '/' && characters.get(index + 1) == Some(&'*') {
            index += 2;
            cleaned.push(' ');
            let mut depth = 1usize;
            while index < characters.len() && depth > 0 {
                if characters.get(index) == Some(&'/') && characters.get(index + 1) == Some(&'*') {
                    depth += 1;
                    cleaned.extend([' ', ' ']);
                    index += 2;
                } else if characters.get(index) == Some(&'*')
                    && characters.get(index + 1) == Some(&'/')
                {
                    depth -= 1;
                    cleaned.extend([' ', ' ']);
                    index += 2;
                } else {
                    cleaned.push(if characters[index] == '\n' { '\n' } else { ' ' });
                    index += 1;
                }
            }
            if depth != 0 {
                return Err("Gradle settings contain an unterminated block comment".to_string());
            }
            continue;
        }
        cleaned.push(character);
        index += 1;
    }

    Ok(cleaned)
}

pub(super) fn find_unquoted_gradle_fragment(
    content: &str,
    fragment: &str,
    search_start: usize,
) -> Option<usize> {
    let mut quote = None;
    let mut escaped = false;
    for (relative, character) in content[search_start..].char_indices() {
        let index = search_start + relative;
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
        if content[index..].starts_with(fragment) {
            return Some(index);
        }
    }
    None
}

pub(super) fn is_gradle_include_start(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("include") else {
        return false;
    };
    rest.is_empty()
        || rest.chars().next().is_some_and(|character| {
            character.is_whitespace() || matches!(character, '(' | '\'' | '"')
        })
}

pub(super) fn gradle_paren_balance(value: &str) -> i32 {
    let mut balance = 0i32;
    let mut quote = None;
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && quote.is_some() {
            escaped = true;
            continue;
        }
        if matches!(character, '\'' | '"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            }
            continue;
        }
        if quote.is_none() {
            match character {
                '(' => balance += 1,
                ')' => balance -= 1,
                _ => {}
            }
        }
    }
    balance
}

pub(super) fn gradle_string_arguments(mut value: &str) -> Result<Vec<String>, String> {
    let mut values = Vec::new();
    loop {
        value = value.trim_start();
        if value.is_empty() {
            return Ok(values);
        }

        let (parsed, remainder) = gradle_string_literal(value)?;
        values.push(parsed);
        value = remainder.trim_start();
        if value.is_empty() {
            return Ok(values);
        }
        let Some(remainder) = value.strip_prefix(',') else {
            return Err("Unsupported or dynamic Gradle expression".to_string());
        };
        value = remainder;
    }
}

pub(super) fn gradle_string_literal(value: &str) -> Result<(String, &str), String> {
    let Some(delimiter) = value
        .chars()
        .next()
        .filter(|character| matches!(character, '\'' | '"'))
    else {
        return Err("Unsupported or dynamic Gradle expression".to_string());
    };
    let mut parsed = String::new();
    let body = &value[delimiter.len_utf8()..];
    let mut characters = body.char_indices().peekable();
    while let Some((index, character)) = characters.next() {
        if character == delimiter {
            let end = delimiter.len_utf8() + index + character.len_utf8();
            return Ok((parsed, &value[end..]));
        }
        if delimiter == '"' && character == '$' {
            return Err("Unsupported or dynamic Gradle expression".to_string());
        }
        if character != '\\' {
            parsed.push(character);
            continue;
        }

        let Some((_, escaped)) = characters.next() else {
            return Err("Gradle settings contain a dangling string escape".to_string());
        };
        match escaped {
            '\\' | '\'' | '"' | '$' => parsed.push(escaped),
            'n' => parsed.push('\n'),
            'r' => parsed.push('\r'),
            't' => parsed.push('\t'),
            'b' => parsed.push('\u{0008}'),
            'f' => parsed.push('\u{000c}'),
            'u' => {
                let mut value = 0u32;
                for _ in 0..4 {
                    let Some((_, digit)) = characters.next() else {
                        return Err(
                            "Gradle settings contain an incomplete Unicode escape".to_string()
                        );
                    };
                    let Some(digit) = digit.to_digit(16) else {
                        return Err("Gradle settings contain an invalid Unicode escape".to_string());
                    };
                    value = value * 16 + digit;
                }
                let Some(decoded) = char::from_u32(value) else {
                    return Err("Gradle settings contain an invalid Unicode escape".to_string());
                };
                if delimiter == '"' && decoded == '$' {
                    return Err("Unsupported or dynamic Gradle expression".to_string());
                }
                parsed.push(decoded);
            }
            '0'..='7' => {
                let mut value = escaped.to_digit(8).unwrap_or_default();
                for _ in 0..2 {
                    let Some((_, digit)) = characters.peek().copied() else {
                        break;
                    };
                    let Some(digit) = digit.to_digit(8) else {
                        break;
                    };
                    characters.next();
                    value = value * 8 + digit;
                }
                let Some(decoded) = char::from_u32(value) else {
                    return Err("Gradle settings contain an invalid octal escape".to_string());
                };
                if delimiter == '"' && decoded == '$' {
                    return Err("Unsupported or dynamic Gradle expression".to_string());
                }
                parsed.push(decoded);
            }
            _ => {
                return Err(format!(
                    "Gradle settings contain an unsupported string escape: \\{escaped}"
                ));
            }
        }
    }
    Err("Gradle settings contain an unterminated quoted string".to_string())
}
