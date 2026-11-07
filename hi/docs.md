---
hi: 1
families: [DOCS]
---

# Documentation you can follow

## Intent

The first thing a new user does is copy what the README and the landing page show them. Those examples should work when followed in the order written, with the hand-done steps named rather than skipped, and they should agree with what the pipeline and Chough actually enforce. When an example stops matching the tool, the pipeline should find out before a reader does.

DOCS-1 through DOCS-3.a were drafted from audit findings D5 (#881) and D11 (#883). Leif approved them as drafted on 2026-11-05 at 20:22 MST in the orc tab ("Approve all"), relayed by orc t18.

DOCS-3.b and DOCS-3.c were drafted by the agent for issue #958. Leif approved them as drafted in the Herdr orc tab on 2026-11-06 at about 19:35 MST ("1A 2yes": option A for the single-choice answer, and one placeholder rule for status as well), relayed by orc t18. Recorded by the ss-triage Claude agent (t15w).

## Criteria

- **DOCS-1**  What the README and the landing page show me runs as written, once I do the steps they tell me to do by hand.
  - **DOCS-1.a**  CI fails when one of those examples stops working, so a new user is not the one who finds out.
- **DOCS-2**  The documented closeout has me finish it locally (verify, implementation review, archive) before I push and wait for CI, because CI refuses a pull request whose change is still open.
  - **DOCS-2.a**  It says the same thing Chough teaches.
- **DOCS-3**  The landing page shows what I must do between creating a change and approving it, so approval does not refuse me for a step it never mentioned.
  - **DOCS-3.a**  If I skip one of those steps anyway, approval's refusal names the command that finishes it.
  - **DOCS-3.b**  When the open question can be answered none, the command approval names already says none, so I can run it as printed, and the refusal still tells me I can name the lessons that apply instead.
  - **DOCS-3.c**  Status names the same command for the next question that approval's refusal does.
