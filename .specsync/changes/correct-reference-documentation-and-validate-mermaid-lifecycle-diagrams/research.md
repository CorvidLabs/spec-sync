# Evidence and decisions

## Audit

The original read-only findings report is /Users/leif/Development/_CorvidLabs/_worktrees/ss-docs-findings.md. It records each claim, source location and actual command/output, every Mermaid review, protected-owner routing and limits. The runtime fixtures used an independently built e6b1ce11 SpecSync 6.2.0 binary; preparation uses the newly built 7ae7245f checkout.

Confirmed behavioral examples: check --commit leaves unrelated untracked files out; review followed immediately by finalize creates the archive commit itself; send-back returns Approved to Draft; the archive gate rejects an open change; active missing source fails while a planned draft mapping is a notice; bare agents install installs all four integrations; inherited SPECSYNC_VERIFICATION_CONTEXT refuses check, change and lifecycle.

Source review refines ownership wording: approval checks existing production paths against declared modules, but justified no-spec-change records with no modules return early; finalization still enforces production ownership. The docs must not overclaim an unconditional early gate.

## Mermaid approach

Offline bunx could not find mmdc, so the audit claimed manual syntax review only. For the future CI guard, use the official syntax-validation API documented at https://mermaid.js.org/config/usage.html#syntax-validation-without-rendering. Its parse result is separate from lifecycle semantic assertions. A pinned dev dependency plus lockfile makes the check reproducible after dependency install. Add a DOM adapter only if required by the parser in Bun; no headless browser or external renderer is required by the intended test design.

## Ownership

All current open PR diffs were read on 2026-11-06; their copied receipts are in /private/tmp/ss-docs-plan/prs. #935 owns the inherited-context HLD row until merged. #936 owns the landing quick start and harness. #900 owns identity/relay prose, #918 read-timeout prose, and #903 task-gate prose. #941 owns sibling/legacy Chough session recovery. Other protected code/spec hunks remain untouched. This change introduces no competing source fixes.

## Scope choices

Preserve dated historical release claims rather than replacing every old number. Distinguish wrapper revision from downloaded binary version. Correct installation guidance to the requested macOS/Linux/WSL 2 policy without inventing native Windows qualification. Do not add flaky network-link availability checks to routine CI. Update generator prose and tracked copies together, following the agents module's version/manifest lesson. Keep all user approvals and reported evidence bounded to what actually happened.
