## MODIFIED

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
35. This module defines no frontmatter reader of its own. Lesson counting, archived lesson bundles, and artifact completeness all read through `parser::strip_frontmatter`, the single canonical implementation, which ends frontmatter at its CLOSING delimiter LINE in either LF or CRLF encoding and never at the next `---` elsewhere in the document. A Markdown horizontal rule therefore never truncates a body to a fragment, a CRLF-authored companion is stripped exactly as an LF one is, a leading BOM never hides the opening delimiter, and a delimiter line padded with trailing whitespace still ends the block at BOTH ends. Four failure modes of the strippers this module used to own are gone with them: a written CRLF artifact is no longer refused as incomplete; an artifact that is only frontmatter is no longer accepted as written when it is closed at end of file, prefixed with a BOM, or opened with a delimiter carrying a trailing space; and prose above the first horizontal rule in a body is no longer deleted when the CLOSING delimiter carries one. One residual is stated rather than guessed at: an artifact opened with `----`, a Markdown thematic break and not a delimiter, still reads as written even when it holds nothing but frontmatter — accepting it as a delimiter would cut real bodies at their first rule, which is the worse failure, and deriving the gate from the generated scaffold instead would not close it either, because a file with a mangled opener no longer equals that scaffold.
36. A `###` heading inside an open semantic-delta item is section CONTENT and does not end that item. Only `### REQUIREMENT <id>` and `### SPEC SECTION <name>` start a new item, and classification happens before the previous item is flushed — otherwise one section carrying subheadings becomes several items under one key and application keeps only the last, silently discarding documented behaviour the change never touched.
37. A semantic delta declaring the same operation, target and key more than once is REFUSED. Applying it would keep the last body and discard the earlier ones with no diagnostic.
38. Semantic delta bodies are bound to the definition approval that signed them: approval records a per-module digest over each delta file's body, and materialization and acceptance refuse to rewrite a canonical spec when a body no longer matches, naming every module that drifted. The body is hashed with `\r\n` folded to `\n` and NOTHING ELSE folded, so the binding asks the question delta application already asks: `markdown_block_matches` compares ignoring line-ending style and `parse_delta` reads through `lines()`, so a CRLF and an LF delta materialize byte-identical canonical specs and a checkout that rewrote the line endings cannot invalidate an approval. The equality stays STRICTLY NARROWER than the applier's: trailing whitespace, blank lines and a lone carriage return are wording a reviewer signed and still move the digest, and Git rewrites none of them on its own. An approval recording no such digest predates the binding and reads as unknown, never as tampering, so every historical archive remains valid.
39. Markdown under `.specsync/` is pinned to `eol=lf` in `.gitattributes`, beside the JSON already pinned there and for the reason that file already states: change artifacts and semantic delta bodies are read as lifecycle evidence, so a working tree that rewrites them into CRLF makes honest, unmodified work arrive in non-canonical form. The pin governs this repository's own working trees; it is not a substitute for readers that tolerate CRLF, because an adopter's repository, a tarball, or an archive extracted without Git is never covered by it.
40. A recorded delta binding is MONOTONE within one approval ledger. Every writer of a `definition` gate records the per-module delta digest it approved — ordinary approval, the normalizing approval inside explicit acceptance, and both members of the portable SpecSync 5.0.1 pair alike — so within a ledger the binding only ever goes from absent to present. An effective definition approval that records no delta wording while another definition approval in the same ledger records it is a claim being withdrawn, not evidence predating the binding, and materialization and acceptance refuse it and name the re-approval remedy. Absence across a whole ledger still reads as unknown, because that is the only shape recorded history has: a change is either from before the binding existed or from after it, never both.
41. `canonical_applied` records that materialization RAN, never that it ran for the delta bodies on disk now, so `change check` and acceptance decide from the canonical artefacts instead of from the flag alone. Materialization produces three outputs per module — the delta applied to the canonical files, the spec's `version:` bump, and the spec's Change Log row — and the flag's short-circuit skipped ALL THREE, so a delta corrected after review and re-approved satisfied the delta binding (a new approval signs the new body) and then left changed contract text with no bump and no row while `change check`, `change audit --strict` and `specsync check --strict` all passed. A module is materialized again when its delta is not fully reflected in the canonical files or when its Change Log carries no row for the change, and is left untouched when both hold: a byte-identical re-approval still writes nothing, and one change bumps one module's version exactly once. Convergence is scoped to an already-applied change — on a first materialization every application refusal still fires, and only afterwards does an already-reflected item, such as a `## REMOVED` block that is already absent, read as done rather than as an error.

## ADDED

### REQUIREMENT REQ-change-092

Canonical materialization SHALL be decided from the canonical artefacts rather than from the
`canonical_applied` flag alone, so that every output materialization produces is present for the
delta bodies the change has currently approved.

Acceptance Criteria
- A semantic delta corrected after review and re-approved is materialized into the canonical spec
  by the next `change check`, and the superseded wording does not survive beside the correction.
- A module whose canonical spec carries the change's contract text but carries neither the
  `version:` bump nor a Change Log row naming the change receives both on the next `change check`.
  Neither is derivable from a delta digest, so re-applying the delta alone would not close this.
- A re-approval whose delta body is byte-identical writes nothing at all: the canonical spec and
  requirements stay byte for byte as they were, the version stands at one bump, and the Change Log
  carries exactly one row for the change. Re-materializing unconditionally is refused as a fix,
  because it would rewrite every canonical spec on every check.
- Re-materialization does not refuse the work its own earlier run performed: a `## REMOVED` item
  whose block is already absent reads as applied. That reading is available ONLY to a change that
  has already materialized once; on a first materialization every application refusal still fires
  unchanged, including removing a block that was never present.
- The refusal for a semantic delta that changed after its approval names `specsync change check`
  after `specsync change approve`. Approval binds the wording and only `check` puts it in the
  canonical spec, so a remedy naming approval alone walked the author into the silent skip.
