---
change: CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414
artifact: research
---

# Research

- JSON-RPC 2.0 requires a Request object with `jsonrpc: "2.0"` and a string method; malformed
  envelopes are Invalid Request (`-32600`), not notifications eligible for silent dispatch.
- Canonicalizing an arbitrary absolute path before containment authorization performs outside
  metadata work and creates an existence oracle. Lexical rejection must occur first.
- Git's `.git` file, `commondir`, alternates, config includes, symlinks, and Windows junctions can
  redirect reads. MCP issue verification avoids this class by requiring explicit repository identity.
- A bounded inbound line alone does not protect project parsers or stdout clients; project files,
  cumulative inputs, and serialized responses require independent limits.
- The deterministic generator's legacy API reports only generated paths/count. MCP therefore
  preflights collisions and verifies all expected files after the call to prevent false success.
- A GitHub 404/`Could not resolve` is ambiguous for private or inaccessible repositories. Repository
  access must be confirmed separately before an absent issue can be classified as not_found.
- Manifest discovery and provider fanout are input-processing work themselves; bounding only the
  later snapshot copy or response serialization leaves denial-of-service work outside the budget.
- Charging a mutable manifest path is insufficient if copy rereads different bytes; the bounded
  snapshot must publish the exact buffer that discovery charged.
- Killing only a provider's direct process does not close pipes retained by descendants. GitHub
  issue reads/listing/verification therefore execute in-process with no provider subprocess.
- Identity comparison followed by name-based unlink is a check/delete race. Rollback first
  atomically quarantines the current public transaction entry and verifies its identity; a public
  mismatch is preserved and reported. Portable filesystems cannot atomically create/open or
  remove-by-handle against a same-user writer racing private names, so failed empty parents are
  retained and private-name mutation by an independently authorized root writer is outside the
  MCP caller/path threat boundary.
- Text scanning for TOML headers is unsafe because valid comments and strings can contain
  `[workspace]`. Parse bounded Cargo bytes as TOML and reject malformed workspace shapes.
