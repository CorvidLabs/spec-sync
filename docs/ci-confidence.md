# CI confidence architecture (no duplicate suites)

**Goal:** approximately 95% merge confidence for ordinary product PRs **without** running the same
expensive suite twice. Release validation and explicitly sensitive changes may add stricter checks.

## Ownership (single source of truth)

| Confidence need | Owner | Where |
|-----------------|-------|--------|
| Format (`rustfmt`) | **CI** | `fmt` job |
| Lint (`clippy -D warnings`) | **CI** | required Ubuntu product lane |
| Unit + integration tests | **CI** | required Ubuntu product lane; Tier B adds macOS + Windows |
| Typecheck | **CI** | covered by `cargo test` / build; local `check-types` |
| Release binary build | **CI** (consumer) + **Trust** (identity) | CI: action-consumer; Trust: packages PR binary for contract |
| Spec contract + 100% path coverage | **CI** + **Trust contract gate** | CI `spec-check` proves tree; Trust re-checks with **PR release binary** (identity, not a second matrix) |
| Security advisories | **CI** | `audit` |
| Coverage measurement | **CI** | Tier B when expensive; strict 100% spec/path coverage stays Tier A |
| Site / VS Code extension | **CI** | `site`, `vscode-extension` |
| Action packaging consumer | **CI** | `action-consumer` |
| Deterministic risk (Augur) | **Trust only** | Trust action risk gate |
| Provenance (Attest) | **Trust only** | Trust action provenance |
| Lifecycle *re-suite* (full test again) | **None — removed** | Was duplicate of CI |

## Confidence tiers

### Tier A: every ordinary product PR

- `cargo fmt --check`
- Ubuntu `cargo clippy -- -D warnings` and full `cargo test`
- `specsync check --strict --require-coverage 100`
- `cargo audit`
- cheap path classification, Action validation, and required readiness gates
- Trust's release-binary identity, contract, Augur, and Attest gates

### Tier B: immutable release candidates

- Ubuntu, macOS, and Windows integration and release validation against one exact candidate SHA
- expensive line coverage/tarpaulin
- any additional release or security matrix required by project policy

The Trust split in this change is non-protected and lands first. Moving macOS, Windows, and expensive
coverage from every PR into Tier B changes `.github/workflows/**`; that requires the repository's
separately pinned required-workflow process. Until that follow-up lands, the existing CI workflow
continues running those jobs on every product PR.

The future protected-workflow follow-up should make Ubuntu the authoritative integration platform
for ordinary development and product PRs. macOS and Windows should not consume runner time on those
PRs. Instead, a release-candidate cycle should:

1. Freeze an exact candidate commit on an RC branch and create an immutable RC marker/tag for that
   SHA.
2. Run the required Ubuntu, macOS, and Windows integration/release gates against that same SHA.
3. Refuse the final release tag and uploads unless every required platform is green for the unchanged
   candidate SHA.

If the candidate changes, create a new immutable RC marker and rerun the cross-platform gate. Do not
create the final release tag first and use its uploads to discover platform failures afterward.

## Wall-clock model

Before this change, Trust was roughly 20 minutes, including roughly 17 minutes spent re-running the
full Rust suite. After this change, Trust should take roughly 3–8 minutes for release build, light
lifecycle, contract, risk, and provenance.

The immediate product-tip critical path remains:

```text
Parallel critical path ≈ max(
  test/windows,   # historically 15–45m
  test/macos,     # ~16m
  trust,          # should be ~3–8m after this redesign (release build + light lifecycle + contract + augur)
  spec-check,     # ~10–12m
  coverage        # ~10m
)
```

After the separately pinned Tier B workflow update, the ordinary PR target is approximately 5–15
minutes. This PR does not claim that matrix scheduling improvement before the protected change lands.

Trust must **not** re-run `cargo test` / clippy / full verify after CI already did.

## Trust lifecycle policy

| Lane | Command | When |
|------|---------|------|
| `verify` | full fmt+lint+types+**test**+release build+spec-check | **Local** `fledge lanes run verify` / agent complete |
| `trust-lifecycle` | **types only** (no test suite) | **GitHub** Trust action via `.trust.toml` |

`.trust.toml` `[lifecycle]` points at `trust-lifecycle` so the Trust GitHub job does not duplicate CI tests.

Trust still:

1. Builds **this PR’s** `cargo build --release` binary (identity artifact)
2. Runs **contract** against that binary (`require_coverage = 100`)
3. Runs **Augur** + **Attest**

That preserves “this binary is the contract” without a second multi-OS suite.

## Tip classes (unchanged)

| Tip | CI | Trust |
|-----|----|-------|
| Product / full | Full matrix + gates | Full Trust action (light lifecycle) |
| `review_only` | Reuse / skip heavy | Reuse ancestor trust |
| `archive_only` | archive-integrity | Reuse ancestor trust |

## Local agent checklist

```bash
bash scripts/pre-push-gate.sh          # fast: fmt + check + path coverage
fledge lanes run verify                # one full local completion suite
fledge trust verify                    # contract + risk + light lifecycle + attest
```

Do **not** expect `fledge trust verify` alone to replace multi-OS CI; GitHub CI is the multi-OS authority.

## Anti-patterns (do not reintroduce)

1. Putting `test` / full `lanes.verify` back into Trust’s GitHub lifecycle
2. Making Trust the only place that runs tests (drops release-candidate multi-OS validation)
3. Dropping Windows/macOS without an immutable-SHA release-candidate gate
4. Running `cargo test` in both CI and Trust “just to be safe” (doubles cost, same bugs)

## Related

- Protected matrix scheduling and ancestor-reuse follow-ups require a separately pinned workflow update
- `fledge.toml` lanes: `verify` vs `trust-lifecycle`
- `.trust.toml` lifecycle command
