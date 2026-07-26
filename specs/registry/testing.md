---
spec: registry.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/registry.rs` | `fledge run test -- registry::tests::` | `[specs]` and `[[modules]]` parsing, malformed TOML, wrong known-field types/shapes, duplicate mappings, inert-stub compatibility, generation, and lookup |

## Coverage Gaps

- Integration gap: add a fixture for "Fetch remote registry" before changing user-visible CLI output, generated files, or error handling in registry.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Fetch remote registry | a GitHub repo "corvid-labs/algochat" with a `specsync-registry.toml` at root | `fetch_remote_registry("corvid-labs/algochat")` is called | returns `Ok(RemoteRegistry)` with parsed module-to-path mappings |
| Generate registry from local specs | specs at `specs/auth/auth.spec.md` and `specs/messaging/messaging.spec.md` | `generate_registry(root, "myproject", "specs")` is called | returns TOML string with `[registry]\nname = "myproject"\n\n[specs]\nauth = "specs/auth/auth.spec.md"\nmessaging = "specs/messaging/messaging.spec.md"\n` |
| Check module existence | a `RemoteRegistry` with specs for "auth" and "messaging" | `has_spec("auth")` is called | returns `true` |
| Resolve documented mapping | a named registry with `[[modules]]` mapping `lib` to `custom/lib.spec.md` | load the local registry and look up `lib` | returns the declared custom path |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| HTTP request fails | Error: "HTTP request failed: {details}" | Keep or add a focused assertion before changing this behavior |
| Repo has no registry file | Error: "HTTP 404 — {repo} may not have a specsync-registry.toml" | Keep or add a focused assertion before changing this behavior |
| Malformed TOML with a surviving `name` line | `load_local_registry` returns `Err` | `load_local_registry_fails_closed_on_malformed_toml_with_name_line` |
| Non-string `[specs]` path | field-specific parse and load error | `specs_mapping_with_non_string_path_is_an_error` |
| Non-empty legacy `[modules]` table | not inert; parse error | `nonempty_legacy_modules_table_is_not_inert` |
| Duplicate mapping across `[specs]` and `[[modules]]` | parse error rather than silent selection | `duplicate_mapping_across_supported_shapes_is_an_error` |
| Local registry file unreadable | `load_registry` returns `None` | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/registry.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
