# Executable ordered changes

This example proves that a dependent change cannot start until its prerequisite
is accepted, then completes both changes in dependency order.

```bash
SPECSYNC_BIN="$PWD/target/release/specsync" ./examples/sdd-concurrent-changes/run.sh
```

The same ordering is used when SpecSync builds the effective contract from
multiple active semantic deltas.

