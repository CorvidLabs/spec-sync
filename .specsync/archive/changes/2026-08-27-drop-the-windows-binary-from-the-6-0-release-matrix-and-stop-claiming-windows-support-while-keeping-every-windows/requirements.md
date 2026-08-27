---
change: drop-the-windows-binary-from-the-6-0-release-matrix-and-stop-claiming-windows-support-while-keeping-every-windows
artifact: requirements
---

# Requirements

`REQ-github-00N` — the published release asset set SHALL contain no Windows executable, and
the exact-set artifact gate SHALL name exactly the artifacts the build matrix produces, so
the gate and the matrix cannot disagree about what a release contains.

`REQ-github-00N` — the packaged Action SHALL refuse a Windows runner with a message naming
the unsupported platform and the supported alternative, rather than attempting a download
that cannot succeed.

`REQ-change-083` (modified) — the slug legality guarantee SHALL be scoped to the platforms a
SpecSync repository may be checked out on, not to the platforms SpecSync ships a binary for,
so narrowing the shipped set cannot narrow the guarantee.

`REQ-change-084` (modified) — likewise for the reserved-name refusal.

`REQ-commands-013` (modified) — likewise for the single shared definition of names that
cannot be a directory component.

Out of scope: the release-candidate qualification lane and its `REQUIRED_PLATFORMS` set,
which still qualify on Ubuntu, macOS and Windows; `linux-aarch64` and `linux-musl`; and every
CRLF, reserved-name, path-separator, junction, and `#[cfg(windows)]` behaviour, all of which
are unchanged by construction.
