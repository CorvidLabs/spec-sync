# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **Verification currency is a content question only.** Whether recorded verification evidence
  is still current is now decided by three content equalities — the evidence passed, the plan
  on disk is the plan that was verified, and the tree on disk is the tree that was verified.
  The git-ancestry walk over descendants of the verification commit, and its `REQ-change-016`
  path allowlist, are removed, as is the `merge-base --is-ancestor` binding between the
  verification commit and the implementation. `verification.commit` is still recorded, but as
  an informational correlation key rather than a gate.

  Two long-standing failures dissolve as a result. **Squash merges no longer orphan
  verification evidence** — the recorded commit is unreachable after a squash, so the ancestry
  check could never pass again. And the lifecycle no longer instructs an author to make a
  commit that its own gate then refuses (the allowlist forbade moving `approvals.json` and
  `change-sequence.json` between verification and review).

  **Reduced detection, stated plainly:** a source change followed by a revert no longer stales
  evidence. The tree is byte-identical to the one that was verified, so the content answer is
  "still current"; previously the ancestry walk saw the intermediate commit and staled it.
  Detecting that work happened in between is a provenance question and belongs to `attest`,
  which keys signed records to commit SHAs in git notes.

- **BREAKING (exit codes): drift now gates by default.** The default enforcement mode changes
  from `warn` to `strict`, so a bare `specsync check` exits 1 on a validation error instead of
  reporting it and exiting 0. Warnings still pass unless `--strict` is supplied, and
  `--enforcement warn` restores the previous non-blocking behaviour.

  Previously `warn` was the default and documented as "always exit 0 regardless of errors or
  warnings", which meant a repository with a deleted source file and undocumented exports
  passed `check`. Paired with the severing above, this moves the gate onto the thing the tool
  exists to check. **Repositories with pre-existing spec errors will start failing; run
  `specsync check` locally before upgrading, or set `--enforcement warn` while you clear them.**

- **BREAKING (exit codes): `specsync check` no longer fails because of SDD lifecycle state.**
  Lifecycle state is now reported, not enforced: `check` prints the active-change count and
  emits lifecycle findings as warnings on stderr, and its exit status is determined solely by
  spec validation results, the effective enforcement mode, `--strict`, and `--require-coverage`.
  A repository that was red *only* for lifecycle reasons — stale verification evidence, a
  squash-orphaned evidence commit, a diverged sequence ledger — now exits 0.

  Previously the lifecycle gate exited 1 unconditionally while the default enforcement mode
  (`warn`) always exits 0, which made the lifecycle a stricter gate than the specs it guarded:
  a repository with a deleted source file and undocumented exports passed `check`, while a
  bookkeeping problem failed it. Every trust-layer failure also presented to users as "the
  drift check is broken."

  Lifecycle gating is unchanged and still available through the `change` verbs and
  `specsync change audit`. **If CI relied on `specsync check` to block on lifecycle state, add
  `specsync change audit --strict` to the pipeline.**

- **`specsync comment` reports spec-check results only** — SDD lifecycle errors and warnings
  are no longer folded into the reported totals (previously prefixed `.specsync/sdd.json:`)
  and no longer decide whether the comment reports a pass.

- **Mandatory local pre-push gate (fast)** — `fledge lanes run pre-push` / `./scripts/pre-push-gate.sh` runs fmt + cargo check + strict path/spec coverage with timing (target: ~seconds–2 min warm; no full test/clippy). Full suite remains `fledge lanes run verify` / CI.

### Fixed

- **An unparseable `.specsync/config.toml` no longer reports success**
  (CorvidLabs/spec-sync#570). A single missing bracket turned a failing project green: the
  loader fell back to built-in defaults, warned on **stderr only**, and stdout printed
  `✓ All required sections present` — a claim about a section list that had been thrown
  away. A CI job capturing stdout saw a clean pass, and `--strict`, `--force`, `--json` and
  `score --min-score` all agreed with it. `score` actually *rose* from 96 to 100, because
  the project lost the very rule it was failing.

  This disabled every rule the project had configured — `required_sections`, `[rules]`
  thresholds, `exclude_patterns` — simultaneously, from one typo. A project writes that
  file precisely because the defaults are not enough, so silently substituting the defaults
  is never the safe reading.

  A config file that exists and cannot be loaded is now an error, refused at the shared
  entry point every spec-reading command passes through. A project with **no** config file
  is unaffected and still runs on the built-in defaults.

- **Coverage no longer displays `0/0` as 100%** (CorvidLabs/spec-sync#562). A project whose
  configured `source_dirs` contained no source files printed `File coverage: 0/0 (100%)` — in
  the same run that `--require-coverage` correctly failed. The gate and the display
  contradicted each other, and the number is the half that ends up on badges and dashboards.

  `compute_exit_code` already carried the reasoning: *"a `--require-coverage` gate over zero
  source files is a vacuous pass … fail loud so a broken config cannot pass CI."* The hazard
  was understood and defended in the gate, while the display kept reporting success.

  Zero denominators now read `0/0 (no source files to measure)`, and the two affirmative
  lines that were true only of an empty set — `✓ All source files referenced by specs` and
  `✓ All source modules have spec directories` — are replaced by the actionable cause:
  `⊘ No source files were found to measure — check source_dirs and exclude_patterns`.
  A project with source files is unaffected.

- **`view --spec <unknown>` no longer succeeds silently** (CorvidLabs/spec-sync#551).
  Asking for a module that does not exist printed **zero bytes** and exited 0, in text and
  in JSON alike — indistinguishable from a module that exists and renders empty. A script or
  agent fetching spec context for a mistyped name got an empty payload and no signal to
  retry.

  The filter loop skipped every non-matching spec, so a filter matching nothing left the
  loop body unexecuted and the command returned normally. It now exits non-zero, names the
  unknown module, and suggests the near match:

  ```
  error: no spec module named `alph`
    did you mean: alpha
  ```

  When nothing is close it lists the modules that do exist instead. The same run also
  surfaced a second defect with the identical shape: a spec that failed to render printed
  its error to stderr and was then ignored by the exit code, so a caller could not tell a
  rendered spec from an unrenderable one. Both now gate.

- **`check --fix` no longer reports success when it could not write**
  (CorvidLabs/spec-sync#549). A spec file that was not writable produced exit 0, no error,
  and nothing on stderr, while the byte-identical writable spec reported
  `✓ … added 1 export(s)`. The user asked for a mutation and was handed a clean bill of
  health instead.

  The write error was discarded by an `if let Ok(())`. It is now reported with the path and
  the OS error, and the command exits non-zero. The same applies to a spec `--fix` could not
  read, which was previously skipped without a word.

  `--fix --dry-run` on an unwritable spec still exits 0, because it writes nothing and so
  fails at nothing.

- **A scaffolded spec's sections can now be changed through the lifecycle**
  (CorvidLabs/spec-sync#564). `specsync scaffold` writes `### Structs & Enums`,
  `### Traits`, and `### Functions` inside `## Public API`, and `### Consumes` inside
  `## Dependencies`. The delta grammar treated *every* `###` line as an item heading, so
  `specsync change approve` rejected the section the tool had just generated:

  ```
  error: invalid delta item heading `### Structs & Enums`
  ```

  That put the second half of the SDD loop out of reach for the section that matters most —
  the one describing the public contract — with no hint that a section body may not contain
  `###` at all.

  The grammar identifies its own items by keyword (`REQUIREMENT`, `SPEC SECTION`), so depth
  was never what distinguished them. A `###` that is not a known item heading, met while
  inside an item, is now section content. Before any item it remains an error, and that
  error now names the two valid forms. Fixing the parser rather than the generator repairs
  existing projects too, not only newly scaffolded ones.

- **`stale` no longer reports every spec up to date in a repository with no commits**
  (CorvidLabs/spec-sync#558). Staleness is decided against git history, and an unborn `HEAD`
  has none — nothing can be newer or older than a history that does not exist. The command
  reported `✓ All specs are up to date with their source files` and exited 0 having compared
  nothing.

  That state is not exotic: it is exactly what `git init` plus `specsync init` produces,
  which is where the quick start begins, and it is common in CI for a freshly created
  repository. The no-repository case was already handled correctly, so "no history to work
  with" had been considered — an unborn `HEAD` simply was not recognised as an instance of
  it. Both now report the same way, and distinguishably: `Repository has no commits` and
  `Not a git repository`, in text and in the machine-readable payload.

- **`check --strict` no longer exits 0 on a tree with source and zero specs**
  (CorvidLabs/spec-sync#560). A project with real source and an empty `specs/` passed
  strict validation silently, and the coverage footer `check` prints in every other path
  was omitted entirely — so a CI log carried no figure at all, while `coverage` on the
  identical tree reported 0%.

  The branch already carried a comment stating that `--require-coverage`,
  `--enforcement enforce-new`, and `--strict` must all gate here. The first two did.
  `--strict` did not, because it escalates *warnings* and a project with no specs produces
  none.

  The coverage line is now printed unconditionally rather than only when a gate already
  failed, `--strict` gates when the tree has source files, and the JSON payload carries
  `total_source_files` and `coverage_percent` so `specs_checked: 0` can be told apart from
  an empty project. Confined to trees that have source: an empty project, or one whose
  specs simply have not been generated yet, still exits 0.

- **`check` no longer prints green result lines for checks that could not run**
  (CorvidLabs/spec-sync#553). When a spec's frontmatter failed to parse, validation still
  reported `✓ All source files exist`, `✓ All DB tables exist in schema`,
  `✓ All required sections present`, and `✓ All dependency specs exist` — for a spec
  missing six of the eight default required sections.

  Each line inferred success from the *absence of errors in its category*, and unparseable
  frontmatter yields no `files:`, no `db_tables:`, no `depends_on:`, and no section
  validation at all, so every category was empty for want of input rather than for want of
  problems.

  The section line was not merely vacuous but **false**: the identical body with valid
  frontmatter reports five missing sections. The DB-table line hid behind a differently
  shaped guard — it tested whether the *project schema* had tables, not whether the spec's
  `db_tables:` could be read. All four are now reported as skipped, reusing the
  `⊘ … skipped` wording the draft path already used. Exit status is unchanged: invalid
  frontmatter was an error before and remains one.

- **`deps` no longer reports a malformed dependency graph as valid**
  (CorvidLabs/spec-sync#550). Frontmatter that `check` rejects with
  `depends_on must be a YAML list, got a mapping` was parsed by `deps` into an *empty*
  dependency list, contributing no edges — after which the command affirmatively printed
  `✓ All dependency declarations are valid` and exited 0. Two commands, one tree, opposite
  verdicts, and the wrong one came from the command specifically about dependencies.

  The parser already returns those errors; `deps` discarded them. It now reports them with
  the same wording `check` uses, so the two agree. Two further silent drops in the same
  walk are now reported as well: a spec whose frontmatter cannot be parsed at all, and one
  declaring no `module`. Both were skipped without a word, so a project could carry an
  unparseable spec indefinitely with `deps` green and edges missing from both the graph and
  the computed build order.

- **`specsync init` no longer leaves a project failing its own coverage gate.** Initialization
  writes `.specsync/config.toml`, `.specsync/version`, and `.specsync/sdd.json` — all three are
  protected SDD paths, so the first commit after `specsync init` was reported as uncovered
  meaningful delivery (`meaningful changed paths are not covered by an active change`) on a gate
  no change workspace could satisfy, because none existed when the files were written. `init` now
  records what it created in a committed `.specsync/bootstrap.json`, and the coverage gate honours
  that record.

  The exemption is pinned, not blanket: a recorded path counts only while it is a protected SDD
  path, is absent at the delivery comparison base (so only *creation* is ever exempt, never a
  modification of an already-tracked policy file), has a recorded base commit that is an ancestor
  of `HEAD`, and still matches its recorded digest. The policy digest covers the enforcement
  surface rather than the file bytes — filling in `verification_commands`, which `init` asks you
  to do, keeps the exemption, while changing `enabled`, `require_change_for_meaningful_files`,
  `meaningful_paths`, or `ignored_paths` revokes it. `change adopt`'s existing single-path
  bootstrap record is honoured through the same, now stricter, validator.

- **The coverage gate no longer breaks in a one-commit repository.** With no active change to
  supply a base commit, the gate fell back to `HEAD~1...HEAD`; in a repository whose first commit
  is also its only one that does not resolve, and a dirty tree — exactly the state `specsync init`
  leaves behind — produced `error: unable to inspect changed paths for SDD coverage`. The fallback
  now resolves to `HEAD`: there is no earlier commit to review, so the committed delivery is empty
  and the working tree is what needs coverage.

- **The coverage-gate failure names the way out.** `meaningful changed paths are not covered by an
  active change` now prints a runnable `specsync change new … --path …` line carrying every
  reported path, names the `--no-spec-change --rationale` variant, and — when a reported path
  matches a configured `ignored_paths` entry — states why the entry did not apply: protected SDD
  policy files and the configured specs tree are always meaningful and cannot be ignored away.

- **A quoted path in frontmatter is no longer taken literally**
  (CorvidLabs/spec-sync#545). `files:` containing `- "src/alpha.rs"` — valid YAML, and the
  documented answer for a path with a space in it — resolved to the literal path
  `"src/alpha.rs"`, reported `✗ Source file not found`, and then cascaded into a bogus
  `✗ Spec documents 'one' but no matching export found in source`, because the file had
  never been opened. Frontmatter was still reported `✓ Frontmatter valid`, so nothing
  pointed at the quotes.

  Flow-style lists (`files: ["a", "b"]`) already unquoted; block lists and plain scalars
  did not. The fix closes that asymmetry at the parse layer, so it covers `files:`,
  `depends_on:`, `db_tables:`, and every scalar — `status: "active"` had the same defect.
  An opening quote with no match is now a frontmatter error rather than a silently
  retained literal. The hash cache's separate `files:` extractor was unquoted to match;
  otherwise it would key entries on a path no file exists at.

- **A `draft` spec whose source is present no longer passes `--strict` without being
  validated** (CorvidLabs/spec-sync#547). `status: draft` skips section and export
  validation, so a spec whose Public API documented a function that existed nowhere still
  reported `1 passed`, `File coverage: 100%`, `"passed": true`, and exit 0. Since
  `specsync generate` writes new specs as draft, that was the day-one state of an adopting
  project.

  The two meanings of `draft` are now separated. A draft whose files **do not exist yet**
  is spec-first authoring — the spec is deliberately written before the code, nothing could
  have been validated, and it still passes `--strict` exactly as before. A draft whose
  files **are present** now emits a warning, so bare `check` stays exit 0 and `--strict`
  gates on it.

- **`check` no longer invents drift warnings on a cold cache**
  (CorvidLabs/spec-sync#548). `.specsync/hashes.json` is untracked, so a fresh clone has no
  baseline, and an absent cache entry was classified as "changed" — correct for deciding
  what to re-validate, wrong for telling a person something drifted. A fresh clone of a
  33-spec project printed 33 × `requirements changed — spec may need re-validation`, none
  of them real, and CI always starts cold. Drift is now reported only against a baseline
  that actually exists; selection is unchanged, so the same specs are still re-validated.

  Note for anyone who noticed `check --strict` reporting *fewer* warnings than bare
  `check`: that was this, not `--strict` suppressing drift. `--strict` re-validates every
  spec and so never consulted the classification that produced the phantom warnings.

- **The coverage-gate remediation no longer emits a wall of flags.** One `--path` per
  changed file on a single line reached over 8000 characters on a wide branch. It now
  spells out the first twelve and names the remainder, suggesting a covering prefix.

- **`specsync scaffold` output no longer fails `specsync check`.** Scaffolding a module emits
  placeholder sections, and the effective-contract gate treated every unfinished section as a stub
  warning — fatal under `--strict`. The tool's own output failed the tool's own gate, on the second
  command of the quick start. The exemption is keyed to **authorship, not content shape**: a
  generated section no active change has authored no longer gates, while a section an active change
  authored and then emptied stays fatal, through both the pending and the applied delta paths.
  Unknown authorship fails closed and exempts nothing, and suppressions are reported as warnings
  rather than dropped silently.

- **A directory in a spec's `files:` block is now an error instead of silent success**
  (CorvidLabs/spec-sync#472). A directory mapping extracts zero exports, so the Public API
  comparison had nothing to compare and passed: `specsync check --strict --force` exited 0 with
  zero warnings while measuring nothing — a green result indistinguishable from a real one. Filed as
  Kotlin-specific; it was language-independent. Validation now reports
  `Source file <path> is a directory — files: must list source files, not directories`, with a fix
  naming the source files beneath it, expanded by the same rule `generate` and `scaffold` apply to a
  `[modules."x"] files` directory, so the remedy matches what generation would have written.

  The snapshot validation path had the same defect in a more misleading form: it refused the
  directory but reported it as an out-of-root **security escape**, telling authors their confined
  path had left the project root. Both paths now name the real cause. Symlink and reparse-point
  rejection is unchanged and still evaluated first.

- SpecSync 6.0 reopen/finalize paths address stranded accepted-record deadlocks reported via CorvidLabs/spec-sync#481 (fledge PR CorvidLabs/fledge#506 worked around the 5.x strand by manual archive).

- **Draft `next_action` no longer recommends approve while artifacts are incomplete** — when the interview is done but selected artifacts still contain `<!-- TODO -->` stubs (or are empty), status/show guidance prefers completing those artifacts first (sandbox #16).
- **Approve rejects `## ADDED` of living requirement IDs** — definition approval and delta validation fail closed with a `## MODIFIED` remediation hint when the requirement already exists in the living tree (sandbox #14).

### Notes

- Private CorvidLabs/spec-sync-sandbox dogfood for workflow-v2 adopt is recorded in sandbox scenario 019; tagging 6.0.0 remains gated on this PR.

## [6.0.0] - 2026-07-29

### Added

- **One guided change workflow for SpecSync 6.0** — `change status` always names one next action,
  `change check` applies approved deltas and runs affected-component verification, one independent
  scoped PR review binds the implementation, and `change finalize` creates the dated archive in the
  same PR without merging externally.
- **Additive strict validation** — global `--strict`, project policy, and deterministic
  release/security classification add validators to the same workflow/evidence instead of
  selecting another lifecycle, approval count, or artifact layout.
- **Positive archive-only CI and merge binding** — a lightweight child lane proves parent checks,
  exact archive shape, unchanged delivery tree, ownership, review, and finalization digest; a
  post-merge job records a compact check/comment bound to the actual merge commit before release.

### Changed

- **Low-churn lifecycle evidence** — ordinary work uses one human scope approval and one scoped
  review. Historical two-approval evidence and repair commands remain readable without re-signing.
- **Targeted lifecycle reads** — invocation-scoped snapshots, bounded Git/evidence memoization,
  stable graph ordering, and one-pass owner batches remove the modest-scale performance cliffs.
- **Safer installation and parsing UX** — Git hook installation honors effective Git hook paths
  and project-keyed managed blocks; generated agent artifacts preserve customizations; ignore and
  schema diagnostics fail visibly; TypeScript Unicode export identity is canonicalized.
- **MCP now starts read-only and fails closed at its trust boundary** (#414) — mutating tools require
  the explicit `mcp --allow-write` opt-in; read paths are confined beneath the canonical server
  root before filesystem probing; operations use bounded capability-safe snapshots and confined
  writes to resist symlink/junction races; configuration and actual copied input bytes share one
  operation budget; configured source roots remain visible; requests and responses are bounded;
  GitHub issue access requires an explicit configured repository and `GITHUB_TOKEN` instead of Git metadata discovery;
  unavailable Git freshness is reported and scored conservatively; malformed JSON-RPC envelopes and
  resource arguments are rejected; startup root acquisition is identity-bound; root-wide and
  manifest-derived inputs cannot disappear behind snapshot ignores; request IDs are bounded; and
  generation is count/content bounded, staged, synced, and atomically published with identity-safe
  failed-batch file rollback while conservatively retaining ambiguous empty parents; startup captures
  the root handle and identity before canonicalization and rejects any mismatched canonical reopen;
  read-root selection and generation rollback stay bound to retained parent capabilities and exact
  filesystem identities even when ambient paths are replaced; generated-file identity also binds
  exact staged bytes so immediate Unix inode reuse cannot authorize a replacement, with fail-closed
  hashing bounded at the generated-output limit. Generic MCP project files now use no-follow,
  non-blocking, identity-continuous retained reads for both tools and resources, rejecting special,
  linked/reparse-backed, and replacement entries without consuming attacker bytes. Read-root
  components and staged public parents are now reopened as regular no-link directories with
  identity checks; null/fractional request IDs and malformed initialize negotiation are rejected;
  and test helpers require successful MCP process exits before accepting protocol output.
  Selected read-root component routes are revalidated again before successful responses. Every
  post-link destination/public-parent failure cleans the exact quarantined staged identity before
  returning, while generated batches share one transaction-wide root capability instead of
  retaining one additional root handle per output.
- **MCP manifest and issue checks fail closed under adversarial input** — bounded Cargo workspace
  discovery uses real TOML, while shared checked Gradle discovery handles Groovy/Kotlin comments,
  escapes, includes, and supported project directories; malformed discovery is inconclusive for
  gates. Cargo snapshot paths come only from semantic target, workspace, and dependency tables;
  unrelated metadata `path` keys are ignored. Manifest-relative sibling paths such as `../b` and
  confined Windows-native forms such as `..\b` remain valid when normalization keeps them beneath
  the retained server root, while drive, UNC, rooted, traversal, symlink, and junction escapes
  still fail. Private quarantine cleanup
  consumes its final retained directory capability before removal so Windows does not turn
  successful init/generation into sharing-violation failures. Manifest discovery shares the 64 MiB
  input budget and snapshots exact preflighted bytes. Every present Gradle filename, including a
  lower-precedence shadowed variant, is preflighted and identity-bound across its retained read;
  invoked unsupported inclusion APIs fail closed without rejecting unrelated control flow. CLI
  checked coverage shares one retained project authority across caller-selected spec mappings,
  every recognized manifest/workspace probe, spec-module enumeration, and source discovery,
  applying iterative 8 MiB/file, 64 MiB total, 100,000-entry, and 256-component bounds with strict
  UTF-8. Root retention precedes configuration and omitted-source autodetection, while explicit
  source roots avoid unrelated autodetection. Nested config/manifest parents remain reachable,
  selected-spec inventory identities remain authoritative through ownership parsing, and shared
  spec/source bytes plus entries are bounded. Cargo member declarations and Node workspace
  patterns consume bounded expansion work before deduplication and reuse normalized completed
  results in both manifest and MCP-specific traversal. Retained Cargo and Node manifests are parsed
  structurally and workspace directory listings remain identity-bound through child consumption.
  Selected source directories are retained before the post-manifest checkpoint, selected MCP
  configuration parents are revalidated through their complete edge chain after bounded reads, and
  authority-bearing recursive snapshot directories reject regular-directory replacement. Recursive
  MCP and checked-coverage traversal records sibling identities before opening children
  sequentially, bounding live handles by depth; Node workspace discovery likewise consumes
  identity-matching child capabilities without swap/read/restore mixing. Object-form Node
  workspaces require `packages`, and every recognized nested package manifest is strictly parsed.
  Separate
  early and post-discovery checkpoints protect the checked-coverage operation and propagate failures
  to gate callers. The root dispatcher preserves the caller-requested spelling for those gates so
  eager canonicalization cannot hide a symlink/junction replacement; generation retains that
  authority through publication so a redirect after checked coverage cannot redirect output.
  Hosted Tarpaulin executes the
  unchanged suite with one harness thread to avoid reproducible overlapping ptrace trap crashes
  while retaining the 50% threshold.
  Command-wide immutable CLI snapshots and generic structured discovery outcomes remain deferred
  to later CLI/outcome/generation work outside issue #414; hosted-Windows junction/reparse runtime
  remains required for final acceptance.
  Issue reads, listing, and verification require `GITHUB_TOKEN`, use in-process GitHub REST, and
  never spawn a `gh` provider process; `gh` remains only the explicit issue-creation write path.
  Verification globally deduplicates/caps IDs, includes repository preflight in the complete batch
  deadline, and revalidates repository access after apparent absence. Inaccessible repositories,
  authentication failures, timeouts, and malformed responses are inconclusive rather than
  not_found; issue-list pages above 100 raw provider entries fail before item parsing. All-error
  CLI batches report their error count instead of claiming no references, and CLI/MCP issue scans
  now fail inconclusive with content-free path attribution when a discovered spec is unreadable or
  has malformed/missing frontmatter. A maintained `serde-saphyr` checked parser validates complete
  real-YAML frontmatter for issue references: duplicate keys or malformed YAML anywhere and
  blank/null/wrong-shaped known fields fail closed, comments and valid trailing commas work, and
  only top-level `implements`/`tracks` lists are authoritative while nested extension/block-scalar
  lookalikes are ignored. Recursive traversal and non-UTF-8 filename failures cannot silently
  disappear. CLI `specs_dir` is confined beneath the project, and specs are captured through
  retained capability-rooted, same-handle identity checks. The crate-visible
  `validate_spec_content` API lets `issues --create` preserve normal drift validation and issue
  creation against those exact immutable bytes without reopening discovered paths. Configured
  repository syntax is validated even with zero references while Git/provider access stays
  skipped. Hostile diagnostics escape controls, bidirectional formatting, and Unicode line/
  paragraph separators and use valid Markdown code spans; MCP errors expose only bounded relative
  paths and stable content-free reasons. Issue-list `pull_request` markers must be objects when
  present, so explicit `null` rejects the page instead of becoming an issue. Every raw issue or
  pull-request item is validated as open with exact repository/resource/number URL identity, and
  duplicate raw identities fail within/across pages before pull-request filtering.
  Single and batch GitHub imports use the same explicit-token typed REST path, with no authenticated
  `gh` fallback. Batch import follows every valid page, bounded to 100 pages of 100 raw provider
  entries, and fails on oversized or malformed pages, malformed links, duplicate issue IDs, or cap
  truncation rather than returning a partial issue set.

## [5.2.0] - 2026-07-19

### Added

- **Native `specsync migrate 5.0` change-ledger migration** (#396) — backfills the 5.1 reopening
  `stale`/`current` acceptance-input digest fields on 5.0.1-era ledgers, deterministically
  (stale from the embedded prior verification, current from the superseding verification or a
  live manifest-aware recomputation), idempotently, with a verification pass before any write,
  per-change failure isolation, `--dry-run`, and an actionable `check` diagnostic that names the
  migration instead of a raw serde error.
- **Batch `specsync change correct-owner`** (#398) — one transactional invocation appends many
  audited exact canonical owner corrections (repeated `--path`/`--spec`, `--manifest`, or
  `--all-missing` discovery); every entry validates independently and a failed entry leaves the
  ledger untouched, ending the 11–19 sequential-correction loops seen during the Trust rollout.
- **Squash-merged accepted-evidence archival** — `specsync change archive` now trusts any
  in-history commit recording a change as accepted with byte-identical evidence when no
  first-acceptance transition anchor matches, so squash-merged pull requests never block
  archival while the exactly-one-eligible rule stays fail-closed.

### Fixed

- **Adoption-era archived ledgers validate without repair** (#397) — legacy acceptance-manifest
  reconstruction assigns the exact delivery owner to production-source inputs with no canonical
  owner, so 5.0.1-era archived changes (e.g. spec-less repos) pass historical-integrity checks
  while newly signed evidence stays fail-closed.
- **Inert 5.0.1 registry stubs are tolerated** (#399) — a local `registry.toml` with no registry
  `name` and no `[specs]` mappings loads as absent during canonical module path resolution,
  falling back to conventional `specs/<module>/` paths, while invalid non-inert registries still
  fail closed with the established diagnostic.

## [5.1.1] - 2026-07-16

### Changed

- **Verified GitHub Action 5.x promotion contract** — the maintained Action now defaults to
  SpecSync 5.1.1, documentation distinguishes immutable `@v5.1.1` installs from the compatible
  floating `@v5` channel, and release promotion requires exact-version Linux, macOS, and Windows
  smoke checks before advancing the floating ref.
- **Deterministic hosted JavaScript runtime** — Pages, site CI, and VS Code extension CI pin Bun
  1.3.14 instead of resolving the newest Bun tag during every run, with a repository guard that
  prevents the three workflow jobs from drifting apart.

### Fixed

- **Accepted lifecycle evidence remains fail-closed under successors and corrections** — exact
  per-input ownership, append-only correction evidence, deterministic canonical ownership, and a
  committed legacy-baseline ledger prevent stale or ambiguous accepted changes from being treated
  as current while preserving auditable recovery for already-applied changes.
- **Verification freshness and integration identity are consistent** — descendant evidence-only
  commits no longer create false stale results, distinct verifying-to-accepted transitions remain
  required, numeric change ordering works beyond `CHG-9999`, and hosted lifecycle checks evaluate
  the exact pull-request head before integration.
- **Strict checks are fast, private, and portable** — Git history inspection is bounded, sensitive
  command output is not written to logs, and Git/path handling covers Windows CRLF, literal
  pathspecs, invalid filenames, and index-only entries without platform-specific failures.

## [5.1.0] - 2026-07-14

### Added

- **Audited correction of accepted interview metadata** — `specsync change correct <id> <field> <value> --actor <human> --reason <text>` can correct the supported `public_contract` and `architecture_risk` classifications without rewriting accepted history. Original answers and evidence remain inspectable in an append-only correction chain, newly required artifacts are added monotonically, and the change must receive fresh definition approval, verification, and closing approval without replaying an already-applied semantic delta.
- **CommonJS export contracts** — `.cjs` modules and CommonJS-style JavaScript now expose statically named `exports.<name>`, `module.exports.<name>`, and top-level object-assignment keys to regex and AST validation without changing existing ESM or TypeScript behavior.

### Fixed

- **Generated create-spec commands stay synchronized** — Claude, Cursor, and Gemini now classify the complete non-flag input regardless of where `--minimal` appears, preserving bare module identifiers and deriving a deterministic slug from free text instead of using its first word. A byte-for-byte installer parity test prevents the checked-in command assets from drifting from their shared templates again (#367).
- **Draft specs can map planned source files** — safe normalized missing paths in `draft` specs now produce non-failing planned-mapping notices and remain outside current file and LOC coverage. Creating the file or activating the spec restores normal validation, while `require_draft_files = true` keeps immediate existence enforcement available for strict repositories.

- **Complete module JavaScript discovery and barrels** — default TypeScript-family source discovery now includes `.mjs` and `.cjs`, so mapped files contribute to real file and LOC totals and uncovered module files correctly fail strict 100 percent coverage. Extensionless export-star targets in module-JavaScript barrels also resolve sibling `.mjs` and `.cjs` modules in regex and AST modes.

## [5.0.2] - 2026-07-14

### Added

- **Explicit extensionless source discovery** — `include_extensionless = true` adds files such as `bin/tool` to coverage and generation scans without changing the established default behavior of omitted or empty `source_extensions`. Extensionless-only and mixed projects now produce non-vacuous strict file and LOC coverage when enabled.
- **Audited recovery for stale accepted changes** — `specsync change reopen <id> --actor <human> --reason <text>` now moves only stale accepted delivery evidence back to `verifying`, preserves the prior verification and superseded closing approval in versioned append-only audit metadata, keeps strict checking red until fresh verification, and requires a new closing approval before returning to `accepted`. Reacceptance does not reapply or version-bump semantic deltas that are already canonical, rejects modified definitions that would otherwise be silently ignored, recognizes squash-integrated acceptance and complete later canonical governance recorded in current history, rejects arbitrary off-history evidence, and returns deterministic change and audit objects through global `--json`.
- **Repository-backed change sequence claims** — protected `.specsync/change-sequence.json` records the latest numeric claim so parallel branches collide during Git integration instead of silently reusing a `CHG-NNNN` sequence. Strict lifecycle checking supports sequences beyond four digits, scans active and archived records together, reports every conflicting full ID and path, rejects acknowledgements containing mutable records, and preserves the exact historical `CHG-0016` accepted/archive collision as an explicit immutable baseline.

### Fixed

- **Lifecycle intent and scope preservation** — change interviews keep acceptance-criteria prose intact unless callers explicitly provide a JSON string array, recursive Cargo verification honors safe `--manifest-path` selection before mutating evidence, and affected spec modules cover only their canonical spec and standard companion files.
- **Section-only semantic changes verify correctly** — Non-removed requirement and spec-section delta items now both satisfy semantic acceptance evidence when observable criteria are present. Requirement IDs still require their own test or declared evidence, and verification now distinguishes missing semantic evidence from a configured command failure.
- **Legacy definition approvals survive the lifecycle schema extension** — false `canonical_applied` values remain absent from new persisted state and deterministic definition serialization, while validation recognizes both the original omitted form and the transitional explicit-false form. Explicit acceptance appends a stable definition approval when it encounters compatible transitional evidence, keeping older contract checkers interoperable without rewriting audit history. Upgrading either active schema-v1 encoding no longer invalidates its existing human approval or verification; reopened and accepted changes still persist true values.
- **Verification cannot recursively re-enter the lifecycle** — direct lifecycle commands are rejected before execution, indirect re-entry carries a process context that fails once, and failed attempts are retained in append-only history while a later corrected retry can become the current successful projection.
- **Canonical successors can govern stale predecessors without deadlock** — an exact later implementing successor must have current definition approval and complete scope; a verifying successor additionally needs fresh passed evidence. Draft, partial, no-spec, failed, stale, or abandoned work never hides unrelated stale acceptance evidence.
- **Semantic deltas honor registry-backed module paths** — acceptance resolves canonical spec and adjacent requirements files from the committed registry, preserves the conventional fallback for unregistered modules, and rejects absolute, traversing, malformed, or escaping paths.
- **Static content participates in coverage** — HTML, HTM, and CSS files are auto-detected and measured by default, so zero-config static projects report real covered files and unmapped content fails requested coverage gates instead of appearing as vacuous success.
- **Strict validation rejects unfinished companion scaffolds** — generated context, requirements, testing, tasks, and design markers now produce artifact-specific path-and-line diagnostics; similar prose and fenced examples remain valid.

## [5.0.1] - 2026-07-11

### Fixed

- **Portable Windows release checksums** — Windows `.sha256` assets now use the same LF-only ASCII record as macOS and Linux, and every packaged checksum is verified byte-for-byte before upload.
- **Public API tables preserve complete extractor symbols** — table parsing previously captured only `\w+`, so documented GitHub Actions paths such as `inputs.config`, `outputs.atlas-enabled`, `permissions.id-token`, and `jobs.deploy-atlas` were truncated or ignored and strict validation reported false drift. The parser now reads the complete nonempty backtick-delimited symbol from the first table cell without imposing a second character allowlist, preserving dots, hyphens, selectors, operators, apostrophes, spaces, Unicode, and other spelling emitted by supported extractors while malformed rows, prose, later-column code, and excluded subsections remain ignored. The immutable `v5.0.0` tag remains unchanged.

### CI

- **Path-aware validation** — true lifecycle archive moves now run classification, strict SpecSync validation, and the stable required gate while source, dependency, workflow, Action, release, site, editor-extension, and unknown-path changes continue to select their full or targeted checks.

## [5.0.0] - 2026-07-11

### Added

- **Full verified SDD lifecycle** — versioned `CHG-NNNN-slug` workspaces move through `draft → approved → implementing → verifying → accepted → archived`, with deterministic interviews, adaptive built-in/custom artifacts, explicit scope, semantic deltas, and equivalent text/JSON clients.
- **Two mandatory human gates** — definition and closing approvals record actor, timestamp, note, and SHA-256 artifact digest. Approved content changes invalidate the gate; no force or emergency bypass exists.
- **Layered requirement traceability** — stable `REQ-<module>-<number>` identities require normative SHALL statements and acceptance criteria, then connect semantic deltas to tests or declared testing evidence.
- **Effective-contract validation** — active implementation is checked against canonical specs plus every approved, non-conflicting delta without prematurely mutating canonical truth.
- **Commit-bound verification and atomic acceptance** — configured test commands run without a shell, evidence is tied to HEAD and the contract digest, accepted deltas update requirements/spec sections, bump versions, and add change-log provenance with rollback on write failure.
- **Native AI-first workflow** — Claude Code, Cursor, Codex, and Gemini skills conduct the deterministic interview, respect human gates, and use the same CLI state machine; Claude, Cursor, and Gemini also receive create-change commands.
- **Guided bootstrap and adoption** — new `init` projects enable SDD and offer agent/first-change setup; existing projects remain unchanged until `specsync change adopt`, which previews policy and OpenSpec/Spec Kit active/canonical import provenance.

### Changed

- **Unified `specsync check` gate** validates active lifecycle state, approval freshness, delta conflicts, effective contracts, and meaningful changed-path coverage before canonical bidirectional validation.
- **Companion files are adaptive** — requirements, research, design, plan, tasks, context, testing, docs, and project templates exist when policy or change risk requires them rather than as empty mandatory ceremony.
- **Agent-native, secret-free generation** — `specsync generate` is deterministic and local. Embedded provider/model selection, API-key and endpoint configuration, automatic source transmission, `corvid-ai`, and the `aiCommand`/`SPECSYNC_AI_COMMAND` shell path are removed. Native agent skills and MCP remain the enrichment boundary.
- **Astro 6 documentation site** — the repo-local site now uses Astro 6.4.6 or newer with compatible MDX/content APIs, closing the five tracked Astro advisories.
- The project and crate version are now 5.0.0; new layouts write a 5.0.0 version stamp and a versioned `.specsync/sdd.json` policy.

### Fixed

- **Rust multi-file module contracts include `pub(crate)` again** — regex and AST scanning now preserve both plain `pub` and crate-visible `pub(crate)` declarations and re-exports across every file listed by a spec, while narrower visibility remains excluded. This fixes issue #334 and restores the 4.7.1 contract.
- **PR comment output is protocol-clean and bounded** — configured verification commands no longer leak their stdout into `specsync comment`, rendered markdown is UTF-8-safely capped, and the mascot workflow cannot exceed Linux argument limits.

## [4.7.1] - 2026-07-04

### Fixed

- **TOML literal (single-quoted) config values are parsed, not silently dropped** — the hand-rolled config reader recognized only double-quoted `"..."` strings, so a valid TOML literal string like `source_dirs = ['lib']` (single-line or a formatter-style multi-line array) was mis-parsed: it scanned a directory named `'lib'` quotes-and-all, found no source files, and reported vacuous `File coverage: 0/0 (100%)` on a green build — the same silent-drop failure class as the multi-line-array and scalar-comment fixes in 4.7.0. Both TOML string kinds are now handled across every scalar and array value: basic `"..."` (escapes decoded) and literal `'...'` (taken verbatim, so a Windows path or regex keeps its backslashes); a `#`, `,`, `[`, or `]` inside either kind is content, not a comment or array structure. Configs that used single quotes are now scanned against their real directories, so `check` reports actual — and possibly failing — coverage where it previously passed vacuously.
- **`deps --strict --format json` no longer emits the human diagnostic note** — the `--strict` failure note (`N dependency warning(s) treated as errors`) introduced in 4.7.0 was written to stderr for every output format, including JSON. It is now suppressed in JSON mode so a JSON consumer gets fully machine-readable output with nothing extraneous to parse around — the offending dependencies are already in the `warnings` array and signalled by the non-zero exit code. Human-readable formats still print the note on stderr, and the `--strict` exit-1 gating is unchanged. Also refreshes the `cmd_deps` spec and its companion docs to match the current `cmd_deps(root, strict, format, mermaid, dot)` signature.

## [4.7.0] - 2026-07-04

> **Upgrade note:** This release closes a family of bugs where enforcement gates silently exited `0` in exactly the states they exist to catch — `check`, `score`, and `coverage` on warm caches or spec-less projects, `deps --strict` on undeclared imports, `lifecycle enforce` configured with the documented camelCase keys, and `--require-coverage` measured against zero source files. After upgrading, CI that passed on one of those false-greens may now correctly fail; each entry below gives the specific remedy. Shallow CI checkouts that run `specsync diff --base <ref>` should set `fetch-depth: 0`.

### Security

- **`add-spec`/`scaffold`/`new`/`wizard` reject path-traversal module names** — the module-creating commands wrote the user-supplied module name verbatim into `<specs_dir>/<name>/<name>.spec.md` and joined it onto source directories with no validation, so a name like `../../PWNED/evil` escaped the project root, wrote the spec file and its companion directory to arbitrary locations on disk, and then panicked (exit 101). A new `validate_module_name` guard now requires the name to be a single path segment, refusing empty names, path separators (`/`, `\`), `.`/`..`, and absolute paths with a clear error and exit 1 before any filesystem write; every module-creating entry point (`add-spec`, `scaffold`, `new`, and the interactive `wizard`) is gated. Legitimate names such as `auth`, `auth-service`, or `user_profile` are unaffected.

### Fixed

- **`migrate` aborts on an unparseable config instead of silently discarding it** — `specsync migrate` converted the legacy config to `.specsync/config.toml` and then deleted the original, but the JSON loader swallowed parse errors and fell back to `SpecSyncConfig::default()`. A single malformed field (such as a trailing comma) therefore wrote a pure-default config, deleted `specsync.json`, and exited 0 reporting success — silently losing settings like `source_dirs` and flipping `enforcement = strict` to `warn`, quietly turning a failing CI gate green, and `--strict` did not catch it. A pre-flight parse check now runs before any mutation: if the JSON config exists but does not parse, `migrate` prints the parse error, leaves the project byte-for-byte unchanged, and exits 1. Unknown keys are still accepted, and the lenient legacy `.specsync.toml` parser is exempt.
- **`hooks uninstall` no longer destroys user content after the managed block** — `hooks install` wrote its instruction block without a closing marker, so `hooks uninstall` deleted everything from the block header down to the next level-1 `# ` heading or end of file, silently discarding any level-2 sections (e.g. `## Deploy Notes`) or prose that followed and erasing the whole file when spec-sync had created it — all while exiting 0. Uninstall now scopes removal precisely: new installs wrap the block in `<!-- specsync:hook:begin -->` / `<!-- specsync:hook:end -->` sentinels and remove exactly between them, pre-sentinel (legacy) installs are matched against their exact known snippet text so the existing installed base is protected, and a heuristic fallback of last resort still refuses to delete a file that has content before the block. CRLF line endings are now preserved so Windows files no longer show a spurious diff.
- **`check` no longer exits 0 without evaluating a requested coverage/enforcement gate** — two paths passed CI in exactly the states a gate exists to catch. With a warm hash cache and no specs to re-validate, `check --require-coverage 90` printed the coverage line and exited 0 where a cold run exited 1, so a pre-commit hook or CI that caches `.specsync/` got a false green. Separately, a project with source but no specs (the default state right after `init`) bypassed every gate — `--require-coverage 100` at 0% coverage and `--enforcement enforce-new` both exited 0, and `--format json` emitted plain text instead of JSON. `check` now evaluates the gate against freshly computed coverage before the warm-cache early-out, and handles the empty-project case itself: it fails when coverage is below the threshold or `enforce-new` flags unspecced files, and emits proper JSON. A bare `check` with no gate still exits 0 in the default `warn` mode. Runs that previously false-passed now exit 1, so any CI or pre-commit relying on that green must raise coverage, add specs, or drop the gate flag.
- **`lifecycle enforce` now honors the documented camelCase config keys** — the hand-rolled TOML reader matched only snake_case, so a `.specsync/config.toml` written with the camelCase names shown in `lifecycle enforce --help` and the README (`maxAge`, `allowedStatuses`, `trackHistory`, plus the guard keys `minScore`/`requireSections`/`noStale`/`staleThreshold`) was silently dropped, leaving the CI gate a no-op that exited 0 even on a wildly overdue draft — anyone who configured it from the docs got a silently green build. Both camelCase and snake_case forms are now accepted as aliases, and an unknown `[lifecycle.*]` subsection (e.g. a typo'd `maxAgee`) now warns instead of being silently ignored. A repo that relied on a camelCase lifecycle config was effectively unguarded and will now fail `lifecycle enforce` on non-compliant specs.
- **Multi-line TOML arrays no longer corrupt `.specsync.toml`** — `load_toml_config` parsed the config line-by-line, so a formatter-style multi-line array (`source_dirs = [` with entries on the following lines) was read as the bare value `[` and collapsed every array key — `source_dirs`, `exclude_patterns`, and the rest — into a single bogus `["["]` entry. specsync then scanned a nonexistent directory named `[`, found zero source files, and reported vacuous 100% coverage on a green build; worse, `migrate` reads and rewrites the config, so it persisted the corruption and destroyed the valid file. The parser now accumulates continuation lines until the array closes, reads the span between the first `[` and the last `]` (tolerating a trailing comment and bracketed globs like `**/[abc]/**`), and strips quote-aware inline `#` comments, so multi-line arrays parse to their real entries while scalar parsing stays byte-for-byte unchanged. Configs that used multi-line arrays are now scanned against their real directories, so a `check` that previously passed on false 100% coverage may now report actual — and possibly failing — results.
- **`score` now honors the `--require-coverage`, `--enforcement`, and `--strict` gates** — these global gate flags were parsed but never wired into the `score` subcommand, so `specsync score --require-coverage 100` printed A/B/C grades and exited `0` even on an under-covered project; any CI job that used `score` as a gate passed silently green. `score` now computes file coverage and resolves enforcement (CLI over config) through the same `compute_exit_code`/`exit_with_status` path as `check` and `coverage`, exiting non-zero when a requested gate fails while keeping `--format json` stdout valid. When a gate is requested, `score` no longer takes the no-spec early-exit, so a spec-less project now fails `--require-coverage 100` instead of exiting 0; a plain `score` with no gate flags or an advisory Warn config stays exit `0` and is unchanged.
- **`deps --strict` gates on undeclared-import warnings** — `--strict` was a silent no-op for `deps`: the flag was never passed into `cmd_deps`, and undeclared-import warnings (a module importing another it does not list in `depends_on`) were printed but never affected the exit code, so `deps --strict` exited 0 and CI stayed green despite the violation. Now `deps --strict` exits 1 when any dependency warning is present, appending a `N dependency warning(s) treated as errors` note on non-JSON formats while leaving JSON output valid and gating purely via the exit code. Default `deps` remains advisory (exit 0), and `--mermaid`/`--dot` return before validation so visualization never gates; a project whose imports are all declared still passes under `--strict`.
- **`hooks install --claude-code-hook` no longer clobbers existing Claude Code hooks** — installing the specsync hook blindly overwrote the entire `hooks` object in an existing `.claude/settings.json`, so anyone who already had their own `PreToolUse`, `PostToolUse`, or other hooks configured lost them silently the moment they ran the command. Install now performs a per-event deep merge: events the user lacks are added, an event they already have gets specsync's matcher group appended to the existing array, and other events and unrelated top-level settings (`permissions`, `model`, …) are left untouched. A `hooks` value that is present but not an object is reset to a working object instead of being merged into a non-map, and the existing idempotency guard still prevents a second install from double-appending.
- **`coverage` and `score` no longer bypass enforcement gates on spec-less projects** — on a project with source files but no specs, `coverage` never evaluated its gate: the no-spec early exit returned 0 and the `--format json` branch unconditionally called `process::exit(0)`, so `coverage --require-coverage 100` (text or JSON) reported a green pass at 0% coverage while silently ignoring `--require-coverage`, `--enforcement`, `--strict`, and config-level enforcement. `score` had a narrower leak — its no-spec early exit was gated only on CLI flags, so a config-only `enforce-new`/`strict` was bypassed even though `check` failed correctly. Both commands now compute and report 0% coverage, evaluate the gate, and exit non-zero when enforcement is requested (the JSON path stays valid JSON on stdout), while a `warn` config with no flags still exits 0 as a friendly advisory; `coverage`, `score`, and `check` are now consistent. CI that ran `coverage`/`score` on projects with source but no specs and relied on a green exit will now fail — add specs or relax the requested enforcement/threshold.
- **`migrate` no longer silently drops config data during JSON→TOML conversion** — the converter never serialized `parseMode`, `modules`, or the security-relevant `customRules`, and an omitted `track_history` loaded as `false` instead of its documented `true`, so migrating a 3.x project silently discarded AST parse mode, module groupings, threat-model gates, and history tracking. Because `migrate` deletes the source `specsync.json` (unrecoverable under `--no-backup`, and the default backup is gitignored), the loss was irreversible. `parseMode` and `modules` now round-trip losslessly and `track_history` defaults to `true` to match serde and the docs. Since `customRules` can't be faithfully represented in the hand-rolled TOML, `migrate` now refuses rather than drop it: a preflight exits 1 before any mutation, names the blocking field, and leaves `specsync.json` byte-for-byte intact.
- **Inline `#` comments on scalar TOML config values** — the hand-rolled config reader stripped inline comments only from array values, so a scalar like `specs_dir = "specs" # note` parsed as the literal `"specs" # note`. That mis-resolved the specs directory, hid every spec, and made `check` silently exit `0` on a project it should have validated; every scalar key was affected (`schema_pattern`, `ai_timeout`, and others). The quote- and escape-aware `strip_inline_comment` now runs on scalar values too, dropping the trailing comment while preserving a `#` inside quotes (`"a#b"` stays `a#b`), so specs are discovered and validated again. Projects that leaned on the previous silent pass will now see `check` actually check those specs and may exit non-zero where it used to exit `0`.
- **Export scanner recognizes Swift `final class`, Go grouped `const`/`var`/`type` blocks, Kotlin top-level `const val`, and Rust exports hidden by doc-comment quotes** — four parser gaps made a correctly-documented public symbol report as "no matching export found" (failing `check --strict`, exit 1) while an undocumented one was silently dropped from coverage. The Swift decl regex allowed `static`/`class` but not `final`; items inside grouped Go `const (...)`/`var (...)`/`type (...)` blocks carried no keyword prefix and were skipped; top-level Kotlin `const val` was missing from the modifier chain; and the Rust scanner stripped string literals before comments, so a `"` inside a `//`/`///` doc comment (any odd number of quotes) was read as a string opener that swallowed every `pub fn` up to the next real `"` — which broke specsync's own self-check. The scanners now match `final`, capture grouped exported (uppercase) identifiers with brace-depth tracking so struct/interface fields aren't miscounted as top-level exports, accept `const val` (while still excluding `internal`/`private const`), and tokenize Rust `//` and `/* */` comments before strings. Because previously-hidden exports are now detected, a `check --strict` run that passed before may newly flag those symbols as undocumented.
- **Leading UTF-8 BOM no longer breaks frontmatter parsing** — a spec saved with a leading BOM (`U+FEFF`, common from Windows/Notepad and some editors) failed validation with a misleading `Missing or malformed YAML frontmatter (expected --- delimiters)` error and scored 0% file coverage, because the invisible byte sat before the opening `---` and defeated the `^---` frontmatter anchor even though the delimiters were present. `parse_frontmatter` now strips a single leading `U+FEFF` before matching, so BOM-prefixed specs parse and validate correctly and the returned body is BOM-free. The fix lives at the one choke point, so every caller benefits (validator, scoring, deps, registry, merge); a `U+FEFF` anywhere other than the start is a genuine zero-width no-break space and is preserved.
- **`--require-coverage` no longer passes vacuously when zero source files are found** — coverage was reported as 100% whenever no source files were discovered, so `--require-coverage N` was silently satisfied against nothing measured. A misconfigured project — a wrong `source_dirs`, or an over-broad `exclude_patterns` such as `**/**` — passed its coverage gate in CI while covering no code. `check` now exits 1 when `--require-coverage N` is set with `N > 0` and no source files are found, printing a message that points at `source_dirs` and `exclude_patterns`. A require of `0` (or no flag) still passes, and real projects at 100% with non-zero files are unaffected.
- **Config files with a leading UTF-8 BOM** — a `.specsync/config.toml` or JSON config saved with a leading UTF-8 byte-order mark (`U+FEFF`) was mis-parsed, so a setting the user wrote was silently dropped. In TOML the BOM attached to the first key, which was then discarded as an unknown key (`Warning: unknown key "﻿specs_dir" (ignored)`); in JSON the BOM broke `serde_json` entirely and the config fell back to defaults. Every config-file read now strips leading BOMs before parsing, and the `migrate` command's preflight and `discover_specs` paths route through the same helper — so `migrate` no longer hard-refuses a BOM'd JSON config and no longer silently skips specs under a BOM-obscured custom `specsDir`/`specs_dir`.
- **`diff` now fails loud on a bad base ref** — `diff` only checked whether `git diff` spawned, never its exit status. When git ran but exited non-zero (a bad base ref or a non-git directory) it produced empty stdout, which the command reported as `No files changed since <base>` and exited 0. A comparison that never happened was silently treated as "no drift", so `specsync diff --base <bad-ref>` — even under `--strict` — could green-light a PR in CI that was never actually diffed. The command now inspects `output.status`, surfaces git's stderr with the offending base ref, and exits 1; a valid base ref (with or without changes) is unaffected. Shallow CI checkouts may need `fetch-depth: 0` so the base ref resolves.
- **`check` no longer panics on a `**/**` coverage exclude pattern** — the `**/dir/**` branch of the exclude matcher extracted the inner directory with `&pattern[3..len-3]`, but for the degenerate `**/**` (length 5) the `**/` prefix and `/**` suffix overlapped, producing a reversed byte range `[3..2]` that panicked and aborted the run with exit 1. The matcher now peels the `**/` prefix and `/**` suffix independently, so `**/**` yields an empty directory fragment that matches every path and excludes all source files (coverage `0/0`) instead of crashing. Normal patterns like `**/gen/**` are byte-for-byte unchanged.
- **`is_test_file` misclassified whole projects as tests under a test-named parent directory** — coverage and the generator pass absolute, canonicalized paths, and the check walked *every* path component, so a project living beneath a directory named `test`, `tests`, `spec`, `specs`, `testing`, or `__tests__` (including repos checked out under `test/` in CI) had all of its source files treated as tests and silently excluded. Coverage reported `File coverage: 0/0 (100%)` while the real source went unchecked, and the `--require-coverage` gate then failed loud on the wrongly empty source set. `is_test_file` now takes `root` and bounds the test-directory check to components below it via `strip_prefix`, so ancestors above the project are ignored while in-project test dirs (`src/tests/`, `__tests__/`) and `.test.ts`/`.spec.ts` filename patterns are still excluded.

## [4.6.1] - 2026-07-02

### Security

- **`aiCommand` is now honored only from the `SPECSYNC_AI_COMMAND` environment variable** — it is no longer read from any config file (committed `.specsync/config.toml`/`specsync.json` or the per-developer `.specsync/config.local.toml`). Because `aiCommand` is run via `sh -c`, sourcing it from a repo file let a malicious repository — delivered by `git clone`, a ZIP/tarball download, or extraction into an existing checkout — achieve arbitrary code execution the moment an AI path ran (`generate`, `check --fix`, or the MCP `generate` tool). If you previously set `aiCommand` in config, export it instead: `export SPECSYNC_AI_COMMAND="…"`. Other `ai_*` fields (provider, model, etc.) are unaffected and still load from config.
- **`files:` entries that escape the project root are now rejected** — a spec `files:` path resolving outside the project via an absolute path (`/etc/passwd`), `..` traversal, or an in-repo symlink pointing out was previously read and validated: the out-of-root file counted as covered and its exported identifiers leaked into `check`/`score`/`diff` output, PR comments, and the MCP tools (a hostile-repo information-disclosure vector). Every site that reads a `files:` entry's content now requires it to resolve inside the project root; out-of-root entries are reported as an error. In-root relative paths and in-root symlinks still work.

### Fixed

- **Non-UTF-8 source files no longer pass validation silently** — a file listed in a spec's `files:` that exists but is not valid UTF-8 caused export extraction to yield nothing, so undocumented exports went unchecked and the spec falsely passed. `check` now reports an error naming the file and how to fix it.
- **Incremental `check` re-validates when schema or config files change** — the default cached `check` only re-checked specs whose own tracked files changed, so editing a schema/migration to drop a documented column, or changing the config, was silently skipped as "nothing to validate." Schema-directory files and the config file are now part of the incremental fingerprint, so such changes trigger re-validation. (`check --force` was already correct.)
- **`merge` no longer corrupts or silently drops spec content** — `specsync merge` reported `✓ resolved` while (depending on the conflict shape) deleting the YAML frontmatter fences, deleting the spec body, dropping frontmatter list items, or deleting a table/changelog section. It now auto-resolves only conflict hunks it can merge losslessly — a pure interior frontmatter-field conflict (fences left in the surrounding clean regions) or a pure table/changelog-row conflict — and leaves anything else for manual resolution with the file untouched. Common cases (a `version` bump, a `files:` list extension where the key is in the hunk, unioned changelog rows) still auto-resolve.

## [4.6.0] - 2026-07-01

### Added

- **`specsync agents install`/`uninstall`/`status`** — native skill and slash-command integrations for Claude Code, Cursor, Codex, and Gemini CLI, distinct from the prose-instruction-file mechanism `specsync hooks install` already provides. Installs a `SKILL.md` the tool auto-discovers (all four tools) and a `/specsync:create-spec` slash command (`/specsync-create-spec` on Cursor; Claude, Cursor, and Gemini — Codex's command mechanism is deprecated and global-only, so it gets the skill only). `create-spec` accepts either a bare module name or a natural-language feature description (e.g. `/specsync:create-spec "I want a feature that lets users export their data as CSV"`), defaults to a full scaffold (spec + companion files), and supports `--minimal` to create a spec-only draft instead. Re-running `install` also refreshes any already-installed skill/command file whose content has drifted from the current template, so upgrading spec-sync updates existing installations instead of leaving them stale.

## [4.5.0] - 2026-06-11

### Fixed

- **`specsync new`/`scaffold` auto-detect the source in single-source-file projects** — when no directory or file matches the module name but the project has exactly one non-test source file (the README quickstart's fresh cargo crate with only `src/lib.rs`), that file is used, so `new greeter` produces a spec with real `files:` and pre-populated exports instead of an empty `files: []` that immediately fails validation. When nothing can be detected, `new` prints a warning explaining that the `files:` list must be filled in instead of silently writing a spec that fails `check`.
- **`check --fix` is never a silent no-op** — `--fix` now bypasses the hash cache's unchanged-skip (like `--strict` and `--force` already did). Previously a spec that failed `check --strict` was still recorded in the cache, so a follow-up `check --fix` printed "All specs unchanged", fixed nothing, and exited 0.
- **`check --fix` no longer appends a duplicate export table for symbols a human already documented** — bare API-kind headings under `## Public API` (`### Functions`, `### Methods`, `### Types`, …) are promoted to `### Exported <Kind>` during `--fix` so the existing rows become the recognized export table, and `--fix` skips any symbol that already appears in *any* table within the Public API section.
- **Warning count matches the warnings shown** — the partial "N/M exports documented" summary line is counted as a warning, so it now prints with ⚠ instead of a green ✓ (previously the summary could say "2 warning(s)" while only one ⚠ line was visible).
- **Fresh `specsync init` now creates the v4 layout** — init writes `.specsync/config.toml`, a `4.0.0` version stamp, `.specsync/.gitignore`, and the `lifecycle/`/`changes/`/`archive/` directories (what `specsync migrate` produces) instead of a legacy root-level `specsync.json`, so a brand-new project no longer sees the "Legacy 3.x layout detected" migration nag on its first `check`.
- **`init-registry` respects the v4 layout** — the registry is written to `.specsync/registry.toml` in v4 projects instead of recreating a root-level `specsync-registry.toml` (which re-triggered the legacy nag after migration). Un-migrated 3.x projects keep the legacy path. `load_registry`/`register_module` now resolve the same location via the new `registry::local_registry_path`.
- **Draft specs no longer pass validation silently** — when a draft skips section/export checks (by design), `check` now prints explicit "Section validation skipped (status: draft)" / "Export validation skipped (status: draft)" notices instead of misleading "✓ All required sections present" lines, plus a summary hint: "N draft spec(s) skipped section and export validation — set `status: active` to enable full checks".
- **`check --fix` routes exports to the matching table** — functions/values are appended to the "… Functions"/"… Methods" table and type exports to the "… Types" table (previously everything landed in the last export table, e.g. functions `add`/`subtract` in "Exported Types"). New rows are padded to the target table's column count.
- **`generate` exits non-zero when AI generation fails** — a failed provider call (e.g. missing API key) still falls back to the template, but the failures are re-printed prominently on stderr *after* the check report and the command exits 1 instead of burying the error and exiting 0. JSON output gains an `ai_errors` array; the MCP `specsync_generate` tool reports `ai_errors` too. Both generation entry points now return a `GenerationOutcome` (count, paths, AI errors).
- **`watch` footer no longer contradicts the report** — the footer parses the child check's summary line instead of trusting only its exit code (which is 0 under the default `enforcement = warn`), so "All checks passed!" is never printed beneath a "… 1 failed" summary.
- **Failing checks render negated labels** — a failing frontmatter check now prints "✗ Frontmatter invalid" instead of "✗ Frontmatter valid".
- **`check <name>` with an unmatched spec filter exits 1** — and no longer follows the "No specs matched" warning with a contradictory "No spec files found in specs/" message when specs exist.
- **`--root` pointing at a nonexistent path now errors (exit 2)** — previously the CLI silently exited 0 having checked nothing.
- **Spec scoring no longer false-flags documented HTML-comment syntax** — the `placeholder_free` check strips fenced and inline code before counting `<!-- ... -->`, so a spec that *documents* an HTML-comment directive (e.g. ``a `<!-- specsync-ignore -->` directive``) isn't penalized for showing real syntax.

## [4.4.0] - 2026-06-07

### Added

- **`openrouter` and `ollama` API providers.** OpenRouter joins the OpenAI-compatible family; Ollama now runs over its OpenAI-compatible HTTP endpoint (local server keyless, or Ollama Cloud via `OLLAMA_API_KEY`) instead of shelling out to `ollama run`.
- **`SPECSYNC_AI_PROVIDER` env var** to pick a provider by name (env outranks config — `flag > env > config`, 12-factor), plus `OLLAMA_HOST` support and `-cloud` model routing to Ollama Cloud.
- **`generate --model <id>` flag** and `SPECSYNC_AI_MODEL` env to choose the model (precedence: `--model` > `SPECSYNC_AI_MODEL` > `aiModel` config > provider default), matching fledge.

### Fixed

- **`generate` now uses AI when a provider is configured** — a `aiProvider`/`aiCommand` in config (or `SPECSYNC_AI_PROVIDER`/`SPECSYNC_AI_COMMAND` env) invokes AI without having to repeat `--provider`. The `--provider` flag still overrides config. With nothing configured, `generate` stays template-only.

### Changed

- **AI API calls now go through the shared [`corvid-ai`](https://crates.io/crates/corvid-ai) client.** spec-sync's three hand-rolled HTTP paths (`call_anthropic_api` / `call_openai_api` / `call_gemini_api`, ~250 lines) are replaced by `corvid_ai::complete`. corvid-ai owns the provider registry — endpoints, default models, `<PROVIDER>_API_KEY` resolution, and secret redaction in errors — so the Anthropic default model is now the current `claude-sonnet-4-6` (was `claude-sonnet-4-20250514`). Minimum supported Rust version is now **1.89**.
- **New auto-detect ladder (never auto-selects a CLI), shared with fledge.** With no explicit selection: **none configured → keyless local Ollama** (`http://localhost:11434`) — the most useful zero-config default; **exactly one key → use it**; **multiple keys → prompt** for provider + model when interactive, otherwise the deterministic order (Ollama, Anthropic, OpenAI, OpenRouter, Gemini, DeepSeek, Groq, Mistral, xAI, Together). A set API key beats unkeyed local Ollama (no network probe). Ollama requests honor `OLLAMA_HOST` and `-cloud` routing to Ollama Cloud.

### Deprecated

- **The agentic CLI providers.** `claude` now routes to the `anthropic` API (with a warning) and no longer shells out to `claude -p` — removing the prompt-injection→tool-execution surface; `copilot`/`cursor` warn and are slated for removal in the next major. The `aiCommand` config remains the explicit, trusted shell escape hatch.

### Removed

- `AiProvider::default_model` and `AiProvider::default_base_url` — corvid-ai is now the single source of truth for API endpoints and default models.

## [4.3.5] - 2026-06-07

### Security

- **Auth tokens redacted from import/GitHub error messages** — Jira, Confluence, and GitHub API request failures now strip any verbatim token from the surfaced error as defense-in-depth, mirroring the AI client's sanitization. Tokens travel in `Authorization` headers (not URLs), so this guards against a misbehaving proxy or redirect echoing them back.
- **`git diff` hardened against argument injection** — the drift command passes `--end-of-options` so a user-supplied base ref starting with `-` is always parsed as a revision, never as a git flag.

### Performance

- **Eliminated N+1 git subprocess spawns in staleness checks** — `git_commits_between` re-ran `git log` to resolve the spec's commit for *every* source file. Replaced with `git_commits_since`, which takes a precomputed spec commit so callers (`stale`, `check`, `report`, `scoring`, `lifecycle`) spawn one `git rev-list` per source file instead of an extra `git log` each. For a spec with N source files this drops N+1 `git log` calls to 1.
- **Cached spec-scoring regexes** — `count_placeholder_todos` no longer recompiles its two regexes on every spec scored; they are built once via `LazyLock`.

### Fixed

- **`print_summary` integer underflow** — `total - passed` could panic on `usize` underflow in debug builds when `passed` exceeded `total`; now uses `saturating_sub`.

### Tests

- Added 25 tests covering git utilities (commit resolution and counting edge cases), CLI argument parsing, `ensure_hashes_gitignored` (including the write-failure error path), migration step application, output boundary cases, and the `stale` command outside a git repository.

## [4.3.4] - 2026-06-07

### Security

- **GitHub Action command execution hardened** — marketplace action now builds `specsync` invocations as bash argv arrays instead of shell strings, eliminating `eval` around user-provided `args`.
- **Release checksum verification fails closed** — downloaded release archives now require matching `.sha256` files before extraction.

### Fixed

- **Action input validation** — `require-coverage` is validated as an integer from 0 to 100 before command execution.
- **MCP generated-spec test assertion** — replaced a tautological unsigned comparison with a meaningful generated-spec count assertion.
- **VS Code extension license packaging** — extension package includes the MIT license file so VSIX builds are complete.

### CI

- **Repo-wide validation expanded** — CI now builds/tests/lints the Astro docs site and compiles/packages the VS Code extension.
- **Spec gate requires full coverage** — project spec CI now runs `check --strict --require-coverage 100 --force`.
- **Coverage threshold raised** — tarpaulin minimum coverage increased from 40% to 43%.
- **Fledge tasks expanded** — repository lanes now cover Rust, specs, docs, extension packaging, and audit checks.
- **Known transitive audit warning tracked** — `RUSTSEC-2024-0384` is ignored explicitly while it remains pulled in through `notify`.

### Specs

- **Utility helpers specced** — added a dedicated spec for `src/util.rs`.
- **Companion files completed** — backfilled `testing.md` companions and missing `tasks.md`/`context.md` files for legacy specs.

## [v4.3.3] - 2026-05-18

### Fixed

- **Word boundary added to RAW_STR regexes** — prevents false matches where raw string patterns were incorrectly matching substrings of longer tokens (#262).

### Site

- New Astro-based marketing site at corvidlabs.github.io/spec-sync — replaces the prior mdBook. Includes a Languages registry, examples, blog, and migrated docs.

## [4.3.2] - 2026-04-20

### Fixed

- **Bare `depends_on` module names now resolve correctly** — entries like `run` (no path separator) are resolved under `specs/` instead of the project root, matching the behavior of `deps.rs` (#257, #258).

## [4.3.1] - 2026-04-18

### Added

- **`--fix` dry-run and backup mode** — `specsync check --fix --dry-run` previews changes without writing; `--fix` now creates timestamped backups before modifying specs (#243, #248).
- **Config path in error messages** — validation errors now show which `.specsync/config.toml` is in effect (#244, #248).
- **Large file warnings** — configurable size threshold (default 512 KB) warns when spec files are unusually large (#245, #248).

### Fixed

- **ReDoS protection on custom regex rules** — user-supplied patterns are now checked for catastrophic backtracking before compilation (#240, #247).
- **Deduplicated Levenshtein implementation** — single shared function replaces two near-identical copies (#241, #247).
- **Scoring weight constants** — magic numbers replaced with named constants (#242, #247).
- **Backup error propagation** — `--fix` now aborts if backup creation fails instead of silently continuing (#249, #252).
- **Duplicate size warnings** — single configurable check replaces redundant size validation (#250, #252).
- **`--dry-run` without `--fix` warning** — emits a helpful warning instead of silently doing nothing (#251, #252).
- **Production unwrap elimination** — all `.unwrap()` calls in production code replaced with proper error propagation (#253).
- **Collapsible-if clippy warnings** — resolved across the entire codebase (#253).

### Security

- **Cleartext logging remediation** — sensitive information no longer appears in log output (#238, #239).
- **`time` crate bumped to 0.3.47** — addresses CVE in time crate (#253).

### Changed

- **CI: cargo-audit job** — new CI step using `rustsec/audit-check` catches dependency CVEs on every PR (#253).
- **CI: coverage threshold** — `cargo-tarpaulin` with 40% minimum coverage enforced in CI (#253).

## [4.3.0] - 2026-04-18

### Added

- **`--explain` per-criterion breakdown** — `specsync score --explain` now prints a per-criterion table showing the score, weight, and a one-line rationale for every dimension. Makes it easy to see exactly why a spec lands at a given grade (#234).
- **Stub/TBD depth penalty** — sections whose content is `TBD`, `Coming soon`, or equivalent placeholders are now penalized in the depth score. A spec filled with stubs can no longer score A (#235).

### Fixed

- **Near-miss required headers** — `specsync check` now reports near-miss section headers (e.g. `Overviews` instead of `Overview`) as actionable errors rather than silently missing them (#236).
- **A-grade stub cap** — specs with a high ratio of stub sections are capped below A regardless of other scores (#236).

## [4.2.1] - 2026-04-17

### Fixed

- **Hardened section header matching** — regex anchors, whitespace tolerance, and word boundaries prevent false positives and mismatched headers on sections with leading/trailing spaces or similar names (#232).
- **`--fix` insertion point corrected** — new rows no longer append after non-export subsections on repeated runs; near-miss header detection broadened to catch more variants (#231).

### Security

- **Redact API keys in `Debug` output** — sensitive keys are now masked in debug/trace logs (#230).
- **Bumped `time` and `rustls-webpki`** — addresses upstream advisories in both crates (#230).

## [4.2.0] - 2026-04-12

### Added

- **`testing.md` companion file** — a new default companion file scaffolded alongside every spec. Contains sections for automated test locations, manual QA checklists, and edge cases/boundary conditions. Generated by `specsync generate`, `specsync add-spec`, and `specsync new --full` (#225).
- **`design.md` companion file (opt-in)** — layout, component hierarchy, design tokens, and asset references. Enabled via `[companions] design = true` in `.specsync/config.toml`. Generated alongside other companions when enabled (#226).
- **YAML as source language** — `Language::Yaml` variant added to export extraction. Detects `.yaml`/`.yml` files and extracts top-level mapping keys as symbols (#224).

## [4.1.3] - 2026-04-11

### Fixed

- **`specsync merge` now detects conflicts in all spec `.md` files** — Previously, merge conflict detection only matched `*.spec.md` files, silently skipping `tasks.md`, `requirements.md`, `context.md`, and other markdown files under the specs directory. Now matches any `.md` file in the specs path (#215).

## [4.1.2] - 2026-04-11

### Fixed

- **`specsync comment` now respects `--strict` and `--require-coverage`** — Previously, `specsync comment` hardcoded pass/fail as `total_errors == 0`, ignoring strict mode, enforcement level, and coverage requirements. PR comments could show "✅ Passed" even when `specsync check --strict` correctly failed with exit code 1. Now uses the same `compute_exit_code()` logic as `check` (#213).

## [4.1.1] - 2026-04-11

### Fixed

- **Unified comment and check validation pipelines** — `specsync comment` now uses the same `run_validation()` pipeline as `specsync check`, ensuring identical output. Previously, `comment` skipped `.specsyncignore` rules, inline `specsync-ignore` directives, and staleness checks (#209).
- **Stripped ANSI codes from PR comments** — CI comments no longer contain color escape sequences from cargo build output (#209).
- **Marketplace action now uses `specsync comment`** — the GitHub Action (`action.yml`) now uses `specsync comment` instead of `specsync diff --format markdown` when `comment: true` is set, producing identical PR comment output to the project's own CI workflow.
- **Fixed YAML parse error in `action.yml`** — unindented `---` inside a block scalar was treated as a YAML document separator, breaking the action. Comment body now uses `printf` instead of a raw heredoc (#211).

### Added

- **YAML validation CI step** — `action.yml` is now validated with `python3 -c "import yaml; yaml.safe_load(open('action.yml'))"` in CI to catch parse errors before release (#211).

## [4.1.0] - 2026-04-11

### Added

- **`specsync rehash` command** — regenerates `.specsync/hashes.json` from scratch without running validation. Useful after manual spec edits or when the hash cache is stale (#208).
- **Auto-gitignore hashes.json** — `specsync init` and `specsync migrate` now automatically add `.specsync/hashes.json` to `.gitignore`. The hash cache is a local artifact and should not be committed (#208).
- **Force hash rebuild in CI** — GitHub Action now runs `specsync rehash` before `specsync check` to ensure CI always validates against fresh hashes, not a stale cache (#208).

### Changed

- **hashes.json removed from version control** — `.specsync/hashes.json` is no longer tracked in git. Existing tracked copies are removed during migration (#208).

### Documentation

- **Updated all docs for v4.0.0** — CLI examples, config paths, GitHub Action usage, quickstart, and architecture docs now reflect the `.specsync/` directory structure and v4 commands (#206).

## [4.0.0] - 2026-04-11

### Breaking Changes

- **Directory restructure** — all spec-sync metadata moves into `.specsync/`: config, registry, lifecycle history, change records, and archives. Root-level `specsync.json` and `specsync-registry.toml` are relocated automatically by `specsync migrate`.
- **Config format change** — `specsync.json` is converted to `.specsync/config.toml` (TOML). Legacy JSON/TOML files at the root still work as fallback.
- **`lifecycle_log` removed from frontmatter** — lifecycle history is extracted from spec YAML frontmatter into `.specsync/lifecycle/*.json` files. The `lifecycle_log` field is removed from specs during migration.
- **GitHub Action version** — update workflows from `@v3` to `@v4`.

### Added

- **`specsync migrate` command** — automated 3.x → 4.0.0 migration with 10 steps: version detection, backup, directory creation, config conversion, registry relocation, lifecycle extraction, frontmatter cleanup, gitignore, cross-project ref scanning, and version stamping. Supports `--dry-run`, `--no-backup`, `--format json`. Idempotent and safe to re-run (#198).
- **Full spec lifecycle management** — `specsync lifecycle` subcommands: `status`, `promote`, `demote`, `set`, `history`, `guard`, `auto-promote`, `enforce`. Specs track lifecycle stages (draft → review → stable → deprecated → archived) with configurable transition guards.
- **Lifecycle enforcement in CI** — `specsync lifecycle enforce --all` validates lifecycle rules in CI. Available via GitHub Action with `lifecycle-enforce: 'true'`.
- **Change records** — `.specsync/changes/` directory for tracking spec modifications over time.
- **Spec archival** — `.specsync/archive/` directory for retired specs. Archive contents are version-controlled (not gitignored).
- **Migration backup** — `.specsync/backup-3x/` with timestamped manifest preserves original 3.x files for rollback.
- **Cross-project reference scanning** — migration detects `depends_on` refs to external repos and records them in `.specsync/cross-project-refs.json`.

### Fixed

- **Archive not gitignored** — `.specsync/archive/` is no longer excluded from git. Users who want to remove archived specs can delete them explicitly (#202).

### Documentation

- **MIGRATION.md** — comprehensive upgrade guide with breaking changes, step-by-step instructions, and FAQ.

## [3.8.0] - 2026-04-10

### Added

- **Staleness detection** — new `specsync stale` command identifies specs that haven't been updated since their source files changed. Also available via `specsync check --stale` (#189).
- **AST-based export parsing** — tree-sitter powered export extraction replaces regex-based parsing for more accurate and reliable results across all supported languages (#192).
- **Batch operations** — `specsync import --all-issues` and `--from-dir` for bulk import; `specsync score --format table|csv` for tabular output; `specsync generate --uncovered` and `--batch` for generating specs in bulk (#191).
- **Declarative custom validation rules** — define project-specific validation rules in config that are checked alongside built-in rules (#190).
- **Cross-repo spec content verification** — `specsync resolve --verify` fetches and validates referenced specs from remote repositories, ensuring cross-project refs point to real, valid content (#159, #195).
- **MCP resource support** — agents can browse the spec tree via 5 new MCP resources (`specsync:///specs`, `specsync:///specs/{module}`, etc.) without knowing file paths (#194).

### Fixed

- **Requirements convention docs** — clarified that requirements belong in companion `requirements.md` files, not inline in specs (#163, #193).

## [3.7.0] - 2026-04-10

### Added

- **`--no-cache` flag** — discoverable alias for `--force` that skips the hash cache (#178).
- **Cache location hint** — when specs are skipped due to caching, the path to `.specsync/hashes.json` is printed so users know where the cache lives (#178).

### Fixed

- **Absolute paths in error messages** — "No spec files found" now shows the full resolved path, making it immediately clear if you're in the wrong directory (#177).

### Changed

- **Clearer help text for spec filters** — `check` and `score` help now documents all four matching modes: module name, filename stem, relative path, and absolute path (#179).

### Closed

- **`--json` output for `score`** — already supported via the global `--json` / `--format json` flags since v3.5.0 (#172).

## [3.6.2] - 2026-04-09

### Fixed

- **`specsync diff` in PR context** — auto-detects `GITHUB_BASE_REF` in GitHub Actions so diff compares against the PR base branch instead of `HEAD` (the merge commit), which previously always reported "No files changed" (#180).

### Changed

- **Strict spec enforcement** — spec-sync now dogfoods its own `--enforcement-mode=strict` in CI, catching spec drift in the tool itself (#182).
- **100% spec file coverage** — added specs for all 62 source files (26 new spec modules), up from 58% (#183).

## [3.6.1] - 2026-04-08

### Fixed

- **`specsync new` frontmatter formatting** — `files:` and `db_tables:` fields no longer merge onto one line when source files are auto-detected (#174).
- **Empty dependency graph hint** — `specsync deps --mermaid` and `--dot` now print a helpful message when no `depends_on` relationships exist, instead of rendering only disconnected nodes (#174).

## [3.6.0] - 2026-04-08

### Added

- **Individual spec path filtering** — `specsync check` and `specsync score` now accept spec file paths or module names as positional arguments, allowing validation/scoring of specific specs instead of the entire project (#170).
- **Dependency graph visualization** — `specsync deps --mermaid` and `specsync deps --dot` output the dependency graph as Mermaid flowchart or Graphviz DOT diagrams for documentation and debugging (#152).
- **`specsync new` command** — quick-create a minimal spec with auto-detected source files and pre-populated exports. Use `--full` to also generate companion files (tasks.md, context.md, requirements.md) (#151).


## [3.5.0] - 2026-04-08

### Added

- **Stub/placeholder detection** — sections containing only "TBD", "N/A", "TODO", "Coming soon", or similar placeholders are now flagged as warnings and no longer inflate quality scores (#162).
- **Source-attributed export warnings** — undocumented export warnings now show which source file the export comes from, making them actionable in large codebases (#165).
- **Requirements companion validation** — warns when specs contain inline requirements sections (should be in `requirements.md`) and when companion files are missing (#163).
- **Score diagnostics** — `specsync score` now shows per-category breakdowns (completeness, structure, cross-references) with actionable improvement suggestions (#167).

### Fixed

- **Header matching flexibility** — fuzzy matching for common header variations like "Public API" → "Exports", "Tech Stack" → "Dependencies", reducing false negatives (#166).
- **Frontmatter parser edge cases** — correctly handles tabs, trailing whitespace, and inline YAML comments in spec frontmatter (#161).
- **`--fix` header renaming** — near-miss headers are now renamed in-place instead of duplicating the section (#164).

## [3.4.1] - 2026-04-07

### Fixed

- Added 6 missing `depends_on` entries to CLI and validator specs, resolving all `specsync deps` warnings.

## [3.4.0] - 2026-04-07

### Added

- **`specsync scaffold` command** — enhanced module scaffolding with auto-detected source files, custom template directories, and automatic registry registration (#138).
- **`specsync deps` command** — cross-module dependency graph validation detecting cycles, missing deps, and undeclared imports (#139).
- **`specsync comment` command** — post spec-sync check summaries as actionable PR comments with spec links, or print for piping (#140).
- **`specsync changelog` command** — generate changelogs of spec changes between two git refs (#141).
- **`specsync report` command** — per-module coverage report with stale and incomplete detection.
- **Graduated enforcement mode** — new `--enforcement-mode` flag with three levels: `warn` (default), `enforce-new` (errors only for new specs), and `strict` (all warnings are errors) (#134).
- **External importers** — `specsync import` supports GitHub Issues, Jira, and Confluence as spec sources (#123).
- **Interactive wizard** — `specsync wizard` for step-by-step guided spec creation (#122).
- **167+ new unit tests** across config, parser, validator, generator, and export modules.
- **100% spec coverage** — resolved 9 undocumented export warnings and added 3 missing specs.
- **Community scaffolding** — CONTRIBUTING.md, CODE_OF_CONDUCT.md, issue/PR templates.
- **Standalone workflow guide** and onboarding documentation.

## [3.1.0] - 2026-03-30

### Added

- **`requirements.md` companion file** — a new per-module companion file scaffolded alongside `tasks.md` and `context.md` by `specsync generate` and `specsync add-spec`. The template includes User Stories, Acceptance Criteria, Constraints, and Out of Scope sections. This keeps the spec focused as a technical contract (authored by Dev/Architect) while giving Product/Design their own space for user stories and acceptance criteria.
- **AGENTS.md hook target** — `specsync hooks install --agents` installs spec-sync instructions into `AGENTS.md`, the emerging standard for multi-agent instruction files.

## [3.0.0] - 2026-03-30

### Added

- **VS Code extension** — first-class editor integration for SpecSync, published on the VS Code Marketplace as `corvidlabs.specsync`.
  - **Inline diagnostics** — errors and warnings from `specsync check --json` mapped directly to spec files with proper severity levels.
  - **CodeLens quality scores** — spec quality scores (0–100 with letter grades) displayed inline above spec files via `specsync score`.
  - **Coverage webview** — rich HTML report showing file and LOC coverage with VS Code theme-aware styling.
  - **Scoring webview** — detailed quality breakdown per spec with improvement suggestions.
  - **Five commands** — Validate Specs, Show Coverage, Score Quality, Generate Missing Specs, Initialize Config — all accessible from the Command Palette.
  - **Status bar indicator** — persistent status bar item showing pass/fail/error/syncing state with color coding.
  - **Validate-on-save** — debounced (500ms) automatic validation when spec or source files are saved.
  - **Configurable settings** — `specsync.binaryPath`, `specsync.validateOnSave`, `specsync.showInlineScores`.
  - Activates automatically in workspaces containing `specsync.json`, `.specsync.toml`, or a `specs/` directory.

### Breaking Changes

- Major version bump to v3. GitHub Action users should update to `CorvidLabs/spec-sync@v3`.

## [2.5.0] - 2026-03-30

### Added

- **Schema column validation** — SpecSync now parses SQL migrations (CREATE TABLE, ALTER TABLE ADD COLUMN) and validates documented columns in spec `### Schema` sections against the actual database schema. Catches phantom columns (documented but missing from schema), undocumented columns (in schema but not in spec), and column type mismatches. Opt-in via `schema_dir` in `specsync.json`.
- **Destructive DDL support** — migration parser correctly handles DROP TABLE, ALTER TABLE DROP COLUMN, ALTER TABLE RENAME TO, and ALTER TABLE RENAME COLUMN, ensuring the schema map accurately reflects state after all migrations replay in order.
- **Multi-language migration files** — schema extraction now supports 16 file types (SQL, TypeScript, JavaScript, Python, Ruby, Go, Rust, PHP, Swift, Kotlin, Java, C#, Dart, and more), not just `.sql`.
- **PHP language support** — full export extraction for PHP: classes, interfaces, traits, enums, public functions/constants, with visibility filtering and magic method exclusion.
- **Ruby language support** — full export extraction for Ruby: classes, modules, public methods with visibility toggle tracking, `attr_accessor`/`attr_reader`/`attr_writer`, constants, and `=begin/=end` comment handling.
- Expanded export parser test coverage for Go, Python, Java, C#, and Dart.
- Achieved 100% spec coverage across all modules.

## [2.4.0] - 2026-03-28

### Changed

- **Export validation uses allowlist** — only `### Exported ...` subsections under `## Public API` now trigger export validation. Non-export subsections (`### API Endpoints`, `### Route Handlers`, `### Component API`, `### Configuration`, etc.) are treated as informational and skipped. This fixes false errors when specs document private route handlers, component signals, service methods, or infrastructure concepts alongside validated exports (#60).

## [2.3.3] - 2026-03-28

### Documentation

- **Companion files populated** — all 28 companion files (`context.md` and `tasks.md`) across 14 modules now contain real content: architectural decisions, key files, implementation status, open tasks, known gaps, and completed work (#58).

## [2.3.2] - 2026-03-28

### Fixed

- **`action.yml` YAML parse fix** — quoted `${{ github.token }}` default value to prevent YAML stream parse errors when external repos use the action (#56).
- **`spec:check` in CI** — added spec validation to the CI pipeline so spec drift is caught automatically (#54).

### Added

- **`manifest.spec.md`** — spec for the manifest module, achieving **100% file coverage** across all 23 source files (#55).
- **Config spec update** — added `manifest` to config's `depends_on` for accurate cross-module references.

## [2.3.1] - 2026-03-28

### Added

- **`specsync-registry.toml`** — published module registry for cross-project spec resolution. Other projects can now verify refs to `CorvidLabs/spec-sync@<module>` via `resolve --remote`.

### Documentation

- **New docs page: Cross-Project References** — dedicated guide covering `owner/repo@module` syntax, registry publishing, remote verification, and CI usage.
- **CLI Reference** — added missing commands: `add-spec`, `init-registry`, `resolve`, `hooks`. Added `--format` flag documentation.
- **Spec Format** — documented cross-project ref syntax in `depends_on` field.
- **Quick Start** — added `add-spec`, `resolve`, `init-registry`, and `hooks` commands.

## [2.3.0] - 2026-03-28

### Added

- **`--format markdown` output** — `check` and `diff` commands now accept `--format markdown` to produce clean, human-readable Markdown tables instead of plain text or JSON. Useful for pasting into PRs, docs, or chat.
- **SHA256 release checksums** — release workflow now generates and publishes SHA256 checksums for all release binaries, improving supply chain verification.

### Changed

- Rolled up all v2.2.1 changes (manifest-aware modules, export granularity, language templates, robustness fixes) into this release.

## [2.2.1] - 2026-03-25 (unreleased — rolled into 2.3.0)

### Added

- **Manifest-aware module detection** — parses `Package.swift`, `Cargo.toml`, `build.gradle.kts`, `package.json`, `pubspec.yaml`, `go.mod`, and `pyproject.toml` to auto-discover targets and source paths instead of just scanning directories.
- **Export granularity control** — `"exportLevel": "type"` in `specsync.json` limits exports to top-level type declarations (class/struct/enum/protocol) instead of listing every member.
- **Configurable module definitions** — `"modules"` section in `specsync.json` lets you define module groupings with explicit file lists.
- **Language-specific spec templates** — `generate` and `--fix` produce Swift, Rust, Kotlin/Java, Go, and Python templates with appropriate section headers and table columns.
- **AI context boundary awareness** — generation prompt instructs the provider to only document symbols from the module's own files, not imported dependencies.

### Fixed

- **Test file detection** — expanded Swift patterns (Spec, Mock, Stub, Fake), added Kotlin/Java/C# patterns, and detect well-known test directories (`Tests/`, `__tests__/`, `spec/`, `mocks/`).
- **Check command no longer hangs on empty specs** — returns clean JSON/exit 0 when `--fix` is used with no spec files.
- **Exit code 101 panic → friendly error** — wraps main in `catch_unwind`, converts panics to actionable error messages with bug report link.

## [2.2.0] - 2026-03-25

### Added

- **`--fix` flag for `check` command** — automatically adds undocumented exports as stub rows in the spec's Public API table. Creates a `## Public API` section if one doesn't exist. Works with `--json` for structured output of applied fixes. Turns spec maintenance from manual bookkeeping into a one-command operation.
- **`diff` command** — compares current code exports against a git ref (default: `HEAD`) to show what's been added or removed since a given commit. Human-readable by default, `--json` for structured output. Essential for code review and CI drift detection.
- **Wildcard re-export resolution** — TypeScript/JS barrel files using `export * from './module'` now have their re-exported symbols resolved and validated. Namespace re-exports (`export * as Ns from`) are detected as a single namespace export. Resolution is depth-limited to one level to prevent infinite recursion.

### Changed

- Spec quality scoring now accounts for `--fix` generated stubs (scored lower than hand-written descriptions).
- Expanded integration test suite with 12 new tests covering `--fix`, `diff`, and wildcard re-exports (74 total integration tests, 131 total).
- Updated `cli.spec.md` and `exports.spec.md` to 100% coverage for all new features.

## [2.1.1] - 2026-03-25

### Fixed

- **Rust export extractor** — strip raw strings, char literals with `"`, and multi-line string literals before scanning for `pub` declarations. Fixes false positives from test data inside `r#"..."#` blocks, and false negatives where `'"'` char literals confused the string regex into consuming subsequent source code.
- **CLI spec** — added spec coverage for `main.rs` (CLI entry point).
- **Exports spec** — expanded to 100% file coverage across all language extractors.

## [2.1.0] - 2026-03-24

### Added

- **`specsync hooks` command** — manage agent instruction files and git hooks for spec awareness. Supports Claude Code (`CLAUDE.md`), Cursor (`.cursor/rules`), GitHub Copilot (`.github/copilot-instructions.md`), pre-commit hooks, and Claude Code hooks. Subcommands: `install`, `uninstall`, `status`.

### Security

- Updated `rustls-webpki` from 0.103.9 → 0.103.10 to fix RUSTSEC-2025-0016 (CRL Distribution Point matching logic).

### Fixed

- Spec scoring now distinguishes placeholder TODOs from descriptive references (#37).

## [2.0.0] - 2026-03-20

### Breaking Changes

- **`--ai` flag removed** — replaced by `--provider auto|claude|openai|ollama`. Use `specsync generate --provider auto` for auto-detection, or `--provider claude` for a specific provider. Plain `specsync generate` remains template-only.

### Added

- **Cross-project spec references** — specs can now reference modules in other repos via `cross_project_refs` in config. Validated locally with `specsync check`, verified remotely with `specsync resolve --remote`.
- **Companion files** — associate non-code files (migrations, configs, protos) with spec modules via `companion_files` config.
- **Spec registry** — `specsync registry` reads `specsync-registry.toml` to list and discover specs across a project.
- **`specsync resolve`** — new command to resolve cross-project references. `--remote` flag opt-in fetches registry files from GitHub repos.
- **Project scope definition** — `SCOPE.md` explicitly defines what spec-sync does and doesn't do.

### Changed

- Unified AI provider selection under `--provider` flag with auto-detection support.
- Remote ref verification groups HTTP requests by repo to minimize fetches.
- Updated all docs, examples, and tests for the new CLI surface.

## [1.3.0] - 2026-03-19

### Added

- **MCP server mode** — run `specsync mcp` to expose spec-sync as a Model Context Protocol server, enabling any AI agent (Claude Code, Cursor, Windsurf, etc.) to validate specs, check coverage, and generate specs via tool calls.
- **Direct API support** for Anthropic and OpenAI — `specsync generate --provider anthropic|openai` can call Claude or GPT APIs directly, no CLI wrapper needed. Set `ANTHROPIC_API_KEY` or `OPENAI_API_KEY`.
- **Auto-detect source directories** — spec-sync now automatically discovers `src/`, `lib/`, `app/`, and other common source directories, so it works out-of-the-box on any project without manual config.
- **Spec quality scoring** — `specsync score` rates spec files on completeness, API coverage, section depth, and staleness, outputting a 0–100 quality score with actionable improvement suggestions.
- **TOML configuration** — `specsync.toml` is now supported alongside `specsync.json`. See `examples/specsync.toml`.
- **VS Code extension scaffold** — `vscode-extension/` directory with diagnostics, commands, and CodeLens integration (ready for Marketplace packaging).
- **Actionable error messages** — all errors and warnings now include fix suggestions.
- Expanded integration test suite (+884 lines).

### Fixed

- Resolved clippy and fmt CI failures on main (#29).

## [1.2.0] - 2026-03-19

### Added

- **`specsync generate --ai`** — AI-powered spec generation. Reads source code, sends it to an LLM, and generates specs with real content (Purpose, Public API tables, Invariants, Error Cases) instead of template stubs. Configurable via `aiCommand` and `aiTimeout` in `specsync.json`, or `SPECSYNC_AI_COMMAND` env var. Defaults to Claude CLI, works with any LLM that reads stdin and writes stdout.
- **LOC coverage tracking** — `specsync coverage` now reports lines-of-code coverage alongside file coverage. JSON output includes `loc_coverage`, `loc_covered`, `loc_total`, and `uncovered_files` with per-file LOC counts sorted by size.
- **Flat file module detection** — `generate` and `coverage` now detect single-file modules (e.g., `src/config.rs`) in addition to subdirectory-based modules.
- `aiCommand` and `aiTimeout` config options in `specsync.json`.

### Changed

- Rewrote README for density — every line carries new information, no filler.
- Documented `generate --ai` workflow, AI command configuration, and LOC coverage in README and docs site.
- Streamlined docs site pages to complement rather than duplicate the README.
- Updated CHANGELOG with previously missing 1.1.1 and 1.1.2 entries.

## [1.1.2] - 2026-03-19

### Fixed

- Resolved merge conflict markers in README.md.
- Removed overly broad permissions from CI workflow (code scanning alert fix).

### Changed

- Bumped `Cargo.toml` version to match the release tag.

## [1.1.1] - 2026-03-18

### Fixed

- Corrected GitHub Marketplace link after action rename.
- Renamed action from "SpecSync Check" to "SpecSync" for Marketplace consistency.
- Updated all marketplace URLs to reflect the new action name.

### Added

- GitHub Marketplace badge and link in README.

## [1.1.0] - 2026-03-18

### Added

- **Reusable GitHub Action** (`CorvidLabs/spec-sync@v1`) — auto-downloads the
  correct platform binary and runs specsync check in CI. Supports `strict`,
  `require-coverage`, `root`, and `version` inputs.
- **`watch` subcommand** — live spec validation that re-runs on file changes.
- **Comprehensive integration test suite** — end-to-end tests using assert_cmd.

### Changed

- Updated crates.io metadata (readme, homepage fields).

## [1.0.0] - 2026-03-18

### Added

- **Complete rewrite from TypeScript to Rust** for language-agnostic spec validation
  with significantly improved performance and a single static binary.
- **9 language backends** for export extraction: TypeScript/JavaScript, Rust, Go,
  Python, Swift, Kotlin, Java, C#, and Dart.
- **`check` command** — validates all spec files against source code, checking
  frontmatter, file existence, required sections, API surface coverage,
  DB table references, and dependency specs.
- **`coverage` command** — reports file and module coverage, listing unspecced
  files and modules.
- **`generate` command** — scaffolds spec files for unspecced modules using
  a customizable template (`_template.spec.md`).
- **`init` command** — creates a default `specsync.json` configuration file.
- **`--json` flag** — global CLI flag that outputs results as structured JSON
  instead of colored terminal text, for CI/CD and tooling integration.
- **`--strict` flag** — treats warnings as errors for CI enforcement.
- **`--require-coverage N` flag** — fails if file coverage percent is below
  the given threshold.
- **`--root` flag** — overrides the project root directory.
- **CI/CD workflows** with GitHub Actions for testing, linting, and
  multi-platform release binary publishing (Linux x86_64/aarch64,
  macOS x86_64/aarch64, Windows x86_64).
- Configurable required sections, exclude patterns, source extensions,
  and schema table validation via `specsync.json`.
- YAML frontmatter parsing without external YAML dependencies.
- API surface validation: detects undocumented exports (warnings) and
  phantom documentation for non-existent exports (errors).
- Dependency spec cross-referencing and Consumed By section validation.

[Unreleased]: https://github.com/CorvidLabs/spec-sync/compare/v6.0.0...HEAD
[6.0.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v6.0.0
[5.2.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v5.2.0
[5.1.1]: https://github.com/CorvidLabs/spec-sync/releases/tag/v5.1.1
[5.1.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v5.1.0
[5.0.2]: https://github.com/CorvidLabs/spec-sync/releases/tag/v5.0.2
[5.0.1]: https://github.com/CorvidLabs/spec-sync/releases/tag/v5.0.1
[5.0.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v5.0.0
[4.0.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v4.0.0
[3.8.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v3.8.0
[3.7.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v3.7.0
[3.6.2]: https://github.com/CorvidLabs/spec-sync/releases/tag/v3.6.2
[3.6.1]: https://github.com/CorvidLabs/spec-sync/releases/tag/v3.6.1
[3.6.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v3.6.0
[3.5.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v3.5.0
[3.4.1]: https://github.com/CorvidLabs/spec-sync/releases/tag/v3.4.1
[3.4.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v3.4.0
[3.1.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v3.1.0
[3.0.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v3.0.0
[2.5.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.5.0
[2.4.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.4.0
[2.3.3]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.3.3
[2.3.2]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.3.2
[2.3.1]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.3.1
[2.3.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.3.0
[2.2.1]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.2.1
[2.2.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.2.0
[2.1.1]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.1.1
[2.1.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.1.0
[2.0.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v2.0.0
[1.3.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v1.3.0
[1.2.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v1.2.0
[1.1.2]: https://github.com/CorvidLabs/spec-sync/releases/tag/v1.1.2
[1.1.1]: https://github.com/CorvidLabs/spec-sync/releases/tag/v1.1.1
[1.1.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v1.1.0
[1.0.0]: https://github.com/CorvidLabs/spec-sync/releases/tag/v1.0.0
