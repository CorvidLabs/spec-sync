---
change: the-windows-rc-sidecar-must-be-readable-by-the-tool-that-verifies-it
artifact: context
---

# Context

`RC assets` run 32545344017 was dispatched against `v6.0.0-rc.2` to give the release candidate
installable binaries. All six targets built. The `attach` job then verified checksums, as it is
designed to, and refused:

    specsync-linux-aarch64.tar.gz:     OK
    specsync-linux-x86_64-musl.tar.gz: OK
    specsync-linux-x86_64.tar.gz:      OK
    specsync-macos-aarch64.tar.gz:     OK
    specsync-macos-x86_64.tar.gz:      OK
    shasum: *specsync-windows-x86_64.exe.zip: No such file or directory

The tag was left with zero assets for a second time.

## The mechanism

msys `sha256sum` defaults to BINARY mode and emits `HASH *name`. The Windows packaging step
normalised that with `awk '{print $1"  "$2}'`. `$2` is `*name` — the asterisk is not a separate
field — so the rewritten line was `HASH  *name`: two spaces, which spells TEXT mode, with the
binary marker still glued to the filename. That form is valid in neither mode, and `shasum -c`
read the whole thing as a filename.

| sidecar line | `shasum -a 256 -c` |
|---|---|
| `HASH  name` (text) | OK |
| `HASH *name` (binary) | OK |
| `HASH  *name` (what the step wrote) | No such file or directory |

## Why this is not an awk bug

`release.yml` has had a correct Windows packaging step since before v5.2.0: `Get-FileHash` for
the digest alone, the line assembled from parts that carry no mode marker, `WriteAllBytes` so
PowerShell cannot prepend a BOM. When `rc-assets.yml` was written it did not reuse that step —
it reimplemented the same job in bash, and the reimplementation was the broken one.

This is the pattern that has cost this release repeatedly: a fix or a feature lands beside an
existing implementation of the same thing rather than reusing it, and the two drift. So the fix
deletes the second implementation rather than patching it.

## Already ruled out

- **Patching the awk** (e.g. `sed 's/ \*/  /'`). It would work, and it would leave two
  implementations of Windows packaging that must now be kept in agreement forever.
- **Changing `7z` to `Compress-Archive` only.** The zip was never the problem; both produce a
  single top-level `specsync-windows-x86_64.exe`, which is what `action.yml:112` moves into
  place. Adopting `Compress-Archive` is a consequence of taking the proven step whole, not an
  independent fix.
- **Relaxing the attach-job verification.** It is deliberately stricter than the consumer and
  it is what caught this.

## Scope note worth keeping

This did not endanger consumers. `action.yml`'s Windows path compares `awk '{print $1}'` against
a recomputed digest and never parses the filename field, so the malformed sidecar would have
installed fine. The guard refused to publish a sidecar a human running `sha256sum -c` by hand
could not read. That is the guard working, not a near miss.
