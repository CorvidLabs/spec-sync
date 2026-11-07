---
change: print-a-single-choice-interview-answer-literally-in-the-approval-refusal-so-the-named-change-answer-command-runs-as
artifact: context
---

# Context

Issue #958, from the Codex review of #936: since #917 the unfinished-interview refusal shows a single choice as `<none>`, which Bash reads as an input redirection, so the printed `specsync change answer <id> constraints <none>` fails before SpecSync runs.

The issue says `none` is the only valid answer. It is not: `constraints` is free text with a `none` shortcut, asked only when lessons or principles exist (#917 answered it with a lesson citation). Leif chose option A (DOCS-3.b): print `none`, and keep telling the reader the lessons can be named instead.

While drafting, status was found to disagree with approve: JSON `next_action` printed `<answer>` for every question (even yes/no), text `change status` passed no questions to `text_mode_next_action` and so skipped `constraints` and named the artifacts step, and with both specs and paths missing approve asked for paths first while status asked for specs. Leif chose to fix status in the same change (DOCS-3.c, "2yes"), which adds the cmd_change spec.

The landing-page probe in `documented_workflows.rs` now runs the single-choice command exactly as printed and checks text and JSON status after every refusal.
