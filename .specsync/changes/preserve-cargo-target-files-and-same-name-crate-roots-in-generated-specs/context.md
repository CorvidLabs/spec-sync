# Context

Two automated PR #814 review findings reproduce against dd6eb3e. Package utils with src/lib.rs and src/utils/mod.rs drops lib.rs. Binary server at src/foo.rs binds src/lib.rs and duplicates the library owner. The user explicitly approved fixing issue #813 with local validation, confidence at least 95 percent, and no regressions. These are corrections to that same bug-fix request. The initial definition remains immutable; this narrow package owns only manifest parsing and parser tests. The reopened initial change owns generation, coverage and process regressions.
