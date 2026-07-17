---
change: CHG-0048-add-static-x86-64-unknown-linux-musl-build-to-the-release-matrix-so-prebuilt-bin
artifact: context
---

# Context

The prebuilt Linux x86_64 binary from the releases page is built on `ubuntu-latest` against
glibc 2.39. On any distro with an older glibc (for example Debian 12, glibc 2.36), it fails at
startup with `version 'GLIBC_2.39' not found (required by specsync)`. The "single binary, no
toolchain needed" install path silently only works on bleeding-edge distros; everyone else
falls back to `cargo install specsync` (roughly an 11 minute compile with the tree-sitter
grammars). Reported in GitHub issue #392.

A static `x86_64-unknown-linux-musl` artifact runs on any Linux regardless of glibc version,
which matches the tool's "deterministic, single-binary, works everywhere" distribution story.
The release workflow's packaging, checksum-verification, upload, and release-glob steps are all
artifact-name-driven, so only the matrix entry and one musl toolchain step are required.
`musl-gcc` is needed because the tree-sitter grammar crates compile C via the `cc` crate; the
musl target otherwise links fine with Rust's bundled std. The `dtolnay/rust-toolchain` step
already installs the musl std via its `targets:` input.

aarch64-musl was considered but left out to keep this minimal: it needs a cross musl toolchain
rather than `musl-tools`.
