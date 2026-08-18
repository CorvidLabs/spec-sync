# Tasks

- [x] Reproduce on 34ade838 and measure which verbs break (5 of 12 swept)
- [x] Enumerate every `create_dir_all` site to find which can leave an empty directory (1 of 9)
- [x] Enumerate every reader of `deltas/` to confirm a missing directory is tolerated (5 of 5)
- [x] Enumerate every archive-side `state.json` reader to find both hard-fail sites
- [x] Add `prune_empty_package_directories` after post-move validation
- [x] Add `is_untrackable_husk` and wire it into both archive readers
- [x] Add four unit tests, two of which must fail on unfixed source
- [x] Confirm the corruption control passes on both binaries
- [x] Gate drill 050 self-flips
- [x] Invert pin drill 007 in the sandbox (CorvidLabs/spec-sync-sandbox branch drills/invert-007-archive-husk)
