---
change: correct-remaining-specsync-6-0-docs-after-the-stable-ship
artifact: context
---

# Context

`promote` executed for `v6.0.0` on 2026-09-09 (run 34418860417) at `b9ff3231`, qualified as `v6.0.0-rc.18`. crates.io serves 6.0.0; GitHub Latest is `v6.0.0`; Homebrew still serves 5.2.0. The immutable `@v6.0.0` Action tag still embeds default `6.0.0-rc.14`.

Operator and first-user copy still described the pre-ship state: cut `rc.17` and promote, `promote` never executed, and the final tag missing. Site examples still invented `specsync check --explain` for language detection, `[[language_overrides]]`, and Action inputs `score_threshold` / `annotations`. GitHub spec companions still required Windows for RC qualification even though `REQUIRED_PLATFORMS` is Ubuntu and macOS.

Constraints: no tags, no retag of `v6.0.0`, no floating `v6`, ASCII hyphens in site copy, dual-pin `uses: CorvidLabs/spec-sync@v6.0.0` with `version: '6.0.0'`, do not claim Homebrew is 6.0.0.
