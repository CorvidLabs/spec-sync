## ADDED

### REQUIREMENT REQ-config-002

Configuration source-directory autodetection SHALL recognize default measurable static files in addition to language exports.

Acceptance Criteria

- Static-only root projects resolve to `.`.
- Static-only nested projects resolve to the containing top-level directory.
- Empty projects retain the `src` fallback.

## MODIFIED

### SPEC SECTION Purpose

Loads canonical project configuration from `.specsync/config.toml`, with compatibility fallbacks for `.specsync/config.json`, `.specsync.toml`, and `specsync.json`, then auto-detects source directories from supported language and default static HTML, HTM, and CSS files when configuration does not provide them.
