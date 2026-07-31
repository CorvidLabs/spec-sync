# Executable SDD lifecycle

This example creates an isolated Git repository and drives the packaged
`specsync` binary through the SpecSync 6.0 single workflow:

```text
draft → approved → implement → check → scoped review → finalize → archived
```

Run it from the SpecSync repository:

```bash
SPECSYNC_BIN="$PWD/target/release/specsync" ./examples/sdd-lifecycle/run.sh
```

`change check` verifies **this change only**. `change audit` reports project
health over active workspaces and living specs. `change finalize` creates the
dated archive on the same branch (same-PR finalization); GitHub still owns the
merge when you use this workflow in a real repository.
