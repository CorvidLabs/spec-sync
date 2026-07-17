---
change: CHG-0044-harden-canonical-numeric-change-ordering-across-chg-9999-to-chg-10000-and-correc
artifact: testing
---

# Testing

Focused unit evidence SHALL prove:

- `change_sequence("CHG-9999-...")` returns 9999 and `change_sequence("CHG-10000-...")` returns 10000;
- the numeric successor helper orders 10000 after 9999;
- same-sequence canonical IDs use their full ID as the tie-breaker;
- too-short, nondigit, wider leading-zero, and overflow sequences fail closed;
- malformed predecessor and candidate IDs both make successor ordering false.

Canonical evidence updates `REQ-change-026` and its testing companion. Existing `extensionless_mjs_barrel_passes_strict_in_regex_and_ast_modes` remains the implementation proof for the changelog correction.

Documentation inspection confirms both adversarial-proof matrix headers name SpecSync 5.1, no stale SpecSync 5.0 table label remains, and the Trust pin comment does not claim a nonexistent v1.0.1 tag.

Verification runs focused filters, the full unit and integration suite, format, type-check, Clippy with warnings denied, release build, lifecycle-aware strict SpecSync checking, and Trust doctor/verify. Hosted CI and closing approval are recorded only after they actually succeed.
