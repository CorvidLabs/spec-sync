---
change: split-source-files-so-none-is-longer-than-5-000-lines
artifact: context
---

# Context

The long source files were split on an older main. This branch replays that split on current main. `include!` keeps each module one module. Cuts are on whole items, so a part is not a broken token tree. `include_str` and `#[path]` gain one `../` because rustc resolves them from the included file.
