## MODIFIED

### REQUIREMENT REQ-change-117

The system SHALL permit an audited reopen of an active workflow-v2 accepted record with a historical scoped-review workspace mismatch only when its entire raw package exactly matches the current remote-default Git tree, that commit is an ancestor of HEAD, and all other definition, execution, identity, closing and finalization digest checks pass.

#### Acceptance Criteria

- The mismatched old review remains invalid for ordinary acceptance and archive.
- Recovery preserves every original byte through a recorded immutable Git commit and returns only to verifying.
- Uncommitted changes, extra or missing files, mode or content tampering, missing remote-default evidence, and other invalid closing fields refuse without mutation.
- Fresh verification and a newly current scoped review are mandatory before closing again.
- Existing bounds and legacy serialized cause values are unchanged; old records serialize identically when the optional anchor is absent.
- Within the exact recorded workspace-mismatch recovery, the inconsistent old review hash is retained as untrusted evidence rather than authenticated. The finalization self-digest and all non-review digests remain strict.
- A review-hash mismatch without a review-workspace mismatch remains refused without mutation.


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
| Later files expand a previously accepted directory scope beyond the evidence payload bound | Recursive accepted-input validation captures only authenticated signed manifest entries; fresh acceptance and reopen keep full discovery and unchanged resource bounds, and signed input drift remains fail-closed |
| An active v2 finalization has a mismatched historical review workspace but its entire raw package matches the current remote-default tree | Reopen may record that exact Git anchor and downgrade to verifying after all other authentication checks; ordinary acceptance/archive remain strict and fresh verification plus scoped review are required |
| The mismatched package is modified, unanchored, not active v2, or has any other invalid finalization digest | Recovery refuses without changing lifecycle or evidence bytes |
| The exactly recorded mismatched old review also has an inconsistent review hash | Reopen-only recovery preserves both as untrusted at the same anchor; finalization self-digest and all non-review checks remain strict, and a pure review-hash mismatch remains refused |
