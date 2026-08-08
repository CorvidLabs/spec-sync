---
change: CHG-0099-keep-successful-legacy-reconstruction-when-scratch-worktree-cleanup-fails-511
artifact: testing
---

# Testing

```bash
cargo test legacy_reconstruction_deduplicates -- --nocapture
```

Asserts identical legacy reconstruction succeeds, and succeeds again with forced
worktree-remove failure (product #511).
