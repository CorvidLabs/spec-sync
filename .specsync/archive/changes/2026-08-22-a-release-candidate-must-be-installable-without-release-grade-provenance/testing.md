---
change: a-release-candidate-must-be-installable-without-release-grade-provenance
artifact: testing
---

# Testing

Both workflow files parse as YAML, and each repair is checked by inspection against the evidence
that identified it:

| Repair | Evidence it was needed |
|---|---|
| explicit tag fetch | `actions/checkout` assigns `fetchTags` only when `fetchDepth > 0`, then force-fetches `+<commit>:refs/tags/<tag>` for a tag ref whose resolved commit differs from `git rev-parse refs/tags/<tag>` — always true for an annotated tag. Measured on `v6.0.0-rc.1`: API `type=tag`, run 8s later refused it |
| `shell: python3 {0}` | macOS runner images install `python3` only; `shell: python` is a literal PATH lookup with no fallback |
| run bound by identity | GitHub discards a posted `details_url` and persists its own `/runs/<check-run-id>`; the old code both failed the prefix test and would have used a check-run id as a run id |
| pinned toolchain | the only bare `cargo` in any workflow; `rust-toolchain.toml` is candidate-controlled |

`rc-assets.yml` is verified by dispatch after merge, which is also the only way to exercise it.
Its guard runs before the six builds so a wrong target costs seconds rather than 45 minutes, and
every checksum is verified against its own sidecar before anything is attached — a mismatch must
fail here rather than at every consumer.

The lane repairs cannot be verified without a tag containing them: push events run the workflow
file as it exists at the pushed ref, which is also why `v6.0.0-rc.1` can never be qualified and
`rc.2` is required.
