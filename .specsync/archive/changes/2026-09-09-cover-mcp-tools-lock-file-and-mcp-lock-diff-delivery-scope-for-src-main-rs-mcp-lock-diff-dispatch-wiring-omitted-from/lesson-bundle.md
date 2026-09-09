# Lesson bundle — cover-mcp-tools-lock-file-and-mcp-lock-diff-delivery-scope-for-src-main-rs-mcp-lock-diff-dispatch-wiring-omitted-from

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Cover mcp-tools-lock-file-and-mcp-lock-diff delivery scope for src/main.rs MCP lock/diff dispatch wiring omitted from the parent change affected_paths
- **Kind**: BugFix
- **Specs**: cli
- **Paths**: src/main.rs
- **Acceptance**: src/main.rs is covered by an active change in delivery-diff coverage; specsync change audit --strict reports no uncovered meaningful changed paths for this path; no canonical spec content changes

## Evidence

- Verification commit: `d1463f28d16bb2bc6cc4b05adfd19189e513be64`
- Base commit: `9d20c4c6f3f51534732cdf798315999bf3374126`
- Verified by: `specsync check --spec cli`

## From the change's context.md

# Context

`mcp-tools-lock-file-and-mcp-lock-diff` delivered MCP `lock`/`diff` CLI dispatch wiring in
`src/main.rs`, but its declared `affected_paths` omitted that file, so delivery-diff coverage
and `change audit --strict` report it as uncovered. Lifecycle freezes scope at definition
approval, so a small bookkeeping change declares ownership of `src/main.rs`. No code or
canonical spec content changes.

## From the change's design.md

# Design

No design change. This is bookkeeping delivery-scope coverage only: declare `src/main.rs` on
an active change so the parent feature's delivery diff is fully covered. Implementation and
specs already landed under `mcp-tools-lock-file-and-mcp-lock-diff`.

## From the change's testing.md

# Testing

- `specsync change audit --strict` passes with no `meaningful changed paths are not covered`
  error naming `src/main.rs`.
- No product code changes; existing parent-change verification and CI gates remain the
  product proof.

## Where these lessons go

- `specs/cli/context.md`
