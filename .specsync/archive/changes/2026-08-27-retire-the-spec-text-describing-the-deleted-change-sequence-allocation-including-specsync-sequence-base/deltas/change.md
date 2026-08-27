# Retire the change-sequence allocation the ordinal retirement deleted

## MODIFIED

### REQUIREMENT REQ-change-022

The lifecycle SHALL prevent parallel branches from silently merging duplicate numeric change sequences while preserving exact historical collision evidence.

Acceptance Criteria

- Active and archived records are scanned together by numeric `CHG-NNNN` sequence.
- Unacknowledged duplicate sequences fail with every conflicting full ID and path.
- Nothing claims a next ID, so the duplicates this gate finds are historical ordinals brought together by a merge rather than two branches minting the same number.
- Existing accepted collisions can be baselined exactly without rewriting accepted state or evidence.

### REQUIREMENT REQ-change-026

The lifecycle SHALL treat canonical numeric sequence claims and historical collision acknowledgements as protected exact repository evidence across arbitrarily wide numeric sequences.

Acceptance Criteria

- Numeric change sequences contain at least four ASCII digits, use exactly four zero-padded digits below 10000, and use unpadded decimal digits at or above 10000.
- Successor identity ordering compares parsed numeric sequence first and full canonical ID second, so `CHG-10000-*` follows `CHG-9999-*` while acknowledged same-sequence collisions remain deterministic.
- Malformed, noncanonical-width, and numerically unrepresentable IDs fail closed instead of participating in successor ordering.
- The committed sequence ledger always requires lifecycle coverage even when `.specsync/` is ignored.
- A change that edits the ledger covers it as a delivery input; no change generates a claim to cover, because nothing allocates a sequence.
- An acknowledgement matches the exact currently located ID set and remains valid only when every member is accepted or archived.
- Removed IDs, added IDs, single surviving records, and draft, approved, implementing, or verifying collision members fail closed.

### REQUIREMENT REQ-change-070

A lifecycle commit SHALL NOT record a change sequence ledger below the highest sequence already committed, and SHALL disclose any raise it performs.

Acceptance Criteria
- Before staging, a working-tree ledger lower than the committed high-water mark is raised to it, so no lifecycle commit can lower the recorded mark.
- A working-tree ledger at or above the committed mark is left exactly as the author wrote it, because raising is the only direction this rule may move a ledger and a mark that is already higher is not a regression to repair.
- The raise is reported on a stream that survives quiet output and does not contaminate a machine-readable payload, naming both the previous and the adopted value.
- Acknowledged collisions recorded on either side are preserved across the raise rather than replaced by one side's copy.
- Every staging site in the lifecycle applies the rule, so a commit path added later cannot reintroduce the regression by bypassing one of them.

### REQUIREMENT REQ-change-072

The change sequence ledger gate SHALL judge a ledger against the highest mark the current branch has itself recorded, and SHALL NOT refuse a branch for trailing the default branch.

Acceptance Criteria
- A branch whose ledger is older than the default branch's, but consistent with its own history, is accepted. Nothing mints an ordinal any more, so trailing the default branch cannot lead to reminting one.
- A ledger below the highest mark the branch itself recorded is refused, including when the branch raised the ledger and then rewrote it downwards to a value still above the point at which it diverged.
- The gate consults no remote, so a repository without an origin is judged by the same rule rather than having the gate silently disabled.
- The refusal names the mark that was lost and a recovery command that applies to the branch's own history.

### SPEC SECTION Invariants

1. Change IDs are minted from the change description as a slug and are unique across active and archived workspaces; the historical `CHG-NNNN-slug` ordinals are read for collision accounting and never allocated.
2. No emergency or force transition bypass exists.
3. Approval digests exclude volatile lifecycle state. The workflow-v1 definition digest hashes every selected artifact and semantic delta body; the workflow-v2 stable scope digest hashes intent and boundary only, and delta bodies are bound instead by a per-module content digest recorded on the definition approval event.
4. Any addition, removal, or replacement in approved stable scope invalidates approval until the new digest is approved.
5. Finalization rejects stale commits, contracts, reviews, incomplete tasks, failed tests, and missing requirement evidence.
6. Overlapping active semantic keys are blocked unless changes declare ordering dependencies.
7. Canonical spec versions increment and changelogs reference the accepted change ID.
8. A failed multi-file write restores all prior canonical content.
9. Change dependencies are acyclic and must be accepted or archived before dependent implementation begins.
10. Meaningful-path coverage compares the branch with the current GitHub/remote default base after a rebase, falling back to the recorded creation commit only when no remote base is available.
11. Approval digests hash repository-relative artifact paths so identical Git content validates across checkout locations and operating systems.
12. Verification command detection prefers portable project-manifest commands and uses Fledge only when no native manifest is available.
13. Persisted and hashed project paths use forward slashes on every operating system.
14. Quiet reporting executes every configured command and preserves failures while suppressing only child stdout and stderr; normal checking and verification continue streaming diagnostics.
15. Reopening accepted evidence is rejected only when its delivery inputs are current AND its verification commit is still anchored in current history; reopening stale evidence never reapplies an already canonical semantic delta.
16. Reacceptance of an already-applied change requires the definition digest captured by the latest audited reopen event unless every difference is a validated additive exact-owner correction.
17. False default lifecycle fields remain absent from new persisted state, while definition validation recognizes both omitted and transitional explicit-false encodings so upgrades preserve existing approvals and verification; explicit acceptance appends stable definition evidence when the latest compatible approval uses the transitional encoding.
18. An unreachable verification commit is itself an admissible staleness axis for audited reopen, recorded as an explicit cause in the reopen ledger; every other authentication check stays fatal, and reacceptance still requires canonical acceptance or deterministic semantic succession provable from trusted history.
19. Acceptance appends a Change Log row matching the canonical table's existing column schema and uses the post-bump version when the schema includes `Version`.
20. Generated bookkeeping never replaces explicit delivery scope; registry authority, policy enablement, and native command identity are evaluated consistently before lifecycle enforcement.
21. Trusted correction-history discovery ignores unresolved remote-default references and parses Git tree paths without quoting ambiguity; regression fixtures preserve quoted-path coverage where supported while remaining valid on Windows.
22. Local and hosted verification freshness inspect every intervening commit against every parent, permit only `state.json`, `verification.json`, `verification-attempts.json`, `review.json`, and `review-attempts.json` below canonical active-change IDs, and never infer safety from a net diff or broad volatile-path exclusion.
23. Exact-owner corrections are additive, restricted to an original affected path and a current canonical source owner, and cannot mutate semantic definition fields or prior evidence.
24. A fully valid later accepted sequence owner covers only historical sequence-ledger drift; reconstruction reuses exact committed collision-owner ledger bytes when available, while the current owner and every non-ledger input remain exact.
25. A structurally valid audited delivery reopen preserves immutable sequence-collision history while fresh verification and closing approval remain mandatory.
26. Accepted-change archival trusts an in-history commit recording the change as accepted with byte-identical evidence when no first-acceptance transition anchor matches, so squash-merged evidence remains archivable while the exactly-one-eligible rule stays fail-closed.
27. Legacy acceptance-manifest reconstruction assigns the exact delivery owner to production-source inputs with no deterministic canonical owner, so adoption-era archived ledgers validate without remediation while newly signed evidence stays fail-closed.
28. Batch exact-owner correction validates every proposed path/module pair independently and fails closed with zero persisted mutations when any entry is invalid.
29. The 5.0 ledger migration backfills reopening digest fields idempotently from recorded evidence only, verifies each repair before writing, and never mutates ledgers it cannot repair deterministically.
30. Canonical module path resolution treats missing and inert local registries as absent fallbacks while non-inert unparsable registries still fail closed with the established parse diagnostic.
31. Immutable workflow-origin validation follows every bounded reachable canonical dated archive path for the exact change ID, preserving identity across archive, reopen, and cross-date rearchive moves.
32. The workflow-v2 baseline retains its exact introduction bytes at every bounded touching commit and readable parent, rejecting rewrite-then-restore history.
33. Answer, dependency, and supersession mutations load and validate correction history only after acquiring the lifecycle project lock.
34. `finalize_change` assembles `lesson-bundle.md` into the archive on a best-effort basis: a bundle failure never undoes a completed archival, and the material is read entirely from disk so finalize keeps working offline and in CI. SpecSync assembles and never authors the lessons; the agent that ran `finalize` writes them, guided by `next_action`.
35. This module defines no frontmatter reader of its own. Lesson counting, archived lesson bundles, and artifact completeness all read through `parser::strip_frontmatter`, the single canonical implementation, which ends frontmatter at its CLOSING delimiter LINE in either LF or CRLF encoding and never at the next `---` elsewhere in the document. A Markdown horizontal rule therefore never truncates a body to a fragment, a CRLF-authored companion is stripped exactly as an LF one is, and the two failure modes the module's own strippers had are gone with them: a written CRLF artifact is no longer refused as incomplete, and an artifact that is only frontmatter closed at end of file is no longer accepted as written.
36. A `###` heading inside an open semantic-delta item is section CONTENT and does not end that item. Only `### REQUIREMENT <id>` and `### SPEC SECTION <name>` start a new item, and classification happens before the previous item is flushed — otherwise one section carrying subheadings becomes several items under one key and application keeps only the last, silently discarding documented behaviour the change never touched.
37. A semantic delta declaring the same operation, target and key more than once is REFUSED. Applying it would keep the last body and discard the earlier ones with no diagnostic.
38. Semantic delta bodies are bound to the definition approval that signed them: approval records a digest over each delta file's exact bytes keyed by module, and materialization and acceptance refuse to rewrite a canonical spec when a body no longer matches, naming every module that drifted. An approval recording no such digest predates the binding and reads as unknown, never as tampering, so every historical archive remains valid.
39. Markdown under `.specsync/` is pinned to `eol=lf` in `.gitattributes`, beside the JSON already pinned there and for the reason that file already states: change artifacts and semantic delta bodies are read as lifecycle evidence, so a working tree that rewrites them into CRLF makes honest, unmodified work arrive in non-canonical form. The pin governs this repository's own working trees; it is not a substitute for readers that tolerate CRLF, because an adopter's repository, a tarball, or an archive extracted without Git is never covered by it.
40. A recorded delta binding is MONOTONE within one approval ledger. Every writer of a `definition` gate records the per-module delta digest it approved — ordinary approval, the normalizing approval inside explicit acceptance, and both members of the portable SpecSync 5.0.1 pair alike — so within a ledger the binding only ever goes from absent to present. An effective definition approval that records no delta wording while another definition approval in the same ledger records it is a claim being withdrawn, not evidence predating the binding, and materialization and acceptance refuse it and name the re-approval remedy. Absence across a whole ledger still reads as unknown, because that is the only shape recorded history has: a change is either from before the binding existed or from after it, never both.

## REMOVED

### REQUIREMENT REQ-change-055

Retired. Every clause it stated was deleted by the ordinal retirement (#665): the local allocation
floor, the remote default-branch high-water floor, and the `SPECSYNC_SEQUENCE_BASE` range. Change
identity is now a slug minted from the description (REQ-change-086), nothing allocates a sequence,
and `SPECSYNC_SEQUENCE_BASE` has no reader. The one invariant worth keeping from its subject — that
the ledger may never be recorded downwards — is stated by REQ-change-070 for the commit side and
REQ-change-072 for the validation gate, so nothing is lost by removing it rather than rewriting it.

### REQUIREMENT REQ-change-071

Retired. Its normative SHALL — refuse a ledger below the mark the default branch has published — is
directly reversed by REQ-change-072, which requires that a branch NOT be refused for trailing the
default branch and that the gate consult no remote. Its remaining acceptance criterion named the
allocation floor as the shared source for the published mark, and that floor was deleted by the
ordinal retirement (#665). REQ-change-072 is the sole live statement of this gate.
