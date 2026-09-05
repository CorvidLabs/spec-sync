## MODIFIED

### REQUIREMENT REQ-change-020

Audited reacceptance SHALL preserve compatible legacy definition evidence while enforcing immutable reopened definitions, fresh evidence, explicit semantic succession, and validation of every current canonical contract it reapproves.

Acceptance Criteria

- A prior verification digest using the transitional explicit-false lifecycle encoding remains compatible with the stable omitted-false encoding during reopened reacceptance.
- An accepted no-spec change cannot satisfy successor governance even when its paths and specs overlap.
- A supported pre-approval supersede transition records a durable definition-bound predecessor edge with explicit path/module/predecessor-digest obligations.
- Closing evidence binds each adopted obligation only when the same successor has the module's semantic delta and an exact old/new transition from its trusted definition-signed base tree to its descendant unique accepted-transition tree — or, while that transition is not yet in history, to the working tree its closing evidence was signed against; the acceptance commit's immediate parent is not the before tree.
- Every owner of a changed input requires its own same-successor path/module obligation; owner intersection and cross-record path/spec unions fail closed.
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

### REQUIREMENT REQ-change-036

Stale accepted-change verification diagnostics SHALL name the offending delivery input and state the concrete remediation, without changing the underlying freshness model.

Acceptance Criteria

- A changed covered input that no accepted or archived successor claims reports the input path, its owner module, and the `specsync change reopen <id>` remediation.
- A changed covered input that one or more successors claimed and were refused for reports the input path, its owner module, and each refused successor with the reason it was refused — its evidence did not authenticate, its manifest could not be resolved, its evidence carries no tuple for the input, its manifest does not carry the tuple's successor entry, its delta has no semantic item, its tuple does not hold or could not be evaluated, or its own delivery-input evidence is stale — sorted by successor ID; a refusal is never reported as the absence of a successor.
- When the stale change is workflow v1 and a refused successor is workflow v2, the diagnostic directs the operator to finish that successor (`specsync change status <successor>` names its next step) and does not offer `specsync change reopen` of the legacy change, whose replayed canonical delta would overwrite the successor's materialization; every other combination directs the operator to verify and accept a covering successor or reopen the accepted change.
- A covered input that disappeared from the current inventory reports the missing path and the restore-or-reopen remediation; a changed exact-only input reports the path and the audited-reopen remediation; missing delivery-input evidence keeps its established phrase and gains the reopen remediation.
- Every stale reason remains deterministic: sorted successor IDs, no timestamps, and no environment-dependent content.
- The `accepted change verification is stale for current delivery inputs` check prefix, the terminal-evidence validity values, and every freshness predicate remain unchanged.

### REQUIREMENT REQ-change-audit-project-001

The change module SHALL expose `audit_project` that validates active change workspaces and living SDD policy/spec coherence without rewalking archived terminal evidence by default.

Acceptance Criteria

- `audit_project` does not load or re-authenticate every archived change's terminal evidence.
- An active terminal record is judged against every accepted or archived change as a successor candidate: archived records are loaded only when such a record exists, only as candidates and never for evaluation, and only one that declares a matching obligation is authenticated. A legacy accepted change superseded by a finalized successor is successor-covered on the audit path exactly as on the full walk, and a refused archived successor is named with its reason.
- `check_project` remains available for full integrity including archives (tests / rare callers).
- CLI project-health surface uses the active-only path.

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
| Covered delivery input disappears from the current inventory | Unified check names the missing path and the restore-or-reopen remediation |
| Non-inert local registry cannot be parsed while resolving a module | Canonical path resolution fails closed with `failed to parse local registry {path} while resolving `{module}`` |
| A repeated stage-zero path has a different mode or object ID | Git candidate inspection fails closed without replacing the first observation |
| Correction ledger is invalid when a definition mutation acquires the project lock | Mutation emits the safe integrity diagnostic and persists no lifecycle update |
| Effective definition approval records no semantic delta wording while an earlier definition approval in the same ledger recorded it | Materialization and acceptance refuse the withdrawn claim and name `specsync change approve <id>` as the remedy |
