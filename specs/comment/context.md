---
spec: comment.spec.md
---

## Key Decisions

- **Single rendering function**: `render_check_comment` is the only public rendering entry point. The marketplace GitHub Action and the project's own CI workflow both call it (via `specsync comment` / `cargo run -- comment`) so PR comments are byte-identical regardless of invocation.
- **String-based suggestion classification**: `suggestion_for_error`/`suggestion_for_warning` match on message prefixes (e.g. `"Missing required section: "`, `"Export '"`). This couples the comment module to the validator's message wording — changing a validator message means updating these matchers.
- **Spec-prefixed messages**: Validator messages arrive as `"spec/path.md: message"`. `group_by_spec` clusters them per spec for the Errors/Warnings sections; `strip_spec_prefix` removes the prefix for the flat Action Items checklist.
- **Branch detection shells out to git**: `detect_branch` runs `git rev-parse --abbrev-ref HEAD`; failures (not a repo, git missing) return `None` rather than erroring.
- **Integration-safe size budget**: `render_check_comment` caps complete output at 49,152 bytes, leaving headroom beneath GitHub's 65,536-byte limit for wrappers such as the CI summary/details block. Truncation walks back to a UTF-8 boundary and appends local reproduction guidance.

## Files to Read First

- `src/comment.rs` — entire module: rendering, suggestion classification, grouping helpers, `detect_branch`.

## Current Status

Stable and complete. Public API is `render_check_comment` and `detect_branch`. Unit coverage includes oversized Unicode reports and validates the byte ceiling, UTF-8 safety, and remediation notice. The `#[cfg(test)]` module contains a private `SpecViolation` struct and `render_comment_body` helper used only to exercise the renderer.

## Notes

- Consumed by the `cli` module (and through it, `cmd_comment`).
- Depends only on `types::CoverageReport`.
- Unspecced-files list is capped at 15 with an "...and N more" suffix.
