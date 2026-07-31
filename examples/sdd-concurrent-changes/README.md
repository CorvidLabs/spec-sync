# Executable ordered changes

This example proves that a dependent change cannot complete scoped verification
until its prerequisite is accepted or archived, then finalizes both changes in
dependency order using the SpecSync 6.0 workflow (`check` → independent
`review` → `finalize`).

```bash
SPECSYNC_BIN="$PWD/target/release/specsync" ./examples/sdd-concurrent-changes/run.sh
```

The same ordering is used when SpecSync builds the effective contract from
multiple active semantic deltas.
