## MODIFIED

### SPEC SECTION Purpose

Detects and conservatively auto-resolves git merge conflicts in spec files using context-aware strategies. Only lossless conflict shapes are resolved: known YAML lists are unioned, numeric versions take the maximum value, changelog tables are merged chronologically, and non-conflicting generic table rows are unioned. Ambiguous content leaves the entire file untouched.
