use super::*;

#[test]
fn test_get_spec_symbols() {
    let body = r#"## Purpose
Something

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `createAuth` | config: Config | Auth | Creates auth |
| `validateToken` | token: string | bool | Validates |

### Exported Types

| Type | Description |
|------|-------------|
| `AuthConfig` | Config type |

## Invariants
"#;
    let symbols = get_spec_symbols(body);
    assert_eq!(symbols, vec!["createAuth", "validateToken", "AuthConfig"]);
}

#[test]
fn fenced_example_backticks_are_not_documented_exports() {
    let body = r#"## Public API

```markdown
| Export | Description |
|--------|-------------|
| `quotedLogin` | fenced example, not the contract |
```

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `realLogin` | user: string | bool | Real export |
"#;
    assert_eq!(get_spec_symbols(body), vec!["realLogin"]);
    assert_eq!(get_all_api_table_symbols(body), vec!["realLogin"]);
}

#[test]
fn fenced_exported_heading_is_not_a_public_api_subsection() {
    let body = r#"## Public API

```
### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `ghost` | | | quoted heading and row |
```

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `real` | | | real row |
"#;
    assert_eq!(get_spec_symbols(body), vec!["real"]);
    assert_eq!(get_all_api_table_symbols(body), vec!["real"]);
    let blanked = blank_fenced_code(body);
    assert_eq!(blanked.len(), body.len());
    assert_eq!(
        blanked.matches("`ghost`").count(),
        0,
        "blank_fenced_code must remove fenced rows"
    );
}

#[test]
fn test_get_spec_symbols_preserves_complete_punctuated_symbols() {
    let body = r#"## Public API

### Exported Symbols

| Symbol | Description |
|--------|-------------|
| `inputs.config` | Dotted YAML input |
| `inputs.working-directory` | Dotted and hyphenated YAML input |
| `outputs.atlas-enabled` | Dotted and hyphenated YAML output |
| `permissions.id-token` | Dotted and hyphenated permission |
| `jobs.deploy-atlas` | Dotted and hyphenated job |
| `login` | Ordinary identifier |
| `active?` | Predicate-style identifier |
| `save!` | Mutating identifier |
| `name_=` | Setter identifier |
| `setName:age:` | Objective-C selector |
| `~Widget` | C++ destructor |
| `map'` | Apostrophe identifier |
| `*global-var*` | Lisp identifier |
| `%+%` | R operator |
| `>>=` | Haskell or OCaml operator |
| `operator==` | C++ operator |
| `operator[]` | C++ subscript operator |
| `operator new` | C++ operator containing a space |
| `<|>` | Elixir or Scala operator containing a pipe |
| `!!!` | Punctuation-only operator |
| `MyApp.Auth` | Dotted module path |
| `módulo` | Unicode identifier |
| `with space` | Quoted atom or string export name |

## Invariants
"#;

    let expected = vec![
        "inputs.config",
        "inputs.working-directory",
        "outputs.atlas-enabled",
        "permissions.id-token",
        "jobs.deploy-atlas",
        "login",
        "active?",
        "save!",
        "name_=",
        "setName:age:",
        "~Widget",
        "map'",
        "*global-var*",
        "%+%",
        ">>=",
        "operator==",
        "operator[]",
        "operator new",
        "<|>",
        "!!!",
        "MyApp.Auth",
        "módulo",
        "with space",
    ];

    assert_eq!(get_spec_symbols(body), expected);
    assert_eq!(get_all_api_table_symbols(body), expected);
}

#[test]
fn test_api_table_symbol_parser_rejects_empty_or_malformed_rows() {
    let body = r#"## Public API

### Exported Symbols

| Symbol | Description |
|--------|-------------|
| `` | Empty code span |
| `   ` | Whitespace-only code span |
| `unterminated | Missing closing delimiter |
| plain | `laterColumn` |
| `trailing` text | Closing span does not occupy the cell |
This prose contains `proseSymbol` but is not a table row.
| `login` | Valid control row |

## Invariants
"#;

    assert_eq!(get_spec_symbols(body), vec!["login"]);
    assert_eq!(get_all_api_table_symbols(body), vec!["login"]);
}

#[test]
fn test_get_spec_symbols_skips_non_exported_subsections() {
    let body = r#"## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `authenticate` | token: string | User | Validates token |

### API Endpoints

| Endpoint | Method | Handler | Description |
|----------|--------|---------|-------------|
| `/login` | POST | `login` | Login route |
| `/logout` | POST | `logout` | Logout route |

### Component API

| Signal | Type | Description |
|--------|------|-------------|
| `activeTab` | string | Current tab |

### Route Handlers

| Handler | Description |
|---------|-------------|
| `registration_status` | Check registration |

### Exported Types

| Type | Description |
|------|-------------|
| `AuthConfig` | Config type |

### Configuration

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `timeout` | number | 30 | Request timeout |

### Internal Functions

| Function | Description |
|----------|-------------|
| `hashPassword` | Internal hashing |

## Invariants
"#;
    let symbols = get_spec_symbols(body);
    // Only symbols under "### Exported ..." subsections should be extracted
    assert_eq!(symbols, vec!["authenticate", "AuthConfig"]);
}

#[test]
fn test_get_all_api_table_symbols_includes_informational_tables() {
    let body = r#"# Auth

## Public API

### Functions

| Function | Description |
|----------|-------------|
| `greet` | Says hello |

### API Endpoints

| Endpoint | Description |
|----------|-------------|
| `login` | Login route |

## Error Cases

| Condition | Behavior |
|-----------|----------|
| `OutsideApiSection` | Not collected |
"#;
    let symbols = get_all_api_table_symbols(body);
    // Tables under ANY ### heading within ## Public API count, but tables
    // in other ## sections do not.
    assert_eq!(symbols, vec!["greet", "login"]);
}

#[test]
fn test_get_all_api_table_symbols_no_api_section() {
    assert!(get_all_api_table_symbols("# Title\n\n## Purpose\n\nText.\n").is_empty());
}

#[test]
fn test_is_export_header() {
    // Should match
    assert!(is_export_header("### Exported Functions"));
    assert!(is_export_header("### Exported Types"));
    assert!(is_export_header("### TypeScript Exports"));
    assert!(is_export_header("### Exports"));
    assert!(is_export_header("### Public Functions"));
    assert!(is_export_header("### Public Types"));
    assert!(is_export_header("### Export Types"));
    assert!(is_export_header("### Exported Symbols"));
    assert!(is_export_header("### exported functions")); // case-insensitive

    // Should NOT match
    assert!(!is_export_header("### API Endpoints"));
    assert!(!is_export_header("### Component API"));
    assert!(!is_export_header("### Route Handlers"));
    assert!(!is_export_header("### Configuration"));
    assert!(!is_export_header("### Internal Functions"));
    // Should NOT match — "unexported" contains "exported" as a substring
    assert!(!is_export_header("### Unexported Functions"));
    assert!(!is_export_header("### Unexported Types"));
}

#[test]
fn test_get_spec_symbols_accepts_header_variations() {
    let body = r#"## Public API

### TypeScript Exports

| Function | Description |
|----------|-------------|
| `createAuth` | Creates auth |
| `validateToken` | Validates |

### Public Types

| Type | Description |
|------|-------------|
| `AuthConfig` | Config type |

### API Endpoints

| Endpoint | Method |
|----------|--------|
| `/login` | POST |

## Invariants
"#;
    let symbols = get_spec_symbols(body);
    // Should extract from "TypeScript Exports" and "Public Types" but not "API Endpoints"
    assert_eq!(symbols, vec!["createAuth", "validateToken", "AuthConfig"]);
}

#[test]
fn test_get_spec_symbols_top_level_table() {
    // Tables directly under ## Public API (no ### header) should be validated
    let body = r#"## Public API

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `helper` | input: string | string | Helps |

## Invariants
"#;
    let symbols = get_spec_symbols(body);
    assert_eq!(symbols, vec!["helper"]);
}
