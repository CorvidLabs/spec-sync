---
change: CHG-0062-harden-mcp-root-confinement-write-authorization-argument-validation-and-notif
artifact: research
---

# Research

- GitHub issue #414 reproduces arbitrary file creation by passing an outside `root` to
  `specsync_generate` while the server is configured for a different root.
- `src/mcp.rs` currently selects `args.root` before dispatch and treats canonicalization failure as
  usable input rather than failed authorization.
- `specsync_generate` and `specsync_init` are mutation capabilities and are currently always
  advertised.
- JSON-RPC notifications are requests without `id`; recognized notifications must not receive a
  response. Parse errors are the exception and use `id: null`.
- Canonicalization closes ordinary traversal and symlink escapes. Concurrent hostile filesystem
  replacement is outside this change and would require capability/dirfd-based IO.
- Independent review showed that confinement must run before config-driven manifest autodetection:
  Cargo members, package workspace bases/entries, Gradle modules, Python package paths, cache files,
  dependency references, and default schema discovery can otherwise touch outside paths before a
  later validation rejects the result.
- Recursive full-tree canonicalization is an avoidable compatibility and performance regression.
  The hardened design scans only symlinks, honors exclusions, bounds work, and mirrors the existing
  four-level no-config source scan.
- Coverage performs manifest module discovery even when an explicit SpecSync config supplies source
  directories. Manifest path preflight must therefore run unconditionally; only the fallback
  four-level source-tree scan is conditional on config source-directory autodetection.
