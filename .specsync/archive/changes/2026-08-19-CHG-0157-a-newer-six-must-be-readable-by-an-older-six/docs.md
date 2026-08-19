# Documentation

No user-facing documentation changes. One user-visible string changes: an unrecognised workflow
version now names a newer writer and an upgrade instead of reporting an invalid change state.

The reasoning is recorded above the first persisted-evidence struct in `src/change.rs` — why
evidence is tolerant, why caches are not, and precisely what tolerance does and does not buy for
the three structs that are digest preimages.
