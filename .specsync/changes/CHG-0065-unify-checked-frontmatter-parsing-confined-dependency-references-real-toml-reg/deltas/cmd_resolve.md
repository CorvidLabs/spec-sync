## ADDED

### REQUIREMENT REQ-cmd-resolve-002

`specsync resolve` SHALL return one explicit report and trustworthy exit status for local and
remote dependency verification.

Acceptance Criteria

- Local missing, malformed, unsafe, registry-load, and module-identity failures are findings and
  exit 1 with or without `--strict`.
- `--remote` treats registry fetch failure, absent registry, missing remote module, malformed
  registry, and unsafe registry mapping as inconclusive or failing findings and exits 1.
- `--verify` treats remote spec fetch or parse failure and breaking drift as findings and exits 1;
  warning-only compatibility findings exit 0 by default and exit 1 under `--strict`.
- The command prints `verified — no drift detected` only when at least one remote registry was
  fetched and every requested remote reference and spec content required by the mode was verified.
- `--verify` implies `--remote`; no network request occurs without either flag.
- Text output uses one `Local dependencies` heading and never drops malformed references.
- JSON is honored and contains `valid`, `gate_passed`, mode, checked counts, cache status, and every
  finding with originating spec, raw reference, normalized identity when available, category,
  severity, and message; no ANSI or human preamble contaminates JSON.
- Remote transport findings preserve their category in every renderer and cache state, including
  bounded deadline expiry as `timeout` rather than `connection_refused`.
- Trustworthy or advisory success exits 0, findings or inconclusive gates exit 1, and CLI misuse
  exits 2.

## MODIFIED

### SPEC SECTION Public API

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_resolve` | `root: &Path, remote: bool, verify: bool, cache_ttl: u64, strict: bool, format: OutputFormat` | `()` | Resolve typed local and remote dependency references, render one complete report in the selected format, and apply the uniform command outcome |

### SPEC SECTION Invariants

1. Every declaration is parsed by the shared typed dependency-reference parser and retained in one
   report with its originating spec and exact raw spelling.
2. Local references use the shared registry-aware confined resolver; malformed, missing, unsafe,
   registry-load, and module-identity failures are findings regardless of `--strict`.
3. No network request occurs without `--remote` or `--verify`, and `--verify` implies `--remote`.
4. Remote registry absence, fetch failure, malformed TOML, unsafe mapping, or missing module is an
   inconclusive or failing finding; zero successful requested registry fetches cannot be success.
5. Deep verification treats remote spec fetch or checked-parse failure and breaking drift as
   findings; advisory compatibility warnings are promoted only by `--strict`.
6. Remote work deduplicates repositories, uses bounded concurrency under one invocation deadline,
   and preserves authentication, authorization, rate-limit, timeout, malformed-response,
   body-limit, and transport categories.
7. Live and cached content use identical parsing, confinement, identity, and verdict rules.
8. Success text claiming verified no drift requires at least one successful registry fetch and
   complete verification of every reference and required remote spec.
9. JSON is the only stdout payload in JSON mode and contains the full report, validity and gate
   fields, mode, counts, cache status, raw references, normalized identities, categories,
   severities, and messages.
10. Text output contains one `Local dependencies` heading and never omits malformed declarations.
11. Trustworthy or advisory success maps to exit 0, findings and inconclusive requested work map to
    exit 1, and usage errors map to exit 2.
