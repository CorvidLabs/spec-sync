---
change: align-public-documentation-and-the-release-runbook-with-the-shipped-specsync-6-0-0
artifact: docs
---

# Docs

Surfaces corrected, all verified against the 6.0 binary and workflows:

- README.md: navigation anchor, real `check --strict` output, install prose without candidate-window wording, `<change-id>/` tree line.
- site/src/content/docs/**: slug identities and `check --commit` / `review` / `finalize` in every walkthrough; `init` documented as writing `sdd.json` off with no interview; real `check` and JSON samples; `import` caveat (#416); `sdd.json` version 2; Action examples pin `@v6.0.0` with `version: '6.0.0'`; Ubuntu and macOS only; `status` enum complete; `.cursorrules` and `AGENTS.md` hook outputs.
- MIGRATION.md: 5.x to 6.0 path with binary and Action pins, the strict enforcement default, `check` no longer walking SDD, fresh-init policy defaults, and the pre-adopt requirement to land every workflow-v1 change (#674).
- docs/ADOPTING.md, SECURITY.md, CONTRIBUTING.md, SCOPE.md, AGENTS.md, vscode-extension/README.md, examples/*/README.md: version pins, lifecycle verbs, platform claims and links aligned.
- action.yml: description text only (what the Action runs, pin an exact version, Linux and macOS runners).
- docs/ci-confidence.md: two-platform evidence, RC16 covers ffba9a32 only, the final tree needs a fresh RC before `promote`.
- docs/RELEASING.md (new): preconditions and validators, cutting an annotated RC, what "qualified" means, dry run, `promote` with its idempotent and refusal cases and never-executed status, post-tag verification, crates.io and Homebrew steps, changelog dating, and what cannot be undone.
- CHANGELOG.md: one `[6.0.0]` Changed entry describing this sweep. fledge.toml: lane comment says Ubuntu and macOS.
