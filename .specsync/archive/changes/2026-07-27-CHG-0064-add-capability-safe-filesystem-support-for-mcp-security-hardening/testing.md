---
change: CHG-0064-add-capability-safe-filesystem-support-for-mcp-security-hardening
artifact: testing
---

# Testing

- `fledge run fmt`
- `fledge run lint`
- `fledge run check-types`
- `cargo +1.89.0 check --locked`
- `cargo test mcp:: -- --nocapture`
- `cargo check --locked --target x86_64-pc-windows-gnu`
- `fledge lanes run verify`
- `fledge lanes run repo`
- `fledge trust verify --range origin/main..HEAD`

The dependency change passes only if Cargo resolves one runtime `tempfile` declaration, the lockfile
is reproducible, all capability-snapshot and confined-write regressions pass, and the Windows target
compiles. The final repository and trust lanes run after CHG-0062 is reaccepted and CHG-0063's
successor binding is refreshed; a lifecycle-ordering failure is not dependency verification evidence.
