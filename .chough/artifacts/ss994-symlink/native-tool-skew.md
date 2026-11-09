# Native tool comparison for #994

Archive: add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync.
Baseline full HEAD: ba934e74ef8997fea35ca6e1f3987b10e7c68379.
Red-test full HEAD: 9532a256b4b2e6c39fee88e39ebb212bb3125e18.
Both direct status probes were launched at the red-test HEAD; the installed probe had the in-progress P1 working edits. The archive was unchanged throughout. `git diff ba934e74ef8997fea35ca6e1f3987b10e7c68379 -- .specsync/archive` was empty after both probes. These are native history receipts, not human acceptance of the new implementation.

## Exact probes and complete output

Required resume: `chough next --environment local` -> chough-next.log. It reported `corrupt-history` using installed tooling and refused the accept stage.

Branch: `target/release/specsync change status add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync --json` -> native-status.json / native-status.err, exit 0. `specsync 6.2.0`, SHA256 `8da76345df285ca9b8ae58f428fa5198e116bc5def4beeb093cc1d4383d958a7`. Archived, approval_valid true, correction_valid true, terminal_evidence.validity `authenticated-history`.

Installed: `/Users/leif/.cargo/bin/specsync change status add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync --json` -> installed-status.json / installed-status.err, exit 0. `specsync 6.2.0`, SHA256 `b42217b88871a16217f0d37dcc0da5c0fb3f6d0c2b61e672a028a96d363a6398`. Archived, approval_valid true, correction_valid true, terminal_evidence.validity `corrupt-history`; reason: `scoped review does not bind a recorded definition approval`. Suggested next action: restore the archive from trusted Git history, then run change check.

PATH for both direct probes: `$HOME/.cargo/bin:/opt/homebrew/bin:$HOME/.bun/bin:$PATH`.

Orc t19f directed: branch-build authenticated-history is authoritative here; retain the installed diagnostic as tool-skew evidence. No archive restoration, rewriting, re-archival, approval, review or weakened check was performed. A newly built final-head status and strict audit remain required.
