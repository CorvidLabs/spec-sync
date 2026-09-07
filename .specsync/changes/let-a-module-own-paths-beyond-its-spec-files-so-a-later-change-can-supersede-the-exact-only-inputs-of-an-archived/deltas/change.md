## ADDED

### REQUIREMENT REQ-change-095

The change lifecycle SHALL let a module own delivery paths beyond its spec's `files:` through `[modules."<name>"] owns` in the project configuration, and SHALL judge a successor's eligibility to supersede a predecessor entry signed under a reserved exact owner by the module that owns the path now rather than by the frozen label.

Acceptance Criteria

- An `owns` entry is a project-relative file, or a directory that owns everything beneath it, matched the way `affected_paths` scopes are; an acceptance manifest signs a matching path under every declared module that owns it, ahead of the reserved `@exact:test` and `@exact:delivery` classes, and a directory entry takes ownership like a file.
- Configured ownership reaches acceptance manifests and semantic succession only: an owned path is not a source mapping, `specsync check` demands no spec coverage for it, a spec's `files:` list still does not lift a mapped test out of `@exact:test`, and no path under `.specsync/` or among the protected SDD paths is configurable.
- `change supersede` accepts a module for a predecessor entry whose signed owners are all reserved exact labels when the module owns the path under the current configuration, refuses it otherwise naming the frozen label and the `owns` remedy without persisting anything, and keeps refusing a module that is not a signed owner of an entry a module signed.
- The succession tuple is unchanged — the successor's module, the predecessor entry digest, and the successor entry digest, with the digest-matches-base-tree rule intact — and no owner correction, reopen, or additional audit record is required to supersede an exact-only entry.
- A workflow-v2 successor that edits, deletes, and re-signs exact-only inputs of an archived bootstrap change finalizes, and the bootstrap is successor-covered on the full walk and on the active-only audit before and after the archive commit.

## MODIFIED

### REQUIREMENT REQ-change-020

Audited reacceptance SHALL preserve compatible legacy definition evidence while enforcing immutable reopened definitions, fresh evidence, explicit semantic succession, and validation of every current canonical contract it reapproves.

Acceptance Criteria

- A prior verification digest using the transitional explicit-false lifecycle encoding remains compatible with the stable omitted-false encoding during reopened reacceptance.
- An accepted no-spec change cannot satisfy successor governance even when its paths and specs overlap.
- A supported pre-approval supersede transition records a durable definition-bound predecessor edge with explicit path/module/predecessor-digest obligations.
- Closing evidence binds each adopted obligation only when the same successor has the module's semantic delta and an exact old/new transition from its trusted definition-signed base tree to its descendant unique accepted-transition tree — or, while that transition is not yet in history, to the working tree its closing evidence was signed against; the acceptance commit's immediate parent is not the before tree.
- Every owner of a changed input requires its own same-successor path/module obligation; owner intersection and cross-record path/spec unions fail closed.
- A changed input whose signed owners are all reserved exact labels requires one obligation from a successor whose module owns the path under the current configuration; a `supersede` declaration for such an entry is admitted only for a module that owns the path now, and refused otherwise with the frozen label and the `[modules."<name>"] owns` remedy named, while a module that is not a signed owner of an entry a module signed is still refused.
- A reopened canonical-applied change validates its current canonical modules without replaying its already-applied semantic delta.
- Strict project checks reject a reopened definition that reacceptance would reject.
- Definition reapproval keeps a canonical-applied reopened record in verifying so fresh evidence remains mandatory.
- Nested project history lookup anchors repository-relative workspace state paths at the Git repository top.
- Reopen rejects a request when the shared validator reports exact or successor-covered evidence.

### REQUIREMENT REQ-change-024

Strict lifecycle checking SHALL permit only explicit closing-valid terminal semantic successors to govern changed inputs of an accepted predecessor without hiding unrelated stale evidence.

Acceptance Criteria

- Draft, approved, implementing, verifying, failed, stale, tampered, no-spec, semantically empty, and partial successors never suppress predecessor errors.
- Accepted or authenticated archived successors selected as candidates require valid definition, verification, closing approval, history integration, and recursive exact-or-successor-covered current inputs; standalone archives require historical integrity without equality to today's inputs.
- Every changed input expands to one obligation per signed canonical owner and every obligation matches one exact predecessor/path/module/old-digest/new-digest tuple from the same successor.
- Multiple terminal successors may cover disjoint obligations, while cycles fail closed and completed validity results are memoized.
- The archive preflight forwards its closing token into the successor walk, so the package being closed authenticates as a successor of the changes it supersedes exactly as its own historical-integrity preflight authenticates it; every reading path passes no token and is judged by history alone.
- A successor whose acceptance transition is not in history — the package being closed, or an archive whose commit has not been made — is admitted only through its working-tree closing evidence, and its succession tuple is then checked against the working tree that evidence signed, with base ancestry checked against HEAD; once the archive commit exists, history is again the sole anchor.
- A changed input whose signed owners are all reserved exact labels names no module of its own: its claimants are the modules under which later accepted or archived successors declared it, each claimant is judged by every check a signed owner's successor passes, one authenticated claimant covers the input, and otherwise every refused claimant is named with its reason and the frozen label as the input's owner.

### REQUIREMENT REQ-change-036

Stale accepted-change verification diagnostics SHALL name the offending delivery input and state the concrete remediation, without changing the underlying freshness model.

Acceptance Criteria

- A changed covered input that no accepted or archived successor claims reports the input path, its owner module, and the `specsync change reopen <id>` remediation.
- A changed covered input that one or more successors claimed and were refused for reports the input path, its owner module, and each refused successor with the reason it was refused — its evidence did not authenticate, its manifest could not be resolved, its evidence carries no tuple for the input, its manifest does not carry the tuple's successor entry, its delta has no semantic item, its tuple does not hold or could not be evaluated, or its own delivery-input evidence is stale — sorted by successor ID; a refusal is never reported as the absence of a successor.
- When the stale change is workflow v1 and a refused successor is workflow v2, the diagnostic directs the operator to finish that successor (`specsync change status <successor>` names its next step) and does not offer `specsync change reopen` of the legacy change, whose replayed canonical delta would overwrite the successor's materialization; every other combination directs the operator to verify and accept a covering successor or reopen the accepted change.
- A covered input that disappeared from the current inventory reports the missing path and the restore-or-reopen remediation; a changed exact-only input that no successor claims reports the path and the audited-reopen remediation, and names the supersede alternative under `[modules."<name>"] owns` wherever the configuration can grant the path; a changed exact-only input whose claimants were all refused is reported like a signed owner's input, with the frozen label as its owner; missing delivery-input evidence keeps its established phrase and gains the reopen remediation.
- Every stale reason remains deterministic: sorted successor IDs, no timestamps, and no environment-dependent content.
- The `accepted change verification is stale for current delivery inputs` check prefix, the terminal-evidence validity values, and every freshness predicate remain unchanged.

### SPEC SECTION Error Cases

| Condition | Behavior |
|-----------|----------|
| Missing acceptance criteria or affected scope | Definition approval fails |
| Missing or invalid semantic delta | Approval, verification, and unified check fail |
| Populated semantic delta with no recognized operation heading | Approval and historical validation name the allowed `## Added`, `## Modified`, and `## Removed` headings instead of reporting the file empty |
| Spec documents an export that is not in source | `change check` fails with the spec finding; configured test commands are not run |
| HEAD changes after verification | Acceptance requires re-verification |
| Any intervening commit changes a disallowed path, even if later reverted | Status and strict checking require re-verification in every environment |
| Accepted delivery evidence is still current AND its verification commit is still anchored | Reopen is rejected without changing lifecycle or audit state |
| Accepted verification commit is unreachable and no reachable history records the acceptance | Reopen is admitted and records `VerificationCommitUnanchored`, even when delivery inputs are byte-identical |
| Reopen actor or reason is empty | Reopen is rejected before any mutation |
| Concurrent changes edit the same semantic key | Progress requires dependency ordering or rebase |
| Ownership correction is not exact, additive, in-scope, and canonically provable | Correction is rejected transactionally |
| Covered delivery input of an accepted change changes and no successor claims it | Unified check names the input path, its owner, and the `change reopen` remediation |
| Covered delivery input changes while every successor that claimed it was refused | Unified check names the input and each refused successor with its reason, sorted by ID; a workflow-v1 predecessor with a refused workflow-v2 successor is directed to finish that successor, never to `change reopen` |
| Archive preflight finds that the package being closed is the only successor covering a legacy change it supersedes | The preflight authenticates that package with its closing token and checks its succession tuples against the working tree its closing evidence signed; finalize succeeds and the predecessor is successor-covered before and after the archive commit |
| `change audit` judges an active legacy accepted change whose changed inputs a finalized (archived) successor covers | Archives are offered as successor candidates without being evaluated; the predecessor is reported successor-covered, and a refused archived successor is named with its reason |
| `change supersede` names a module for a predecessor entry signed under reserved exact owners that the module does not own now | The obligation is refused, naming the frozen label and the `[modules."<name>"] owns` remedy, and nothing is persisted |
| Changed exact-only delivery input that no successor claims | Unified check names the input, the audited-reopen remediation, and — wherever the configuration can grant the path — the supersede alternative |
| Changed exact-only delivery input whose claimants were all refused | Unified check names the input with its frozen label as owner and each refused claimant with its reason, exactly as for a signed owner's input |
| Covered delivery input disappears from the current inventory | Unified check names the missing path and the restore-or-reopen remediation |
| Non-inert local registry cannot be parsed while resolving a module | Canonical path resolution fails closed with `failed to parse local registry {path} while resolving `{module}`` |
| A repeated stage-zero path has a different mode or object ID | Git candidate inspection fails closed without replacing the first observation |
| Correction ledger is invalid when a definition mutation acquires the project lock | Mutation emits the safe integrity diagnostic and persists no lifecycle update |
| Effective definition approval records no semantic delta wording while an earlier definition approval in the same ledger recorded it | Materialization and acceptance refuse the withdrawn claim and name `specsync change approve <id>` as the remedy |
