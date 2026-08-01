# CI confidence architecture (no duplicate suites)

**Goal:** 100% merge confidence **without** running the same expensive suite twice.

## Ownership (single source of truth)

| Confidence need | Owner | Where |
|-----------------|-------|--------|
| Format (`rustfmt`) | **CI** | `fmt` job |
| Lint (`clippy -D warnings`) | **CI** | every `test` matrix OS |
| Unit + integration tests | **CI** | `test` matrix: ubuntu + macos + windows |
| Typecheck | **CI** | covered by `cargo test` / build; local `check-types` |
| Release binary build | **CI** (consumer) + **Trust** (identity) | CI: action-consumer; Trust: packages PR binary for contract |
| Spec contract + 100% path coverage | **CI** + **Trust contract gate** | CI `spec-check` proves tree; Trust re-checks with **PR release binary** (identity, not a second matrix) |
| Security advisories | **CI** | `audit` |
| Coverage measurement | **CI** | `coverage` |
| Site / VS Code extension | **CI** | `site`, `vscode-extension` |
| Action packaging consumer | **CI** | `action-consumer` |
| Deterministic risk (Augur) | **Trust only** | Trust action risk gate |
| Provenance (Attest) | **Trust only** | Trust action provenance |
| Lifecycle *re-suite* (full test again) | **None — removed** | Was duplicate of CI |

## Wall-clock model (product tip)

```text
Parallel critical path ≈ max(
  test/windows,   # historically 15–45m
  test/macos,     # ~16m
  trust,          # should be ~3–8m after this redesign (release build + light lifecycle + contract + augur)
  spec-check,     # ~10–12m
  coverage        # ~10m
)
```

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

## Local agent checklist (100% confidence)

```bash
bash scripts/pre-push-gate.sh          # fast: fmt + check + path coverage
fledge lanes run verify                # full local suite when needed
fledge trust verify                    # contract + risk + light lifecycle + attest
```

Do **not** expect `fledge trust verify` alone to replace multi-OS CI; GitHub CI is the multi-OS authority.

## Anti-patterns (do not reintroduce)

1. Putting `test` / full `lanes.verify` back into Trust’s GitHub lifecycle
2. Making Trust the only place that runs tests (drops multi-OS)
3. Dropping Windows/macOS from CI without an explicit alternate confidence plan
4. Running `cargo test` in both CI and Trust “just to be safe” (doubles cost, same bugs)

## Related

- PR ship thrash / tip dance: `docs` GOAL-6-buttery in sandbox; product #487–#489
- `fledge.toml` lanes: `verify` vs `trust-lifecycle`
- `.trust.toml` lifecycle command
