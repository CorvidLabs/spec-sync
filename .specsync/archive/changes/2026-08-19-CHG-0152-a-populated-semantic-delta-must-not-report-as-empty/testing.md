---
change: CHG-0152-a-populated-semantic-delta-must-not-report-as-empty
artifact: testing
---

# Testing

## Strategy

The lie is a wording bug with a case-asymmetry rider. Every new assertion is paired with a
control that must stay green on both the old and new parser.

## Cases

| case | before | after |
|---|---|---|
| prose, no `##` heading (`# greeter` plus a sentence) | `is empty` | names allowed operation headings; does not say empty |
| whitespace-only or truly empty file | `is empty` | still `is empty` |
| `## added` + `### REQUIREMENT REQ-…` | approves | still approves |
| `## Added` + `### requirement REQ-…` | invalid item heading | parses as REQUIREMENT |
| `## Added` + `### spec section Public API` | invalid item heading | parses as SPEC SECTION |
| `## ADDED` with no items | `is empty` | names required item forms; does not say empty |
| `## REMVOED` | invalid operation heading | still invalid; message names Added, Modified, Removed |
| `### Structs & Enums` inside an open item | content (#564) | still content |
| `### requirement REQ-…` inside an open item | content or error | opens a new REQUIREMENT item |
| `### leftover` before any item | invalid item heading naming both forms | unchanged |
| valid uppercase ADDED/REQUIREMENT | items unchanged | items unchanged |
| historical `plain garbage\n` | `historical semantic delta is empty` | historical wrapper plus no-recognized-headings; not `is empty` |

The existing test `historical_tombstone_corruption_fails_closed` writes `plain garbage\n`
and asserts the old empty wording. That assertion must flip with the product.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-075 | Unit tests in `src/change_tests.rs` covering the table above through `parse_delta` and `validate_delta_files`, including the flipped historical tombstone case |
