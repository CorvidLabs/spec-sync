---
change: CHG-0033-close-final-5-0-2-lifecycle-review-and-intent-preservation-gaps
artifact: design
---

# Design

## Cargo command classification

Extend the existing structured Cargo argument classifier rather than invoking a shell. Capture either supported `--manifest-path` form while scanning `cargo run` options, resolve relative paths from the project root, and inspect that manifest instead of always inspecting the root manifest. Unsafe path traversal and malformed explicit manifest selections fail closed. The classifier continues to distinguish the SpecSync package, `default-run`, `--package`, and `--bin` from unrelated Cargo targets.

## Canonical companion scope

Keep implicit registry coverage exact. Starting from the registry-resolved canonical spec path, compare a changed path with the spec itself or with an allowlist of standard companion basenames under that spec's parent directory. Do not use a directory-prefix fallback. Explicit affected paths remain authoritative for additional files.

## Question-aware answer parsing

Replace the unconditional comma/newline splitter with parsing selected by question semantics:

- prose acceptance criteria: one trimmed scalar, or an explicit JSON string array;
- affected specs and paths: the existing comma/newline list syntax;
- booleans and scalar answers: unchanged scalar parsing.

This preserves backward-compatible list entry where lists are expected while preventing ordinary English punctuation from silently changing the contract.
