---
change: keep-3md-files-from-being-classified-as-roff
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-cmd-init-007 | `adds_3md_linguist_rule_to_missing_gitattributes` and `keeps_an_existing_3md_linguist_rule` pass. |

- `cargo test --bin specsync -- adds_3md_linguist_rule_to_missing_gitattributes` passed.
- `cargo test --bin specsync -- keeps_an_existing_3md_linguist_rule` passed.
