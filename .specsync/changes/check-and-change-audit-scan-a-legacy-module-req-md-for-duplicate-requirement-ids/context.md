---
change: check-and-change-audit-scan-a-legacy-module-req-md-for-duplicate-requirement-ids
artifact: context
---

# Context

Issue #877, a follow-up from the #865 review. `duplicate_requirement_id_errors` (the project
audit) and the per-spec check in `validate_spec` open only `requirements.md`. A legacy
`{module}.req.md` companion is still a requirements file: `hash_cache::find_companion_files`
recognizes it and `merge_driver::contract_kind` merges it as one. So a duplicate ID that such a
merge keeps passed both `check` and `change audit`. REQ-validator-045 and REQ-change-125 refuse a
requirements file that repeats an ID. The fix takes the issue's first option: both scans read
`.req.md` companions too. No requirement text changes.
