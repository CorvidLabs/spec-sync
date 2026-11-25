---
change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
artifact: context
---

# Context

- #1163 added the warning for a lesson file `check` cannot read or parse (REQ-cmd-check-023). Codex reviewed its head `24d553f6` and left four P2 findings on that code. Each was run on the CLI with the binary built from that head before anything was changed.
- Confirmed: after a clean cached `check`, breaking the lesson file is not reported by a plain `check`, because the hash cache does not know the file; `--force` and `--strict` report it.
- Confirmed: for a spec file not named for its module (`main.spec.md`, module `product`), `check` looks at `main.3md` while the lifecycle reads `product.3md`.
- Partly confirmed: a directory named `<module>.3md` is skipped silently. A FIFO or a dangling link there already fails closed in coverage discovery.
- Confirmed, and kept: `change check` passes and `change check --strict` fails on the warning. Strict means warnings fail; the requirement and the changelog said lifecycle commands never fail, which is true of reading lessons and not of a strict check.
- Leif chose to fix the three in one successor change and keep the strict behaviour (release-lead chat, 2026-11-24); he is shown the two deltas before the definition is approved.
