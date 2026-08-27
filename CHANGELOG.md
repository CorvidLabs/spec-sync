# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Removed

- **Windows is no longer a supported target; no Windows binary is published.** SpecSync 6.0 ships
  five prebuilt artifacts — `linux-x86_64`, `linux-x86_64-musl`, `linux-aarch64`, `macos-x86_64`,
  `macos-aarch64`. Run SpecSync under WSL, or build it from source: `cargo install specsync` still
  works on Windows. The packaged GitHub Action now refuses a Windows runner with that message
  instead of requesting an asset that no longer exists.

  The decision rests on two facts, and the second matters more than the first. Across five weeks of
  v5.2.0 the Windows asset was downloaded once, against 462 for `macos-aarch64` and 446 for
  `linux-x86_64`; every 6.0 release candidate sat at 0-2 with a uniformity that reads as automated
  verification rather than people. And every job in ordinary CI runs on `ubuntu-latest`, so the
  Windows executable was published without ever being exercised. That is what allowed the defect
  fixed in rc.7 — `specsync view` failing with "Cannot parse frontmatter" on *every spec in the
  project* in a checkout with `core.autocrlf=true` — to survive for weeks on the one platform that
  shipped a binary nothing tested. A major version is where a platform may be dropped, so it is
  dropped here.

  **No Windows correctness was removed, and none may be.** Dropping the binary is not dropping the
  bug class: a teammate on Windows commits CRLF files and a colleague on Linux reads them. CRLF
  frontmatter tolerance in `parser::parse_frontmatter`, the single canonical `strip_frontmatter`,
  the `.gitattributes` `eol=lf` pins, Windows-reserved and Windows-invalid filename guards,
  `MAX_SLUG_BYTES` and its `MAX_PATH` justification, path-separator handling, junction and
  reparse-point rejection, and every `#[cfg(windows)]` block and Windows-shaped fixture are all
  unchanged.

  Requirement wording that had scoped those guarantees to "every platform SpecSync ships a binary
  for" (REQ-change-083, REQ-change-084, REQ-commands-013) is rebound to the platforms a repository
  may be *checked out on*. Read literally, the old phrasing would have narrowed each guarantee the
  moment the shipped set narrowed, which is the opposite of the intent.

  The release-candidate qualification lane still runs on Ubuntu, macOS **and** Windows. It is the
  only place the retained `#[cfg(windows)]` code is compiled and run, and removing it would recreate
  exactly the condition that produced the `view` defect.

### Added

- **6.0 release candidates ship installable binaries** (CorvidLabs/spec-sync#669).
  `v6.0.0-rc.1` was tagged correctly — annotated, right commit, marked pre-release — and carries
  zero assets, because the release lane refused at its first job and every job downstream skipped.
  Every consumer of the packaged GitHub Action got a 404.

  A final tag and a release candidate are different promises. `release.yml` qualifies a candidate
  against tag rulesets and a three-platform evidence matrix and should keep doing so; a
  candidate's whole job is to be installed by people who have agreed to test it, and gating that
  behind release-grade provenance meant it could not be installed at all. `rc-assets.yml` is a
  `workflow_dispatch` lane that does exactly one thing: build the same targets `release.yml`
  builds, under the same names `action.yml` downloads, and attach them to a release that already
  exists and is already a published pre-release. It creates no tag, promotes nothing, and depends
  on no ruleset, environment, or App identity.

  A shortcut past a trust gate needs guards of its own. The target must match `vX.Y.Z-rc.N`, must
  exist, must be marked pre-release, and must not be a draft — checked in a separate `guard` job
  ahead of the builds, so a wrong target costs seconds rather than 45 minutes. Every checksum is
  verified against its own sidecar before anything is attached, because a mismatch must fail
  there rather than at every consumer. The Rust cache is `save-if: false`, since the tree is a
  candidate.

  `v6.0.0-rc.1` itself could not be rescued. A push event runs the workflow file as it existed at
  the pushed ref, so it replays the broken lane forever, and the release was left as a draft —
  which this lane's own guard refuses. `v6.0.0-rc.2` was the first candidate with assets. It
  carried six, including a Windows build; 6.0 ships five (see Removed).

- **Lessons written into `specs/<module>/context.md` are now surfaced at each of the three
  moments a lesson exists** (CorvidLabs/spec-sync#697). Lessons were already being written
  there — the place a module's knowledge is supposed to live, precisely so the next change to
  that module can read it. Nothing surfaced them, so they accumulated where nobody looked, which
  is indistinguishable from never having written them.

  At **proposal**, `change new` prints one line per affected module —
  `specs/<module>/context.md (N line(s)) — read before scoping this change` — under a `Lessons:`
  heading. A pointer with a size, not a dump: a wall of text at creation gets scrolled past. It
  is silent when no declared module has substantive prose, and text mode only; `--json` output is
  unchanged.

  At **build**, a *failed* `change check` writes to stderr that if the failure taught you
  something it goes in `.specsync/changes/<id>/context.md` while it is fresh. Only on failure — a
  hint on green is noise. Two placement details were found by running it rather than reading it:
  the hint sits on the `Err` path from `verify_change`, because a first attempt sat in the
  `passed: false` display branch and never fired for real verification failures; and
  `active_change_id` had to select `Approved | Implementing | Verifying`, the same states
  `check_change` selects, or the hint stayed silent on a failing *first* check of an approved
  change.

  At **archive**, `finalize` assembles `lesson-bundle.md` into the archive — title, kind, specs,
  paths, acceptance criteria, the verification commit and the commands that produced it, and the
  bodies of the change's own `context.md`, `design.md` and `testing.md` — and `next_action` names
  the fold-back before the merge. `finalize --json` gained a `lesson_bundle` field.

  SpecSync **assembles and never authors**: writing the lesson would mean shelling out to a
  particular agent, and it does not need to, because whoever just ran `finalize` is right there.
  Every path **fails open** — an unreadable context yields no pointer rather than an error, and a
  bundle that cannot be written leaves a successful archive intact. That is deliberately the
  opposite posture from evidence validation, which fails closed; the distinction is whether the
  artifact is load-bearing for trust or an aid to the author.

  The loop caught two design flaws in the change that adds the loop. `change new` surfaced
  `specs/cmd_change/context.md`, which states three separate ways that the command layer holds no
  lifecycle policy — and the first implementation had put the lessons policy there. Policy now
  lives in `src/change.rs` (`accumulated_lessons`, `lesson_fold_targets`, `module_context_path`)
  and the command layer renders and decides nothing. Separately, scaffold detection was
  documented as "prompts are HTML comments" while the real `CONTEXT_TEMPLATE` is plain bullets,
  so every untouched `specs/<module>/context.md` counted as four lines of knowledge: the proposal
  stage would have pointed every new adopter at a file that had learned nothing, which is the way
  to kill the surface. It survived dogfooding because all 62 specs in this repository already
  have authored prose, so no untouched scaffold existed here to trip over. The generator now owns
  what a scaffold looks like.

  One sibling was collapsed on the way. The surfacing count and `write_lesson_bundle` had drifted
  apart *inside a single feature*, both using `split("---").nth(2)`. `---` is a legal Markdown
  horizontal rule, so that truncates any body containing one — and truncated material in a lesson
  bundle is indistinguishable from material nobody wrote. Both now use one delimiter-based
  helper, pinned by a horizontal-rule test that failed against the first fix.

- **`docs/ADOPTING.md` — one page for a repository taking spec-sync on**
  (CorvidLabs/spec-sync#671). Written to be pasted wholesale into an agent session and readable
  on its own: install, initialise and generate, fill the specs and set them active, configure
  verification, drive one real change end to end, and wire CI.

  Every verb, flag, and path in it was checked against the RC binary rather than recalled — the
  `change` subcommands, `approve --actor`, `review --reviewer`, `check --commit`, `check
  --strict`, the `.specsync/config.toml` and `.specsync/sdd.json` layout,
  `.specsync/archive/changes/`, and the three action inputs it names. A guide that names a verb
  the binary has dropped is exactly the drift this tool exists to catch, so it should not be the
  tool's own documentation that carries it.

  The "Things that will bite you" section is not speculative; each entry is something hit while
  adopting spec-sync in a real package, ordered by when it lands: scope freezing at approval with
  no withdraw verb, a declared production path with no owning module refused at `ship`,
  separation of duties refusing a solo adopter at review, build directories staling verification
  evidence, widening to `pub(crate)` becoming a documented export, and a repeated description
  colliding on its slug. Separation of duties is stated as a rule rather than a hint after
  confirming it is genuinely enforced rather than advisory — case-insensitive, at both the
  attempt history and the review itself — because a solo adopter hits it with everything else
  already green, which is the worst moment to discover it.

  The CI section says why *both* pins are needed: the `uses` ref pins the action code, the
  `version` input pins the binary it downloads. Pinning one and not the other is the easy
  mistake.

### Changed

- **The effective checkout overrides are read in one `git config` query, bounded for what that
  query can actually return** (CorvidLabs/spec-sync#646, CorvidLabs/spec-sync#649).
  `effective_checkout_overrides_uncached` spawned four `git config --get` processes for
  `core.autocrlf`, `core.eol`, `core.symlinks` and `core.filemode`. They are now one `git
  config -z --get-regexp '^core[.](autocrlf|eol|symlinks|filemode)$'`. At roughly 15 ms per
  spawn on the measuring hardware, one instrumented suite run went from 15,359 `git config`
  spawns to 3,842. It is **not a cache**: every call still spawns, so configuration edited
  between two reads is still observed — the saving is asking once for four answers, never
  remembering an answer.

  Equivalence with four `--get` calls was checked against git 2.50.1 rather than assumed, for
  each behaviour where the two could differ: a multi-valued key lists in order and last-wins
  matches `--get`; a valueless key yields a record with no `\n`, i.e. the empty value; a
  mixed-case section is emitted lowercased; whitespace is trimmed. The row that matters is the
  last one — "no matching key" (rc=1, empty stdout *and* stderr) stays distinguishable from
  "config is malformed" (rc=128 with stderr), because reading the second as unset would turn a
  broken repository into a silently default one. `core.fsmonitor` is deliberately excluded: it
  resolves through `configured_git_command`, which scrubs system, global and injected
  configuration, while this query is built on `rooted_git_command` and must be, to keep the
  precedence its callers depend on.

  The batched call then inherited the **128-byte stdout bound** the four single-key calls had
  shared. Four keys at about six bytes each were never near it; `--get-regexp` returns every
  occurrence of all four keys across every scope, and the ordinary global-plus-local layout is
  already 144 bytes, which tripped the deterministic-output guard and turned a routine read
  into a hard error. Because `effective_checkout_overrides` feeds `inspect_git_candidates` and
  so the git-evidence and workspace-digest capture, every lifecycle command that captures
  evidence failed outright on an affected machine. The equivalence test written alongside the
  batching checked six git behaviours and total output volume was not one of them, and it used
  a single scope — structurally incapable of seeing this — while the development machine had
  only `core.filemode` set. The bound is now 16 KiB, matching the sibling `core.fsmonitor` read
  which has the same "one query, unknown number of records" shape; the guard itself stays, so a
  genuinely unbounded response is still refused. Both landed before `v6.0.0-rc.1` was tagged,
  so no published build carried the overflow.

- **Internal: the reopen-then-close guard is pinned by tests** (CorvidLabs/spec-sync#650). No
  behaviour changes. The fix for #540 — a change reopened after finalize could never be closed
  again — shipped with no tests at all: 21 lines of `src/change.rs`, no test file touched, and
  reverting it left the entire suite green. Its refusal string appeared exactly once in the
  tree, in the product. The only protection was a sandbox drill that scores clean **on a binary
  with the guard deleted outright**, because it asserts `rc=0`, `state=archived` and
  `archives=1`, all of which deleting the guard satisfies.

  The two removals are not the same removal: dropping the archive-to-active direction
  reintroduces #540, while deleting the guard entirely passes the drill. One makes the guard
  stricter, the other makes it absent, and a single assertion cannot see both. The tests are
  therefore a deliberate pair that fails *differently* — a round-trip test that fails when the
  archive-to-active term is removed, a third-location test and a deletion test that fail when
  the guard itself is removed. `validate_scoped_review_history_transition` takes its inputs
  directly, so none of them needs a repository fixture.

- **BREAKING (change identity): `specsync change new` mints a slug, with no `CHG-NNNN`
  ordinal** (CorvidLabs/spec-sync#665). A change created now is identified by its description
  alone, so two people working from the same base no longer need to coordinate to avoid
  claiming the same identity. Historical archives keep their `CHG-NNNN-slug` identities and
  directory names permanently — there is no migration and no renaming.

  **What this breaks.** Tooling that parses an ordinal out of a change ID will not find one.
  `SPECSYNC_SEQUENCE_BASE` no longer does anything: allocation is gone, so there is nothing to
  floor. `.specsync/change-sequence.json` is frozen — nothing writes it any more — and it is no
  longer auto-added to a new change's `affected_paths`. A description that would produce an
  identity already in use is now refused by naming the existing change — its ID, its workspace
  path, and its description and state — instead of exhausting a 10,000-iteration allocation
  retry that reported "exhausted change sequence allocation retries" for what was simply a
  taken name. Refusing rather than disambiguating with a `-2` suffix is deliberate: the archive
  directory is `<date>-<id>`, so two same-named changes archived on different days would
  produce two archives with one `record.id`, after which `find_change_dir` reports ambiguous
  locations and every command in the repository fails with no clean recovery. A description
  containing no ASCII letters or digits is now refused outright rather than collapsing to a
  shared fallback, because under ordinals each such description became a distinct
  `CHG-NNNN-untitled-change` and without them the first one created would permanently own the
  only ID any of them can produce.

  **The guarantee the ordinal was providing by accident.** It was the repository's only
  assurance that two changes could not share an identity, and the numeric collision gate
  enforced that as a side effect. A slug is unique only by convention, and two clones can
  archive the same slug on different days into differently dated directories that git merges
  without a conflict. The gate is now explicit in `validate_change_sequences` and does not
  route through the ordinal, because the identities that need it no longer have one. It cannot
  live in `list_all_changes_uncached`, which already refuses the same shape, because `change
  audit` runs with `include_archive_integrity = false` and never loads the archive at all.

  `located_change_ordinal` separates "claims no ordinal" from "claims one badly": a slug-only
  ID is simply absent from numeric accounting, while a `CHG-`-prefixed ID whose leading segment
  is digits in a non-canonical width still fails closed. A blanket skip would have dropped the
  malformed ones out of the acknowledged-collision ID-set check that guards the archived
  collision members.

  Three surfaces were broken before this landed and all three through one line in
  `located_change_sequences`: `change audit --strict` could not even count a slug-only
  workspace, `change new` failed repo-wide with the same error, and `change status` reported a
  *healthy* next action because `sequence_ledger_freeze_next_action` ended `Err(_) => None` — a
  bricked repository that looked fine. That fall-through now reports the real problem.

  The ledger file itself cannot be deleted, for two independent reasons:
  `acknowledged_collisions` lives only there (five groups covering 11 archived changes), and
  120 of 164 archives sign it, each having signed different content. Roughly 400 lines of
  history-reading machinery therefore survive, now commented to explain why they look dead. Net
  −89 lines of production code, +52 of comment.

  The commit carries a second lifecycle change covering `tests/integration/change.rs` and
  `tests/integration/comment.rs`: the retirement rewrote fixtures that hard-coded `CHG-NNNN`
  identities, and those paths were not in its declared scope. Scope freezes at approval, so the
  archived change could not be widened after the fact.

- **Final-tag creation is no longer restricted to a release GitHub App, and every release run
  states what that costs** (CorvidLabs/spec-sync#718, CorvidLabs/spec-sync#720).

  The design in REQ-github-007 called for a `SpecSync final tag creation` ruleset naming a
  dedicated release GitHub App as its only bypass actor, and a protected `release` deployment
  environment holding that App's private key. Neither was ever provisioned, and the decision is
  now not to create them. `promote` mints `refs/tags/vX.Y.Z` with the workflow's own
  `GITHUB_TOKEN` under `contents: write` declared on that job alone; the workflow-level default
  stays `contents: read` / `actions: read` / `checks: read`, exactly two jobs hold
  `contents: write` — `promote` for the tag and `release` for publication — and a test pins that
  count so a third cannot appear unnoticed. The `actions/create-github-app-token` step,
  `vars.SPECSYNC_RELEASE_APP_ID` and `secrets.SPECSYNC_RELEASE_APP_PRIVATE_KEY` are removed from
  `.github/` entirely rather than left unset and failing closed on every dispatch.

  What that costs is stated, not implied: anyone who can run `release.yml` from the default
  branch can cause a final release tag to be created. An App key is the one credential a workflow
  author cannot reach by editing the workflow, and there no longer is one — running the release
  lane and holding release authority are now the same permission.

  `environment: release` was removed rather than kept with a comment. It named an environment that
  has never existed, and GitHub materializes a referenced environment on first use with no
  protection rules, so the reference would have published a `release` entry in the repository's
  Environments and Deployments UI that gates nothing while looking like a gate — to an audience no
  workflow comment reaches. The route to a real gate is recorded at the job in order: create the
  environment with required reviewers and a `main`-only deployment branch policy first, then
  re-add the reference, then restore a check that proves those rules still hold.

  The disclosure is enforced rather than documentary. `validate-release-candidate.py rulesets`
  emits an `unenforced` array — the three fixed `UNENFORCED_TAG_POLICIES` entries, plus one
  notice per ruleset whose `bypass_actors` the run's token could not read — and `resolve` prints
  each as a `::warning::` annotation and into the step summary on every run, green ones included.
  **`release.yml` fails when that array is empty.** The validator hard-codes three entries, so an
  empty array cannot mean "everything is enforced"; it can only mean the disclosure path itself
  broke — a renamed flag, a changed `jq` path, an emptied tuple. The tripwire guards the
  announcement, not the policy. A future maintainer who provisions the App ruleset and empties
  `UNENFORCED_TAG_POLICIES` must remove the `unenforced_count` check in the same change, or the
  lane will fail on a repository that is strictly better protected than it is today.

  Nothing else weakens. `promote` still `needs: [resolve, validate, authorize-release]`, so a tag
  from that job still follows a candidate qualified on three platforms. Both immutability
  rulesets are still validated with no bypass actor admitted, so once `vX.Y.Z` exists nobody —
  this token included — can move or delete it. The checkout still runs with
  `persist-credentials: false` behind the same one-remote credential helper, so no token lands in
  `.git/config`; only the credential changed.

  Internal to this repository's CI, not to the shipped binary: the `environment` subcommand of
  `.github/scripts/validate-release-candidate.py` is removed with its tests, and
  `--release-app-id` and `--final-creation-ruleset-json` are removed rather than ignored, so a
  stale caller fails loudly instead of believing the App policy is still checked.

<!-- DISCREPANCY: the shipped section contradicts itself, and still does on `main`. Its opening
     paragraph says a squash-merge makes a change "read as unverified the moment its own PR lands
     — forcing a full re-verify AND a fresh independent review" (docs/ADOPTING.md:109-112), while
     a paragraph fifteen lines later says "a squash no longer forces a re-verification"
     (:126-129). The PR's third commit corrected the cost downward but left the lead claim
     standing, so the first thing an adopter reads is the pre-#689 behaviour. -->
- **The adoption guide leads with merge strategy, and no longer advises one this repository
  cannot use** (CorvidLabs/spec-sync#692). `ADOPTING.md` was written before a full day of real
  adoption on another repository and mentioned none of what that adoption actually hit — squash,
  rebase, reopen, finalize, `db_tables`, `schema_dir`, or lessons.

  Merge strategy is now the first entry, with the `gh api repos/OWNER/REPO --jq '{merge:…,
  squash:…, rebase:…}'` invocation to check it *before* adopting, and the warning that `gh pr
  merge --rebase` silently falls back to squash when rebase is disabled — so following the advice
  is not enough, you have to check the setting.

  The first draft of this section told adopters to prefer rebase-merge or merge-commits for
  branches carrying lifecycle commits. spec-sync's own repository is squash-only (`merge: false,
  rebase: false, squash: true`) and 89% of its own archives — 19 of 172 — have an unreachable
  verification commit. Advice the tool's own repository cannot follow reads as a supported path
  and is not one, so it was replaced before merge with what is actually true.

  The second draft then overstated the cost, telling adopters to budget for a re-verify *and* a
  fresh review after every merge. #689 had already made ship readiness content-based, so a squash
  no longer forces a re-verification — measured across squash, rebase, and merge-commit, all
  three reach `ready to finalize`. What a squash still costs is the independent review, whose
  check walks the commits between the review and `HEAD` to prove nothing changed except the
  change's own records; a squash makes that walk impossible rather than false (#694). Advice that
  overstates the cost is not a safe default either — it tells people to budget for work the tool
  no longer asks of them.

  Also added, all from measured adoption: that merging before `finalize` blocks every earlier
  accepted change sharing a delivery input, not only the one merged (#687); the two state traps
  whose escapes are not in the error text — `check` *without* `--commit` for the reopened
  workflow-v1 deadlock, and `finalize` rather than `ship` when a review has staled (#685); and
  that `db_tables` is checkable only against `.sql` migrations, so declaring it without
  `schema_dir` is a notice rather than a `strict`-gating warning as of rc.5 (#684). Plus a "Close
  the learning loop" section for #697's three stages, with the reason it matters: a change's own
  `context.md` is archived and read by nobody, while a spec's is read before every future change
  to that module.

- **The adoption guide names the remedy for a path with no owning module, not only the trap**
  (CorvidLabs/spec-sync#682). The page already warned that production source declared under
  `--no-spec-change` is refused, but did not say what to do instead, which makes the warning
  describe a wall rather than a door. The refusal arrives at `ship`:

      error: acceptance input `src/change.rs` is production source without
             deterministic canonical ownership

  Scope freezes at approval and there is no withdraw verb, so by then the only exit is resetting
  the branch and redoing the whole lifecycle — found that way while shipping #677/#678.

  The remedy is that `--spec` and `--no-spec-change` **coexist**, and that pairing is the correct
  one for production source with no spec text changes. Nothing in the docs said so, and the flag
  names actively suggest otherwise. The page now gives the full `change new` invocation, says how
  to find a path's owning spec (grep the `files:` list in each `specs/<module>/*.spec.md`), and
  says that a path with no owner is itself the defect to fix first.

- **The lesson fold-back's own recursion, and the flag combination that terminates it, are
  documented** (CorvidLabs/spec-sync#710). The fold-back that `finalize` and `ship` instruct is
  itself a change touching tracked paths, so it needs its own lifecycle record — and if that
  record declares the same specs, `lesson_fold_targets` returns the same context paths and the
  author is told to fold again. There is no cycle detection, no warning, and no error; the
  instruction simply repeats.

  What stops it is a change that declares no affected specs. `lesson_fold_targets` maps
  `affected_specs` to `specs/<module>/context.md`, so an empty list yields no targets and both
  `lessons_next_action` and `ship_next_action` fall through to their plain merge guidance —
  "merge the PR on GitHub" for `finalize`, and ship's own push/CI/sibling tail unprefixed. The
  mechanism is the **absence of `--spec`**, not the presence of `--no-spec-change`: the two
  coexist (see #682), so a fold that declares specs is told to fold again. `ADOPTING.md` now
  supplies the `change new --kind documentation --path specs/<module>/context.md
  --no-spec-change` invocation with rationale wording, so the terminating combination is not
  improvised, and says to keep such a change to `context.md` paths — a spec companion is not
  production source and so does not trip the owning-module refusal, but anything alongside it
  would be, and would have lessons of its own.

  It also corrects a stale claim in the same bullet, which credited only `finalize` with naming
  the step; since #700 both verbs do, ahead of their remaining guidance. The section ends on the
  measured number: 6 of 183 archived changes have ever touched a spec's `context.md`.

  Not built here: `finalize` recognising a companion-only change and omitting the clause. That
  needs a discrimination this does not build, since a change touching a companion *and*
  production source must still be told to fold (#703).

- **REQ-change-016 describes commit ancestry as it is actually used**
  (CorvidLabs/spec-sync#717). Spec text only — no source file changed, and nothing an adopter
  runs behaves differently. It is here because it is the governed statement of the guarantee
  #689 relies on, and it was false.

  `specs/change/requirements.md` said `verification.commit` "is retained as an informational
  correlation key and is never a gate". `verification_commit_is_accepted_current` —
  `merge-base --is-ancestor` and nothing else — is consulted at three sites, so the requirement
  described behaviour the code does not have: the drift this project exists to catch, in its own
  spec.

  The obvious narrowing — "may consult ancestry as one basis among several" — would have replaced
  a false sentence with a different false sentence. Two of the three sites are hard conjuncts
  inside `staged_accepted_snapshot_is_closing_authenticated` (the workflow-v2 branch and the
  legacy branch); only `accepted_evidence_is_anchored` is a disjunct of three, alongside the
  integrated accepted workspace and the acceptance recorded on the remote default branch. A
  conjunct can only block and a disjunct can only widen, so one wording cannot honestly cover
  both.

  The requirement now separates the two questions: `verification.commit` is never a gate on
  verification currency or ship readiness, and a squash that discards it invalidates nothing;
  archival *anchoring* — whether an acceptance is anchored in history a reader can reach — MAY
  consult ancestry as one basis, and "Ancestry MUST NOT be the only basis on which anchoring can
  be established." That last clause is deliberately testable, and it is the one the two conjuncts
  violate; it is tracked as #706 rather than silently blessed. A coverage proof commissioned for
  #706 then inverted its own premise — those conjuncts are vacuous rather than merely weak, since
  accept sets `verification.commit` to `HEAD` moments before archive, and requiring a real
  in-history anchor would break `finalize` for every workflow-v2 change, because accept and
  archive happen in one process with no commit between them.

### Security

- **An acceptance anchor must be the commit where the evidence entered history, not any later
  commit that re-introduces it** (CorvidLabs/spec-sync#663). Exploitable by anyone able to land
  a commit.

  `authenticated_accepted_transition` authenticated an archived change by finding a commit that
  *added* its `accepted-state.json` whose committed evidence bytes equalled the
  **working-tree** bytes — with no cutoff, no ancestry bound and no ordering rule. The check is
  circular: it authenticates working-tree bytes against a commit that contains those bytes, so
  any commit of the current state qualifies. `--diff-filter=A` was the only thing keeping that
  from being trivially true, since a tampering commit is a modify — and re-introducing the
  package produces an addition.

  The issue was filed as "renaming an archive directory launders tampering", and that framing
  was too narrow. Three attack shapes were demonstrated against the shipped behaviour and the
  third contains no rename at all: `reopen` moves a package to `.specsync/changes/<id>/` and
  `archive` moves it back, so tampering in between produces a fresh introduction at a path
  SpecSync itself writes, with the archive directory's name unchanged throughout. **A fix
  scoped to the archive path would have closed two of three and looked complete.**

  For an archived change, an acceptance anchor must now be the earliest reachable commit that
  introduced that change's package, and the active-workspace stages and the working-tree
  fallback are admitted only for commits preceding it. Three details are load-bearing, each
  having been a defect in a rejected candidate: the index is built from `git_repo_prefix`
  rather than a bare archive path, because comparing a project-relative prefix against Git's
  repo-relative output makes the whole fix a silent no-op wherever the project is not at the
  repository root; rename detection is disabled with `--no-renames` rather than followed with
  `--follow`, because `diff.renames` has defaulted on since Git 2.9 so a `git mv` reports
  `R100` and vanishes from `--diff-filter=A`, and a `--follow`-based fix would look closed
  while resting on a similarity heuristic *the attacker controls*; and identity comes from the
  `id` inside the committed `state.json`, matching how `find_change_dir` resolves a package,
  because the directory name is not part of a package's identity anywhere else in the code base
  and so must not be part of the trust decision.

  Every archive that authenticated before this rule continues to authenticate after it,
  verified per risk class against a 161-row baseline captured beforehand — including the 90
  archives whose only eligible anchor is the archived-package stage, which is the class that
  breaks if the bound is drawn too tightly, and the seven pre-existing corrupt archives, which
  still fail downstream of the anchor logic and so confirm nothing was papered over.

- **A later generation of terminal evidence is trusted only when it extends the generation
  already committed** (CorvidLabs/spec-sync#666). Repairs two defects the anchor fix above
  introduced. One was a live laundering hole; the other broke reopen-then-re-finalize. Both
  were found after the Rust suite, a per-risk-class corpus sweep and three adversarial passes
  had all cleared that fix.

  The anchor fix distinguished a genuine reopen from a copy by `approvals.json`'s
  `reopenings.len()` — **a number written by whoever writes the file**, next to the evidence it
  is supposed to qualify. Appending one hand-made `ReopenRecord` promoted a rewritten package
  past the introduction that contradicts it, re-opening all three attack shapes the fix had
  closed. The count is now used nowhere in the decision. `ArchiveIntroduction` carries the
  committed `approvals.json` bytes instead, and `ledger_succession` admits a candidate only
  when `reopenings` grows, `approvals` is at least as long, **both prefixes are
  byte-identical**, and the first added reopen event's `superseded_approval` equals the earlier
  ledger's terminal approval — so a new generation names the package it supersedes rather than
  merely counting past it. Comparison is on raw `serde_json::Value`, never round-tripped
  through `ApprovalLedger`, because the typed form drops unknown fields and that is exactly
  where a difference would hide. `scope_adoptions` is deliberately excluded: `append_approval`
  clears it whenever a renewed definition approval lands, so it legitimately shrinks across a
  reopen. There is no extra git cost — the index already ran one `git show` per introduction
  and now keeps those bytes rather than reducing them to a number.

  The second defect was a regression, bisected rather than assumed: in a reopen lifecycle the
  active-path and recording stages are empty by construction, because acceptance is reached in
  the working tree between `review` and `finalize` and never committed, and at the second
  finalize the new generation's package is not yet in history — it is what the finalize is
  about to create. The archived-package stage can only offer the previous generation, which by
  definition of a genuine reopen no longer matches. So the sole surviving stage was the
  working-tree closing-evidence fallback, and the anchor fix had switched it off.

  The repair is two conjuncts, because either alone fails and both candidate one-conjunct
  repairs proved it — one let any working tree speak for a committed package, the other let
  `finalize` bless a package it merely found. **Who is speaking**: a `PendingArchiveClose`
  token is minted only by the process writing a package out of the change's own active
  workspace, and every reading path (`status`, `audit`, `list`, `ship-status`, the corpus
  census, the successor and legacy-baseline checks) passes `None` and is judged entirely by
  history. The token is deliberately not minted for a post-move resume that found its package
  already in the archive, because that shape cannot be told from an attacker flipping a
  committed package's `state.json` back to `accepted` and re-running `finalize`. **What it
  says**: the ledger about to be committed must contain, unrewritten, every ledger history
  already holds.

  The scoped-review history walk is widened in the same change. A change has two homes but
  occupies several archive *directories* across a reopen round trip, since the next `finalize`
  creates a directory dated by the day of the second close. The walk read "the ledger is absent
  from every path I know" as deleted evidence, so a path set built only from where the package
  sits now reported the reopen's own move as a deletion — which is what refused the second
  reopen of any change, and the first from inside `reopen` itself. Every directory the package
  has occupied in reachable history is now admitted, and archive-to-archive joins the two
  permitted directions. Append-only growth is still required byte-for-byte; only the
  one-attempt-per-commit restriction is relaxed across a move, where a squash can legitimately
  collapse several attempts. A repository whose introduction index cannot be built — a shallow
  clone, say — degrades to the narrower set, which is the behaviour that shipped.

  Worth recording, because it is the second time in this release: the anchor fix *anticipated*
  the reopen hazard, built the generation term for it, and verified that term as "dormant today
  — no archive in the corpus has two introductions." Dormant meant untested, and only a reopen
  creates the state that would have tested it.

- **The release lane proves a candidate before running any of its code** (rode in with
  CorvidLabs/spec-sync#666). The tag/package version check ran `cargo metadata` — which
  executes the candidate's own build scripts and manifests — *before* the check that the
  release commit is integrated into `origin/main`. The ancestor check is what makes the
  checkout trustworthy at all, so it now runs first and cargo touches nothing until it passes.
  Nothing about a legitimate candidate changes.

  The qualification job's Rust cache is switched to restore-only (`save-if: false`). That
  runner checks out a commit that on a `workflow_dispatch` is derived from an operator-supplied
  tag, while the run carries default-branch privileges — writing a cache entry from that tree
  is what would let a candidate that is not what the operator believes seed a cache later
  restored by default-branch workflows. Reading one is harmless. The cost is a cold build on a
  lane that runs rarely.

### Fixed

- **`ADOPTING.md` no longer leads with the squash cost that #689 removed**
  (CorvidLabs/spec-sync#729). The page an adopter is pointed at first said a squash-merge forces
  "a full re-verify AND a fresh independent review", and corrected itself seventeen lines later
  under a heading reading "Half of this is now fixed". #689 shipped the fix; #692 added the
  correction as a later commit and left the lead claim standing — so the first thing a reader met
  was the pre-#689 behaviour, in the paragraph written to alarm them, with the retraction below a
  heading that reads like a footnote. A page that states both the old and the new behaviour and
  leads with the old one is worse than one that states only the old, because a reader who stops
  early is confidently misinformed and a reader who continues cannot tell which paragraph is
  current.

  The section now leads with what a squash actually costs — the independent review, and only that
  — and states separately that re-verification is no longer among it. The framing changed from the
  maintainer's question (what did we fix) to the adopter's (what does this cost me), which is what
  had put the stale claim first.

  The statistic in the same paragraph was also unreadable: "89% of its own archived changes have an
  unreachable verification commit — 19 of 172" invites the reader to take 19 as the unreachable
  count, which would be 11%. The 89% was right and the parenthetical was the *reachable* count.
  Re-measured by walking every archived change's recorded verification commit against
  `git merge-base --is-ancestor`: **21 of 198 reachable**. Now stated in one direction only.

- **Declaring an additional module can no longer reduce the verification a change receives**
  (CorvidLabs/spec-sync#617). `verification_commands_for_change` walked `affected_specs`,
  collected whatever `component_verification_commands` entry each module had, and fell back to
  the project-wide `verification_commands` only `if commands.is_empty()`. That test was taken
  over the whole change, so a single routed module made the list non-empty and suppressed the
  project-wide list for every other module in scope — including modules nobody had routed at
  all. Verification was therefore non-monotonic in declared scope: `--spec routed --spec
  unrouted` ran strictly less than `--spec unrouted` alone.

  Measured on this repository, on two real changes minutes apart. A change declaring `--spec
  validator --spec manifest` received `["cargo test validator::tests::"]` — 63 tests, the
  integration binary 0 passed / 400 filtered out — and reported `✓ verified`. A change
  declaring no module at all received all four project-wide commands, including one that had
  never executed under this gate on this repository. The more carefully an author named the
  modules they touched, the less the gate ran.

  A declared module with no routing entry is now tracked separately, and the project-wide list
  is added whenever any such module is present: a module nobody routed is not a module that
  needs no verification. Targeted verification survives rather than being deleted to make the
  property hold — a change scoped entirely to routed modules still receives only its component
  commands and does not fall back. Strict escalation continues to append without removing.

  `--strict` was the interim mitigation and only ever narrowed the hole. It appends
  `strict_verification_commands` without restoring the globals, so a project that configured
  `verification_commands` and never populated the strict list got no protection from it.

<!-- DISCREPANCY: the commit subject names only the lane-classification fix; the same commit
     also carries two unrelated coverage-job changes for CorvidLabs/spec-sync#624, described in
     the last paragraph below. -->
- **Repository-internal (nothing an adopter runs changes): a tip-only CI classification may
  narrow the lane, never contradict the pull request** (CorvidLabs/spec-sync#626). This touches
  only spec-sync's own `.github/`, but it bears directly on the verification evidence behind
  this release, so it is recorded rather than dropped. `ci.yml` classified changed paths twice
  — once over the whole pull request, once over the tip commit alone — and unconditionally took
  the tip answer whenever it was `archive_only`, `legacy_archive_only` or `review_only`.
  `specsync change ship` always produces an archive commit last, so the tip answer won on every
  lifecycle pull request. PR #567 changed `src/commands/check.rs` and merged with `test`, `fmt`,
  `coverage`, `audit` and `spec-check` all skipped and the required aggregate green, because
  that aggregate counts a skipped job as a pass; PR #629, whose two fixes are entries in this
  same release, changed nine source files the same way.

  The whole-PR classification is now computed first and unconditionally, and
  `.github/scripts/select-ci-lane.sh` arbitrates: a tip answer is a candidate that may narrow
  the lane, but if the whole pull request selected the product lane, an archive-shaped tip does
  not deselect it. The second half of #626 — making the aggregate assert which jobs *ran*
  rather than only that none failed — was not done; `skipped` still satisfies `Require every
  selected gate`.

  The change was shipped as its own test and immediately found a second way to produce the same
  symptom: `select-ci-lane.sh` was staged by `change check --commit` before `chmod +x` ran, so
  git recorded mode 100644, the classify job died with `Permission denied`, and every
  downstream job skipped — identical visible outcome, unrelated cause.

  The two coverage changes the subject does not mention (#624): `cargo tarpaulin
  --follow-exec` instrumented every child process, and the integration suite spawns the binary
  for nearly every test, producing ~1000 profraw files (999 and 994 measured on two runs) whose
  single merge killed the runner on every run — always at "Merging coverage reports", never
  during the tests. Steering the naming with `LLVM_PROFILE_FILE=%m` did not work, because
  tarpaulin sets that variable per child itself. Coverage now runs `--bins` and instruments only
  the binary target's in-process unit tests. The integration suite still runs, in the `test`
  job; it is simply no longer instrumented, so the reported percentage no longer includes paths
  reached only by driving the binary end to end.

- **`migrate` no longer deletes an explicit `enforcement = "warn"`** (CorvidLabs/spec-sync#625).
  `config_to_toml` skipped `EnforcementMode::Warn` with the comment `// default, omit`. `Warn`
  has not been the default since `#[default]` moved to `Strict`, so a project that had
  deliberately chosen the non-blocking policy lost the line on write and became gating on the
  next load. One tree, one `specsync migrate` in between: identical findings, identical output,
  rc=0 before and rc=1 after.

  This is close to undiagnosable from the diff, because the config did not *gain* a `strict`
  line, it *lost* a `warn` one, and the person debugging a newly red CI looks for something
  that was added. It also voided the documented mitigation for 6.0's `warn` → `strict` default
  change, which is to set `enforcement = "warn"` explicitly — exactly the value `migrate`
  deleted.

  The key is now written unconditionally instead of omitted on equality with a literal.
  Omit-on-default is safe only while the default never moves, and an absent key is
  byte-identical on disk to a key holding the default, so nothing downstream can tell the two
  apart afterwards. One consequence worth knowing: `init` and `migrate` both write an explicit
  `enforcement = "strict"` for a project that never expressed a preference. The effective
  policy is unchanged; it is now pinned against a future default move rather than tracking it.

  `site/src/content/docs/configuration.md` had documented the default as `warn` and now says
  `strict`.

- **A reopened change can be finalized again** (CorvidLabs/spec-sync#540).
  `validate_scoped_review_history_transition` walks committed history and, for an unchanged
  review ledger, accepted only two shapes: the path did not move, or it moved from
  `.specsync/changes/` to `.specsync/archive/changes/`. A change has exactly two homes and the
  evidence crosses between them twice in a round trip — `finalize` carries it active → archive,
  `reopen` carries the same bytes back. Only the first direction was admitted.

  The refusal therefore surfaced at the *next* `finalize`, not at the `reopen` that performed
  the move, because it comes from a walk over committed history rather than from the command
  doing the work — which is also why re-running `review` could never clear it. What the user
  saw was a dead end: `finalize` failed with `scoped review history moved evidence outside
  finalization` and restored the source, after which `check`, `review`, `reopen`, `accept`,
  `archive` and `ship` each refused for their own reason while `status` and `ship-status` went
  on naming `finalize` as the next action. The workspace was intact; it simply could not be
  closed.

  Archive → active is now admitted on the same terms as active → archive. A move to any third
  location is still refused, so the check continues to detect evidence relocated outside the
  lifecycle: the allowance names the two canonical homes rather than loosening the predicate.

- **An archived change package no longer leaves an untrackable husk, and enumeration no longer
  dies on one** (CorvidLabs/spec-sync#536, CorvidLabs/spec-sync#412). `change ship` wrote an
  empty `deltas/` directory into the dated archive package. Git cannot represent an empty
  directory, so checking out any commit that predates the archive removed every tracked file in
  the package and stranded the directories — a husk that `git status` reports as clean. The
  next `specsync check`, `change new` or `change audit` then died on a raw OS error for a
  `state.json` in a directory git says is not there. Recovery was `rm -rf` of a path the tool
  never named.

  The two issues are one failure mode reached from two directions, which is why one commit
  closes both: #536 is the tool manufacturing the husk through its own normal output, #412 is
  the same enumeration hard-failing on a hand-made `mkdir` under `.specsync/archive/changes/`.
  Both halves are addressed. `archive_change_with_options` prunes directories holding no regular
  file at any depth, deepest first so a parent emptied by its children goes in the same pass,
  and after validation so the rollback paths still restore an intact source; failure to remove
  one is ignored rather than undoing an archive that already validated.
  `located_change_sequences` and `list_all_changes_uncached` skip such a directory alongside
  the existing legacy-tombstone allowance.

  The read-side allowance is deliberately narrower than #412 asked for. That report wanted any
  stray directory skipped with a warning; only a directory git could never have committed is
  skipped. One holding at least one regular file but no `state.json` is still refused, so the
  tolerance cannot be satisfied by ignoring corruption, and directories in an archived package
  that do hold files are preserved.

- **A populated semantic delta no longer reports as empty** (CorvidLabs/spec-sync#537).
  `parse_delta` returned `Ok(vec![])` for any content it did not recognise, and both callers
  turned an empty item list into ``semantic delta for `<module>` is empty``. A three-line prose
  file was therefore reported as an empty file, which sends the author looking for a write that
  did not land rather than at a heading grammar they have never seen — the grammar appears in
  no `--help` output and in no generated `SKILL.md`.

  `parse_delta` now makes the distinction itself. Whitespace-only content still reports empty;
  content with no recognized operation heading reports that and names `## Added`, `## Modified`
  and `## Removed`; content that *has* an operation heading but no item under it gets its own
  message naming `### REQUIREMENT <id>` and `### SPEC SECTION <name>`. The `invalid delta
  operation heading` refusal names the allowed values too. The historical-delta walk inherits
  the distinction for free, because it calls the same parser rather than reimplementing the
  test.

  The same commit removes an asymmetry its subject does not mention. `## Added` was matched
  after `to_ascii_uppercase`, so operation headings were already case-insensitive, while item
  headings used a raw `strip_prefix("REQUIREMENT ")` and were not — `## added` approved and
  `### requirement` was refused, in the same file. Item headings now match through
  `strip_ascii_prefix_ignore_case`, so `### requirement` and `### spec section` parse. A `###`
  line that is not an item keyword, met while an item is open, remains that item's content as
  before.

<!-- DISCREPANCY: the commit subject, "ship-status must name the action the lifecycle
     requires", is unqualified, but the diff scopes the deferral to Draft, Accepted and
     Archived. The `verifying` cases CorvidLabs/spec-sync#534 documented — a stale verification
     told to run `change review`, a "ready to finalize" told to run `change ship` — still stand,
     as do #534's stage findings (two simultaneous `[current]` stages, `product_tip` gated on
     git ancestry rather than the contract digest). REQ-cmd-change-014 states the narrow scope
     correctly; only the subject overclaims. -->
- **`change ship-status` defers to the lifecycle next action outside the shipping window, and
  never restates a blocker as one** (CorvidLabs/spec-sync#534). `ship_status_report` computed
  the correct state-aware `lifecycle_next` and emitted it in the JSON, but the text renderer
  printed `ship_next`, which is derived from tip stage and git ancestry. On a draft that meant
  `Next: run specsync change check <id> --commit` while the change was still in its interview;
  obeying it produced `cannot check the change while ... is draft` from the same binary that
  printed the line. On an archived change it printed the same instruction for work that was
  finished.

  Two rules now apply. Outside the shipping window — `Draft`, `Accepted`, `Archived` — the ship
  lane defers to `lifecycle_next`: the lane may narrow the next action, never contradict the
  lifecycle state, the same rule applied to CI lane classification in #626. And `ship_next` no
  longer falls back to `blockers[0]` whenever any blocker exists. A blocker says what is wrong,
  not what to do, and it already renders on its own `Blocker:` line; at `approved` that arm
  printed `Next: no verification evidence recorded yet`, a restatement where a runnable command
  belongs.

  Inside the shipping window `ship_next` still comes from the stage table, so the two
  `verifying` instances in #534's told-then-refused table are unchanged.

- **`change ship-status` resolves an archived change's evidence from its archive package**
  (CorvidLabs/spec-sync#534). Both evidence reads built the path to `verification.json` and
  `review.json` under a hard-coded `.specsync/changes/<id>/` — a parallel implementation of
  `change_dir` that is correct exactly until the change is archived and moves out of it. A
  finalized change with recorded evidence reported `Verification: none` and `Review: missing`
  for artifacts sitting in its own archive package.

  `find_change_dir` already answers active-or-archive and is now the single resolver, made
  `pub` so the command layer reuses it instead of growing a third path idiom beside it. The read
  is lenient by design: an unreadable or unparseable archived artifact degrades to "no evidence
  recorded" rather than propagating with `?`, because turning `ship-status` and `ship` from rc=0
  into rc=1 on an already-damaged repository would make the fix for an inspection command the
  thing that breaks inspection. Resolution failure on an ambiguous or malformed id falls back to
  the active path, so a status command always renders.

- **The change-sequence ledger gate judges a branch by its own history, not by origin**
  (CorvidLabs/spec-sync#533, PR CorvidLabs/spec-sync#629). The read-side gate added with the
  #533 fix compared the working ledger against `remote_sequence_high_water`, so a branch merely
  *behind* the default branch was diagnosed as corrupt:

      error: change sequence ledger claims CHG-0001 but the default branch has already
      recorded CHG-0002; restore it with `git checkout origin/HEAD -- ...`

  Nothing was wrong with that branch, the prescribed recovery was not needed, and `check`
  warned on every run. The gate also prevented nothing: allocation is already floored against
  the same mark, so with the gate removed that branch allocates CHG-0003, not a colliding
  CHG-0002.

  `branch_sequence_high_water` now reads every revision of `.specsync/change-sequence.json`
  reachable from HEAD in one `git log -p` — one invocation rather than one per revision, and
  bounded to 200 revisions of the ledger so `check` and `audit` stay cheap on a long-lived repo
  — and takes the maximum over *added* `"sequence":` lines only; counting removed ones would
  make every ordinary increment look like a rewrite. This is the branch asking a question about
  itself, which is the only question whose answer can convict it. A branch that has never
  recorded anything higher than it holds is never accused. A branch that raised the ledger and
  then rewrote it downwards is caught even when the whole episode postdates its divergence —
  the case a merge-base comparison acquits, because the merge-base predates the raise. No
  remote is consulted, so the gate cannot silently disable itself on a repository without an
  origin, and the refusal names a recovery command that applies to the branch's own history.

  The write-side floor from #533 is unchanged and still consults the remote mark. It also
  gained the test it was missing: `floor_sequence_ledger_to_committed` had unit tests calling
  it directly, but nothing asserted that `git_commit_all` invoked it, so deleting the call left
  the whole suite green while every lifecycle commit went back to staging a stale ledger over a
  higher committed mark — the exact #533 regression. The new test drives the real staging path
  and inspects the sequence that landed in the commit. The 55-drill sandbox board had stayed
  green through the origin-comparison regression for the same reason: the #533 drill exercised
  only the write path.

- **BREAKING (exit codes): a cited source file that cannot be measured is no longer reported as
  freshness** (PR CorvidLabs/spec-sync#629). Every consumer of the drift primitive guarded with
  `if !root.join(source_file).exists() { continue; }` and then reported the number computed over
  whatever files remained. A spec whose sources had all been deleted measured zero commits
  behind, so `specsync stale` printed `✓ All specs are up to date with their source files` at
  rc=0 on a tree where `specsync check` exited 1 — two commands, one tree, opposite verdicts,
  and the wrong one is the reassuring one.

  A deletion is not an absence of evidence: `git cat-file -e <spec-commit>:./<path>` proves the
  file was there and is gone. That fact rules out the obvious repair, because a deletion
  measures as *one* commit against a default threshold of five, so merely removing the guard
  leaves the spec "fresh" and the bug intact. `source_was_deleted` is now the one shared
  predicate — a deletion is stale regardless of threshold, and a path git never knew is
  unmeasurable rather than stale.

  Five call sites in three disguises, found only by enumerating them: `stale`, `report` and
  `check` skipped and reported zero; `scoring` skipped and then reported the git half
  `Measured` at zero, which looked correct from outside because a separate file-existence
  criterion does penalise the missing file — only the drift half lied; `lifecycle` had no guard
  at all and let the threshold bury the deletion. All five are corrected together rather than
  where the problem was noticed.

  `scoring` deliberately applies **no** second penalty. The file-existence criterion already
  charges for a cited file that is gone; charging again would bill one defect twice and move
  every affected spec's score. What was wrong was the claim, so `git_freshness` reports
  `Withheld` instead of `Measured` and scores are unchanged. `stale` reports `deleted_files` per
  spec, separates partially-measured specs from wholly unmeasurable ones, withholds the
  all-clear in the text *and* markdown renderers, and carries `unmeasurable_count`,
  `unmeasurable_specs` and `deleted_source_specs` in `--format json` so a consumer computing
  `total - stale - fresh` cannot silently absorb them. `report` returns `stale: null` rather
  than `false` with zero, and derives `staleness_inconclusive` from the unmeasured count rather
  than from missing history alone.

  **The exit codes move.** `specsync stale` now exits 1 when any spec is unmeasurable or only
  partially measurable, not only when one is over threshold — the same rule
  `refuse_without_history` already applied one level up, where the missing input was the
  history rather than the file. `specsync report` counts those modules among its failures. A
  `lifecycle` guard now fails on a cited file that no longer exists, whatever the threshold
  tolerates. `--enforcement warn` is unaffected and still exits 0 throughout, and a spec whose
  files are all measurable is scored and gated exactly as before.

- **A file written by a newer 6.x is readable by an older 6.x** (CorvidLabs/spec-sync#652).
  Seventeen persisted-evidence structs in `src/change.rs` carried
  `#[serde(deny_unknown_fields)]` — `ScopedReviewRecord`, `FinalizationRecord`,
  `CorrectionRecord`, `ApprovedScopeV1`, `WorkflowV2Baseline`, `LegacyArchiveBaselineV1` and
  the scope-adoption family among them. An older 6.x binary therefore could not parse a file a
  newer 6.x binary had written with an added field, so no evidence shape could be extended
  during 6's lifetime without breaking installations already deployed. That is the mechanism by
  which "just add a field in 6.4" becomes "we need 7.0". The attribute is removed from all
  seventeen.

  The design line is that **tolerance is for what cannot be recreated**. A cache that cannot be
  understood should be discarded; evidence should not. `hash_cache.rs` keeps the attribute —
  its file is gitignored and `HashCache::load` returns `Self::default()` on any parse error, so
  an unrecognised shape costs one rebuild.

  Stated rather than glossed: `ApprovedScopeV1`, `CorrectionRecord` and `ScopedReviewRecord`
  are digest preimages (`scope_digest`, the correction digests, `finalization.review_digest`).
  Adding a field to one of those still changes its serialized bytes and therefore its digest.
  Tolerance lets an older reader *parse* such a file instead of erroring; it does not make
  field addition digest-safe for those three. The other fourteen are freely extensible. **No
  digest moved** — read-time tolerance was never part of any preimage.

  Separately, a record carrying a `workflow_version` outside `{1, 2}` reported `invalid change
  state <path>: unsupported workflow version 3`, indistinguishable from corruption. Three sites
  — `validate_loaded_change`, `validate_workflow_version_anchor` and its historical twin — now
  report it as written by a newer SpecSync and name the upgrade as the remedy, which is what
  lets a later workflow version exist without every older 6.x install reporting the repository
  as broken.

- **The forward-compatibility valve is now true in all three places it was claimed, and works
  in both directions** (CorvidLabs/spec-sync#655). An adversarial pass over #652 could not
  break its digest-invariance claim, but found the valve asserted where it does not hold.

  `.specsync/agent-artifacts.json` was classified with the hash cache as a regenerable cache,
  and the code comment said the opposite of what the code does: `load_agent_artifact_manifest`
  returns `Err` rather than rebuilding, the file is git-tracked and shared, and
  `.specsync/hashes.json` is gitignored. So one teammate upgrading to a later 6.x and adding a
  field stopped `agents install` and `init` for every teammate still on the older binary —
  precisely the lockout #652 existed to remove. The manifest is also not recomputable: it
  records the digest of exactly the bytes SpecSync last generated, which is the only thing
  distinguishing an untouched artifact from an edited one. It is evidence, and it is now
  tolerant. The three fields it needs stay required, so tolerance cannot be mistaken for
  accepting any shape.

  The test meant to guard the regenerable-cache side guarded nothing: it fed an `entries` key
  when the field is `hashes`, so the parse failed on the missing field whether or not the
  attribute was present. And the `WorkflowV2Baseline` case was true at type level and false in
  operation — `read_workflow_v2_baseline` and `validate_legacy_archive_baseline_bytes`
  re-serialize what they parsed and require `bytes_match_canonical_json` against the bytes on
  disk, so an added field survives `from_slice` and is then dropped by the re-serialization and
  fails the comparison. That gate is deliberate for the two files that anchor history; the
  limit is now pinned by its own test rather than left to be discovered.

  The mirror defect: `deny_unknown_fields` is the old-reads-new door, and `SddPolicy` had the
  new-reads-old one. None of its eight fields was optional on deserialize, which works only
  because SpecSync writes all of them — the day 6.x adds a ninth, every `sdd.json` written
  before it becomes unreadable by the binary that added it. It now carries a container-level
  `#[serde(default)]` over the existing `Default`, which is the *enforcing* policy (`enabled:
  true`, `require_change_for_meaningful_files: true`), so a policy that loses a field enforces
  more, not less.

- **A damaged archive package is refused as damaged whatever it is named, and CI decides which
  changes need review by reading state rather than globbing** (CorvidLabs/spec-sync#658). Two
  gates decided identity from the shape of a name and both failed open.

  `is_positive_legacy_tombstone` used `name.contains("-CHG-")` to mean "real lifecycle package,
  therefore not a pre-lifecycle tombstone". That is false for the undated form:
  `2026-08-19-CHG-0001-foo` contains it, `CHG-0001-foo` does not. An archived package named
  that way which had lost its `state.json` and its four marker files while keeping
  `deltas/*.md` was silently skipped by `list_all_changes_uncached` and
  `located_change_sequences` instead of being refused as corrupt — hiding damage as absence.

  The replacement is a **union, not a substitution**: three signals now say "real package" — it
  holds a regular file outside `deltas/`, it holds a lifecycle marker file, or its name carries
  an ordinal (`name_carries_a_lifecycle_ordinal`, which accepts both the dated and undated
  forms). Each signal can only move a directory from *skipped* to *refused*, so adding one
  cannot weaken the gate. That mattered: the first implementation *replaced* the name check
  with the content check, and a dated package holding only `deltas/auth.md` went from refused
  to skipped — trading a fail-closed behaviour for a fail-open one while claiming to fix two.
  The code says plainly that signal 3 is the one that cannot survive an identity scheme without
  ordinals, and hands that over as a stated problem.

  The second gate is this repository's own CI rather than shipped product:
  `classify-ci-paths.sh` globbed `.specsync/changes/CHG-*/state.json` to decide whether the one
  mandatory independent implementation review was required, so any identity shape the glob
  missed yielded `review_required=false` and a pull request that merged unreviewed — while CI
  went green *faster*, the worst possible shape for a gate failure. All four sites now read
  `.id` from `state.json`, and when jq is unavailable or the state unreadable the archive fast
  lane is withheld: no identity, no shortcut.

- **Succession is ordered by when a change was created, and every ordering of its edges agrees
  with the one that is signed** (CorvidLabs/spec-sync#659). `succession_change_key(id)` was
  `(change_sequence(id).unwrap_or(u64::MAX), id)` and had six callers. Under an identity scheme
  without ordinals `change_sequence` returns `None` for every ID, every key collapses to
  `(u64::MAX, id)`, and all six degrade to *alphabetical* order — silently, with no error, no
  compile failure and no failing test. `retire-auth` would have "happened before" `add-billing`
  because `a` sorts before `r`.

  Three sites asked a genuine happens-before question and now compare `created_at` through
  `happens_after`, which tie-breaks on ID so the relation stays a total order for two changes
  sharing a timestamp — the surrounding gates enforce strict sorts, so a tie would make a valid
  record unrepresentable. Both records are already loaded at each of those sites, so this costs
  no I/O the ordinal saved.

  The other three exist for canonical serialization and digest stability, where any
  deterministic total order will do, and they now sort lexicographically by `predecessor_id` to
  align with `approved_scope` — the one whose result is hashed into `scope_digest`. **That
  alignment fixes a live bug.** The two orderings already disagreed at five digits: numerically
  `CHG-9999 < CHG-10000`, lexicographically `CHG-10000 < CHG-9999`. `approved_scope` sorted
  lexicographically and hashed the result while `validate_supersedes_edges` enforced a numeric
  strict sort, so `approved_scope` could emit an order the gate then rejected. The refusal text
  changes accordingly, from "strictly sorted by numeric sequence and full predecessor ID" to
  "strictly sorted by predecessor ID and must not repeat".

  No historical digest can move: the succession subsystem has never been exercised in this
  repository's history — of 160 archived records, none carries a `supersedes` edge and none
  carries `semantic_succession` evidence.

- **A minted change slug is a legal directory component and stays readable when it has to be
  cut** (CorvidLabs/spec-sync#661). The guarantee is scoped to the platforms a repository may
  be **checked out on**, Windows included, not the platforms SpecSync publishes a binary for —
  the directory a slug becomes is created in someone else's clone. (REQ-change-083 was
  originally worded "every platform SpecSync ships a binary for"; it was rebound to the
  checkout sense when the Windows binary was dropped, since read literally the old phrasing
  would have narrowed the guarantee the moment the published set narrowed. No Windows content
  guarantee was relaxed.)

  Three properties of `slugify` did not survive the slug becoming the whole path component.
  `for character in value.chars().take(80)` bounded *input characters*, not emitted bytes —
  runs of punctuation collapse to single hyphens, so a "capped" slug finishes well under 80 and
  the cap never actually bounded the path component. It truncated mid-word, producing names
  reading like `…preserved-audited-guara`. And it could emit a reserved name: `slugify("NUL")`
  gave `nul`, a directory Windows cannot create or open, matched case-insensitively so
  lowercasing is no escape. The empty-input fallback was literally `"change"` — itself
  reserved, and a collision with `.specsync/changes/`.

  The cap is now 120 **bytes**, and the binding constraint is `MAX_PATH` (260) rather than the
  255-byte filesystem component limit: the deepest path a change produces is
  `.specsync/archive/changes/<slug>/deltas/<module>.md`, which at a 120-byte slug is 174
  characters and clears `MAX_PATH` inside an 80-character repository root, while a 255-byte
  slug is 309 before any root prefix at all. Truncation trims back to a word boundary when one
  is near enough for the result to stay legible. The empty-slug fallback is `untitled-change`,
  and a slug that reduces to a reserved name gets a `-change` suffix. Measured against this
  repository's 159 archived descriptions, raising 80 → 120 takes intact slugs from 77 to 110 —
  and slug uniqueness saturates at 50 bytes, so the cap is purely a readability knob.

  `is_reserved_module_name` already existed with the full list in `src/commands/mod.rs`; it is
  made `pub(crate)` and reused rather than restated, because a second copy of that list is
  exactly how the two would drift apart. That is a change to the `commands` module's contract,
  so it carries its own requirement and a documented export.

- **A change identity is validated for what it is, not for how it starts**
  (CorvidLabs/spec-sync#662). `validate_change_id` led with `id.starts_with("CHG-")`, which was
  doing two jobs and was evidence for neither: `CHG-` is four characters any caller can type,
  so it proved neither that an identity was well-formed nor that SpecSync minted it. What it
  did do is hard-reject every identity without an ordinal, from a function that gates
  `find_change_dir` and `validate_loaded_change` and therefore gates the whole system.

  Two checks that actually matter were missing. There was **no length bound at all** —
  survivable only because every ID was minted as `CHG-NNNN-` over a capped slug, and a path
  this process cannot open the moment an arbitrary name is accepted. The ceiling is 255 bytes,
  the filesystem component limit, deliberately *not* the 120-byte slug cap: the slug cap bounds
  what SpecSync mints, this bounds what it will read, and an ID minted by a different version
  or by hand must still load if it is legal. And there was **no reserved-name check**; it now
  reuses the same shared predicate as the slug minter, so `nul`, `con` and `com1` are refused
  as identities too. `.` and `..` were already rejected for free by `Path::components`, and
  that is now pinned by test rather than left for a reader to notice. Every identity shape
  SpecSync has previously minted remains acceptable.

<!-- DISCREPANCY: 77461a4a is titled "fix(release): the lane must be able to read the tag that
     triggered it (#668)" and its entire diff is `fetch-tags: true` on `actions/checkout`. That
     input is a no-op on this code path — checkout assigns `fetchTags` only inside its
     `fetchDepth > 0` branch, so with `fetch-depth: 0` it is silently dropped. The commit did not
     correct the behaviour its title names. #669 established this 39 minutes later, removed the
     line, and replaced it with an explicit `git fetch --force`. The entry below states the fix as
     #669's, not #668's. -->

- **The release lane can resolve and validate a release candidate** (CorvidLabs/spec-sync#635,
  CorvidLabs/spec-sync#668, CorvidLabs/spec-sync#669, CorvidLabs/spec-sync#670,
  CorvidLabs/spec-sync#718, CorvidLabs/spec-sync#720). This is release infrastructure for the
  SpecSync repository, not product behaviour. It is recorded as one entry because it is one
  failure with six layers of cause stacked behind each other, and because it is the reason no
  6.0 candidate could be installed for the first two weeks it existed.

  `release.yml` had not completed a run since 2 August. Every candidate from `v6.0.0-rc.1` to
  `v6.0.0-rc.7` failed inside `resolve` in between eight and thirteen seconds. Each cause was
  invisible until the one in front of it was removed, so the sequence matters more than any
  single fix.

  **Layer 1 — a gate on a check nothing produced (#635).** `validate` waited up to 120 seconds
  for a check run named `SpecSync archive binding` and then exited 1. Its only producer,
  `post-merge-archive.yml`, was deleted in #499 when the CI reimplementation of the SDD lifecycle
  was removed; the consumer stayed. The ~430 lines of embedded Python below the wait — parsing
  `external_id`, pinning the app id and `details_url` prefix, cross-checking the merged pull
  request two ways, reproducing a SHA-256 over a reconstructed event — read as a working system
  and were unreachable, because the input they validated was never sent. `validate` carries no
  mode guard, so this blocked the RC tag as well as the final tag, and nothing downstream of it
  (`qualify`, `authorize-release`, `promote`, `build`, `release`) had ever executed once.

  The wait and the reconstruction are deleted rather than restored: re-adding the producer would
  reinstate the code #499 removed on purpose, including a `deltas/` mis-count that had already
  caused one change to fail to bind to its merge commit. The archive-to-merge binding is enforced
  by SpecSync itself, in `change ship` and archive validation. The rest of `validate` is intact —
  tag version against `Cargo.toml`, checkout matches the resolved candidate, candidate is an
  ancestor of `origin/main`. `test_release_reconstruction_requires_actual_pull_request_event` was
  removed with the block it anchored on, and the other 19 `release.yml` anchors in that file were
  enumerated first to confirm it was the only orphan.

  The same change added a `dry_run` workflow input so the lane can be exercised from
  `workflow_dispatch` without creating a tag: an unrecognised value is rejected rather than
  defaulted, so a typo cannot become a real promotion. The lane going sixteen days unexercised is
  why this stayed invisible, and that input is what finally made a green `release.yml` run
  possible on 27 August — the first since v5.2.0.

  **Layer 2 — the checkout destroyed the annotated tag before the lane read it (#668, #669).**
  `resolve` refused `v6.0.0-rc.1` with "must be an annotated tag, not a lightweight tag" eight
  seconds after a correctly annotated tag was pushed; the GitHub API reported `type=tag` for the
  same ref. `actions/checkout` with `fetch-depth: 0` compares the resolved commit against
  `git rev-parse refs/tags/<tag>`, which for an annotated tag is the tag object's own SHA and so
  never matches, and reacts by fetching `+<commit>:refs/tags/<tag>` — force-overwriting the
  annotated ref with a lightweight one. `resolve_annotated_rc_tag` then asks
  `git cat-file -t refs/tags/<tag>`, sees `commit`, and refuses. The push path now re-fetches the
  tag object explicitly, exactly as the `workflow_dispatch` path already did.

  **Layer 3 — three more defects in jobs that had never run (#669).** With the tag readable, an
  audit of the six untested jobs found three:

  `shell: python` in `qualify`'s "Write SHA-bound platform evidence" and `release`'s "Verify
  packaged checksum" is a literal PATH lookup with no `python3` fallback, and the macOS runner
  image installs only `python3`. Both steps are `if: always()`, so the failure was silent in the
  step list and surfaced three jobs later as a missing artifact: no `macos.json`, so the evidence
  upload hit `if-no-files-found: error`, so `record-qualification` never received its three
  platform files, so the RC gate posted `failure` and could not go green. A missing interpreter
  presented as an unqualifiable candidate. Both are now `shell: python3 {0}`.

  `authorize-release` derived a workflow-run id by string-stripping `details_url`. GitHub
  discards the `details_url` posted to `POST /check-runs` from an Actions token and persists its
  own `/runs/<check-run-id>`, so the prefix test rejected every green gate — and had it passed,
  the extracted id would have been a check-run id used as a workflow-run id, a wrong answer
  rather than an error. The run is now located by the properties that identify it — candidate
  `head_sha`, `event=push`, `status=success`, workflow path `.github/workflows/release.yml`, and
  `head_branch` equal to the RC tag. This repository had already recorded that rule in CHG-0076:
  a display URL is not a stable provenance field. Every assertion the old path made about the
  resolved run still applies.

  `validate` held the only bare `cargo` invocation in any workflow here, so the rustup shim read
  the *candidate's* `rust-toolchain.toml` and downloaded whatever it named — an unpinned
  toolchain fetch driven by candidate-controlled content, inside a five-minute timeout, on the
  step whose job is deciding whether that candidate can be trusted. It is now pinned to
  `dtolnay/rust-toolchain` at 1.89.0, making the toolchain a property of the lane rather than of
  the thing under test.

  **Layer 4 — a checksum sidecar the verifier could not read (#670).** The RC asset lane built
  all six targets and then refused to attach any of them:
  `shasum: *specsync-windows-x86_64.exe.zip: No such file or directory`. msys `sha256sum`
  defaults to binary mode and emits `HASH *name`; the packaging step reformatted that with
  `awk '{print $1"  "$2}'`, which carried the `*` into the filename field while respelling the
  separator as text mode. `HASH  *name` is valid in neither mode, so `shasum -c` looked for a
  file literally named `*specsync-windows-x86_64.exe.zip`. The deeper fault was that the step
  existed at all: `release.yml` had had a correct PowerShell packaging step since before v5.2.0,
  and `rc-assets.yml` reimplemented the same job from scratch in bash. The fix deleted the
  parallel implementation and copied the proven step verbatim — `Get-FileHash` yields the digest
  alone, so the line is assembled from parts that carry no mode marker, and `WriteAllBytes`
  rather than `Set-Content` so PowerShell cannot prepend a BOM.

  This layer is history, not a live guarantee: 6.0 ships no Windows binary and #722 deleted both
  the step and the target (see Removed). It is recorded because it is the layer that stood
  between a working RC asset lane and the first candidate anyone could download. No consumer was
  ever at risk — `action.yml`'s Windows path compared `awk '{print $1}'` against a recomputed
  digest and never parsed the filename field, so the malformed sidecar would have installed
  fine. It was the attach job's own pre-flight, deliberately stricter than the consumer, that
  refused to ship a sidecar a human running `sha256sum -c` could not read.

  **Layer 5 — a ruleset gate that had never once passed (#718).** From rc.2 the tag resolved and
  `resolve` reached its ruleset check, which had never succeeded since it landed in #492. It
  demanded three tag rulesets plus a release GitHub App, and none of the App-shaped pieces was
  ever provisioned: no `SpecSync final tag creation` ruleset, no `SPECSYNC_RELEASE_APP_ID`
  variable, no `SPECSYNC_RELEASE_APP_PRIVATE_KEY` secret, no `release` environment. The App id
  expanded to the empty string and argparse rejected `--release-app-id ""` before a single
  ruleset file was read, so the two rulesets that *do* exist were never verified once. A gate
  that always fails enforces nothing while appearing to.

  Qualification now requires exactly the two live rulesets, both `active` — `SpecSync immutable
  final tags` on `refs/tags/v*.*.*` excluding the RC pattern, and `SpecSync immutable RC tags` on
  `refs/tags/v*.*.*-rc.*`. No strictness was traded to reach a passing gate: both live payloads
  were checked against the unmodified strict validators before any edit, and `validate_tag_ruleset`
  lost its `release_app_id` parameter entirely, so there is no longer any code path by which a
  bypass actor could be admitted. What was given up is disclosed rather than dropped quietly —
  see the Changed entry below.

  **Layer 6 — an unobservable field required as evidence (#720).** rc.7 then failed with
  `final tag immutability ruleset is missing fields: bypass_actors`. GitHub returns
  `bypass_actors` only to a caller with admin access to repository settings; the workflow runs
  with `contents: read, actions: read, checks: read`, so the field is absent from every payload
  it fetches, and the validator listed it in `REQUIRED_RULESET_FIELDS`. This gate could never
  have been satisfied from CI. It was invisible locally for the mirror-image reason: a
  maintainer's `gh` is authenticated as an admin and does see the field.

  This is the release's most repeated defect shape — **a category empty for want of INPUT, read
  as a VERDICT** — appearing in the validator that enforces that distinction elsewhere. It is the
  same shape as #672 (a schema that could not be parsed reported as every table missing), #684
  (a `db_tables` spec with no `schema_dir` gating a release on advice the reader cannot take) and
  #689's first design (absent evidence read as "ready"). Here it ran in the opposite direction
  and was still wrong: an unobservable field was treated as a malformed payload rather than as
  something this caller cannot see.

  `bypass_actors` moved to `OPTIONAL_RULESET_FIELDS`, and absence now means UNOBSERVED. Present
  and empty passes; present and populated is still refused with `must not grant bypass to any
  actor`; absent is reported by `unobserved_bypass_notices` into the enforced disclosure list,
  naming each ruleset whose bypass list this token could not read. Reading absence as emptiness
  would have been worse than requiring it — a ruleset that genuinely grants bypass would then
  pass a green run.

  **Where the lane stands.** `v6.0.0-rc.8` and `v6.0.0-rc.9` clear `resolve` and `validate` and
  then run for twenty-four minutes into the three-platform qualification matrix, where they fail
  inside `qualify (windows)` — a different lane and a different problem. `promote` has still
  never executed, and cannot be rehearsed: `final_tag` is `v{version}` taken from the candidate's
  own RC tag, which `validate` pins to `Cargo.toml`, so any promote run against a real candidate
  mints the real `vX.Y.Z` — and the immutability ruleset then makes it permanent. For that job,
  the proof and the release are the same event.

- **An accepted change whose verification commit a rebase orphaned can be reopened**
  (CorvidLabs/spec-sync#673). `reopen_unarchived_change` gated on exactly one thing: whether
  the current delivery-input digest differed from the stale one. A rebase or a squash that
  leaves the verification commit unreachable changes no content, so `check` refused on
  reachability ("accepted change verification commit is not in current history…") while
  `reopen` refused because the inputs were current. Both statements were true simultaneously
  and no verb moved the record; the reporter rebuilt two changes from draft.

  The reporter's second observation is what turned this from "reopen is too strict" into a
  correctness fix: an unrelated one-line comment edit to an input file satisfied the gate and
  unlocked `reopen`. The digest comparison is a proxy for "is this evidence still good?", and
  the proxy refused the case it should admit while admitting a case it had no reason to. Its
  safety value was approximately zero and its obstruction value high.

  The three-way reachability disjunction — `verification_commit_is_accepted_current`,
  `accepted_workspace_is_integrated`, `accepted_change_is_recorded_on_remote_default` — is
  extracted into one resolver, `accepted_evidence_is_anchored`, which `reopen` asks as a
  question and every other caller still enforces as a refusal. All five existing callers of
  `authenticate_accepted_evidence` keep byte-identical behaviour through a wrapper, so no
  fourth parallel implementation was added. `reopen` now admits on `inputs_drifted ||
  !anchored`, and both refusal messages name both conditions.

  A trap the fix would otherwise have sprung: `reopened_change_preserves_sequence_history`
  independently encoded "a reopen implies the digests differ". An anchor-axis reopen has
  *equal* digests, so without a recorded cause it would have stripped `historical` status and
  frozen `change new` project-wide for any acknowledged-collision member. The reason is
  therefore recorded on the `ReopenRecord` as `ReopenCauseV1::VerificationCommitUnanchored` and
  read there, with `skip_serializing_if` so existing `approvals.json` stay byte-identical.

  The existing squash-merge test asserted `unwrap_err()` — it *pinned the defect*. It now pins
  both directions: anchored evidence with current inputs must still refuse, unanchored evidence
  with byte-identical inputs must succeed. Invariants 15 and 18 and REQ-change-017/018/034/035
  forbade this fix in as many words and were amended deliberately, invariant 18 having
  constrained audited reopen by name.

  Not fixed here, from the same report: the second deadlock, where `reopen` reports a stale
  definition approval while the change is `accepted` and `approve` refuses in that state.

- **A squash-merged change is recognised on the default branch by its archive path**
  (CorvidLabs/spec-sync#677). `accepted_change_is_recorded_in_ref` looked for
  `.specsync/changes/<id>/state.json` — the *active* path — at any commit on the reference. A
  workflow-v2 change is created, verified and archived inside one pull request; squash-merge
  that and the default branch receives a single commit in which the workspace is already under
  the archive, so the active path never appears there at all. Measured across this repository's
  own archives: the active path is present for 83 of 172, the archive path for 172 of 172,
  neither for 0.

  That is also where the alarming "100 of 171 archives would go red under an anchor check"
  figure came from. It had been read as a property of the archives; it was a property of the
  predicate, which is structurally unable to succeed for a squash-merged v2 change. The
  predicate now asks about the archive path as well when the record is `Archived`, and requires
  the state each location can actually hold — `Accepted` at the active path, `Archived` at the
  archive path — rather than accepting either state anywhere. A record the default branch has
  never seen in any location still reads as unrecorded.

  The issue's own central claim, that `validate_archived_integrity_inner` contains no anchor
  check at all, was retracted by its reporter after following the calls: the archived path *is*
  anchored, through `validate_finalization_evidence` on `finalization.implementation_commit`
  rather than through `verification.commit`. What survives is the narrower asymmetry —
  `verification.commit` is never consulted once archived, so orphaning that specific field is
  an error before archival and silent after — and that is unchanged here.

- **`init` and `change new` name the lifecycle when it is the legacy one**
  (CorvidLabs/spec-sync#678). A repository upgraded from 5.x keeps `version: 1` in
  `.specsync/sdd.json`, `init` short-circuits on an existing project and prints "already
  exists" without raising it, and every change created there is workflow-v1. Nothing said so
  until `ship` refused, several verbs later, by which point the change cannot be re-created on
  the other lifecycle without redoing the work.

  `print_legacy_policy_notice` (in `init`) and `print_legacy_workflow_notice` (in `change`
  identity output) announce **state**, not a verb: `workflow v1 (legacy) — this change uses
  change accept and change archive, not change finalize`, with the `change adopt` pointer as
  the minor half. That weighting came from the reporter retracting their own justification.
  They had said no adopt verb existed and that the affected repository was hand-authored; both
  were wrong — `change adopt` is listed plainly in `specsync change --help`, and that
  repository had run adopt properly months earlier. The better argument that replaced it:
  discoverability was never the failure mode, *knowing you needed it* was. Being told a command
  exists does not contradict an assumption you did not know you were making.

  A workflow-v2 project stays completely silent, so the normal path gains no noise. `init`'s
  notice returns early on `.specsync/workflow-v2-baseline.json`, which is deliberate rather
  than incidental: `change adopt` writes that baseline but leaves `.specsync/sdd.json` at
  `version: 1`, so an adopted repository runs v2 purely through the baseline-exists disjunct
  with the policy disjunct permanently false. Two sources of truth for one question,
  disagreeing in a supported configuration the tool's own adoption path produces. Recorded here
  rather than resolved.

- **A schema that cannot be replayed reports its tables as unknown, not as missing**
  (CorvidLabs/spec-sync#672). Two halves, and the first is what made the schema unreplayable in
  the first place.

  SQLite has no `ADD COLUMN IF NOT EXISTS`. The only way to add a column to existing databases
  while keeping fresh ones correct is to carry it in `CREATE TABLE` **and** replay a bare
  `ALTER TABLE ADD COLUMN` whose error the caller discards — in Go, literally `_, _ =
  db.Exec(stmt)`. The duplicate is intentional and load-bearing: remove either half and one of
  the two database populations is wrong. `apply_operation` rejected the second statement
  outright with `ALTER TABLE ADD duplicates existing column`, aborting the whole replay. A
  redeclaration that **agrees** with the existing column's type is now a no-op; one that
  **contradicts** it still fails, and the message names both types rather than saying
  "duplicates". A behaviour-only control asserts the type-conflict refusal on any binary, so
  the narrowing cannot be satisfied by deleting the check.

  The second half is the general one, and it is a shape this release has now hit repeatedly:
  **a category empty for want of an input, read as a verdict.** `get_schema_table_names`
  collapsed three outcomes into one empty `HashSet` — a schema that genuinely declares no
  tables, a schema that failed to replay, and a `schema_pattern` that failed to compile — and
  `add_missing_db_table_error` read the empty set as "these tables do not exist". So one
  unparseable migration reported *every* declared table as absent, including tables created
  correctly in an unrelated file, and each report advised adding a `CREATE TABLE` that was
  already present and correct.

  `schema_table_names_available` now answers whether the declared set is **known** as opposed
  to merely empty, and `Ok(false)` from `schema_table_exists` becomes an error only when it is.
  Resolved lazily behind an `Option<bool>` so the happy path never re-replays the schema. The
  parse failure still reports itself; it no longer gets a second, false story stacked on top of
  it.

- **`db_tables` declared without `schema_dir` is a notice, and no longer gates `--strict`**
  (CorvidLabs/spec-sync#684). Split out of #672 after measuring that #672's fix cannot reach
  it: that fix lives inside `if config.schema_dir.is_some()`, and this is the branch taken when
  it is `None`.

  A project whose schema is defined in application code rather than `.sql` migrations has
  nothing to point `schema_dir` at, and `schema_dir` is not a spec frontmatter field, so it
  cannot be scoped per module. The disclosure fired on every run forever, and warnings escalate
  to errors under `--strict` in `compute_exit_code`. The remaining choices were deleting a
  truthful `db_tables` declaration or abandoning `strict` — that is, giving up drift gating
  everywhere else. It is the same shape as #672 one layer out, and gating on advice the reader
  cannot take is not drift detection.

  The message moves from `result.warnings` to `result.notices`. `compute_exit_code` takes only
  errors and warnings, so a notice structurally cannot gate. Both halves matter: the gate is
  gone **and** the disclosure still prints, because silencing it would have regressed the
  visibility the original code comment was protecting. The suggested fix now says the schema
  may legitimately live in application code. Measured on a fixture matching the reporting
  repository's shape: `check --strict` went from exit 1 with `1 warning(s) treated as errors`
  to exit 0 with `⊘ DB table validation skipped: …`.

  Validation is unchanged wherever it can run — three states, three verdicts: no `schema_dir`
  is a notice; set but unreadable is unknown (#672); set, readable and the table absent is an
  **error**. The last is pinned by a vacuity control, because a fix that simply stopped
  checking `db_tables` would have passed the discriminator.

<!-- DISCREPANCY: the commit's third message narrows the duplicate guard's justification —
     "duplicate `## ADDED` already fails loudly ... MODIFIED is the one that resolves silently,
     so it is the one worth refusing" — but the guard as shipped compares (operation, target,
     key) and refuses a repeat under ANY operation. A duplicate `## ADDED` key is now refused
     at parse rather than at application. The entry describes the shipped guard. -->
- **A delta section carrying `###` subheadings no longer loses everything above the last one**
  (CorvidLabs/spec-sync#699). A delta declaring five `### Scenario` entries under `### SPEC
  SECTION Behavioral Examples` produced a living spec containing one. Three pre-existing
  documented contracts the change never touched — `change new`'s JSON contract, `change
  reopen`'s audit-record contract, and the `finalize` contract — were deleted from
  `cmd_change.spec.md` with exit 0. Which file was targeted decided the outcome:
  `change.spec.md` uses bold `**Scenario**` and survived, `cmd_change.spec.md` uses `###
  Scenario` and lost content, and 59 of 62 spec files use the vulnerable style.

  `parse_delta` called `flush(...)` at the **top** of the `### ` branch, before deciding
  whether the heading was an item heading or content. `flush` pushes an item and clears the
  body, so every content subheading ended the item and began a fresh body under the *same* key:
  one section with three scenarios became three items keyed alike, and application kept the
  last.

  #564 had already taught this grammar that a `###` inside an open item is content — it added
  exactly that branch — and left the flush above the classification. Half a fix, which is
  precisely why the symptom looked like a format limitation. The issue was filed on that
  reading: it claimed the subheading level was unrepresentable and proposed rejecting such
  deltas at `approve`. Implementing that would have reintroduced #564, since `scaffold` itself
  writes `### Structs & Enums` inside `## Public API`. Classification now happens first and
  `flush` runs only when the heading really is `REQUIREMENT` or `SPEC SECTION`.

  A delta declaring the same operation, target and key twice is now refused rather than
  resolved, with an error naming the operation, the target and how many bytes of the earlier
  body would have been discarded. That is the second route into the same silent loss.

  **The two halves must ship together, and the source says so where a backporter would see
  it.** Zero of 424 archived deltas are refused by the new guard — but two of them (CHG-0121
  `types.md`, CHG-0131 `deps.md`) contain duplicate MODIFIED keys that exist *only* because the
  old ordering split one section into several items. With the reordering they parse as single
  items and pass; the guard shipped alone would refuse them and those changes could never be
  re-materialized.

  Still open, found by the review of this fix: the guard compares `(operation, target, key)`,
  so the same key under two *different* operations is not a duplicate and still resolves
  last-write-wins.

- **A semantic delta cannot change after the approval that signed it**
  (CorvidLabs/spec-sync#704). Demonstrated end to end, not inferred: approve a delta, overwrite
  `deltas/<module>.md` with different wording, run `change check`. The canonical spec is
  rewritten with the new text, with no error and no warning, while the ledger records an
  approved definition covering wording no approver ever saw.

  Delta bodies were bound by nothing under workflow v2. The v1 definition digest hashed every
  delta payload through `definition_artifact_snapshot`, so editing one invalidated the
  approval; the v2 stable-scope projection deliberately hashes intent and boundary only, and
  nothing replaced that binding. `validate_delta_files` reads filenames, `project_input_digest`
  excludes `.specsync/changes/` by design, and the descendant walk that would have noticed
  passes 0 of 107 archived reviews. The one mechanism covering that region was inert by
  construction, which is why the gap was invisible. The threat model is stated honestly in the
  report: this needs local write access between approve and materialize, so it is not a remote
  attack. What it breaks is evidence integrity, and the same window is reachable without malice
  — a bad merge, a rebase resurrecting an older delta, an agent editing the wrong file, two
  changes racing on one workspace.

  `approve` now records `approved_delta_digests` on the definition approval event: one digest
  per module over the delta file's exact bytes, with the module name framed into the digest so
  moving a body from `deltas/a.md` to `deltas/b.md` cannot preserve it. Keyed by module because
  "the delta changed" is not an actionable message when a change owns nine specs. Only a
  definition gate records it — closing and finalization gates bind delivery evidence, and
  claiming they reviewed delta wording would be a lie written into the ledger.

  `materialize_change_deltas` and `accept_change_with_gate` verify it before
  `prepare_delta_application` runs. The materialization check sits deliberately **above** the
  `canonical_applied` short-circuit: once the deltas are applied that function stops writing
  specs, so a check below it would never see a swap on any run after the first, and the
  workspace would go on shipping a delta that no longer describes the spec it produced.

  An **absent** binding is unknown, never violated. All 183 archived changes carry none, so the
  check returns early on `None` rather than inventing a verdict from evidence nobody could have
  written — the same trap as #672 and #684 above. `Option` plus `skip_serializing_if` keeps the
  field out of persisted JSON when absent, so no existing digest moves and older ledgers
  re-serialize byte-identically. What that early return then cost is #719, below.

  The feature caught its own delta during development. A rebase onto main renumbered an
  invariant, the delta was regenerated from the merged spec, and materialization refused with
  the message naming `specsync change approve` as the remedy — the accidental case #704
  predicted, in a real rebase rather than a fixture.

<!-- DISCREPANCY: the subject claims "one canonical frontmatter reader". The diff unifies four
     STRIPPERS onto one and makes parse_frontmatter CRLF-tolerant, but two non-canonical
     frontmatter readers survive on main: registry.rs's line-wise `extract_module_name` and
     src/commands/lifecycle.rs's unanchored `find("---\n")`. The commit BODY is honest about
     this (steps 4 and 6 of #696's migration order are named as deliberately out of scope); the
     subject line is not. The entry states what is actually canonical and what is not. -->
- **`specsync view` renders a CRLF checkout, and one `strip_frontmatter` serves every caller**
  (CorvidLabs/spec-sync#696, CorvidLabs/spec-sync#709). `src/view.rs` read the file with
  `fs::read_to_string` and handed the raw bytes to `parser::parse_frontmatter`, whose
  `FRONTMATTER_RE` is `^---\n(.*?)\n---\n(.*)$` — LF only. In a clone with `core.autocrlf=true`
  that returned `None` for every spec, so `view` failed with "Cannot parse frontmatter" on all
  311 of them. A Windows binary was published and all sixteen CI jobs ran on Ubuntu, so the one
  platform that broke was the one never exercised. That is the same fact the `Removed` entry
  above gives as its second and stronger reason for dropping the Windows binary; the CRLF
  correctness fixed here is explicitly among what that entry retains, because a teammate on
  Windows commits CRLF files and a colleague on Linux reads them.

  There was no "normalize then parse" convention to have relied on. Measured: of the 39
  `parse_frontmatter` call sites outside `parser.rs`, 21 normalize and 18 do not. So
  `parse_frontmatter` normalizes itself, guarded on `content.contains('\r')` so an LF document
  allocates nothing and takes the borrowed path, and returns the LF-only `body` all 39 callers
  already assumed. That fixes `view` and the other 17 unnormalized sites without touching one
  of them; an obligation on 18 call sites is unenforceable and fails silently. A lone `\r` is
  content and is preserved.

  Then the four strippers become one. `change::strip_frontmatter` — already CRLF-correct from
  the lessons-pointer fix earlier in this release — was the only implementation correct on all
  six axes that separated them: LF, CRLF, a leading BOM, unterminated frontmatter, a closing
  delimiter at EOF, and a horizontal rule in the body. It is promoted to
  `parser::strip_frontmatter` and the other two are deleted rather than left alongside it. Both
  failed silently, in opposite directions:

  - `view::strip_frontmatter` was LF-only and rejected a closer at EOF, so a CRLF companion
    rendered its raw YAML block on screen under the `## Requirements` heading.
  - `change::strip_yaml_frontmatter` searched the whole document for `\n---\n` before trying
    `\r\n---\r\n`, which made it a **content deleter**: a CRLF artifact with one LF horizontal
    rule in its body lost everything above that rule. Its caller asks "is this artifact
    written?", so a completed design was refused as incomplete — and, in the other direction,
    an artifact that was only frontmatter closed at EOF was accepted as written.

  Blast radius measured, not assumed: all four strippers were simulated over all 2103 tracked
  `.md` files and produced zero disagreements, and no tracked file has CRLF or a leading BOM.
  This changes output for zero specs in this repository, which is exactly why it survived this
  long.

  The canonical stripper requires the delimiter line to be exactly `---`. A malformed opener
  (`---  ` with a trailing space, or `----`) is not frontmatter and the document is returned
  whole. That is deliberate — guessing at a malformed delimiter is how a body gets cut at a
  horizontal rule — but it means a caller counting prose sees the YAML as content, so the
  change closes the empty-artifact hole for well-formed openers and opens it for malformed
  ones. Filed as #716 rather than left in a footnote. `registry.rs` and
  `src/commands/lifecycle.rs` still read frontmatter their own way and were deliberately left
  out of scope.

  `.gitattributes` gained `eol=lf` pins for `.specsync/**/*.md` and `specs/**/*.md`
  (CorvidLabs/spec-sync#709). Delta bodies became byte-compared evidence with #704 above and
  the existing `.specsync/**/*.json` pattern never covered them; canonical specs feed
  `project_input_digest`, so a working tree rewritten to CRLF stales evidence for honest,
  unmodified work. The pins govern this repository's trees and are stated in the file as no
  substitute for readers that tolerate CRLF, since an adopter's repository, a tarball, or an
  archive extracted without Git is never covered by them.

- **A manifest that cannot be parsed no longer vetoes a configured `source_dirs`**
  (CorvidLabs/spec-sync#723). Manifest discovery exists to **infer** a source list the project
  did not state. `compute_coverage_checked` propagated
  `discover_from_manifests_checked_with_root`'s error unconditionally with `?`, and both
  `check` and `coverage` exit on it — before inspecting a single spec. Two other call sites
  already degraded (`config.rs`'s `detect_source_dirs` via `unwrap_or_else`, and
  `retained_config` via `Err(_)`); the one CI gates on did not.

  Measured in the field across the whole release line, not inferred from the call graph: every
  candidate from rc.1 through rc.7 exited 1 on `check --strict` and `coverage` for a Gradle
  project with an in-repo `includeBuild`, **with `source_dirs = ["app/src/main/java"]`
  explicitly configured**. `view` and `change new` worked. So the tool was usable for authoring
  and unusable for gating, which is the worst split available: a project can adopt it, get
  value, and discover the gap only when it tries to enforce anything. Their only working option
  was v5.2.0, which reported 24/29 files (82.76%).

  `retained_coverage_manifest` makes this a **precedence**, not a softening, and both halves
  are asserted together. `source_dirs` stated: the failure costs only manifest-declared module
  names, so it degrades to a notice and coverage proceeds over the declared list. `source_dirs`
  omitted: the list coverage would measure is itself discovery output, so the error still
  propagates and the command is inconclusive, exactly as before. The control is the second half
  — a change that merely stopped failing would pass the first one just as well.

  Telling those apart required knowing what the file said. `source_dirs_set` is a new
  `#[serde(skip)]` field recorded by both config loaders and by `retained_config`, following
  `enforcement_set`, because a configured `source_dirs = ["src"]` is indistinguishable from the
  `["src"]` default once loading is done. `retained_config` sets the flag from the same
  predicate that decides the fallback, so the two cannot disagree and let a *scanned* list be
  treated as a stated one.

  The notice is not optional and travels with the figures rather than to stderr, which is
  precisely what a CI job capturing stdout does not read (#570). Manifest modules seed module
  attribution, so a degraded run names fewer modules without specs than the tree holds — a
  report improved because part of the measurement stopped. `CoverageReport::manifest_notices`
  carries it for the same reason as `skipped_links`, printed by `write_manifest_notices` and in
  `print_check_markdown`, and included in `coverage_json` and `cmd_check`'s JSON payload
  because machine consumers acting on `passed` are exactly who cannot see the text disclosure.

  This is the class-level half, and it earned its keep within days: the parser fix below did
  not cover the form the same adopter reported next, and the precedence rule caught it anyway
  (CorvidLabs/spec-sync#725). Any manifest, in any ecosystem, that a future parser cannot read
  is no longer able to override an explicit declaration.

- **An in-repo Gradle `includeBuild` is judged by its path, configuration block and all**
  (CorvidLabs/spec-sync#723, CorvidLabs/spec-sync#725). Two reports from the same adopter, days
  apart, on one declaration.

  `reject_non_leading_gradle_includes` decided on the token prefix alone: any executable token
  starting with `include` produced `Unsupported Gradle workspace mutator {token}`, and the
  argument was never read. So `includeBuild("vendor/podo-shared")` — an ordinary in-repo
  composite build, the common and correct usage — failed identically to
  `includeBuild("../outside")`, which escapes the repository. Every fixture the guard was
  written against used `../outside`, so no test could fail for this reason: the guard was built
  for escapes, caught every composite build, and the two cases were indistinguishable.

  `gradle_include_build_target` now parses the path from inside the parentheses and confines it
  through `normalize_gradle_project_relative_path`. A single complete string literal resolving
  beneath the project root is accepted and then ignored — it names a separate build, and the
  root build's own `include(...)` list is the only thing this parser derives modules from. A
  `../` escape, an interpolated or otherwise dynamic expression, and more than one argument all
  keep failing closed, and the refusal now names the path rather than the token, which is how a
  reader tells an escape from a form the parser does not model. `includeFlat` and
  `includeWorkspace` stay refused deliberately: `includeFlat` resolves against the *parent* of
  the root, so its argument is outside the project by construction, and `includeWorkspace` is
  not a form this parser models — reading their arguments would not make either supportable.

  Position is not judged, and that asymmetry with `include` is deliberate: a conditional or
  block-scoped `include` makes the module set unknowable from the text, while a composite build
  contributes no module whether or not its branch runs. Getting there took a correction inside
  the same change — the same conditional composite build was accepted written across three
  lines and refused written on one, because only in the second case did the enclosing `}` land
  on the declaration's own line. A verdict that turns on where the author pressed Enter is a
  bug whichever way it is settled.

  **The first fix then refused the common form.** #723 read the path argument while
  deliberately keeping any trailing expression refused, and `includeBuild(path) {
  dependencySubstitution { … } }` is the normal spelling — substituting a local project for a
  published coordinate is the reason to declare a composite build at all. The parser therefore
  accepted the bare minority form and rejected the common one, with `Unsupported trailing
  Gradle includeBuild declaration expression`. Reported against rc.8 by the same adopter, and
  by then a degraded notice rather than an outage, because the precedence fix above already had
  it.

  A configuration block carries substitution rules, not project declarations, so an
  `includeBuild` contributes no module and no source directory with or without one.
  `skip_gradle_include_build_configuration_block` skips a balanced block whole, and the
  skipping is confined to locating where the declaration ends:

  - the path is parsed and confined from inside the parentheses in front of the block, so
    `includeBuild("../outside") { … }` still fails on the path — and now says so, instead of
    blaming the block;
  - the block's text is never removed from the parsed content, so a block-scoped `include`, a
    `projectDir` mutation, or an unrecognized `project(...)` mutation written inside it still
    fails closed exactly as it does anywhere else;
  - the brace scan is quote- and escape-aware and runs after `strip_gradle_comments`, so a
    brace in a string or a comment moves the depth in neither direction;
  - an unbalanced block is refused, because its extent is exactly what is unknown.

- **A definition approval cannot withdraw the delta binding an earlier one recorded**
  (CorvidLabs/spec-sync#719). The binding added for #704 above returns early when the effective
  definition approval records none, because every approval written before the field existed
  carries none and absent evidence must read as unknown rather than as tampering. `change
  approve --portable-5-0-1` defeated that: `append_portable_definition_approval_v501` appended
  two `definition`-gate approvals with `approved_delta_digests: None`, and
  `effective_definition_approval` selects the **last** definition event. A change that had just
  recorded a digest ended up with an effective approval recording none. A compatibility path
  meaning "written before the binding existed" was made to mean "this approver declines to
  say".

  **The filed mechanism is not the one that generalises, and the fix corrected it.** The
  sequence in the report — approve, `approve --portable-5-0-1`, swap the delta, materialize —
  already refuses on unfixed `main`, for an unrelated reason: the portable projection is
  workflow-v1 only, and a v1 definition digest hashes every delta payload through
  `definition_artifact_snapshot`, so `ensure_definition_approval_valid` catches the swap one
  line before the binding is consulted, with `portable definition approval pair is malformed or
  stale`. On v1 the downgrade therefore costs recorded evidence and a correct diagnostic rather
  than a materialization — and the diagnostic that does fire is actively harmful, because it
  points the reader at re-running `--portable-5-0-1`, which re-approves the swapped wording and
  again records no claim about it. The consequence that generalises is **workflow v2's**, where
  the stable-scope digest hashes intent and boundary only and this binding is the whole of what
  stands between a swapped body and the canonical spec. The fix's second discriminator
  therefore constructs the downgraded *shape* directly rather than replaying the filed
  sequence, and asserts on the canonical spec's contents rather than on an error string:
  against unfixed `main` it produces no message at all — `materialize_change_deltas` returns
  with `canonical_applied: true` and the spec contains `BACKDOOR`.

  The rule is monotonicity, on both sides. On the write side the portable pair records the
  wording it approves, on both members, read after `validate_delta_files` so the claim is the
  wording this actor is approving now rather than something inherited from an older event.
  Carrying it forward rather than refusing the approve is what the rest of the module already
  does — `append_approval` records it for every definition gate, and the normalizing approval
  in `accept_change` carries it forward with that reasoning in a comment — and refusing would
  have removed the only route an adopter has to a 5.0.1-verifiable approval on a change the
  current binary already approved. The projection is untouched and pinned rather than assumed:
  `approved_delta_digests` is an input to none of `definition_digest`, the 5.0.1 projection
  bytes, or `definition_approval_pair_id`, and `ApprovalLedger` tolerates unknown fields by
  design, so a 5.0.1 reader still parses the record it came for.

  On the read side absence is still trusted, and now qualified. Absence is a property of a
  **ledger**, not of an event: a ledger that has recorded a delta digest for this change is
  demonstrably new enough to record one again, so a later definition approval carrying none is
  not silence from before the binding — it is a claim being withdrawn. That state is refused,
  naming `specsync change approve <id>` as the remedy, while absence is trusted whenever no
  definition approval in that ledger records a digest.

  Refusing it costs history nothing, measured rather than argued: all 197 `approvals.json`
  files under `.specsync/` were scanned for the newly refused shape and none matches, because
  archived ledgers carry no digest on any definition approval and take the untouched path
  exactly as before. The honestly labelled control — a pre-binding ledger holding several
  silent definition approvals, body swapped, materialization required to succeed — passes on
  the unfixed binary too, which is the point: it is what fails if absence is ever made to fail
  closed. This is the #672 and #684 shape inverted. There, an empty result was read as a
  verdict; here, the guard built to avoid exactly that could be re-armed into silence by a
  later writer.

<!-- DISCREPANCY: the issue's closing comment says "All four sites now name the second-order
     cost" and quotes the extracted wording as though all four shared it. Two do. The other two
     carry hand-written paraphrases that the pinning test does not cover, and both drop exits the
     helper names: `src/cli.rs:519-521` (`change finalize` help) omits "or those are reopened",
     and `src/commands/change.rs:781` (the `verifying` next-action line) omits both exits. The
     fix for "two verbs each composing their own prose for the same step" left two of four sites
     composing their own prose — the exact regression its test was written to guard against,
     present at authorship rather than introduced later. The entry below matches the code. -->
- **The merge-before-finalize warning names the cost that lands on other people's changes**
  (CorvidLabs/spec-sync#687). It said merging first "orphans verification evidence and strands
  the change", which prices the loss as one record — the reader's own, and locally recoverable.
  Measured on a real repository the cost is larger and lands elsewhere: an unfinalized change
  never reaches `accepted` or `archived`, so it never becomes an "accepted or archived
  successor", and every **earlier accepted change sharing a delivery input** with it can no
  longer archive. That second-order effect is why an accepted pile can grow without any single
  merge decision looking wrong — each one individually small and locally recoverable, the
  aggregate a lifecycle that cannot drain.

  Four sites now name that cost: both `ship-status` arms, the `verifying` next-action line, and
  the `change finalize` CLI help. The two `ship-status` arms share one pure
  `merge_before_finalize_warning(still_active)`, pinned by a test asserting whose work is
  blocked, what is blocked, and both exits — "until this one is finalized or those are reopened"
  — because the likeliest regression is a future refactor shrinking it back to "strands the
  change". The next-action line and the CLI help state the same cost in their own words and are
  not covered by that test; each names fewer exits than the helper does.

  Only the wording changed. The issue's own predicted remedy — that finalizing the `verifying`
  successors would let the blocked predecessor archive — was measured and **refuted**: the
  successor could not archive either, because archiving it would invalidate the predecessor. Each
  error named the other side as the thing that must move first, and neither said the counterpart
  was symmetrically blocked, so an operator following the printed next step from either end never
  learns they are in a two-sided block. Coupled set on one spec: 13 changes.

- **A squash-merged change can still be finalized** (CorvidLabs/spec-sync#689).
  `ship_status_report` decided `ready_to_finalize` by asking whether `verification.commit` was
  reachable from `HEAD`. A squash-merge rewrites that commit, so `merge-base --is-ancestor` is
  permanently false for a change whose evidence is perfectly intact — and squash is the only
  strategy this repository permits, as it is for many. Measured here: 19 of 172 archived changes
  have a reachable verification commit, so the "guarantee" held one time in nine.

  The rest of the module had already settled this. `verification_is_current` is content-only, and
  the ancestry walk was removed from those paths long ago with the reasoning recorded inline: it
  is a history-trust question, it is `attest`'s job, and on `verification.commit` it was freshness
  wearing a trust costume. `ship-status` was the one caller that never got the change. Readiness
  now asks `recorded_verification_is_current` — does the recorded plan and tree still match what
  was verified — and the blocker text changed with it, from "verification commit is not an
  ancestor of HEAD" to "verification evidence is stale for the current tree; re-run change check
  --commit before review/finalize".

  The first proposal was returned REDESIGN REQUIRED for this release's most-repeated defect
  shape: its replacement predicate would have reported **ready when verification was simply
  absent** — a category empty for want of input, read as a verdict, which is the same shape as
  the bug being fixed. `recorded_verification_is_current` loads the evidence itself and treats
  missing or unreadable evidence as not current. It is deliberately total rather than `?`-strict,
  because a strict propagate would turn `ship-status` from rc=0 into rc=1 on a workspace whose
  evidence is already damaged, and the fix for an inspection command must not brick inspection.

  Ancestry is preserved where it is load-bearing and could not be tightened away: over
  history-discovered commits it is genuine trust, and it stays.

  Three attempts were needed for an honest discriminator. The first passed on the baseline binary
  and so discriminated nothing; the second measured a thread-local `project_input_digest` memo
  inside a read scope rather than the predicate — a single process cannot observe that digest
  move, so a control that measures a cache is not a control. The third asserts at
  `ship_status_report` level against a binary built from a separate checkout. Its companion unit
  test is labelled a CHARACTERIZATION test in the source rather than a discriminator, because
  content currency was never broken.

- **`ship` names the lesson fold-back, not only `finalize`** (CorvidLabs/spec-sync#700). #697
  shipped the loop's third stage half-wired. `finalize_change` writes `lesson-bundle.md`
  correctly and the `finalize` command names the fold-back in its `next_action`, but `ship`
  composed its own next-action string and did not. `ship` is the verb the tool recommends —
  `ship-status` says to run it — so on the primary path the bundle was assembled and nothing said
  it existed. That is the exact failure the loop was built to end, knowledge produced where
  nobody looks, reproduced inside the loop on its own recommended path. It was found by running
  `ship` for real and reading the output; the unit suite passed throughout, because nothing tested
  *which verb emits which guidance*.

  The cause is the shape #687 already hit: two verbs each composing their own prose for the same
  step, with nothing pinning them together. The push/wait/siblings matrix is now one pure
  `ship_next_action(push, wait, siblings_before, fold_targets, bundle)`; the existing tail is
  preserved exactly and the fold-back clause is **prepended** — first, because the merge is what
  makes skipping it permanent. `lessons_next_action` was deliberately not reused: it ends in
  "then merge the PR on GitHub" while ship's tail is conditional on `--push`, `--wait` and
  siblings, so reuse would have emitted two different merge instructions in one sentence. The
  verbs share the clause, not the sentence.

  The independent review then caught the same asymmetry one level down: `ship --json` omitted
  `lesson_bundle` while `finalize --json` emitted it. Both now emit it. It also caught a doc
  comment stolen by a missing blank line — Rust concatenated the two blocks, leaving
  `lessons_next_action` undocumented and `ship_next_action` inheriting rationale that reads wrong
  for it — and a control test that promised byte-identical guidance while only asserting the
  absence of "write lessons", which a reworded tail would have passed, and that skipped the
  reachable `(false, true)` combination. Tightening the control caught a wrong expectation rather
  than a bug: `--wait` without `--push` collapses into the no-push branch, which is pre-existing,
  and is now pinned with a comment saying the control records that behaviour without endorsing
  it.

  This is also the change that performed the fold for the first time, moving #697's archived
  bundle into the `change`, `cmd_change` and `generator` context companions. Doing it showed why
  it had never happened: only 4 of 178 archived changes had ever touched a `context.md`.

- **A CRLF checkout is no longer told that every untouched module has recorded lessons**
  (CorvidLabs/spec-sync#701). Byte-identical generated scaffolds differing only in line endings
  behaved differently at `change new`: the LF one was correctly silent, the CRLF one printed
  `specs/<module>/context.md (1 line(s)) — read before scoping this change`. A Windows-authored
  project was told every untouched module had recorded knowledge — a pointer to nothing, which
  trains the reader to ignore the pointer, and it lands on new adopters, who are exactly who the
  proposal stage is for. Found by end-to-end sandbox testing in a genuinely new project; every
  unit fixture was LF.

  Two causes, either one silent on its own:

  1. `strip_frontmatter` matched the literal `---\n`, so CRLF frontmatter survived into the
     content count. It now terminates on a closing delimiter *line* in either encoding, without
     reintroducing the horizontal-rule truncation #697 removed, and keeps the whole document when
     frontmatter is unterminated rather than guessing where it ended.
  2. Scaffold comparison used the raw `CONTEXT_TEMPLATE`, whose unexpanded `spec: {module}.spec.md`
     can never equal a real file's `spec: <module>.spec.md`. The generator now hands out the
     expanded scaffold via `generated_context_scaffold(module)`.

  Cause 2 was invisible on LF because frontmatter was stripped before it could matter; it only
  appears when stripping fails. Fixing either alone leaves the other latent.

  Two comments in the first draft claimed things that are false, and were corrected rather than
  left standing once the review caught them. `parser.rs` does not accept `---\r\n` — its
  frontmatter regex is `^---\n`, LF-only, and roughly 28 call sites normalize `\r\n` before the
  parser sees the text. That matters beyond accuracy: handling CRLF here instead of normalizing
  at the boundary keeps the borrowed `&str` return but makes this a parser with its own dialect,
  a fourth position rather than a step toward one definition, so #696 now carries the repo-wide
  decision. And the claim that this helper is behaviourally identical to `view::strip_frontmatter`
  stopped being true the moment it learned CRLF: `view` is still LF-only and still rejects a
  closer at EOF, so unifying them is now a behaviour change for `view` rather than the no-op it
  would have been.

  Discrimination was measured on the shipped binary before the fix rather than by reverting in
  place, and the fixtures are the real `generated_context_scaffold(module)` converted to CRLF
  rather than hand-written lookalikes — following the lesson #697's own loop had folded into
  `specs/generator/context.md`, that a scaffold defect is invisible to dogfooding here because no
  untouched scaffold exists in this repository to trip over.

- **`watch` names the directories it is not watching, and no longer reports a pass over an empty
  check** (CorvidLabs/spec-sync#577). Two claims, both untrue.

  The watch set was built by testing each configured directory with `is_dir()` and keeping the ones
  that passed. A `specs_dir` or `source_dirs` entry that did not exist fell out with no record it had
  ever been named, and the banner then listed the survivors — which reads as the complete list. The
  path the operator got wrong was exactly the one omitted.

  Separately, `watch` forks `specsync check` and read the child's exit status. A check that finds no
  specs prints `No spec files found in <dir>/` and exits 0, because in bare mode that is
  informational — #560 settled that `--strict` is where it gates. `watch` could not tell "examined N
  specs, all clean" from "examined none", so it printed a green `All checks passed!` over a run that
  checked nothing.

  Both halves are the release's defect class: a set empty or short for want of input, read as an
  absence of problems.

  Resolving the watch set now records skipped paths beside watched ones, and each is reported with
  its role on stderr in both human and JSON modes, so the stdout banner stays parseable. `run_check`
  claims a pass only on positive evidence that specs were examined.

  A missing directory is still non-fatal — `watch` is a long-running dev loop and one bad path of
  several should not stop it — an empty watch set still exits 1, `check` is untouched, and a real,
  passing spec set still reports `All checks passed!`.

- **A stale sequence ledger is no longer committed backwards** (CorvidLabs/spec-sync#533).
  A branch created before the default branch advanced carries an older `change-sequence.json`.
  `change check --commit` ran `git add -A` over it, so SpecSync's own materialize commit rewrote
  the high-water mark downwards — no human `git add` involved. The next change then reused an
  allocated number.

  `floor_sequence_ledger_to_committed` reads HEAD's committed ledger before staging and raises the
  working tree to it, merging `acknowledged_collisions` rather than discarding them, and returns
  the before/after so the caller can disclose the raise on stderr — where it survives `--quiet` and
  stays off a `--format json` payload. It refuses rather than guesses when the committed ledger is
  unparseable. All three staging sites are covered: materialize, verification evidence, and archive.

  `validate_change_sequences` gained the matching read-side gate: a ledger below the default
  branch's recorded high-water mark is now an error naming the recovery command, so the divergence
  is caught on read as well as prevented on write.

  A change already at or ahead of the committed mark is untouched, so the repair cannot be
  satisfied by raising every ledger.

- **A directory named in `files:` scores zero, not eighty** (CorvidLabs/spec-sync#573).
  The same spec scored `80/100 [B]` and exited 0 under `score --strict` while `check` on the
  identical tree hard-failed. The 80 was not arbitrary: freshness awarded 15/15 because the path
  existed — `is_dir()` was never asked — and the API dimension scored 0 because `read_to_string`
  on a directory returns `Err`, which the scorer folded into "documents nothing yet". A spec that
  could not be read at all was graded as one that was merely incomplete.

  This is the release's defect class in the scoring path: a dimension empty for want of a readable
  file, read as an absence of documented exports.

  `ExportScan` gains a `Directory` variant, classified by a shared `files_entry_is_directory`
  predicate *before* the read, so the decision is made once at the scan rather than re-derived at
  each consumer. Six consumers handle it and the compiler enforces coverage. `score` remains a
  metric rather than a hard failure — the spec totals 0 with grade F, below every strict and
  minimum floor, so `--explain` and machine-readable output still render for the affected spec.
  `check` is untouched; it was already correct.

  A spec naming a real source file scores exactly as before, so the rule cannot be satisfied by
  lowering scores generally.

### Changed

- **20 integration tests that had never compiled now run** (CorvidLabs/spec-sync#585).
  `tests/integration/regression_w1.rs` arrived in `9a00223b` and was never `#[path]`-declared in
  `tests/integration.rs`. `Cargo.toml` defines no `[[test]]` targets, so Cargo auto-discovers
  `tests/*.rs` — and `tests/integration/` is a subdirectory, whose files must each be registered
  explicitly. One never was.

  It was not compile-rot: the file compiled on the first attempt, against the same helper API it was
  written for. That is what made it invisible. An unregistered file contributes no failures, and no
  failures reads as green, so 20 regression assertions were absent from every passing run they
  appeared to be part of — the release's own defect class, a category empty for want of input and
  read as want of problems.

  Running them found two real defects, filed as #605 (`report --require-coverage` is unreachable
  when staleness is unmeasurable — `--require-coverage 0` also exits 1, so the gate is not failing,
  it is not running) and #606 (`deps` emits two findings for one missing dependency). A third, #607,
  is sharper: the *passing* sibling test would have passed with `--require-coverage` deleted from the
  codebase, because its fixture exited 1 for every argument. Its fixture now varies with the flag.

  A guard asserts set equality between the files in `tests/integration/` and the `#[path]`
  declarations, so an orphan names itself. It lives inline in `tests/integration.rs`, because a guard
  placed in `tests/integration/` could be orphaned and would then be the thing it exists to detect.

- **`src/change.rs` is 17,460 lines, down from 29,983** (CorvidLabs/spec-sync#589). Its 309 tests
  moved to `src/change_tests.rs`, declared with `#[cfg(test)] #[path]` so the module stays inline and
  `use super::*` still reaches every private item.

  The size was not cosmetic. The defect this release has been chasing — a fix landing at the site
  named in the bug report while a parallel implementation survives — has happened **seven times**,
  and you cannot sweep for a sibling in a file you cannot hold in your head. Ten of the thirteen
  remaining known bugs live in this file.

  A pure move, proved by counting rather than by reading a 12,543-line diff: 309 `#[test]` functions
  before and after, 2275 unit and 374 integration tests passing both times, and an unchanged drill
  board. The 24 `#[cfg(test)]` helpers and fault-injection hooks stay — production paths reference
  them, so they are not test code that merely lives near production code.

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

- **Coverage no longer invents a parent module for language-specific specs**
  (CorvidLabs/spec-sync#529). A repository mapping `src/strutil.{py,mjs,rb,lua,sh}` to five
  language-specific specs read `Modules without specs (1): ⚠ strutil/` and `File coverage: 5/5
  (100%)` in the same report — a module nobody wrote, printed with a trailing slash over a
  directory that does not exist, telling a multi-language project it was incompletely specced at
  100% file and LOC coverage.

  "Does this module have a spec?" was decided by looking for a spec **directory of that name**.
  Nothing about the module's files was consulted — `specced_files` is computed a few lines above
  and never reached the module blocks — so the absence of the *name* `strutil` was read as the
  absence of a *spec*. That is this release's defect class with the sign flipped: rather than a
  category emptied for want of input, a value invented where there was no input.

  Four separate blocks made that claim on the same evidence — configured modules, manifest
  modules, source subdirectories, and flat-file stems — and only the last one is named in the
  issue. **All four now have to show a gap in their own files.** A module whose discovered files
  were all looked at and all found mapped is covered, whatever the specs are called.
  Owning no discovered file is *not* such a showing: it is the absence of input, so a directory
  holding nothing measurable keeps its report, and so does any module with even one unmapped
  file. spec-sync's own tree had two of these phantoms — `specsync` from `Cargo.toml` and
  `change_tests` from `src/change_tests.rs` — beside 106/106 files covered.

  `generate` consumes that same list, so `specsync generate` and the MCP `generate` tool no
  longer offer to write `specs/strutil/strutil.spec.md` claiming files another spec already owns.

- **A refused `reopen` no longer destroys the archive** (CorvidLabs/spec-sync#539). This was the only
  confirmed unrecoverable bug in 6.0. `reopen` moved the dated archive package into the active
  workspace and *then* ran its preconditions — every one of which correctly returns an error on a
  healthy tree with no drift. So a **correctly refused reopen consumed the archive**, leaving an
  orphan whose `state.json` still said `archived`. Every recovery verb refused it, and if the archive
  tip had never been committed the package was simply gone.

  Validate-then-move was not available: the checks read the workspace through `find_change_dir`, and
  `authenticate_accepted_evidence` runs `validate_archived_accepted_snapshot` only while the record is
  Archived, so validating first would have silently changed what those checks mean. The fix uses the
  move-then-restore pattern `archive_change` already had for #540 — the correct shape was one function
  away — and the refusal now says `archive restored`, or names the path to recover by hand if the
  restore itself fails.
- **One unreadable change workspace no longer hides every healthy one**
  (CorvidLabs/spec-sync#443). `change list` and `change status` printed
  `No active SDD changes.` and exited 0 when any single workspace was malformed —
  indistinguishable from an empty project, while `change show` and `change audit`
  on the identical tree hard-errored with the path and line:column.

  Two defects compounded. Enumeration aborted on the first bad `state.json`, so
  healthy siblings were never collected; then `list_changes_checked().unwrap_or_default()`
  turned the resulting `Err` into an empty list, so the failure was not merely
  unhandled but unrepresentable. A category was empty for want of input, and the
  listing read that as want of changes.

  The roster now reports both halves: every readable change, and every workspace it
  could not read, named with its reason. It exits non-zero when the view is partial
  — `change audit` already did so on the same tree, and two commands disagreeing
  about whether one repository is healthy is precisely the class of defect this
  release has been closing. A project with genuinely no active changes is
  unaffected and still exits 0.

  **This is also what made mixed-version corruption invisible**
  (CorvidLabs/spec-sync#603). A 6.0 record rewritten by a 5.2 binary loses
  `workflow_version`, and 6.0 detects that correctly — `workflow-v1 change <id> was
  not present at the trusted pre-v2 cutoff <sha>` — but `list` and `status` were
  swallowing the detection and reporting the change as absent rather than as
  damaged. No version stamp had to move: the refusal already existed and was being
  discarded.

  Three further callers were reading the same empty roster as fact. The
  pull-request diff base was selected from it; `ship-status` reported no other
  changes in flight when it could not tell; and `ship` inferred which change to
  ship from whatever remained readable. The last of those writes commits. All three
  now refuse rather than guess.

  **JSON note:** `list` and `status` keep their historical bare-array shape whenever
  every workspace is readable. A degraded roster is reported as an object carrying
  `changes` and `unreadable`, because an array cannot say "and there were three I
  could not read".
- **`cargo test --release` is green again** (CorvidLabs/spec-sync#581). On a clean `main` it exited
  101 with five integration failures — `generate exited before the post-coverage barrier`, `tool
  exited before the directory-enumeration barrier` — which read as real TOCTOU regressions in the
  root-identity guards and were not. `cargo test` passed, so anyone who ran the release profile
  before shipping saw five failures nobody else could reproduce.

  Each of those tests spawns the real binary, waits for it to publish a marker at a synchronisation
  point, swaps a path underneath it, then asserts the command refuses to report a result. Those
  synchronisation points are `#[cfg(debug_assertions)]` and compile to `Ok(())` in release, so the
  marker never appears, the child runs to completion, and the test polls for a file that will never
  exist.

  **The shipped binary was never weaker.** Every `#[cfg(debug_assertions)]` item in `src/` is a test
  *rendezvous*, not a guard. The guards they synchronise — `verify_public_path`,
  `verify_coverage_project_root`, `ConfinedReadRoot::revalidate_before_success`, and the identity
  comparison in `open_server_root_capability` — are compiled unconditionally. A release binary put
  under a live symlink race still refuses, with `Coverage project root … changed during retained
  traversal` and exit 1. Compiling the rendezvous into release instead would have shipped an
  env-var-triggered wait loop of up to 30 seconds — and on `revalidate_before_success` that is
  *every* read-success path, not one — plus a file write at a caller-named path. So the seven
  affected tests (five on Unix, two Windows-only carrying the identical defect) are gated with
  `cfg_attr(not(debug_assertions), ignore)`, which keeps them compiled and type-checked in release
  and **visible in the run output** rather than silently absent, as a bare `cfg` would leave them.

  **Release-profile coverage is lost for three guards, stated plainly.**
  `verify_coverage_project_root`, `verify_public_path` and `revalidate_before_success` now have no
  release-runnable test; `verify_public_path` has no unit coverage in *any* profile, because
  `src/commands/generate.rs` has no `mod tests` at all. The other three keep release coverage
  through in-process `#[cfg(test)]` seams that do run under `--release`. And **no pipeline runs
  `cargo test --release` today** — `ci.yml` runs `cargo test --verbose`, the RC lane runs
  `cargo test`, both debug builds — so nothing caught this and nothing will catch its return.

- **A warm hash cache no longer drops findings** (CorvidLabs/spec-sync#429). The same command over
  the same tree disagreed with itself depending on run history: the first run reported
  `specs_checked: 1` and warned about an undocumented export; the second reported
  `{passed: true, specs_checked: 0, warnings: []}`. `--force` and `--no-cache` restored the warning —
  the cache was working correctly, and that is precisely why the findings vanished.

  This was the most serious of the remaining false-green defects, because it made **every other
  result conditional on run history**. A green board, a clean CI run, a passing gate — each was only
  as trustworthy as whether the cache happened to be cold.

  The cache legitimately skips re-validation rather than only re-extraction, so the previous verdict
  has to survive. It did not: the snapshot types for storing a per-spec result **already existed and
  were never wired to the live path**. Results are now stored alongside the hash and replayed for a
  skipped spec, which is counted in `specs_checked` with its warnings named.

- **`deps` no longer reports a clean graph built from imports it could not resolve**
  (CorvidLabs/spec-sync#477). Kotlin import analysis was skipped entirely: a fixture with real
  imports reported `✓ All dependency declarations are valid` with rc=0, having collected nothing.

  The first fix for that reproduced it one layer down. It collected the imports, then matched each
  package prefix only against directory *suffixes* and let `filter_map` **drop whatever failed to
  resolve, with no record** — so a Kotlin file whose directory did not mirror its package still
  produced zero edges and exit 0. The fix for "zero edges reported as zero problems" reported zero
  edges as zero problems.

  Resolution now reads each file's own `package` declaration before falling back to layout, and
  "unresolvable" is a distinct reported outcome rather than an absent one: an import is owned by a
  module, foreign to the project's namespace, or **unattributed and disclosed**. When nothing is
  known about the project's packages an unowned import is unattributed rather than foreign, so
  silence is never the default. The disclosure never changes the exit code.

- **Gradle module identity now comes from the project name, not a source path segment**
  (CorvidLabs/spec-sync#473). Discovery took the **first** path segment, so `com.example.foo` and
  `com.example.bar` both became a module named `com` and `generate` wrote `specs/com/com.spec.md` for
  an entire package tree.

  The reframing matters more than the fix: every other ecosystem derives identity from its
  **manifest** — Cargo `[package]`, Swift `.target(name:)`, npm `name`, pubspec, pyproject, Go's
  module path — with directory scanning only as the no-manifest fallback. Gradle already used that
  rule for `settings.gradle` includes; a *single-project* build never inserts a module, so the shared
  fallback saw `src/main/kotlin/com`. Gradle was the one language naming a module from a path at all.

  A single-project build is now named from `rootProject.name`, falling back to the project directory
  — Gradle's own default. Children of JVM source roots are not modules, so there is no longer a
  segment to choose wrongly.

- **Ruby private methods are no longer published as exports** (CorvidLabs/spec-sync#479). The issue
  title describes the opposite of the defect: nothing "escapes extraction". Methods sitting *below*
  `private` were wrongly **added** to the export set.

  The extractor's block-opener test was anchored to a line's first token, so an assignment-form
  conditional (`coarse = if seconds < 3600 … end`) never pushed a nesting entry while its `end` still
  popped one. The stack desynced by one, the class's visibility-restore entry popped early, `public`
  flipped back mid-body, and every method after that point leaked — including the private ones.

  This was dangerous rather than noisy: the leak surfaces as a *warning*, and the obvious way to
  silence a warning is to document the symbol — which made `check` accept it, **publishing a private
  method as public contract**. The bug recruited the user into making it permanent.

- **`score`, `new`, `generate`, `scaffold` and `diff` now honour the configured export level**
  (CorvidLabs/spec-sync#474). On a project configuring `export_level = "type"`, `check` reported
  `2/2 exports documented` while `score` deducted 12 points for three "undocumented exports" that
  were not part of the configured surface at all — two commands disagreeing about what a module's
  API is.

  The cause was a convenience wrapper that hard-coded both the export level and the parse mode, and
  it had five callers. The unreported ones matter more than the reported one: **`specsync new`
  generated specs that `specsync check` then rejected** (`Spec documents 'id' but no matching export
  found in source`), and `diff --json` reported `"new_exports": ["id","name","find"]` as drift — in
  PR comments — for symbols the contract never claimed.

  A project configuring `parse_mode = "ast"` also got AST parsing in `check` and regex everywhere
  else. One configuration now produces one answer.

- **Two values that stood in for measurements nobody took** (follow-up to CorvidLabs/spec-sync#572
  and CorvidLabs/spec-sync#583, both caught by their own sandbox gates after the fixes had merged).

  `report` correctly refused a tree whose staleness could not be measured — and then printed
  `1 total, 0 stale` with `"stale_modules": 0`. The per-module fields were already `null`; the
  **aggregate** still said zero, so a dashboard scraping "N stale" read no drift from a run that
  measured none. Text now says `stale unknown` and JSON emits `null`, with a number only when at
  least one module was actually measured.

  And `config.rs`'s hand-rolled scanner **silently skipped any line it did not recognise**, so a
  typo'd `[rules]` header disabled every rule while `check` reported success. An unterminated header
  is now a load failure, worded identically to the unreadable-file refusal so a consumer matching on
  one need not know which door produced it.

- **The config refusal now guards both loaders** (CorvidLabs/spec-sync#583). #570 installed it at
  `load_and_discover` and called that "a single choke point" in the source. It was not:
  `config::load_config` is a second door, and `rules`, `compact` and `rehash` all came through it,
  each exiting 0 over a config file that existed and could not be read.

  **`rehash` is the one that mattered.** It does not merely report — it regenerated
  `.specsync/hashes.json` from specs interpreted under default configuration, and that cache is what
  later `check` runs consult to decide which specs are unchanged and can be skipped. A broken config
  did not produce one wrong answer; it wrote a stale-skip cache that silently shortened every
  subsequent run.

  The default now refuses and the permissive loader must be requested by name, so the repair paths
  keep working while a caller added later gets the guard unless it deliberately opts out.

- **Machine-readable formats no longer report fewer findings than the text output**
  (CorvidLabs/spec-sync#576). On a failing tree, `check --format table` and `--format csv` printed a
  summary and **named no finding while exiting 1** — a CI job parsing the CSV saw zero rows and
  concluded the tree was clean, while the exit code disagreed. `coverage --format json`, the payload
  an agent reads, carried **no findings at all**.

  Table and csv were never implemented for `check`: they shared the text arm, which printed only the
  summary, while per-finding output sat behind a `Text`-only guard. Every format now draws from one
  finding list, and the three hand-built coverage payloads — CLI and both MCP surfaces — collapse to
  a single constructor, which previously disagreed even on key names.

  Staleness findings drive the exit code but are gathered separately, so a format that omitted them
  exited non-zero naming nothing. They now reach every non-text format, including `--format github`,
  the PR-comment renderer.

- **A tree with unresolved merge conflicts no longer passes `check`**
  (CorvidLabs/spec-sync#578). A source file with `sub` on one side of a hunk and `mul` on the other,
  with a spec documenting both, reported `✓ 3/3 exports documented` and exit 0. The extractors
  parsed **both sides as ordinary declarations**, so the union satisfied the spec and spec-sync
  green-lit a tree that does not compile. Spec bodies carrying markers passed the same way.

  The naive guard was unshippable: this repository contains twelve complete, well-formed conflict
  triples inside test string literals, two of them in files a spec maps and `check` scans on every
  run. Detection therefore composes two signals — git's own unmerged list, which is authoritative
  and cannot false-positive, and the symptom itself: declarations read from **both** sides of one
  hunk. A triple inside a string literal yields declarations on neither side, so this repository
  still passes. The error names which side contributed which symbols rather than announcing that
  markers exist.

- **Staleness that cannot be measured is refused, not reported as zero drift**
  (CorvidLabs/spec-sync#572, CorvidLabs/spec-sync#586). On a tree six commits behind its source with
  `.git` removed, `stale` refused while `report` answered `"stale": false, "commits_behind": 0` and
  exited 0 — two commands, one tree, opposite answers. An unborn HEAD did the same.

  There were **four** implementations of the same computation and only `stale.rs` guarded the
  precondition; its guard cites #558, a fix that landed there and nowhere else. `report`,
  `check --stale`, the lifecycle `no_stale` transition and the `score` freshness dimension all read
  the absence of history as an absence of drift. The scoring case was the sharpest: **deleting
  `.git` raised a spec's grade a full letter** (77/100 C to 80/100 B), and `--min-score` gates on
  that number.

  `git_utils` now exposes the precondition as a value distinguishing "no repository" from "no
  commits", every reader refuses in the shape its own file already used, and freshness points that
  could not be measured are withheld rather than awarded.

- **Coverage over zero source files no longer reports 100% anywhere**
  (CorvidLabs/spec-sync#582). #562 was fixed in `src/output.rs` and reached one of nine sites; the
  other eight were each only reachable through a format or transport the original report never
  used. Text said `0/0 (no source files to measure)` while `report`, `coverage --json`, and both
  MCP surfaces said the project was fully covered. Worse, `--require-coverage 80` over such a tree
  **exited 1 while the same run's JSON printed `"coverage_percent": 100`** — the gate reads the
  counts, the payload read a precomputed field.

  The precomputed `coverage_percent` / `loc_coverage_percent` fields are removed from
  `CoverageReport` and replaced by accessors returning `Option`, `None` when the denominator is
  zero. This makes the omission impossible to repeat: every renderer is now a compile error until
  it states what it shows when nothing was measured. JSON emits `null`; `--require-coverage` fails
  closed rather than comparing against a substituted 100.

- **`specsync comment` now exits with the verdict it just printed**
  (CorvidLabs/spec-sync#571). The command computed an exit code precisely so its PR comment
  would agree with CI, rendered `## ❌ SpecSync: Failed` from it — and then returned normally,
  so the process exited `0`. Used as a CI step, `comment` was a permanent pass that posted its
  own failure. It was also the only command ignoring `--require-coverage`: `check`, `score`,
  `report` and `deps` all exit `1` over a 0%-covered tree while `comment` exited `0`.

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
