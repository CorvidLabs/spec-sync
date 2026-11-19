---
spec: config.spec.md
---

## Regression Matrix

| Case | Required Result |
|------|-----------------|
| JSON/TOML deterministic fields | Load and round-trip |
| Legacy AI keys | Ignored; warning names keys but never values |
| Unreadable config | Fail-loud warning then safe defaults |
| Missing config | Auto-detect source directories |
| Static-only root or nested project | Detect `.` or the containing top-level directory from HTML, HTM, or CSS |
| Malformed Gradle settings through checked discovery | Return an error naming the manifest; do not expose partial manifest results |
| Malformed Gradle settings through compatibility discovery | Preserve infallible signatures and use the documented fallback behavior |
| Legacy JSON `github.repo` is a number, boolean, object, or list | Preserve otherwise valid configuration but make repository resolution fail closed without Git auto-detection |
| Exact-byte checked JSON has a non-object `github` or wrong-typed `github.repo` | Return `Err`; retained callers cannot expose sentinel/default success |
| Retained JSON/TOML snapshot | Parse exact caller bytes, preserve BOM and omitted-source compatibility, and reject malformed syntax or wrong-shaped known TOML fields |
| Retained CLI config with an explicit linked source root | Do not pre-scan the root; normal validation reports the unsafe mapping |
| Retained CLI config with invalid UTF-8 | Emit the established fail-loud warning and use safe defaults; MCP selected-config reads still reject it |
| Serialization | No retired inference keys |
| `[modules."<name>"] owns` | Round-trips through `config_to_toml` and `load_config`, alone or beside `files`; a non-array value is rejected by checked TOML validation; coverage ignores it |

## Bounded reads (REQ-config-016)

FIFOs run under a timeout and over-bound files are sparse; nothing reads `/dev/zero`. Unix-only.

| Case | Required behavior | Test |
|------|-------------------|------|
| A project file this module reads is a FIFO, a link out of the project, or over its bound | Refused without waiting or reading past the bound | `tests/integration/project_reads.rs`: `check_does_not_wait_on_a_config_file_that_is_not_a_regular_file`; `tests/integration/project_reads.rs`: `no_project_file_is_read_outside_the_bounded_read`, the source scan that fails on any direct file read outside the bounded project read |

## Configured directories (REQ-config-017)

Every link points inside the test's own temporary directory.

| Case | Required behavior | Test |
|------|-------------------|------|
| `specs_dir`, a `source_dirs` entry or `schema_dir` outside the project, absolute or through `..` | A config error naming the field; nothing outside is listed | `tests/integration/repository_inputs.rs`: `a_configured_directory_outside_the_project_is_a_config_error` |
| The same config under the MCP server | A tool error naming the field for that request; the server keeps serving | `tests/integration/mcp.rs`: `mcp_rejects_configured_read_and_write_path_escapes` |
