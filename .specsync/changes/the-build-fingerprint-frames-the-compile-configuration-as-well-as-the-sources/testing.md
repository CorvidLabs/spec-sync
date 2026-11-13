---
change: the-build-fingerprint-frames-the-compile-configuration-as-well-as-the-sources
artifact: testing
---

# Testing

| Requirement | Evidence |
|---|---|
| REQ-hash-cache-005, REQ-change-134 | `hash_cache::tests::the_build_fingerprint_frames_the_compile_configuration` (red: nothing recorded; green after), `build_fingerprint::tests::two_configurations_of_one_tree_frame_differently`, `build_fingerprint::tests::the_same_configuration_frames_the_same_whatever_else_is_set`, and the existing `only_the_build_that_recorded_a_snapshot_replays_it` |

Locally: `cargo test --bin specsync hash_cache` and `build_fingerprint`; hosted CI runs the full suite at the archived head.
