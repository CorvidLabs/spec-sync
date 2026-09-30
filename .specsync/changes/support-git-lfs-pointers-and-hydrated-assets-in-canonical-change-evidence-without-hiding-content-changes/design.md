---
change: support-git-lfs-pointers-and-hydrated-assets-in-canonical-change-evidence-without-hiding-content-changes
artifact: design
---

# Design

Issue #789: support LFS v1 canonical evidence without filters or network calls.
- Keep entire evidence inventory; do not exclude asset paths through ignored_paths.
- Recognize filter=lfs explicitly and continue refusing custom filters/encoding/ident.
- Read attributed paths directly, including paths Git calls clean, to avoid hiding hydrated changes.
- Canonicalize plain SHA-256 LFS v1 pointers and streamed hydrated bytes to the same pointer payload; preserve index identity and file mode.
- Fail closed on unsupported pointer extensions and malformed pointer-like input; support the empty-file special case.
- Preserve sparse/missing/symlink guards and double-capture race detection.
- Disable LFS process/clean/smudge during evidence Git queries; do not invoke LFS or download content.
- Tests: pointer/hydrated parity, same-length changed bytes, edited pointers, staged changes, deletion, bad/extended pointers, other attributes, custom filter non-execution, full change-check with unchanged out-of-scope LFS.
- Full build/pre-push currently blocked by unavailable cached serde-saphyr; no fetch/install authorized.

