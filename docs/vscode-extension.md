---
title: VS Code Extension
layout: default
nav_order: 5
---

# VS Code Extension
{: .no_toc }

Real-time spec validation, quality scores, and coverage reports inside VS Code.
{: .fs-6 .fw-300 }

<details open markdown="block">
  <summary>Table of contents</summary>
  {: .text-delta }
1. TOC
{:toc}
</details>

---

## Installation

Install from the [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=corvidlabs.specsync) or search **"SpecSync"** in the Extensions panel.

```bash
code --install-extension corvidlabs.specsync
```

{: .note }
The extension requires the `specsync` CLI binary to be installed and on your PATH. See the [CLI Reference](cli.html) for installation instructions.

---

## Activation

The extension activates automatically when your workspace contains any of:

- `specsync.json`
- `.specsync.toml`
- A `specs/` directory

On activation, it runs an initial validation and displays results in the status bar.

---

## Features

| Feature | Description |
|:--------|:------------|
| **Inline diagnostics** | Errors and warnings mapped to spec files in the Problems panel |
| **CodeLens scores** | Quality grade and score (0–100) displayed inline above each spec file |
| **Coverage report** | Rich webview showing file and LOC coverage with uncovered file details |
| **Scoring report** | Per-spec quality breakdown with grade distribution and improvement suggestions |
| **Status bar** | Persistent pass/fail/error indicator — click to re-validate |
| **Validate-on-save** | Automatic validation with 500ms debounce when saving spec or source files |

---

## Commands

All commands are available via the Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`):

| Command | Description |
|:--------|:------------|
| `SpecSync: Validate Specs` | Run `specsync check` and update diagnostics |
| `SpecSync: Show Coverage` | Open the coverage report webview |
| `SpecSync: Score Spec Quality` | Open the scoring report webview |
| `SpecSync: Generate Missing Specs` | Scaffold specs for unspecced modules |
| `SpecSync: Initialize Config` | Create a `specsync.json` in the workspace root |

---

## Settings

| Setting | Default | Description |
|:--------|:--------|:------------|
| `specsync.binaryPath` | `specsync` | Path to the specsync binary (if not on PATH) |
| `specsync.validateOnSave` | `true` | Automatically validate when spec or source files are saved |
| `specsync.showInlineScores` | `true` | Show CodeLens quality scores above spec files |

To configure, open **Settings** (`Ctrl+,` / `Cmd+,`) and search for "specsync".

---

## Status Bar

The status bar item shows the current validation state:

| Icon | Meaning |
|:-----|:--------|
| `$(check) SpecSync: N specs OK` | All specs pass validation |
| `$(warning) SpecSync: NE NW` | Validation found errors/warnings |
| `$(sync~spin) SpecSync` | Validation in progress |
| `$(error) SpecSync` | CLI error (binary not found, crash, etc.) |

Click the status bar item to re-run validation at any time.

---

## CodeLens Scores

When `specsync.showInlineScores` is enabled, each `.spec.md` file displays a CodeLens line at the top showing:

- **Grade** (A–F) and **total score** (0–100)
- **Breakdown**: Frontmatter, Sections, API, Depth, Freshness
- **Top suggestion** for improvement (if any)

Click the CodeLens to open the full scoring report.

---

## Webview Reports

### Coverage Report

The coverage report (`SpecSync: Show Coverage`) shows:

- **File coverage** — percentage of source files with matching specs
- **LOC coverage** — percentage of lines of code covered by specs
- **Uncovered files** — sorted by LOC, largest gaps first
- **Unspecced modules** — modules that need spec files

### Scoring Report

The scoring report (`SpecSync: Score Spec Quality`) shows:

- **Overall grade** and average score
- **Grade distribution** (A/B/C/D/F counts)
- **Per-spec details** — grade, score, sub-scores, and suggestions

---

## Troubleshooting

**"SpecSync" not activating?**
Ensure your workspace contains `specsync.json`, `.specsync.toml`, or a `specs/` directory.

**"Command not found" errors?**
The `specsync` binary must be on your PATH or configured via `specsync.binaryPath`. Check the Output panel (View → Output → SpecSync) for detailed logs.

**Diagnostics not updating?**
Check that `specsync.validateOnSave` is `true` in settings. You can also manually trigger validation via the Command Palette or by clicking the status bar.
