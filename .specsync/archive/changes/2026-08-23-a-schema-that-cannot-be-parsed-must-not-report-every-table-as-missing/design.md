---
change: a-schema-that-cannot-be-parsed-must-not-report-every-table-as-missing
artifact: design
---

# Design

## Unknown is not absent

`get_schema_table_names` keeps returning `HashSet<String>` — changing it would ripple through four
public signatures and their callers in `mcp.rs`, `change.rs`, `commands/issues.rs`. Instead a
sibling answers the question the empty set cannot:

    pub fn schema_table_names_available(root: &Path, config: &SpecSyncConfig) -> bool

The validator consults it ONLY on the error path, lazily, so a healthy repository never re-replays
the schema:

    Ok(false) => {
        let known = *schema_tables_known
            .get_or_insert_with(|| schema_table_names_available(root, config));
        if known { add_missing_db_table_error(table, &mut result); }
    }

`root` and `config` are already in scope in `validate_spec_content_internal`, so no signature moves
and the ~50-path ripple is avoided entirely.

## Agreeing redeclaration is a no-op

`replay_sql`'s `AddColumn` arm changes `.any(...)` to `.find(...)` so the existing column is in
hand, then compares `col_type`. Equal types return `Ok(())`; unequal types still `Err`, with both
types named.

## Alternatives rejected

**Dialect awareness.** There is no dialect setting in the config, so "accept for SQLite only" is
not expressible without inventing one.

**Downgrading the duplicate to a warning.** Warnings gate under `strict`, which is precisely the
mode the reporter needs, so it would not have unblocked them.

**Making the parse failure suppressible via an ignore category.** Considered and rejected: today a
failed replay is fail-CLOSED. Making it suppressible would convert a hard error into something a
project can silence, which is a fail-open hole that does not exist now.
