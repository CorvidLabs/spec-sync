# SpecSync for VS Code

Bidirectional spec-to-code validation directly in your editor. This extension wraps the [SpecSync CLI](https://github.com/CorvidLabs/spec-sync) to show diagnostics, coverage, and quality scores without leaving VS Code.

## Features

- **Validate on save** — runs `specsync check` when you save a spec or source file; errors and warnings appear in the Problems panel
- **Coverage view** — see which files and modules are covered by specs
- **Quality scores** — letter grades (A-F) with score breakdowns and improvement suggestions
- **CodeLens** — inline quality scores on spec files showing grade, component scores, and top suggestion
- **Generate specs** — scaffold specs for unspecced modules
- **Status bar** — persistent indicator showing validation state

## Requirements

Install the `specsync` CLI:

```bash
cargo install specsync
```

Or download a prebuilt binary from [GitHub Releases](https://github.com/CorvidLabs/spec-sync/releases).

## Extension Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `specsync.binaryPath` | `specsync` | Path to the specsync binary |
| `specsync.validateOnSave` | `true` | Run validation on save |
| `specsync.showInlineScores` | `true` | Show CodeLens quality scores on spec files |

## Commands

- **SpecSync: Validate Specs** — run full validation
- **SpecSync: Show Coverage** — open coverage report
- **SpecSync: Score Spec Quality** — open quality report
- **SpecSync: Generate Missing Specs** — scaffold new specs
- **SpecSync: Initialize Config** — create `specsync.json`
