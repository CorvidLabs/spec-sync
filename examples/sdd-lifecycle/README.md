# Executable SDD lifecycle

This example creates an isolated Git repository and drives the packaged
`specsync` binary through:

```text
draft → approved → implementing → verifying → accepted
```

Run it from the SpecSync repository:

```bash
SPECSYNC_BIN="$PWD/target/release/specsync" ./examples/sdd-lifecycle/run.sh
```

The example deliberately leaves the accepted workspace active. Archival belongs
after the delivery diff is merged and no longer relies on the workspace for path
coverage.

