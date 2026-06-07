---
spec: registry.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/registry.rs` | cargo test registry:: | `test_parse_registry`, `test_parse_registry_empty`, `test_extract_module_name`, `test_remote_registry_has_spec` |

## Coverage Gaps

- Integration gap: add a fixture for "Fetch remote registry" before changing user-visible CLI output, generated files, or error handling in registry.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Fetch remote registry | a GitHub repo "corvid-labs/algochat" with a `specsync-registry.toml` at root | `fetch_remote_registry("corvid-labs/algochat")` is called | returns `Ok(RemoteRegistry)` with parsed module-to-path mappings |
| Generate registry from local specs | specs at `specs/auth/auth.spec.md` and `specs/messaging/messaging.spec.md` | `generate_registry(root, "myproject", "specs")` is called | returns TOML string with `[registry]\nname = "myproject"\n\n[specs]\nauth = "specs/auth/auth.spec.md"\nmessaging = "specs/messaging/messaging.spec.md"\n` |
| Check module existence | a `RemoteRegistry` with specs for "auth" and "messaging" | `has_spec("auth")` is called | returns `true` |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| HTTP request fails | Error: "HTTP request failed: {details}" | Keep or add a focused assertion before changing this behavior |
| Repo has no registry file | Error: "HTTP 404 — {repo} may not have a specsync-registry.toml" | Keep or add a focused assertion before changing this behavior |
| Malformed TOML (no name) | `parse_registry` returns `None` | Keep or add a focused assertion before changing this behavior |
| Local registry file unreadable | `load_registry` returns `None` | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/registry.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
