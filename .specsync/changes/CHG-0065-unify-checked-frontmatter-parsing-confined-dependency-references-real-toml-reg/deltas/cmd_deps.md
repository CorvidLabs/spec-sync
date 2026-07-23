## ADDED

### REQUIREMENT REQ-cmd-deps-002

`specsync deps --require-coverage <percent>` SHALL enforce the checked project coverage gate in
every output mode.

Acceptance Criteria

- The dispatcher passes the global coverage threshold to `cmd_deps`; it is not ignored.
- Text, JSON, Markdown or GitHub, Mermaid, and DOT modes compute coverage through the same checked
  coverage API and return exit 1 when measured coverage is below the requested threshold.
- Zero discoverable source files, malformed manifests, unreadable configured inputs, or another
  inconclusive coverage result fail the requested gate instead of reporting 100% or success.
- Thresholds outside 0 through 100, including 101, are usage errors with exit 2.
- JSON remains parseable on every outcome and includes `valid`, `gate_passed`, the requested
  threshold, project coverage, graph counts, diagnostics, and deduplicated edges.
- Diagram output remains the only stdout payload in Mermaid or DOT mode; gate diagnostics use
  stderr and exit status without corrupting the diagram.

## MODIFIED

### SPEC SECTION Public API

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_deps` | `root: &Path, strict: bool, require_coverage: Option<usize>, format: OutputFormat, mermaid: bool, dot: bool` | `()` | Validate and render one checked dependency report, enforcing dependency and requested coverage gates in every output mode |

### SPEC SECTION Invariants

1. Core graph construction and validation delegate to the checked `deps` report.
2. The requested coverage threshold is evaluated through checked coverage in text, structured,
   Markdown or GitHub, Mermaid, and DOT modes.
3. A threshold outside `0..=100` is a usage error with exit 2; a failed or inconclusive requested
   coverage gate exits 1.
4. Zero-source discovery cannot satisfy a requested coverage gate, including a zero-percent gate.
5. Dependency findings exit 1, and `--strict` promotes advisory dependency warnings after the
   complete report is rendered.
6. JSON is the only stdout payload in JSON mode and includes `valid`, `gate_passed`, threshold,
   checked coverage, graph counts, diagnostics, and deduplicated edges on success and failure.
7. Mermaid or DOT syntax is the only stdout payload in diagram mode; all gate diagnostics use
   stderr and process status.
8. Every renderer consumes the same normalized graph and coverage report, so parse, reference,
   registry, identity, and confinement failures cannot be bypassed by output selection.
