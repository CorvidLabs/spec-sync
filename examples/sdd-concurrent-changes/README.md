# Executable ordered changes

This example proves that a dependent change cannot complete scoped verification
until its prerequisite is accepted or archived, then finalizes both changes in
dependency order. Each successful `change check --commit` records the
approval ledger in Git before scoped review and finalization.

```bash
SPECSYNC_BIN="$PWD/target/release/specsync" ./examples/sdd-concurrent-changes/run.sh
```

The same ordering is used when SpecSync builds the effective contract from
multiple active semantic deltas.
