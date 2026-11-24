## MODIFIED

### SPEC SECTION Error Cases

| Condition | Behavior |
|-----------|----------|
| Spec file unreadable | Error: "Cannot read spec" |
| Spec discovery meets a `specs` link out of the repository, or passes 100,000 entries or 256 levels | `find_spec_files_checked` returns `spec discovery stopped: <reason>`; `find_spec_files` prints it and exits 1 |
| The lesson file beside the spec, `<module>.3md` named for the spec's module, does not parse as 3md or cannot be read, which includes an entry of that name that is not a regular file | Warning naming the file, the parser's error and that `context.md` is used for its lessons instead, so `check --strict` fails (REQ-cmd-check-023) |
| Pre-read snapshot passed to `validate_spec_content` | Validates those exact bytes without opening `spec_path` or adjacent companions |
| Supplied source is `Missing`, `Rejected`, or `Unreadable` | Reports the corresponding mapped-source validation outcome without ambient fallback |
| Missing frontmatter delimiters | Error: "Missing or malformed YAML frontmatter" |
| Source file not found | Error with fix suggestion (Levenshtein-based or removal) |
| DB table not in schema | Error: "DB table not found in schema" |
| Missing required section | Error: "Missing required section: ## SectionName" |
| Dependency spec not found | Error: "Dependency spec not found" |
| Malformed, unreadable, unsupported, or unconfined Gradle discovery during checked coverage with `source_dirs` unstated, including unsafe Gradle manifest entries | Returns `Err`; CLI/MCP gate callers report an inconclusive failure rather than coverage success, referent reads, or outside traversal |
| The same discovery failure with `source_dirs` explicitly stated | Coverage runs over the stated list and returns `Ok`; the error is reported as a `manifest_notices` entry beside the figures, and the manifest contributes no modules |
| Coverage selected-spec/source input is linked/reparse-backed, special, replaced, invalid UTF-8, over 8 MiB, or shared traversal exceeds 64 MiB/100,000 entries/256 components | Returns `Err` from the retained project snapshot; no partial totals or ambient fallback |
| Caller-selected coverage spec is outside the retained project, missing, linked/reparse-backed, special, replaced, invalid UTF-8, or over the shared coverage input bounds | Returns `Err`; ownership mappings are never obtained through the ambient spec pathname |
| Valid checked coverage contains more sibling spec/source directories than the process descriptor limit | Children are reopened sequentially through retained parents; coverage completes with handles bounded by traversal depth |
| Valid checked coverage configures more source roots than the process descriptor limit | Root identities are selected without retaining handles, then each root is reopened, identity-checked, traversed, and released sequentially |
| A project file this module reads leaves the project through a link, dangles, is not a regular file, or is over its bound | Refused without waiting or reading past the bound, and handled like that read's other read errors |
