---
spec: cmd_comment.spec.md
---

## Tasks

- [ ] Add an end-to-end CLI test for stdout mode (`comment` without `--pr`) asserting the rendered markdown body on a fixture project
- [ ] Wire up the currently-unused `_base` parameter (diff base), or remove it if there is no planned use

## Done

- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented
- [x] Documented the unified pipeline: marketplace action + CI both use `specsync comment` for identical output
- [x] Verified the wrapper reuses `check`'s validation + `compute_exit_code` and renders via `comment::render_check_comment`
- [x] Confirmed the renderer is covered by `comment` inline tests (`test_render_check_comment_*`, `test_suggestion_for_*`)

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
