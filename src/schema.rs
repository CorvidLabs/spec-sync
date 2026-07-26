use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

/// A column extracted from SQL schema files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaColumn {
    pub name: String,
    /// Normalised to uppercase (e.g. "INTEGER", "TEXT").
    pub col_type: String,
    pub nullable: bool,
    pub has_default: bool,
    pub is_primary_key: bool,
}

/// All columns for a single table, built by replaying migrations in order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecColumn {
    pub name: String,
    /// Raw type string from the spec (e.g. "INTEGER", "TEXT").
    pub col_type: String,
}

/// Stable categories for schema-loading and replay failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaErrorKind {
    MissingDirectory,
    ReadDirectory,
    ReadEntry,
    ReadFile,
    MalformedStatement,
    MissingTable,
    DuplicateTable,
    RenameCollision,
    MissingColumn,
    DuplicateColumn,
}

/// A path- and source-position-aware schema loading or replay failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaError {
    pub kind: SchemaErrorKind,
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl fmt::Display for SchemaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(formatter, "{}: {}", self.path.display(), self.message)
        } else {
            write!(
                formatter,
                "{}:{}:{}: {}",
                self.path.display(),
                self.line,
                self.column,
                self.message
            )
        }
    }
}

impl std::error::Error for SchemaError {}

impl SchemaError {
    fn for_path(kind: SchemaErrorKind, path: &Path, message: impl Into<String>) -> Self {
        Self {
            kind,
            path: path.to_path_buf(),
            line: 0,
            column: 0,
            message: message.into(),
        }
    }

    fn at(
        kind: SchemaErrorKind,
        path: &Path,
        sql: &str,
        offset: usize,
        message: impl Into<String>,
    ) -> Self {
        let bounded_offset = offset.min(sql.len());
        let before = &sql[..bounded_offset];
        let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
        let column = before
            .rsplit_once('\n')
            .map(|(_, tail)| tail.chars().count() + 1)
            .unwrap_or_else(|| before.chars().count() + 1);

        Self {
            kind,
            path: path.to_path_buf(),
            line,
            column,
            message: message.into(),
        }
    }
}

/// The current canonical schema state after deterministic migration replay.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SchemaSnapshot {
    pub tables: HashMap<String, SchemaTable>,
    pub retired_tables: HashSet<String>,
}

// ─── SQL Parsing ─────────────────────────────────────────────────────────

/// One SQL identifier segment: ANSI double quotes, MySQL backticks, SQL Server
/// brackets, or a conservative bare identifier.
const IDENTIFIER_SEGMENT: &str =
    r#"(?:"(?:[^"]|"")+"|`(?:[^`]|``)+`|\[(?:[^\]]|\]\])+\]|[A-Za-z_][A-Za-z0-9_$]*)"#;

fn schema_regex(pattern: &str) -> Regex {
    let qualified_name = format!("{IDENTIFIER_SEGMENT}(?:\\s*\\.\\s*{IDENTIFIER_SEGMENT})*");
    let expanded = pattern
        .replace("{NAME}", &qualified_name)
        .replace("{IDENT}", IDENTIFIER_SEGMENT);
    Regex::new(&expanded).expect("static schema regex must compile")
}

static DDL_START_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(?:CREATE\s+(?:OR\s+REPLACE\s+)?(?:VIRTUAL\s+)?TABLE|ALTER\s+TABLE|DROP\s+TABLE)\b",
    )
    .expect("static DDL start regex must compile")
});

static CREATE_TABLE_RE: LazyLock<Regex> = LazyLock::new(|| {
    schema_regex(
        r"(?is)^CREATE\s+(?P<replace>OR\s+REPLACE\s+)?TABLE\s+(?P<if_not_exists>IF\s+NOT\s+EXISTS\s+)?(?P<table>{NAME})\s*\(",
    )
});

static CREATE_VIRTUAL_TABLE_RE: LazyLock<Regex> = LazyLock::new(|| {
    schema_regex(
        r"(?is)^CREATE\s+VIRTUAL\s+TABLE\s+(?P<if_not_exists>IF\s+NOT\s+EXISTS\s+)?(?P<table>{NAME})\s+USING\s+[A-Za-z_][A-Za-z0-9_]*",
    )
});

static ALTER_ADD_RE: LazyLock<Regex> = LazyLock::new(|| {
    schema_regex(
        r"(?is)^ALTER\s+TABLE\s+(?P<table>{NAME})\s+ADD\s+(?:COLUMN\s+)?(?P<column>{IDENT})\s+(?P<type>[A-Za-z_][A-Za-z0-9_]*(?:\s*\([^)]*\))?)",
    )
});

static DROP_TABLE_RE: LazyLock<Regex> = LazyLock::new(|| {
    schema_regex(r"(?is)^DROP\s+TABLE\s+(?P<if_exists>IF\s+EXISTS\s+)?(?P<table>{NAME})(?:\s|$)")
});

static ALTER_DROP_COL_RE: LazyLock<Regex> = LazyLock::new(|| {
    schema_regex(
        r"(?is)^ALTER\s+TABLE\s+(?P<table>{NAME})\s+DROP\s+(?:COLUMN\s+)?(?P<column>{IDENT})(?:\s|$)",
    )
});

static ALTER_RENAME_TABLE_RE: LazyLock<Regex> = LazyLock::new(|| {
    schema_regex(
        r"(?is)^ALTER\s+TABLE\s+(?P<table>{NAME})\s+RENAME\s+TO\s+(?P<new_table>{NAME})(?:\s|$)",
    )
});

static ALTER_RENAME_COL_RE: LazyLock<Regex> = LazyLock::new(|| {
    schema_regex(
        r"(?is)^ALTER\s+TABLE\s+(?P<table>{NAME})\s+RENAME\s+(?:COLUMN\s+)?(?P<column>{IDENT})\s+TO\s+(?P<new_column>{IDENT})(?:\s|$)",
    )
});

/// Return a canonical table identity by parsing qualified identifier segments,
/// unescaping each supported quoting style, and applying Unicode lowercase.
pub fn canonicalize_table_name(raw: &str) -> Result<String, String> {
    let segments = split_table_identifier_segments(raw)?;
    let canonical_segments = segments
        .iter()
        .map(|segment| normalize_identifier_segment(segment))
        .map(|result| result.map(|segment| render_canonical_table_segment(&segment)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(canonical_segments.join("."))
}

/// Return the canonical final identifier segment of a qualified table name.
/// Validator code uses this for unqualified declarations instead of splitting
/// on `.`, which would corrupt a quoted identifier that itself contains a dot.
#[allow(dead_code)] // Consumed by the validator restack built on this schema slice.
pub fn canonical_table_leaf(raw: &str) -> Result<String, String> {
    let segments = split_table_identifier_segments(raw)?;
    let last = segments
        .last()
        .ok_or_else(|| format!("table identifier `{raw}` has no segments"))?;
    normalize_identifier_segment(last).map(|segment| render_canonical_table_segment(&segment))
}

fn split_table_identifier_segments(raw: &str) -> Result<Vec<String>, String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut characters = raw.chars().peekable();

    while let Some(character) = characters.next() {
        match quote {
            Some('"') if character == '"' => {
                current.push(character);
                if characters.peek() == Some(&'"') {
                    current.push(characters.next().unwrap_or('"'));
                } else {
                    quote = None;
                }
            }
            Some('`') if character == '`' => {
                current.push(character);
                if characters.peek() == Some(&'`') {
                    current.push(characters.next().unwrap_or('`'));
                } else {
                    quote = None;
                }
            }
            Some(']') if character == ']' => {
                current.push(character);
                if characters.peek() == Some(&']') {
                    current.push(characters.next().unwrap_or(']'));
                } else {
                    quote = None;
                }
            }
            Some(_) => current.push(character),
            None if character == '.' => {
                if current.trim().is_empty() {
                    return Err(format!("empty table identifier segment in `{raw}`"));
                }
                segments.push(std::mem::take(&mut current));
            }
            None if character == '"' || character == '`' => {
                quote = Some(character);
                current.push(character);
            }
            None if character == '[' => {
                quote = Some(']');
                current.push(character);
            }
            None => current.push(character),
        }
    }

    if quote.is_some() {
        return Err(format!("unterminated quoted table identifier `{raw}`"));
    }
    if current.trim().is_empty() {
        return Err(format!("empty table identifier segment in `{raw}`"));
    }
    segments.push(current);
    Ok(segments)
}

fn render_canonical_table_segment(segment: &str) -> String {
    if is_bare_identifier(segment) {
        segment.to_string()
    } else {
        format!("\"{}\"", segment.replace('"', "\"\""))
    }
}

fn normalize_identifier_segment(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("identifier segment is empty".to_string());
    }

    let unquoted = if let Some(inner) = trimmed
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    {
        inner.replace("\"\"", "\"")
    } else if let Some(inner) = trimmed
        .strip_prefix('`')
        .and_then(|value| value.strip_suffix('`'))
    {
        inner.replace("``", "`")
    } else if let Some(inner) = trimmed
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    {
        inner.replace("]]", "]")
    } else {
        if !is_bare_identifier(trimmed) {
            return Err(format!("malformed identifier segment `{raw}`"));
        }
        trimmed.to_string()
    };

    if unquoted.is_empty() {
        return Err(format!("identifier segment `{raw}` is empty"));
    }
    Ok(unquoted.to_lowercase())
}

fn is_bare_identifier(identifier: &str) -> bool {
    let mut characters = identifier.chars();
    characters
        .next()
        .map(|first| first == '_' || first.is_ascii_alphabetic())
        .unwrap_or(false)
        && characters.all(|character| {
            character == '_' || character == '$' || character.is_ascii_alphanumeric()
        })
}

/// Normalize a valid table reference for comparison. Invalid input is lowered
/// without structural rewriting; fallible replay uses `canonicalize_table_name`
/// and reports the malformed identifier instead.
#[allow(dead_code)] // Used by the validator restack that consumes this schema slice.
pub fn normalize_table_name(raw: &str) -> String {
    canonicalize_table_name(raw).unwrap_or_else(|_| raw.trim().to_lowercase())
}

/// File extensions that may contain embedded SQL statements.
const SQL_EXTENSIONS: &[&str] = &[
    "sql", "ts", "js", "mjs", "cjs", "swift", "kt", "kts", "java", "py", "rb", "go", "rs", "cs",
    "dart", "php",
];

/// Build a complete schema map from SQL/migration files in the given directory.
/// This compatibility wrapper returns an empty map on error. Validation code
/// must call `build_schema_snapshot` so malformed or unreadable migrations
/// cannot become a vacuous success.
pub fn build_schema(schema_dir: &Path) -> HashMap<String, SchemaTable> {
    build_schema_snapshot(schema_dir)
        .map(|snapshot| snapshot.tables)
        .unwrap_or_default()
}

/// Compatibility wrapper exposing retired names to older validator code.
/// Validation code should migrate to `build_schema_snapshot`.
#[allow(dead_code)] // Used by the validator restack until it adopts SchemaSnapshot.
pub fn build_schema_with_retired(
    schema_dir: &Path,
) -> (HashMap<String, SchemaTable>, HashSet<String>) {
    match build_schema_snapshot(schema_dir) {
        Ok(snapshot) => (snapshot.tables, snapshot.retired_tables),
        Err(_) => (HashMap::new(), HashSet::new()),
    }
}

/// Build one deterministic, fallible schema snapshot.
///
/// Files are sorted by filename and every supported DDL statement is replayed
/// in byte order within each file. Directory, entry, UTF-8, malformed-DDL,
/// missing-object, and canonical-name collision failures are returned instead
/// of being collapsed into an empty schema.
pub fn build_schema_snapshot(schema_dir: &Path) -> Result<SchemaSnapshot, SchemaError> {
    let read_dir = fs::read_dir(schema_dir).map_err(|error| {
        let kind = if error.kind() == std::io::ErrorKind::NotFound {
            SchemaErrorKind::MissingDirectory
        } else {
            SchemaErrorKind::ReadDirectory
        };
        SchemaError::for_path(
            kind,
            schema_dir,
            format!("schema directory could not be read: {error}"),
        )
    })?;

    let mut files = Vec::new();
    for entry in read_dir {
        let entry = entry.map_err(|error| {
            SchemaError::for_path(
                SchemaErrorKind::ReadEntry,
                schema_dir,
                format!("schema directory entry could not be read: {error}"),
            )
        })?;
        let path = entry.path();
        let supported = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| SQL_EXTENSIONS.contains(&extension))
            .unwrap_or(false);
        if supported && path.is_file() {
            files.push(path);
        }
    }
    files.sort();

    let mut snapshot = SchemaSnapshot::default();
    for path in files {
        let content = fs::read_to_string(&path).map_err(|error| {
            SchemaError::for_path(
                SchemaErrorKind::ReadFile,
                &path,
                format!("schema/migration file could not be read as UTF-8: {error}"),
            )
        })?;
        replay_sql(&path, &content, &mut snapshot)?;
    }

    Ok(snapshot)
}

/// Compatibility diagnostics for command paths that have not yet adopted the
/// fallible snapshot directly.
pub fn schema_read_errors(schema_dir: &Path) -> Vec<String> {
    match build_schema_snapshot(schema_dir) {
        Ok(_) => Vec::new(),
        Err(error) => vec![error.to_string()],
    }
}

/// Parse SQL content and merge discovered tables/columns into the map.
#[cfg(test)]
fn parse_sql_into(sql: &str, tables: &mut HashMap<String, SchemaTable>) {
    let mut snapshot = SchemaSnapshot {
        tables: std::mem::take(tables),
        retired_tables: HashSet::new(),
    };
    if let Err(error) = replay_sql(Path::new("<test>"), sql, &mut snapshot) {
        panic!("test SQL must replay successfully: {error}");
    }
    *tables = snapshot.tables;
}

#[derive(Debug)]
enum SchemaOperation {
    CreateTable {
        table: String,
        columns: Vec<SchemaColumn>,
        if_not_exists: bool,
        replace: bool,
    },
    DropTable {
        table: String,
        if_exists: bool,
    },
    RenameTable {
        table: String,
        new_table: String,
    },
    AddColumn {
        table: String,
        column: SchemaColumn,
    },
    DropColumn {
        table: String,
        column: String,
    },
    RenameColumn {
        table: String,
        column: String,
        new_column: String,
    },
}

fn replay_sql(path: &Path, sql: &str, snapshot: &mut SchemaSnapshot) -> Result<(), SchemaError> {
    let searchable = mask_sql_comments(sql);
    let candidate_offsets: Vec<usize> = DDL_START_RE
        .find_iter(&searchable)
        .map(|candidate| candidate.start())
        .collect();

    let mut consumed_until = 0;
    for (candidate_index, offset) in candidate_offsets.iter().copied().enumerate() {
        if offset < consumed_until {
            continue;
        }
        let end = find_statement_end(sql, offset, sql.len());
        for nested_offset in candidate_offsets
            .iter()
            .copied()
            .skip(candidate_index + 1)
            .take_while(|nested_offset| *nested_offset < end)
        {
            if scan_state_at(sql, offset, nested_offset) == SqlScanState::Normal {
                return Err(SchemaError::at(
                    SchemaErrorKind::MalformedStatement,
                    path,
                    sql,
                    nested_offset,
                    "supported DDL statement starts before the previous statement terminates",
                ));
            }
        }
        let statement = &sql[offset..end];
        let operation = parse_operation(path, sql, offset, statement)?;
        apply_operation(path, sql, offset, snapshot, operation)?;
        consumed_until = end.saturating_add(1);
    }

    Ok(())
}

fn parse_operation(
    path: &Path,
    sql: &str,
    offset: usize,
    statement: &str,
) -> Result<SchemaOperation, SchemaError> {
    let upper = statement.to_uppercase();

    if upper.starts_with("CREATE VIRTUAL TABLE") {
        let captures = CREATE_VIRTUAL_TABLE_RE.captures(statement).ok_or_else(|| {
            malformed_statement(
                path,
                sql,
                offset,
                statement,
                "malformed CREATE VIRTUAL TABLE",
            )
        })?;
        let table = capture_table_name(path, sql, offset, statement, &captures, "table")?;
        return Ok(SchemaOperation::CreateTable {
            table,
            columns: Vec::new(),
            if_not_exists: captures.name("if_not_exists").is_some(),
            replace: false,
        });
    }

    if upper.starts_with("CREATE") {
        let captures = CREATE_TABLE_RE.captures(statement).ok_or_else(|| {
            malformed_statement(path, sql, offset, statement, "malformed CREATE TABLE")
        })?;
        if captures.name("replace").is_some() && captures.name("if_not_exists").is_some() {
            return Err(malformed_statement(
                path,
                sql,
                offset,
                statement,
                "CREATE TABLE cannot combine OR REPLACE with IF NOT EXISTS",
            ));
        }
        let table = capture_table_name(path, sql, offset, statement, &captures, "table")?;
        let opening_parenthesis = captures
            .get(0)
            .map(|matched| matched.end().saturating_sub(1))
            .ok_or_else(|| {
                malformed_statement(path, sql, offset, statement, "missing CREATE TABLE body")
            })?;
        let body = extract_paren_body(statement, opening_parenthesis + 1).ok_or_else(|| {
            malformed_statement(
                path,
                sql,
                offset,
                statement,
                "CREATE TABLE has unmatched parentheses",
            )
        })?;
        return Ok(SchemaOperation::CreateTable {
            table,
            columns: parse_column_defs(&body),
            if_not_exists: captures.name("if_not_exists").is_some(),
            replace: captures.name("replace").is_some(),
        });
    }

    if upper.starts_with("DROP TABLE") {
        let captures = DROP_TABLE_RE.captures(statement).ok_or_else(|| {
            malformed_statement(path, sql, offset, statement, "malformed DROP TABLE")
        })?;
        let table = capture_table_name(path, sql, offset, statement, &captures, "table")?;
        return Ok(SchemaOperation::DropTable {
            table,
            if_exists: captures.name("if_exists").is_some(),
        });
    }

    if let Some(captures) = ALTER_RENAME_COL_RE.captures(statement) {
        let table = capture_table_name(path, sql, offset, statement, &captures, "table")?;
        let column = capture_identifier(path, sql, offset, statement, &captures, "column")?;
        let new_column = capture_identifier(path, sql, offset, statement, &captures, "new_column")?;
        return Ok(SchemaOperation::RenameColumn {
            table,
            column,
            new_column,
        });
    }

    if let Some(captures) = ALTER_RENAME_TABLE_RE.captures(statement) {
        let table = capture_table_name(path, sql, offset, statement, &captures, "table")?;
        let new_table = capture_table_name(path, sql, offset, statement, &captures, "new_table")?;
        return Ok(SchemaOperation::RenameTable { table, new_table });
    }

    if let Some(captures) = ALTER_ADD_RE.captures(statement) {
        let table = capture_table_name(path, sql, offset, statement, &captures, "table")?;
        let column_name = capture_identifier(path, sql, offset, statement, &captures, "column")?;
        if is_sql_keyword(&column_name) {
            return Err(malformed_statement(
                path,
                sql,
                offset,
                statement,
                "ALTER TABLE ADD does not contain a valid column name",
            ));
        }
        let column_type = captures
            .name("type")
            .map(|matched| matched.as_str().trim().to_uppercase())
            .ok_or_else(|| {
                malformed_statement(
                    path,
                    sql,
                    offset,
                    statement,
                    "ALTER TABLE ADD is missing a column type",
                )
            })?;
        return Ok(SchemaOperation::AddColumn {
            table,
            column: SchemaColumn {
                name: column_name,
                col_type: column_type,
                nullable: !upper.contains("NOT NULL"),
                has_default: upper.contains("DEFAULT"),
                is_primary_key: upper.contains("PRIMARY KEY"),
            },
        });
    }

    if let Some(captures) = ALTER_DROP_COL_RE.captures(statement) {
        let table = capture_table_name(path, sql, offset, statement, &captures, "table")?;
        let column = capture_identifier(path, sql, offset, statement, &captures, "column")?;
        return Ok(SchemaOperation::DropColumn { table, column });
    }

    Err(malformed_statement(
        path,
        sql,
        offset,
        statement,
        "unsupported or malformed ALTER TABLE statement",
    ))
}

fn capture_table_name(
    path: &Path,
    sql: &str,
    offset: usize,
    statement: &str,
    captures: &regex::Captures<'_>,
    name: &str,
) -> Result<String, SchemaError> {
    let raw = captures
        .name(name)
        .map(|matched| matched.as_str())
        .ok_or_else(|| {
            malformed_statement(path, sql, offset, statement, "table identifier is missing")
        })?;
    canonicalize_table_name(raw)
        .map_err(|message| malformed_statement(path, sql, offset, statement, message))
}

fn capture_identifier(
    path: &Path,
    sql: &str,
    offset: usize,
    statement: &str,
    captures: &regex::Captures<'_>,
    name: &str,
) -> Result<String, SchemaError> {
    let raw = captures
        .name(name)
        .map(|matched| matched.as_str())
        .ok_or_else(|| {
            malformed_statement(path, sql, offset, statement, "column identifier is missing")
        })?;
    normalize_identifier_segment(raw)
        .map_err(|message| malformed_statement(path, sql, offset, statement, message))
}

fn malformed_statement(
    path: &Path,
    sql: &str,
    offset: usize,
    statement: &str,
    reason: impl AsRef<str>,
) -> SchemaError {
    let compact = statement.split_whitespace().collect::<Vec<_>>().join(" ");
    let preview: String = compact.chars().take(120).collect();
    SchemaError::at(
        SchemaErrorKind::MalformedStatement,
        path,
        sql,
        offset,
        format!("{}: `{preview}`", reason.as_ref()),
    )
}

fn apply_operation(
    path: &Path,
    sql: &str,
    offset: usize,
    snapshot: &mut SchemaSnapshot,
    operation: SchemaOperation,
) -> Result<(), SchemaError> {
    match operation {
        SchemaOperation::CreateTable {
            table,
            columns,
            if_not_exists,
            replace,
        } => {
            if snapshot.tables.contains_key(&table) && if_not_exists {
                return Ok(());
            }
            if snapshot.tables.contains_key(&table) && !replace {
                return Err(SchemaError::at(
                    SchemaErrorKind::DuplicateTable,
                    path,
                    sql,
                    offset,
                    format!(
                        "CREATE TABLE collides with existing canonical table `{table}`; use IF NOT EXISTS or OR REPLACE explicitly"
                    ),
                ));
            }
            snapshot.retired_tables.remove(&table);
            snapshot.tables.insert(table, SchemaTable { columns });
        }
        SchemaOperation::DropTable { table, if_exists } => {
            if snapshot.tables.remove(&table).is_none() {
                if if_exists {
                    return Ok(());
                }
                return Err(SchemaError::at(
                    SchemaErrorKind::MissingTable,
                    path,
                    sql,
                    offset,
                    format!("DROP TABLE references missing canonical table `{table}`"),
                ));
            }
            snapshot.retired_tables.insert(table);
        }
        SchemaOperation::RenameTable { table, new_table } => {
            if !snapshot.tables.contains_key(&table) {
                return Err(SchemaError::at(
                    SchemaErrorKind::MissingTable,
                    path,
                    sql,
                    offset,
                    format!("ALTER TABLE RENAME references missing canonical table `{table}`"),
                ));
            }
            if table == new_table || snapshot.tables.contains_key(&new_table) {
                return Err(SchemaError::at(
                    SchemaErrorKind::RenameCollision,
                    path,
                    sql,
                    offset,
                    format!(
                        "ALTER TABLE RENAME target `{new_table}` collides with an existing canonical table"
                    ),
                ));
            }
            let Some(schema_table) = snapshot.tables.remove(&table) else {
                return Err(SchemaError::at(
                    SchemaErrorKind::MissingTable,
                    path,
                    sql,
                    offset,
                    format!("ALTER TABLE RENAME references missing canonical table `{table}`"),
                ));
            };
            snapshot.retired_tables.insert(table);
            snapshot.retired_tables.remove(&new_table);
            snapshot.tables.insert(new_table, schema_table);
        }
        SchemaOperation::AddColumn { table, column } => {
            let schema_table = snapshot.tables.get_mut(&table).ok_or_else(|| {
                SchemaError::at(
                    SchemaErrorKind::MissingTable,
                    path,
                    sql,
                    offset,
                    format!("ALTER TABLE ADD references missing canonical table `{table}`"),
                )
            })?;
            if !schema_table
                .columns
                .iter()
                .any(|existing| existing.name == column.name)
            {
                schema_table.columns.push(column);
            }
        }
        SchemaOperation::DropColumn { table, column } => {
            let schema_table = snapshot.tables.get_mut(&table).ok_or_else(|| {
                SchemaError::at(
                    SchemaErrorKind::MissingTable,
                    path,
                    sql,
                    offset,
                    format!("ALTER TABLE DROP references missing canonical table `{table}`"),
                )
            })?;
            let original_len = schema_table.columns.len();
            schema_table
                .columns
                .retain(|existing| existing.name != column);
            if schema_table.columns.len() == original_len {
                return Err(SchemaError::at(
                    SchemaErrorKind::MissingColumn,
                    path,
                    sql,
                    offset,
                    format!("ALTER TABLE DROP references missing column `{table}.{column}`"),
                ));
            }
        }
        SchemaOperation::RenameColumn {
            table,
            column,
            new_column,
        } => {
            let schema_table = snapshot.tables.get_mut(&table).ok_or_else(|| {
                SchemaError::at(
                    SchemaErrorKind::MissingTable,
                    path,
                    sql,
                    offset,
                    format!("ALTER TABLE RENAME references missing canonical table `{table}`"),
                )
            })?;
            if schema_table
                .columns
                .iter()
                .any(|existing| existing.name == new_column)
            {
                return Err(SchemaError::at(
                    SchemaErrorKind::DuplicateColumn,
                    path,
                    sql,
                    offset,
                    format!(
                        "ALTER TABLE RENAME target column `{table}.{new_column}` already exists"
                    ),
                ));
            }
            let column_to_rename = schema_table
                .columns
                .iter_mut()
                .find(|existing| existing.name == column)
                .ok_or_else(|| {
                    SchemaError::at(
                        SchemaErrorKind::MissingColumn,
                        path,
                        sql,
                        offset,
                        format!("ALTER TABLE RENAME references missing column `{table}.{column}`"),
                    )
                })?;
            column_to_rename.name = new_column;
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SqlScanState {
    Normal,
    SingleQuote,
    DoubleQuote,
    Backtick,
    Bracket,
    LineComment,
    BlockComment,
}

fn mask_sql_comments(sql: &str) -> String {
    let original = sql.as_bytes();
    let mut masked = original.to_vec();
    let mut state = SqlScanState::Normal;
    let mut index = 0;

    while index < original.len() {
        let current = original[index];
        let next = original.get(index + 1).copied();
        match state {
            SqlScanState::Normal if current == b'-' && next == Some(b'-') => {
                masked[index] = b' ';
                masked[index + 1] = b' ';
                index += 2;
                state = SqlScanState::LineComment;
                continue;
            }
            SqlScanState::Normal if current == b'/' && next == Some(b'*') => {
                masked[index] = b' ';
                masked[index + 1] = b' ';
                index += 2;
                state = SqlScanState::BlockComment;
                continue;
            }
            SqlScanState::Normal if current == b'\'' => state = SqlScanState::SingleQuote,
            SqlScanState::Normal if current == b'"' => state = SqlScanState::DoubleQuote,
            SqlScanState::Normal if current == b'`' => state = SqlScanState::Backtick,
            SqlScanState::Normal if current == b'[' => state = SqlScanState::Bracket,
            SqlScanState::SingleQuote if current == b'\'' => {
                if next == Some(b'\'') {
                    index += 2;
                    continue;
                }
                state = SqlScanState::Normal;
            }
            SqlScanState::DoubleQuote if current == b'"' => {
                if next == Some(b'"') {
                    index += 2;
                    continue;
                }
                state = SqlScanState::Normal;
            }
            SqlScanState::Backtick if current == b'`' => {
                if next == Some(b'`') {
                    index += 2;
                    continue;
                }
                state = SqlScanState::Normal;
            }
            SqlScanState::Bracket if current == b']' => {
                if next == Some(b']') {
                    index += 2;
                    continue;
                }
                state = SqlScanState::Normal;
            }
            SqlScanState::LineComment => {
                if current == b'\n' {
                    state = SqlScanState::Normal;
                } else {
                    masked[index] = b' ';
                }
            }
            SqlScanState::BlockComment if current == b'*' && next == Some(b'/') => {
                masked[index] = b' ';
                masked[index + 1] = b' ';
                index += 2;
                state = SqlScanState::Normal;
                continue;
            }
            SqlScanState::BlockComment => masked[index] = b' ',
            _ => {}
        }
        index += 1;
    }

    String::from_utf8(masked).unwrap_or_else(|_| sql.to_string())
}

fn find_statement_end(sql: &str, start: usize, limit: usize) -> usize {
    let bytes = sql.as_bytes();
    let mut state = SqlScanState::Normal;
    let mut index = start;
    let bounded_limit = limit.min(bytes.len());

    while index < bounded_limit {
        let current = bytes[index];
        let next = bytes.get(index + 1).copied();
        match state {
            SqlScanState::Normal if current == b';' => return index,
            SqlScanState::Normal if current == b'-' && next == Some(b'-') => {
                state = SqlScanState::LineComment;
                index += 2;
                continue;
            }
            SqlScanState::Normal if current == b'/' && next == Some(b'*') => {
                state = SqlScanState::BlockComment;
                index += 2;
                continue;
            }
            SqlScanState::Normal if current == b'\'' => state = SqlScanState::SingleQuote,
            SqlScanState::Normal if current == b'"' => state = SqlScanState::DoubleQuote,
            SqlScanState::Normal if current == b'`' => state = SqlScanState::Backtick,
            SqlScanState::Normal if current == b'[' => state = SqlScanState::Bracket,
            SqlScanState::SingleQuote if current == b'\'' => {
                if next == Some(b'\'') {
                    index += 2;
                    continue;
                }
                state = SqlScanState::Normal;
            }
            SqlScanState::DoubleQuote if current == b'"' => {
                if next == Some(b'"') {
                    index += 2;
                    continue;
                }
                state = SqlScanState::Normal;
            }
            SqlScanState::Backtick if current == b'`' => {
                if next == Some(b'`') {
                    index += 2;
                    continue;
                }
                state = SqlScanState::Normal;
            }
            SqlScanState::Bracket if current == b']' => {
                if next == Some(b']') {
                    index += 2;
                    continue;
                }
                state = SqlScanState::Normal;
            }
            SqlScanState::LineComment if current == b'\n' => state = SqlScanState::Normal,
            SqlScanState::BlockComment if current == b'*' && next == Some(b'/') => {
                state = SqlScanState::Normal;
                index += 2;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    bounded_limit
}

fn scan_state_at(sql: &str, start: usize, target: usize) -> SqlScanState {
    let bytes = sql.as_bytes();
    let mut state = SqlScanState::Normal;
    let mut index = start;
    let bounded_target = target.min(bytes.len());

    while index < bounded_target {
        let current = bytes[index];
        let next = bytes.get(index + 1).copied();
        match state {
            SqlScanState::Normal if current == b'-' && next == Some(b'-') => {
                state = SqlScanState::LineComment;
                index += 2;
                continue;
            }
            SqlScanState::Normal if current == b'/' && next == Some(b'*') => {
                state = SqlScanState::BlockComment;
                index += 2;
                continue;
            }
            SqlScanState::Normal if current == b'\'' => state = SqlScanState::SingleQuote,
            SqlScanState::Normal if current == b'"' => state = SqlScanState::DoubleQuote,
            SqlScanState::Normal if current == b'`' => state = SqlScanState::Backtick,
            SqlScanState::Normal if current == b'[' => state = SqlScanState::Bracket,
            SqlScanState::SingleQuote if current == b'\'' => {
                if next == Some(b'\'') {
                    index += 2;
                    continue;
                }
                state = SqlScanState::Normal;
            }
            SqlScanState::DoubleQuote if current == b'"' => {
                if next == Some(b'"') {
                    index += 2;
                    continue;
                }
                state = SqlScanState::Normal;
            }
            SqlScanState::Backtick if current == b'`' => {
                if next == Some(b'`') {
                    index += 2;
                    continue;
                }
                state = SqlScanState::Normal;
            }
            SqlScanState::Bracket if current == b']' => {
                if next == Some(b']') {
                    index += 2;
                    continue;
                }
                state = SqlScanState::Normal;
            }
            SqlScanState::LineComment if current == b'\n' => state = SqlScanState::Normal,
            SqlScanState::BlockComment if current == b'*' && next == Some(b'/') => {
                state = SqlScanState::Normal;
                index += 2;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    state
}

/// Extract text between the opening paren (at `start`) and its matching close.
fn extract_paren_body(sql: &str, start: usize) -> Option<String> {
    let bytes = sql.as_bytes();
    let mut depth = 1;
    let mut i = start;
    let mut state = SqlScanState::Normal;

    while i < bytes.len() && depth > 0 {
        let current = bytes[i];
        let next = bytes.get(i + 1).copied();
        match state {
            SqlScanState::Normal if current == b'(' => depth += 1,
            SqlScanState::Normal if current == b')' => depth -= 1,
            SqlScanState::Normal if current == b'-' && next == Some(b'-') => {
                state = SqlScanState::LineComment;
                i += 2;
                continue;
            }
            SqlScanState::Normal if current == b'/' && next == Some(b'*') => {
                state = SqlScanState::BlockComment;
                i += 2;
                continue;
            }
            SqlScanState::Normal if current == b'\'' => state = SqlScanState::SingleQuote,
            SqlScanState::Normal if current == b'"' => state = SqlScanState::DoubleQuote,
            SqlScanState::Normal if current == b'`' => state = SqlScanState::Backtick,
            SqlScanState::Normal if current == b'[' => state = SqlScanState::Bracket,
            SqlScanState::SingleQuote if current == b'\'' => {
                if next == Some(b'\'') {
                    i += 2;
                    continue;
                }
                state = SqlScanState::Normal;
            }
            SqlScanState::DoubleQuote if current == b'"' => {
                if next == Some(b'"') {
                    i += 2;
                    continue;
                }
                state = SqlScanState::Normal;
            }
            SqlScanState::Backtick if current == b'`' => {
                if next == Some(b'`') {
                    i += 2;
                    continue;
                }
                state = SqlScanState::Normal;
            }
            SqlScanState::Bracket if current == b']' => {
                if next == Some(b']') {
                    i += 2;
                    continue;
                }
                state = SqlScanState::Normal;
            }
            SqlScanState::LineComment if current == b'\n' => state = SqlScanState::Normal,
            SqlScanState::BlockComment if current == b'*' && next == Some(b'/') => {
                state = SqlScanState::Normal;
                i += 2;
                continue;
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
        // Virtual tables use USING syntax, so their columns are not parsed.
        // The snapshot still records table existence instead of relying on a
        // separate CREATE-pattern scan that could diverge from replay.
        let sql = "CREATE VIRTUAL TABLE search_idx USING fts5(content, sender);";
        let mut tables = HashMap::new();
        parse_sql_into(sql, &mut tables);
        let table = tables
            .get("search_idx")
            .expect("virtual table existence must be represented");
        assert!(table.columns.is_empty());
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
    fn test_schema_read_errors_flags_only_unreadable() {
        use std::io::Write;
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();

        // A configured but absent directory must fail loud.
        let missing = schema_read_errors(Path::new("/nonexistent/path"));
        assert_eq!(missing.len(), 1);
        assert!(missing[0].contains("schema directory could not be read"));

        // A readable migration → no error.
        fs::write(dir.join("001_ok.sql"), "CREATE TABLE t (id INTEGER);").unwrap();
        assert!(schema_read_errors(dir).is_empty());

        // A non-source file present alongside → ignored (not a SQL_EXTENSIONS file).
        fs::write(dir.join("README.md"), "notes").unwrap();
        assert!(schema_read_errors(dir).is_empty());

        // A non-UTF-8 migration → exactly one error naming the file.
        let mut f = std::fs::File::create(dir.join("002_bad.sql")).unwrap();
        f.write_all(b"CREATE TABLE u (id INTEGER);\n\xff\xfe")
            .unwrap();
        let errs = schema_read_errors(dir);
        assert_eq!(errs.len(), 1, "only the unreadable file should be flagged");
        assert!(errs[0].contains("002_bad.sql"));
    }

    #[test]
    fn test_schema_read_errors_flags_unenumerable_dir() {
        // A `schema_dir` that exists but is a file rather than a directory makes
        // `read_dir` return `Err`. That must fail loud (hard error), not fail open
        // to an empty schema that would silently skip db_tables/column checks.
        let tmp = tempfile::tempdir().unwrap();
        let not_a_dir = tmp.path().join("schema.sql");
        fs::write(&not_a_dir, "CREATE TABLE t (id INTEGER);").unwrap();

        let errs = schema_read_errors(&not_a_dir);
        assert_eq!(errs.len(), 1, "an unenumerable schema dir must be flagged");
        assert!(errs[0].contains("could not be read"));
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
    fn test_build_schema_snapshot_missing_directory_is_error() {
        let error = build_schema_snapshot(Path::new("/nonexistent/path"))
            .expect_err("a configured missing directory must not become an empty snapshot");
        assert_eq!(error.kind, SchemaErrorKind::MissingDirectory);
        assert_eq!(error.line, 0);
        assert!(error.message.contains("schema directory could not be read"));
    }

    #[test]
    fn test_build_schema_snapshot_reports_malformed_sql_with_path_and_position() {
        let tmp = tempfile::tempdir().unwrap();
        let migration = tmp.path().join("002_broken.sql");
        fs::write(
            &migration,
            "\n-- a preceding line\nCREATE TABLE broken (id INTEGER;\n",
        )
        .unwrap();

        let error = build_schema_snapshot(tmp.path())
            .expect_err("malformed supported DDL must fail the snapshot");
        assert_eq!(error.kind, SchemaErrorKind::MalformedStatement);
        assert_eq!(error.path, migration);
        assert_eq!(error.line, 3);
        assert_eq!(error.column, 1);
        assert!(error.message.contains("unmatched parentheses"));
        assert!(error.message.contains("CREATE TABLE broken"));
    }

    #[test]
    fn test_replay_rejects_unterminated_statement_before_later_ddl() {
        let sql = r#"
CREATE TABLE first_table (id INTEGER PRIMARY KEY)
CREATE TABLE second_table (id INTEGER PRIMARY KEY);
"#;
        let mut snapshot = SchemaSnapshot::default();
        let error = replay_sql(Path::new("001_broken.sql"), sql, &mut snapshot)
            .expect_err("a missing statement terminator must not hide later DDL");

        assert_eq!(error.kind, SchemaErrorKind::MalformedStatement);
        assert_eq!(error.line, 3);
        assert!(
            error
                .message
                .contains("before the previous statement terminates")
        );
        assert!(snapshot.tables.is_empty());
    }

    #[test]
    fn test_replay_ignores_commented_and_string_literal_ddl() {
        let sql = r#"
-- DROP TABLE notes;
/* ALTER TABLE notes RENAME TO hidden_notes; */
CREATE TABLE notes (
    id INTEGER PRIMARY KEY,
    message TEXT DEFAULT 'DROP TABLE notes;'
);
"#;
        let mut snapshot = SchemaSnapshot::default();
        replay_sql(Path::new("001_notes.sql"), sql, &mut snapshot).unwrap();

        let notes = snapshot.tables.get("notes").expect("notes must remain");
        assert_eq!(notes.column_names(), vec!["id", "message"]);
        assert!(snapshot.retired_tables.is_empty());
    }

    #[test]
    fn test_replay_respects_drop_then_recreate_within_one_file() {
        let sql = r#"
CREATE TABLE users (id INTEGER PRIMARY KEY);
DROP TABLE users;
CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT NOT NULL);
"#;
        let mut tables = HashMap::new();
        parse_sql_into(sql, &mut tables);

        let users = tables
            .get("users")
            .expect("the later CREATE must recreate the dropped table");
        assert_eq!(users.column_names(), vec!["id", "email"]);
    }

    #[test]
    fn test_replay_respects_recreate_after_rename_within_one_file() {
        let sql = r#"
CREATE TABLE users (id INTEGER PRIMARY KEY);
ALTER TABLE users RENAME TO archived_users;
CREATE TABLE users (id INTEGER PRIMARY KEY, replacement TEXT);
"#;
        let mut tables = HashMap::new();
        parse_sql_into(sql, &mut tables);

        assert!(
            tables.contains_key("archived_users"),
            "the renamed table must remain in the final snapshot"
        );
        let users = tables
            .get("users")
            .expect("the later CREATE must create a new users table");
        assert_eq!(users.column_names(), vec!["id", "replacement"]);
    }

    #[test]
    fn test_replay_recreate_clears_retired_identity() {
        let sql = r#"
CREATE TABLE users (id INTEGER PRIMARY KEY);
DROP TABLE users;
CREATE TABLE users (id INTEGER PRIMARY KEY, replacement TEXT);
"#;
        let mut snapshot = SchemaSnapshot::default();
        replay_sql(Path::new("001_users.sql"), sql, &mut snapshot).unwrap();

        assert!(snapshot.tables.contains_key("users"));
        assert!(!snapshot.retired_tables.contains("users"));
    }

    #[test]
    fn test_replay_duplicate_canonical_create_is_a_non_mutating_error() {
        let sql = r#"
CREATE TABLE "Users" (id INTEGER PRIMARY KEY);
CREATE TABLE users (replacement TEXT);
"#;
        let mut snapshot = SchemaSnapshot::default();
        let error = replay_sql(Path::new("001_users.sql"), sql, &mut snapshot)
            .expect_err("canonical duplicate CREATE must not overwrite the first table");

        assert_eq!(error.kind, SchemaErrorKind::DuplicateTable);
        assert_eq!(error.line, 3);
        assert!(error.message.contains("canonical table `users`"));
        let users = snapshot.tables.get("users").unwrap();
        assert_eq!(users.column_names(), vec!["id"]);
    }

    #[test]
    fn test_replay_if_not_exists_preserves_the_existing_table() {
        let sql = r#"
CREATE TABLE users (id INTEGER PRIMARY KEY);
CREATE TABLE IF NOT EXISTS "Users" (replacement TEXT);
"#;
        let mut snapshot = SchemaSnapshot::default();
        replay_sql(Path::new("001_users.sql"), sql, &mut snapshot).unwrap();

        let users = snapshot.tables.get("users").unwrap();
        assert_eq!(users.column_names(), vec!["id"]);
    }

    #[test]
    fn test_replay_rename_collision_is_a_non_mutating_error() {
        let sql = r#"
CREATE TABLE source_table (id INTEGER PRIMARY KEY);
CREATE TABLE target_table (id INTEGER PRIMARY KEY);
ALTER TABLE source_table RENAME TO target_table;
"#;
        let mut snapshot = SchemaSnapshot::default();
        let error = replay_sql(Path::new("001_tables.sql"), sql, &mut snapshot)
            .expect_err("rename target collisions must not discard either table");

        assert_eq!(error.kind, SchemaErrorKind::RenameCollision);
        assert_eq!(error.line, 4);
        assert!(snapshot.tables.contains_key("source_table"));
        assert!(snapshot.tables.contains_key("target_table"));
        assert!(snapshot.retired_tables.is_empty());
    }

    #[test]
    fn test_replay_missing_drop_and_rename_sources_fail_truthfully() {
        let mut snapshot = SchemaSnapshot::default();
        let drop_error = replay_sql(
            Path::new("001_drop.sql"),
            "DROP TABLE missing;",
            &mut snapshot,
        )
        .expect_err("plain DROP of a missing table must fail");
        assert_eq!(drop_error.kind, SchemaErrorKind::MissingTable);
        assert!(drop_error.message.contains("DROP TABLE"));

        replay_sql(
            Path::new("002_drop.sql"),
            "DROP TABLE IF EXISTS missing;",
            &mut snapshot,
        )
        .unwrap();

        let rename_error = replay_sql(
            Path::new("003_rename.sql"),
            "ALTER TABLE missing RENAME TO replacement;",
            &mut snapshot,
        )
        .expect_err("rename of a missing table must fail");
        assert_eq!(rename_error.kind, SchemaErrorKind::MissingTable);
        assert!(rename_error.message.contains("RENAME"));
    }

    #[test]
    fn test_drop_table() {
        let sql = r#"
CREATE TABLE temp_data (id INTEGER PRIMARY KEY, value TEXT);
CREATE TABLE keep_me (id INTEGER PRIMARY KEY);
DROP TABLE temp_data;
"#;
        let mut tables = HashMap::new();
        parse_sql_into(sql, &mut tables);
        assert!(!tables.contains_key("temp_data"));
        assert!(tables.contains_key("keep_me"));
    }

    #[test]
    fn test_drop_table_if_exists() {
        let sql = r#"
CREATE TABLE things (id INTEGER PRIMARY KEY, name TEXT);
DROP TABLE IF EXISTS things;
"#;
        let mut tables = HashMap::new();
        parse_sql_into(sql, &mut tables);
        assert!(!tables.contains_key("things"));
    }

    #[test]
    fn test_drop_column() {
        let sql = r#"
CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, legacy TEXT);
ALTER TABLE users DROP COLUMN legacy;
"#;
        let mut tables = HashMap::new();
        parse_sql_into(sql, &mut tables);
        let t = tables.get("users").unwrap();
        assert_eq!(t.columns.len(), 2);
        assert_eq!(t.column_names(), vec!["id", "name"]);
    }

    #[test]
    fn test_rename_table() {
        let sql = r#"
CREATE TABLE old_name (id INTEGER PRIMARY KEY, data TEXT);
ALTER TABLE old_name RENAME TO new_name;
"#;
        let mut tables = HashMap::new();
        parse_sql_into(sql, &mut tables);
        assert!(!tables.contains_key("old_name"));
        let t = tables.get("new_name").unwrap();
        assert_eq!(t.columns.len(), 2);
    }

    #[test]
    fn test_rename_column() {
        let sql = r#"
CREATE TABLE items (id INTEGER PRIMARY KEY, old_col TEXT NOT NULL);
ALTER TABLE items RENAME COLUMN old_col TO new_col;
"#;
        let mut tables = HashMap::new();
        parse_sql_into(sql, &mut tables);
        let t = tables.get("items").unwrap();
        assert_eq!(t.columns.len(), 2);
        assert_eq!(t.columns[1].name, "new_col");
        assert_eq!(t.columns[1].col_type, "TEXT");
    }

    #[test]
    fn test_quoted_and_schema_qualified_table_names() {
        let sql = r#"
CREATE TABLE "quoted" (id INTEGER PRIMARY KEY);
CREATE TABLE `backtick` (id INTEGER PRIMARY KEY);
CREATE TABLE public.schema_table (id INTEGER PRIMARY KEY);
CREATE TABLE MixedCase (id INTEGER PRIMARY KEY);
"#;
        let mut tables = HashMap::new();
        parse_sql_into(sql, &mut tables);
        assert!(tables.contains_key("quoted"), "tables: {:?}", tables.keys());
        assert!(tables.contains_key("backtick"));
        assert!(tables.contains_key("public.schema_table"));
        // Comparison is case-insensitive: stored normalized.
        assert!(tables.contains_key("mixedcase"));
        assert!(!tables.contains_key("MixedCase"));
    }

    #[test]
    fn test_normalize_table_name() {
        assert_eq!(normalize_table_name("users"), "users");
        assert_eq!(normalize_table_name("\"Users\""), "users");
        assert_eq!(normalize_table_name("`Users`"), "users");
        assert_eq!(normalize_table_name("public.\"Users\""), "public.users");
        assert_eq!(normalize_table_name("PUBLIC.USERS"), "public.users");
        assert_eq!(
            canonicalize_table_name(" public . \"User.Events\" ").unwrap(),
            "public.\"user.events\""
        );
        assert_eq!(
            canonicalize_table_name("[Sales].[Order]]History]").unwrap(),
            "sales.\"order]history\""
        );
        assert_eq!(
            canonical_table_leaf("public.\"User.Events\"").unwrap(),
            "\"user.events\""
        );
        assert!(canonicalize_table_name("users\"").is_err());
        assert!(canonicalize_table_name("public..users").is_err());
    }

    #[test]
    fn test_quoted_qualified_names_replay_through_rename_and_drop() {
        let sql = r#"
CREATE TABLE public."Users" (id INTEGER PRIMARY KEY);
ALTER TABLE public."Users" RENAME TO archive.`Users`;
DROP TABLE archive.[Users];
"#;
        let mut snapshot = SchemaSnapshot::default();
        replay_sql(Path::new("001_qualified.sql"), sql, &mut snapshot).unwrap();

        assert!(snapshot.tables.is_empty());
        assert!(snapshot.retired_tables.contains("public.users"));
        assert!(snapshot.retired_tables.contains("archive.users"));
    }

    #[test]
    fn test_rename_and_drop_apply_to_quoted_names() {
        let sql = r#"
CREATE TABLE "old_name" (id INTEGER PRIMARY KEY);
ALTER TABLE "old_name" RENAME TO "new_name";
CREATE TABLE `doomed` (id INTEGER PRIMARY KEY);
DROP TABLE `doomed`;
"#;
        let mut tables = HashMap::new();
        parse_sql_into(sql, &mut tables);
        assert!(!tables.contains_key("old_name"));
        assert!(tables.contains_key("new_name"));
        assert!(!tables.contains_key("doomed"));
    }

    #[test]
    fn test_sql_extensions_list() {
        // Verify key languages are in the supported list
        assert!(SQL_EXTENSIONS.contains(&"sql"));
        assert!(SQL_EXTENSIONS.contains(&"ts"));
        assert!(SQL_EXTENSIONS.contains(&"swift"));
        assert!(SQL_EXTENSIONS.contains(&"kt"));
        assert!(SQL_EXTENSIONS.contains(&"java"));
        assert!(SQL_EXTENSIONS.contains(&"py"));
        assert!(SQL_EXTENSIONS.contains(&"rb"));
        assert!(SQL_EXTENSIONS.contains(&"go"));
        assert!(SQL_EXTENSIONS.contains(&"rs"));
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
