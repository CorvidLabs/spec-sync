---
kind: evidence
id: 2026-11-19-on-base-3a02a5d5-with-only-the-new-tests-added-11-of-17
aliases: []
claim: "On base 3a02a5d5 with only the new tests added: 11 of 17 project-read integration tests fail. A FIFO at .specsync/sdd.json, the transaction journal, .specsync/config.toml, .specsyncignore, the MCP lock or CLAUDE.md made change status, change audit, change new, check, mcp diff and hooks status wait until the 20 s timeout killed them; sdd.json linked outside the project and a 4 MiB+1 sdd.json were read; a 64 MiB+1 hashes.json was read; and the source scan found 187 direct file read(s) bypassing a bounded read."
result: failed
evidence_kind: product-test
method: "cargo test --locked --test integration -- project_reads:: (6 passed, 11 failed); rustc 1.89.0, isolated git config; FIFOs under a 20 s timeout and sparse files, never /dev/zero"
revision: 3a02a5d5903b08612c0dad8c44ecfdf28fe2b0ef
content: ea398bda76bb12c41e999ca2b92fc9e65dece1725519845858be0c10dc8e14b4
environment: local-macos-arm64
criteria: [SETUP-7]
requirements: []
contracts: []
limitations:
  - "The 6 that pass on base are guards already in place: check's confined discovery refuses a FIFO companion and a linked-out source or requirements file, archive-tasks refuses a non-regular tasks.md, change 2 guards the agents manifest, and the inside-link control. Over-bound and not-a-regular-file text cannot be asserted for those modules on base; the source scan covers them."
observed: 2026-11-19T14:30:41Z
actor:
  - agent: claude-code
    terminal: t5r
---
**On base 3a02a5d5 with only the new tests added: 11 of 17 project-read integration tests fail. A FIFO at .specsync/sdd.json, the transaction journal, .specsync/config.toml, .specsyncignore, the MCP lock or CLAUDE.md made change status, change audit, change new, check, mcp diff and hooks status wait until the 20 s timeout killed them; sdd.json linked outside the project and a 4 MiB+1 sdd.json were read; a 64 MiB+1 hashes.json was read; and the source scan found 187 direct file read(s) bypassing a bounded read.**

Recorded as failed in local-macos-arm64 at revision 3a02a5d5903b, observed 2026-11-19T14:30:41Z.

## Key failing lines

- was still running after 20s: a read waited on a file instead of refusing it; the policy was read through a link to a file outside the project; 187 direct file read(s) bypass the bounded project read (REQ-util-003)
