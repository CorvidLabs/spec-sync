use super::super::{validate_spec, validate_spec_content};
use crate::schema::SchemaTable;
use crate::types::SpecSyncConfig;
use std::collections::{HashMap, HashSet};
use std::fs;

#[test]
fn test_validate_spec_schema_columns() {
    let tmp = tempfile::tempdir().unwrap();
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("msg.ts"), "export function send() {}").unwrap();

    let spec = tmp.path().join("msg.spec.md");
    fs::write(
        &spec,
        r#"---
module: msg
version: 1
status: active
files:
  - src/msg.ts
db_tables:
  - messages
---

## Purpose
Messaging

## Requirements

### Schema: messages

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | INTEGER | PRIMARY KEY |
| `content` | TEXT | NOT NULL |
| `ghost_col` | TEXT | NOT NULL |

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `send` | msg: string | void | Sends |

## Invariants
## Behavioral Examples
## Error Cases
## Dependencies
## Change Log
"#,
    )
    .unwrap();

    let mut table_names = HashSet::new();
    table_names.insert("messages".to_string());

    let mut schema_cols = HashMap::new();
    schema_cols.insert(
        "messages".to_string(),
        SchemaTable {
            columns: vec![
                crate::schema::SchemaColumn {
                    name: "id".to_string(),
                    col_type: "INTEGER".to_string(),
                    nullable: false,
                    has_default: false,
                    is_primary_key: true,
                },
                crate::schema::SchemaColumn {
                    name: "content".to_string(),
                    col_type: "TEXT".to_string(),
                    nullable: false,
                    has_default: false,
                    is_primary_key: false,
                },
                crate::schema::SchemaColumn {
                    name: "created_at".to_string(),
                    col_type: "TEXT".to_string(),
                    nullable: false,
                    has_default: true,
                    is_primary_key: false,
                },
            ],
        },
    );

    let config = SpecSyncConfig::default();
    let result = validate_spec(&spec, tmp.path(), &table_names, &schema_cols, &config);

    // ghost_col is in spec but not in schema → ERROR
    assert!(result.errors.iter().any(|e| e.contains("ghost_col")));
    // created_at is in schema but not in spec → WARNING
    assert!(result.warnings.iter().any(|w| w.contains("created_at")));
}

#[test]
fn test_validate_spec_schema_type_mismatch() {
    let tmp = tempfile::tempdir().unwrap();
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("t.ts"), "export function f() {}").unwrap();

    let spec = tmp.path().join("t.spec.md");
    fs::write(
        &spec,
        r#"---
module: t
version: 1
status: active
files:
  - src/t.ts
db_tables:
  - items
---

## Purpose
Test

## Requirements

### Schema: items

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | INTEGER | PRIMARY KEY |
| `price` | TEXT | |

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `f` | | void | Does stuff |

## Invariants
## Behavioral Examples
## Error Cases
## Dependencies
## Change Log
"#,
    )
    .unwrap();

    let mut table_names = HashSet::new();
    table_names.insert("items".to_string());

    let mut schema_cols = HashMap::new();
    schema_cols.insert(
        "items".to_string(),
        SchemaTable {
            columns: vec![
                crate::schema::SchemaColumn {
                    name: "id".to_string(),
                    col_type: "INTEGER".to_string(),
                    nullable: false,
                    has_default: false,
                    is_primary_key: true,
                },
                crate::schema::SchemaColumn {
                    name: "price".to_string(),
                    col_type: "REAL".to_string(),
                    nullable: true,
                    has_default: false,
                    is_primary_key: false,
                },
            ],
        },
    );

    let config = SpecSyncConfig::default();
    let result = validate_spec(&spec, tmp.path(), &table_names, &schema_cols, &config);

    // price type mismatch: spec says TEXT, schema says REAL → WARNING
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.contains("type mismatch") && w.contains("price"))
    );
}

/// #684: a project whose schema lives in application code cannot configure `schema_dir` —
/// there are no `.sql` migrations to point it at, and `schema_dir` is not a spec frontmatter
/// field. Carrying "validation skipped" as a WARNING escalated it to an error under
/// `--strict`, so the only ways to clear it were deleting the `db_tables` documentation or
/// abandoning `strict`. Gating on advice the reader cannot take is not drift detection.
///
/// DISCRIMINATOR. Before the fix this message was pushed to `warnings`.
#[test]
fn db_tables_without_schema_dir_is_a_notice_not_a_warning() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("specs/tracker")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/tracker.rs"), "pub fn track() {}\n").unwrap();
    let spec_path = root.join("specs/tracker/tracker.spec.md");
    let content = "---\nmodule: tracker\nversion: 1\nstatus: draft\nfiles:\n  - src/tracker.rs\ndb_tables:\n  - events\n---\n";
    fs::write(&spec_path, content).unwrap();

    let config = SpecSyncConfig::default();
    assert!(config.schema_dir.is_none(), "premise: no schema_dir");
    let result = validate_spec_content(
        &spec_path,
        content,
        root,
        &HashSet::new(),
        &HashMap::new(),
        &config,
    );

    let skipped = |v: &Vec<String>| v.iter().any(|m| m.contains("DB table validation skipped"));
    assert!(
        skipped(&result.notices),
        "the disclosure must survive as a notice: {:?}",
        result.notices
    );
    assert!(
        !skipped(&result.warnings),
        "it must not be a warning — warnings escalate under --strict: {:?}",
        result.warnings
    );
}

/// VACUITY CONTROL. Behaviour only: when `schema_dir` IS configured, a declared table the
/// schema does not contain must still be an ERROR. A fix that simply stopped checking
/// `db_tables` would pass the discriminator above and fail this.
#[test]
fn a_missing_db_table_is_still_an_error_when_schema_dir_is_configured() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("specs/tracker")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/tracker.rs"), "pub fn track() {}\n").unwrap();
    let spec_path = root.join("specs/tracker/tracker.spec.md");
    let content = "---\nmodule: tracker\nversion: 1\nstatus: draft\nfiles:\n  - src/tracker.rs\ndb_tables:\n  - events\n---\n";
    fs::write(&spec_path, content).unwrap();

    // The schema must be READABLE for "missing" to be a legitimate verdict — an unreadable
    // schema correctly degrades to "unknown" (#672), so a fixture without a real schema
    // directory would prove nothing here.
    fs::create_dir_all(root.join("schema")).unwrap();
    fs::write(root.join("schema/001.sql"), "CREATE TABLE other (id TEXT);").unwrap();
    let mut config = SpecSyncConfig::default();
    config.schema_dir = Some("schema".to_string());
    let mut tables = HashSet::new();
    tables.insert("other".to_string());

    let result =
        validate_spec_content(&spec_path, content, root, &tables, &HashMap::new(), &config);
    assert!(
        result
            .errors
            .iter()
            .any(|m| m.contains("DB table not found in schema: events")),
        "a genuinely absent table must remain an error: {:?}",
        result.errors
    );
}
