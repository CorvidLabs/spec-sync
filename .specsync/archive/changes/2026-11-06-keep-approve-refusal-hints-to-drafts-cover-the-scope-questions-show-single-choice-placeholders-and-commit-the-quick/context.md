---
change: keep-approve-refusal-hints-to-drafts-cover-the-scope-questions-show-single-choice-placeholders-and-commit-the-quick
artifact: context
---

# Context

Issue #917 collects the four P2 findings from the Codex review of #912, the #883 change that added REQ-change-133. All four are real. #883's change is archived, and #912 is held for Leif's morning, so they are fixed here as one follow-up instead of on #912.

1. r4489583812: `interview_incomplete_refusal` gives `"<answer>"` for a single-choice question (`constraints`, choices `["none"]`). REQ-change-133 says the choices are the placeholder.
2. r4489583819: `validate_definition` also runs for approved, implementing and verifying records (check, verification, audit). A corrupted or hand-edited non-draft record is told to run `change answer`, which refuses outside draft. A refusal must name a command that can recover (the orc's standing rule for this work), and REQ-change-133 speaks of drafts.
3. r4489810025: a draft from `change new "..."` without `--path`/`--spec` gets the bare `at least one affected path is required` / `affected specs are required unless no_spec_change is justified`. Both are interview questions (`next_questions` asks them).
4. r4489810033: the landing page committed only `.specsync` after `change adopt`. Run from exactly the documented state, the reader pushes with `src/auth.ts`, `.gitignore` and `.gitattributes` (the last two from `specsync init`) uncommitted. #883's test hid this behind its own scaffold commit.

The branch leif/917-approve-hints is stacked on #912's archive tip 93750834. origin/main is merged in once #912 lands. The orc held #912 overnight (2026-11-06) and asked for #917 to be built to its approval gate, which queues for Leif.
