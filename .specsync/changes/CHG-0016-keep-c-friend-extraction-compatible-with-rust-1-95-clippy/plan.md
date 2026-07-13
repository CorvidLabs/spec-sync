---
change: CHG-0016-keep-c-friend-extraction-compatible-with-rust-1-95-clippy
artifact: plan
---

# Plan

1. Move the existing public-visibility condition into the match arm guard.
2. Run formatting and Clippy with warnings denied.
3. Run the complete focused C++ extractor test module.
