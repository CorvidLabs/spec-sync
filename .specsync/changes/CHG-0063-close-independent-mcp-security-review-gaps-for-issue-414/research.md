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
- Preflighting an ambient config pathname and later reopening it leaves a replacement interval.
  Security-sensitive callers must parse the exact bytes read through the retained handle and
  validate the complete known shape before compatibility defaults can erase configured paths.
- The final adversarial pass demonstrated that syntax-only parsing still accepted a JSON array and
  that ambient config opens could follow links or block on a FIFO. Selected MCP config therefore
  needs the same no-follow, regular-file, identity-through-read discipline as CLI issue config,
  with non-blocking acquisition for special-file races.
- Exact retained config bytes are insufficient if omitted source fields later invoke ambient
  autodetection. Capability callers must supply discovery derived from the same retained project
  identity; a bounded sparse snapshot preserves existing manifest/extension behavior without
  reopening the root pathname.
- Inspecting a regular pathname and then opening it still leaves an identity-substitution interval.
  Compare the opened file to the pre-open identity before reading, reject special files
  non-blocking, and recheck after the read. The same rule applies to recognized manifests.
- Compatibility sentinels cannot replace checked shape validation: exact JSON must reject a
  non-object `github` value and non-string/non-null `github.repo` before compatibility loading.
- Retained source detection must reuse normal ignored-name policy before metadata inspection, but
  recognized non-regular manifests must remain visible as inconclusive configuration evidence.
- A typed issue-details endpoint is issue-only, and safe-name normalization may return an empty
  slug; both conditions must be rejected before importer output construction.
- Early text-only returns violate caller-selected structured output contracts even when the exit
  status is correct; terminal outcomes must flow through one format-aware renderer.
- Windows path metadata cannot provide a trustworthy pre-open file identity for this contract.
  Opening no-follow/non-blocking first and retaining that handle makes its native file identity the
  authority; subsequent path opens are observations, not authority.
- A passing private replay is not reproducible when its executable and untracked drill/fixtures are
  mutable. Exact commit IDs must be paired with content digests for every executed input.
- Confined MCP roots are insufficient if a shared manifest parser can emit ambient source paths
  outside the project to CLI coverage/check callers. Gradle module identities and `projectDir`
  literals must be normalized lexically and rejected on root, drive, UNC, or parent underflow
  before any source probing or partial discovery.
- Gradle colon notation is not a filesystem normalization primitive. Mapping `:` to `/` before
  checking a raw value can transform a drive-qualified identity such as `C:/outside` into a
  non-drive spelling; raw include identities and project selectors must be authorized first.
- Gradle exposes both property assignment and the official `setProjectDir(File)` method. Parsing
  only `.projectDir = ...` creates a false-green omission. The shared parser must recognize the two
  literal confined method arguments or fail closed on unsupported/dynamic mutation syntax.
- Lexical containment does not authorize filesystem traversal through an in-root symlink or
  Windows reparse point. Gradle-derived source directories require component-by-component
  no-follow inspection through a retained root capability before either CLI coverage or MCP
  snapshot traversal receives the path.
