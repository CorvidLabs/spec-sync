---
id: the-windows-rc-sidecar-must-be-readable-by-the-tool-that-verifies-it
state: implementing
type: bug_fix
base_commit: 7c6100fc5e97e92ab83b40ef912c166069366c24
---

# The Windows RC sidecar must be readable by the tool that verifies it

## Intent

the Windows RC sidecar must be readable by the tool that verifies it

## Affected Canonical Specs

- None

## Acceptance Criteria

- The RC assets lane attaches all twelve files to a pre-release tag. The Windows sidecar it produces is HASH, two spaces, bare filename, LF — byte-identical in form to the sidecar shipped with v5.2.0 — and `shasum -a 256 -c` on it reports OK rather than looking for a file named `*specsync-windows-x86_64.exe.zip`. The bash reimplementation of Windows packaging is gone, leaving one implementation shared with release.yml.

## No-spec Rationale

the packaging step writes a checksum sidecar; no module spec text describes CI workflow files
