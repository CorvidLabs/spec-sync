---
change: a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement
artifact: testing
---

# Testing

| Behaviour | Evidence |
|---|---|
| SHALL at a line edge is normative | `change::tests::a_shall_at_a_line_edge_is_a_normative_statement` (red, then green) |
| No SHALL is still refused | the same test |

Locally: `cargo test --bin specsync approved_delta`; hosted CI runs the full suite at the archived head.
