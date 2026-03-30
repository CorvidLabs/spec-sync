use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

/// A column extracted from SQL schema files.
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaColumn {
    pub name: String,
    /// Normalised to uppercase (e.g. "INTEGER", "TEXT").
    pub col_type: String,
    pub nullable: bool,
    pub has_default: bool,
    pub is_primary_key: bool,
}

/// All columns for a single table, built by replaying migrations in order.
#[derive(Debug, Clone, Default)]
pub struct SchemaTable {
    pub columns: Vec<SchemaColumn>,
}

impl SchemaTable {
    #[cfg(test)]
    pub fn column_names(&self) -> Vec<&str> {
        self.columns.iter().map(|c| c.name.as_str()).collect()
    }
}

/// A column documented in a spec's ### Schema section.
#[derive(Debug, Clone, PartialEq)]
pub struct SpecColumn {
    pub name: String,
    /// Raw type string from the spec (e.g. "INTEGER", "TEXT").
    pub col_type: String,
}

// ─── SQL Parsing ─────────────────────────────────────────────────────────

static CREATE_TABLE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)CREATE\s+(?:VIRTUAL\s+)?TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?(\w+)\s*\(").unwrap()
});

static ALTER_ADD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)ALTER\s+TABLE\s+(\w+)\s+ADD\s+(?:COLUMN\s+)?(\w+)\s+(\w+)").unwrap()
});

/// Build a complete schema map from SQL/migration files in the given directory.
/// Files are sorted by name so migrations replay in order.
pub fn build_schema(schema_dir: &Path) -> HashMap<String, SchemaTable> {
    let mut tables: HashMap<String, SchemaTable> = HashMap::new();

    if !schema_dir.exists() {
        return tables;
    }

    // Collect and sort files for deterministic migration ordering.
    let mut files: Vec<_> = fs::read_dir(schema_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            let ext = e
                .path()
                .extension()
                .and_then(|x| x.to_str())
                .unwrap_or("")
                .to_string();
            ext == "sql" || ext == "ts"
        })
        .collect();
    files.sort_by_key(|e| e.file_name());

    for entry in &files {
        let content = match fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };
        parse_sql_into(&content, &mut tables);
    }

    tables
}

/// Parse SQL content and merge discovered tables/columns into the map.
fn parse_sql_into(sql: &str, tables: &mut HashMap<String, SchemaTable>) {
    // Handle CREATE TABLE statements
    for cap in CREATE_TABLE_RE.captures_iter(sql) {
        let table_name = cap[1].to_string();
        let start = cap.get(0).unwrap().end(); // position after opening paren

        // Find matching closing paren (handles nested parens for CHECK constraints etc.)
        if let Some(body) = extract_paren_body(sql, start) {
            let columns = parse_column_defs(&body);
            let entry = tables.entry(table_name).or_default();
            // CREATE TABLE replaces any prior definition (e.g. CREATE OR REPLACE)
            entry.columns = columns;
        }
    }

    // Handle ALTER TABLE ADD COLUMN
    for cap in ALTER_ADD_RE.captures_iter(sql) {
        let table_name = cap[1].to_string();
        let col_name = cap[2].to_string();
        let col_type = cap[3].to_uppercase();

        // Get the full ALTER statement for constraint analysis
        let full_match_start = cap.get(0).unwrap().start();
        let rest = &sql[full_match_start..];
        let stmt_end = rest.find(';').unwrap_or(rest.len());
        let full_stmt = &rest[..stmt_end].to_uppercase();

        let nullable = !full_stmt.contains("NOT NULL");
        let has_default = full_stmt.contains("DEFAULT");
        let is_primary_key = full_stmt.contains("PRIMARY KEY");

        let entry = tables.entry(table_name).or_default();
        // Only add if column doesn't already exist (idempotent)
        if !entry.columns.iter().any(|c| c.name == col_name) {
            entry.columns.push(SchemaColumn {
                name: col_name,
                col_type,
                nullable,
                has_default,
                is_primary_key,
            });
        }
    }
}

/// Extract text between the opening paren (at `start`) and its matching close.
fn extract_paren_body(sql: &str, start: usize) -> Option<String> {
    let bytes = sql.as_bytes();
    let mut depth = 1;
    let mut i = start;
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b'\'' => {
                // Skip string literal
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == b'\'' {
                        if i + 1 < bytes.len() && bytes[i + 1] == b'\'' {
                            i += 2; // escaped quote
                            continue;
                        }
                        break;
                    }
                    i += 1;
                }
            }
            b'-' if i + 1 < bytes.len() && bytes[i + 1] == b'-' => {
                // Skip line comment
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    if depth == 0 {
        Some(sql[start..i - 1].to_string())
    } else {
        None
    }
}

/// Parse column definitions from the body of a CREATE TABLE (between parens).
fn parse_column_defs(body: &str) -> Vec<SchemaColumn> {
    let mut columns = Vec::new();

    // Split on commas that aren't inside parens
    let parts = split_top_level(body, ',');

    for part in &parts {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }

        let upper = trimmed.to_uppercase();

        // Skip table-level constraints (PRIMARY KEY, UNIQUE, CHECK, FOREIGN KEY, CONSTRAINT)
        if upper.starts_with("PRIMARY KEY")
            || upper.starts_with("UNIQUE")
            || upper.starts_with("CHECK")
            || upper.starts_with("FOREIGN KEY")
            || upper.starts_with("CONSTRAINT")
        {
            continue;
        }

        // Parse: column_name TYPE [constraints...]
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        if tokens.len() < 2 {
            continue;
        }

        let col_name = tokens[0].to_string();
        // Skip if the column name looks like a keyword (extra safety)
        if is_sql_keyword(&col_name) {
            continue;
        }

        let col_type = tokens[1].to_uppercase();
        let rest_upper = upper.clone();

        let nullable = !rest_upper.contains("NOT NULL");
        let has_default = rest_upper.contains("DEFAULT");
        let is_primary_key = rest_upper.contains("PRIMARY KEY");

        columns.push(SchemaColumn {
            name: col_name,
            col_type,
            nullable,
            has_default,
            is_primary_key,
        });
    }

    columns
}

/// Split a string on a delimiter, but only at the top level (not inside parens).
fn split_top_level(s: &str, delim: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let ch = bytes[i] as char;
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            '\'' => {
                current.push(ch);
                i += 1;
                while i < bytes.len() {
                    let c = bytes[i] as char;
                    current.push(c);
                    if c == '\'' {
                        if i + 1 < bytes.len() && bytes[i + 1] == b'\'' {
                            current.push('\'');
                            i += 2;
                            continue;
                        }
                        break;
                    }
                    i += 1;
                }
            }
            c if c == delim && depth == 0 => {
                parts.push(std::mem::take(&mut current));
            }
            _ => current.push(ch),
        }
        i += 1;
    }

    if !current.trim().is_empty() {
        parts.push(current);
    }

    parts
}

fn is_sql_keyword(s: &str) -> bool {
    matches!(
        s.to_uppercase().as_str(),
        "PRIMARY" | "UNIQUE" | "CHECK" | "FOREIGN" | "CONSTRAINT" | "INDEX" | "CREATE" | "TABLE"
    )
}

// ─── Spec Schema Parsing ─────────────────────────────────────────────────

static SCHEMA_HEADER_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Matches: ### Schema   or   ### Schema: table_name
    Regex::new(r"(?m)^###\s+Schema(?::\s*(\w+))?\s*$").unwrap()
});

static SCHEMA_TABLE_HEADER_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Matches: #### `table_name`   or   #### table_name
    Regex::new(r"(?m)^####\s+`?(\w+)`?\s*$").unwrap()
});

static COLUMN_ROW_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Matches: | `col_name` | TYPE | ... |
    Regex::new(r"^\|\s*`(\w+)`\s*\|\s*([^|]+?)\s*\|").unwrap()
});

/// Extract column definitions from a spec's ### Schema section.
/// Returns a map of table_name -> Vec<SpecColumn>.
///
/// Supports two formats:
///
/// Format 1 — single table (table name in header):
/// ```markdown
/// ### Schema: messages
/// | Column | Type | ... |
/// | `id` | INTEGER | ... |
/// ```
///
/// Format 2 — multiple tables (table names as #### sub-headers):
/// ```markdown
/// ### Schema
/// #### `messages`
/// | Column | Type | ... |
/// | `id` | INTEGER | ... |
/// #### `users`
/// | Column | Type | ... |
/// ```
pub fn parse_spec_schema(body: &str) -> HashMap<String, Vec<SpecColumn>> {
    let mut result: HashMap<String, Vec<SpecColumn>> = HashMap::new();

    // Find ### Schema sections
    for schema_cap in SCHEMA_HEADER_RE.captures_iter(body) {
        let match_start = schema_cap.get(0).unwrap().start();
        let inline_table = schema_cap.get(1).map(|m| m.as_str().to_string());

        // Find the end of this ### section (next ## or ### that isn't ####)
        let after_header = match body[match_start..].find('\n') {
            Some(pos) => match_start + pos + 1,
            None => continue,
        };

        let section_end = {
            let rest = &body[after_header..];
            // Find next ## or ### heading (but not ####).
            // Scan for "\n## " or "\n### " that isn't "\n#### ".
            let mut end = rest.len();
            let mut pos = 0;
            while pos < rest.len() {
                if let Some(nl) = rest[pos..].find('\n') {
                    let line_start = pos + nl + 1;
                    if line_start >= rest.len() {
                        break;
                    }
                    let after_nl = &rest[line_start..];
                    if (after_nl.starts_with("## ") || after_nl.starts_with("### "))
                        && !after_nl.starts_with("#### ")
                    {
                        end = line_start;
                        break;
                    }
                    pos = line_start;
                } else {
                    break;
                }
            }
            after_header + end
        };

        let section = &body[after_header..section_end];

        if let Some(table_name) = inline_table {
            // Format 1: ### Schema: table_name — all rows belong to this table
            let columns = extract_columns_from_section(section);
            if !columns.is_empty() {
                result.insert(table_name, columns);
            }
        } else {
            // Format 2: ### Schema — look for #### sub-headers per table
            let mut current_table: Option<String> = None;
            let mut current_columns: Vec<SpecColumn> = Vec::new();

            for line in section.lines() {
                if let Some(cap) = SCHEMA_TABLE_HEADER_RE.captures(line) {
                    // Flush previous table
                    if let Some(name) = current_table.take()
                        && !current_columns.is_empty()
                    {
                        result.insert(name, std::mem::take(&mut current_columns));
                    }
                    current_table = Some(cap[1].to_string());
                    current_columns.clear();
                } else if current_table.is_some() {
                    if let Some(cap) = COLUMN_ROW_RE.captures(line) {
                        let name = cap[1].to_string();
                        let col_type = cap[2].trim().to_uppercase();
                        if !is_table_header_word(&name) {
                            current_columns.push(SpecColumn { name, col_type });
                        }
                    }
                } else {
                    // No #### header yet — could be a top-level table
                    // (e.g. when there's only one table and no #### headers)
                    if let Some(cap) = COLUMN_ROW_RE.captures(line) {
                        let name = cap[1].to_string();
                        let col_type = cap[2].trim().to_uppercase();
                        if name.to_lowercase() != "column" && name.to_lowercase() != "name" {
                            current_columns.push(SpecColumn { name, col_type });
                        }
                    }
                }
            }

            // Flush last table
            if let Some(name) = current_table
                && !current_columns.is_empty()
            {
                result.insert(name, current_columns);
            }
            // If columns were found without any #### header, they're orphaned —
            // skip them (we don't know which table they belong to).
        }
    }

    result
}

fn extract_columns_from_section(section: &str) -> Vec<SpecColumn> {
    let mut columns = Vec::new();
    for line in section.lines() {
        if let Some(cap) = COLUMN_ROW_RE.captures(line) {
            let name = cap[1].to_string();
            let col_type = cap[2].trim().to_uppercase();
            if !is_table_header_word(&name) {
                columns.push(SpecColumn { name, col_type });
            }
        }
    }
    columns
}

/// Returns true if the word is a markdown table header label (not a real column name).
/// We detect header rows by checking if the "type" column contains a header-like word.
fn is_table_header_word(name: &str) -> bool {
    // Only skip the exact header word "Column" (case-insensitive)
    name.eq_ignore_ascii_case("column")
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_create_table() {
        let sql = r#"
CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    sender TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    read INTEGER DEFAULT 0
);
"#;
        let mut tables = HashMap::new();
        parse_sql_into(sql, &mut tables);

        let t = tables.get("messages").unwrap();
        assert_eq!(t.columns.len(), 5);

        assert_eq!(t.columns[0].name, "id");
        assert_eq!(t.columns[0].col_type, "INTEGER");
        assert!(t.columns[0].is_primary_key);

        assert_eq!(t.columns[1].name, "content");
        assert_eq!(t.columns[1].col_type, "TEXT");
        assert!(!t.columns[1].nullable);

        assert_eq!(t.columns[3].name, "created_at");
        assert!(t.columns[3].has_default);

        assert_eq!(t.columns[4].name, "read");
        assert!(t.columns[4].nullable);
        assert!(t.columns[4].has_default);
    }

    #[test]
    fn test_parse_create_table_if_not_exists() {
        let sql = "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);";
        let mut tables = HashMap::new();
        parse_sql_into(sql, &mut tables);

        let t = tables.get("users").unwrap();
        assert_eq!(t.columns.len(), 2);
        assert_eq!(t.columns[0].name, "id");
        assert_eq!(t.columns[1].name, "name");
    }

    #[test]
    fn test_parse_create_virtual_table() {
        // Virtual tables use USING syntax — the paren is after the module name,
        // not directly after the table name. Our regex requires `table_name (`
        // so virtual tables are intentionally skipped for column parsing.
        // Table *existence* is still caught by get_schema_table_names() which
        // has a separate regex that handles VIRTUAL TABLE.
        let sql = "CREATE VIRTUAL TABLE search_idx USING fts5(content, sender);";
        let mut tables = HashMap::new();
        parse_sql_into(sql, &mut tables);
        // Virtual tables won't be parsed for columns (different syntax)
        assert!(!tables.contains_key("search_idx"));
    }

    #[test]
    fn test_parse_alter_table_add_column() {
        let sql = r#"
CREATE TABLE tasks (id INTEGER PRIMARY KEY, title TEXT NOT NULL);
ALTER TABLE tasks ADD COLUMN status TEXT NOT NULL DEFAULT 'pending';
ALTER TABLE tasks ADD COLUMN priority INTEGER DEFAULT 0;
"#;
        let mut tables = HashMap::new();
        parse_sql_into(sql, &mut tables);

        let t = tables.get("tasks").unwrap();
        assert_eq!(t.columns.len(), 4);
        assert_eq!(t.columns[2].name, "status");
        assert_eq!(t.columns[2].col_type, "TEXT");
        assert!(!t.columns[2].nullable);
        assert!(t.columns[2].has_default);

        assert_eq!(t.columns[3].name, "priority");
        assert!(t.columns[3].nullable);
    }

    #[test]
    fn test_alter_idempotent() {
        let sql = r#"
CREATE TABLE t (id INTEGER PRIMARY KEY);
ALTER TABLE t ADD COLUMN name TEXT;
ALTER TABLE t ADD COLUMN name TEXT;
"#;
        let mut tables = HashMap::new();
        parse_sql_into(sql, &mut tables);
        assert_eq!(tables.get("t").unwrap().columns.len(), 2);
    }

    #[test]
    fn test_table_constraints_skipped() {
        let sql = r#"
CREATE TABLE edges (
    source_id INTEGER NOT NULL,
    target_id INTEGER NOT NULL,
    weight REAL DEFAULT 1.0,
    PRIMARY KEY (source_id, target_id),
    FOREIGN KEY (source_id) REFERENCES nodes(id),
    UNIQUE (source_id, target_id, weight),
    CHECK (weight > 0)
);
"#;
        let mut tables = HashMap::new();
        parse_sql_into(sql, &mut tables);

        let t = tables.get("edges").unwrap();
        assert_eq!(t.columns.len(), 3);
        assert_eq!(t.column_names(), vec!["source_id", "target_id", "weight"]);
    }

    #[test]
    fn test_string_literal_in_default() {
        let sql = "CREATE TABLE t (status TEXT NOT NULL DEFAULT 'it''s pending');";
        let mut tables = HashMap::new();
        parse_sql_into(sql, &mut tables);
        let t = tables.get("t").unwrap();
        assert_eq!(t.columns.len(), 1);
        assert!(t.columns[0].has_default);
    }

    #[test]
    fn test_parse_spec_schema_inline() {
        let body = r#"## Purpose
Something

### Schema: messages

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | INTEGER | PRIMARY KEY |
| `content` | TEXT | NOT NULL |
| `created_at` | TEXT | DEFAULT |

## Invariants
"#;
        let schema = parse_spec_schema(body);
        assert_eq!(schema.len(), 1);
        let cols = schema.get("messages").unwrap();
        assert_eq!(cols.len(), 3);
        assert_eq!(cols[0].name, "id");
        assert_eq!(cols[0].col_type, "INTEGER");
        assert_eq!(cols[1].name, "content");
        assert_eq!(cols[2].name, "created_at");
    }

    #[test]
    fn test_parse_spec_schema_multi_table() {
        let body = r#"## Purpose
Something

### Schema

#### `messages`

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Row ID |
| `body` | TEXT | Message body |

#### `users`

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Row ID |
| `name` | TEXT | Username |
| `email` | TEXT | Email addr |

## Invariants
"#;
        let schema = parse_spec_schema(body);
        assert_eq!(schema.len(), 2);
        assert_eq!(schema.get("messages").unwrap().len(), 2);
        assert_eq!(schema.get("users").unwrap().len(), 3);
    }

    #[test]
    fn test_parse_spec_schema_no_section() {
        let body = "## Purpose\nSomething\n## Public API\nStuff\n";
        let schema = parse_spec_schema(body);
        assert!(schema.is_empty());
    }

    #[test]
    fn test_build_schema_nonexistent_dir() {
        let tables = build_schema(Path::new("/nonexistent/path"));
        assert!(tables.is_empty());
    }

    #[test]
    fn test_build_schema_migration_ordering() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();

        // Write migrations in numbered order
        fs::write(
            dir.join("001_create.sql"),
            "CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT NOT NULL);",
        )
        .unwrap();
        fs::write(
            dir.join("002_add_col.sql"),
            "ALTER TABLE items ADD COLUMN price REAL DEFAULT 0;",
        )
        .unwrap();

        let tables = build_schema(dir);
        let t = tables.get("items").unwrap();
        assert_eq!(t.columns.len(), 3);
        assert_eq!(t.columns[0].name, "id");
        assert_eq!(t.columns[1].name, "name");
        assert_eq!(t.columns[2].name, "price");
        assert_eq!(t.columns[2].col_type, "REAL");
    }

    #[test]
    fn test_multiple_tables_in_one_file() {
        let sql = r#"
CREATE TABLE a (id INTEGER PRIMARY KEY);
CREATE TABLE b (id INTEGER PRIMARY KEY, ref_a INTEGER);
"#;
        let mut tables = HashMap::new();
        parse_sql_into(sql, &mut tables);
        assert!(tables.contains_key("a"));
        assert!(tables.contains_key("b"));
        assert_eq!(tables.get("b").unwrap().columns.len(), 2);
    }
}
