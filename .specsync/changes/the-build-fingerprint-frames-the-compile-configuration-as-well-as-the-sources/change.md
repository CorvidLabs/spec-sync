---
id: the-build-fingerprint-frames-the-compile-configuration-as-well-as-the-sources
state: implementing
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# The build fingerprint frames the compile configuration as well as the sources

## Intent

The build fingerprint frames the compile configuration as well as the sources

## Affected Canonical Specs

- `hash_cache`
- `cli`

## Acceptance Criteria

- build.rs frames TARGET, PROFILE, OPT_LEVEL, DEBUG and every CARGO_CFG_* and CARGO_FEATURE_* variable into SPECSYNC_BUILD_FINGERPRINT and records them as SPECSYNC_BUILD_CONFIGURATION; two configurations of one tree frame differently, and the same configuration frames the same whatever else is set

## No-spec Rationale

REQ-hash-cache-005 and REQ-change-134 say a cached result is replayed only by the build that recorded it, and CHECK-1.d says a remembered result never hides a failure the running build would report; two builds of one tree for another target, profile or feature set shared one fingerprint (#981)
