# CI confidence architecture (no duplicate suites)

**Goal:** approximately 95% merge confidence for ordinary product PRs **without** running the same
expensive suite twice. Release validation and explicitly sensitive changes may add stricter checks.

## Ownership (single source of truth)

| Confidence need | Owner | Where |
|-----------------|-------|--------|
| Format (`rustfmt`) | **CI** | `fmt` job |
| Lint (`clippy -D warnings`) | **CI** | required Ubuntu product lane |
| Unit + integration tests | **CI** | required Ubuntu product lane; Tier B adds macOS. Windows is neither built nor qualified as of 6.0 (#735) |
| Typecheck | **CI** | covered by `cargo test` / build; local `check-types` |
| Release binary build | **CI** (consumer) + **Trust** (identity) | CI: action-consumer; Trust: packages PR binary for contract |
| Spec contract + 100% path coverage | **CI** + **Trust contract gate** | CI `spec-check` proves tree; Trust re-checks with **PR release binary** (identity, not a second matrix) |
| Security advisories | **CI** | `audit` |
| Coverage measurement | **CI** | ordinary full product PRs; never duplicated in Trust |
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
- line coverage/tarpaulin for full product changes
- cheap path classification, Action validation, and required readiness gates
- Trust's release-binary identity, contract, Augur, and Attest gates

### Tier B: immutable release candidates

- Ubuntu, macOS, and Windows integration and release validation against one exact candidate SHA
- any additional release or security matrix required by project policy

The Trust split remains non-protected. CHG-0075 applies the separately pinned protected-workflow
update: Ubuntu is the authoritative integration platform for ordinary development and product PRs,
while macOS and Windows run only in the immutable release-candidate cycle:

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

The immediate product-tip critical path is:

```text
Parallel critical path ≈ max(
  test/ubuntu,    # authoritative product suite
  trust,          # should be ~3–8m after this redesign (release build + light lifecycle + contract + augur)
  spec-check,     # ~10–12m
  coverage        # ~10m
)
```

With the Tier B workflow update, the ordinary PR target is approximately 5–15 minutes.

Trust must **not** re-run `cargo test` / clippy / full verify after CI already did.

## Tag authority: what is enforced, and what is not

Tag authority rests on **two** active repository rulesets, and RC qualification verifies exactly
those two before anything else runs:

| Ruleset | Include | Exclude | Rules | Bypass actors |
|---------|---------|---------|-------|---------------|
| `SpecSync immutable RC tags` | `refs/tags/v*.*.*-rc.*` | — | `update`, `deletion` | none |
| `SpecSync immutable final tags` | `refs/tags/v*.*.*` | `refs/tags/v*.*.*-rc.*` | `update`, `deletion` | none |

Both must be `active`, `Repository`-sourced, and grant bypass to **nobody**. `resolve` refuses any
broadening — an extra include pattern, a missing exclude, an added rule type, a single bypass actor,
`evaluate` instead of `active` — so the gate cannot be widened without failing. Humans may create
RC markers and final tags; no actor can move or delete either once created. That immutability is
what makes a shipped `vX.Y.Z` reproducible, and it is the protection that survives.

Three protections the original design specified are deliberately **not** enforced. Every release
run states all three as warning annotations and in its step summary, so a green run can never be
misread as proof of a policy nobody enforces:

- **App-only final-tag creation is NOT enforced.** There is no `SpecSync final tag creation`
  ruleset, so any actor with tag-write access can create `refs/tags/vX.Y.Z` directly, without a
  qualified candidate and without this workflow.
- **The final tag is minted by the workflow's own `GITHUB_TOKEN`, NOT by a separate release
  identity.** `promote` creates `vX.Y.Z` with `GITHUB_TOKEN` under a `contents: write` permission
  scoped to that one job. There is no App key that a workflow author cannot reach, so anyone able
  to run `release.yml` from the default branch can cause a release tag to be created. **Running the
  release lane and holding release authority are the same permission here.**
- **Promotion is NOT behind a deployment-environment gate.** `promote` names no environment, so no
  required reviewer, wait timer, or deployment branch policy stands between dispatching a promotion
  and the tag being written.

Why: there is no release GitHub App, and the owner decided not to create one. The App, its
`SpecSync final tag creation` ruleset, and the protected `release` environment that would have held
its private key were never provisioned; demanding all three plus the App failed `release.yml` on
every RC tag from `v6.0.0-rc.1` through `rc.6`, and the check never once passed after it was added
in #492. A gate that always fails verifies nothing, and the two rulesets that *do* exist were never
reached. Rather than leave dead App plumbing that fails closed on every dispatch, `promote` now
pushes the tag with `GITHUB_TOKEN` and the workflow says, at the job and in every run log, exactly
what that costs.

The `environment: release` reference was **removed rather than kept**. GitHub materializes a
referenced environment on first use with no protection rules at all, so naming an environment this
repository does not have would publish a `release` entry in Environments and Deployments that gates
nothing while looking like a gate. An unprotected environment is worse than none, because only one
of the two can mislead a reader. To make promotion a real gate: create the `release` environment
with required reviewers and a `main`-only deployment branch policy **first**, then re-add
`environment: release` to `promote`, then restore a qualification check that proves those rules are
still in place.

What this does **not** cost: no workflow in this repository listens for a final-tag push
(`release.yml` is the only `tags:` trigger and it matches `v*.*.*-rc.*` only), so the usual
objection to `GITHUB_TOKEN` — that pushes made with it do not start other workflows — has nothing
to break here. Anything added later that must react to `vX.Y.Z` has to be called from inside
`release.yml` rather than triggered by the tag.

### What has actually run, and what has not

The section above reasons about tag authority. This one records which of it has been *executed*,
because a design that has only ever been argued is not a design that has been tested, and the
release lane is the one workflow whose most important job runs exactly once.

| Job | Status |
|-----|--------|
| `resolve` | **Executed.** `workflow_dispatch` with `dry_run=true` against `v6.0.0-rc.7` — the first dispatch in this repository's history. |
| `validate` | **Executed**, same run. Both rulesets read and accepted. |
| `qualify` | Ubuntu and macOS only as of #735. It first ran on `rc.8` and failed on Windows there and on `rc.9`; `rc.1`–`rc.7` had died in `resolve` in 8–13 seconds, so the matrix never ran at all. Windows is now dropped from the lane rather than fixed forward, so the retained `#[cfg(windows)]` code is compiled and run **nowhere** — those guarantees are best-effort and unverified. No candidate has yet qualified; `rc.10` is the first to try. |
| `promote` | **Never executed.** |

`promote` cannot be rehearsed here. `final_tag` is derived from the candidate's own `Cargo.toml`
version, so any promote run against a real candidate mints the real `vX.Y.Z` — there is no throwaway
value to aim it at, and the immutability rulesets make whatever it creates permanent. The proof and
the release are the same event.

What *was* proven, ahead of time and separately:

- **Its git mechanics.** The `Create final tag` step was transcribed verbatim against a local bare
  repository and all three branches exercised: a fresh create produces an **annotated** tag pointing
  at `candidate_sha` and not at HEAD; a re-run against the same candidate takes the idempotent path
  and exits 0; a re-run against a different candidate refuses with
  `already points at a different commit` and exits 1.
- **That the rulesets do not block it.** Both carry `update` and `deletion` only. Tag *creation* is
  unrestricted, so the immutability that protects a released tag cannot prevent it being minted.
  This was the failure that would have appeared for the first time at the moment of release.

What remains unproven, stated rather than glossed: the credential helper supplies a credential only
when the remote asks for one, and a local path remote never asks — so the rehearsal proves the
helper's `git -c` syntax does not break the invocation, not that it authenticates. That, and
`GITHUB_TOKEN`'s push against the live ruleset, first execute on the real release.

**A promote failure is recoverable, which is why this residue is acceptable.** The step creates
nothing on the failing paths, and the tag is pushed as the last action of the job; a failed run
leaves the tag namespace untouched, and the idempotent branch means a retry after a *later* job
fails is safe rather than a second release. The one non-recoverable outcome — a tag pointing at the
wrong commit — is the case that refuses.

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
| Product / full | Ubuntu product lane + gates | Full Trust action (light lifecycle) |
| Release candidate | Ubuntu + macOS release lane at one immutable SHA | Release identity and provenance |
| `review_only` | Reuse / skip heavy | Reuse ancestor trust |
| `archive_only` | archive-integrity | Reuse ancestor trust |

## Local agent checklist

```bash
bash scripts/pre-push-gate.sh          # fast: fmt + check + path coverage
fledge lanes run verify                # one full local completion suite
fledge trust verify                    # contract + risk + light lifecycle + attest
```

Do **not** expect `fledge trust verify` alone to replace release-candidate multi-OS CI; the Release
workflow is the cross-platform authority for an exact immutable candidate SHA.

## Anti-patterns (do not reintroduce)

1. Putting `test` / full `lanes.verify` back into Trust’s GitHub lifecycle
2. Making Trust the only place that runs tests (drops release-candidate multi-OS validation)
3. Dropping Windows/macOS without an immutable-SHA release-candidate gate
4. Running `cargo test` in both CI and Trust “just to be safe” (doubles cost, same bugs)

## Related

- Further protected topology or ancestor-reuse changes require a separately pinned workflow update
- `fledge.toml` lanes: `verify` vs `trust-lifecycle`
- `.trust.toml` lifecycle command
