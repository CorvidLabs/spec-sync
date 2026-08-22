---
change: the-windows-rc-sidecar-must-be-readable-by-the-tool-that-verifies-it
artifact: testing
---

# Testing

A workflow step cannot be covered by the Rust suite. It is judged by reproduction and by
evidence from released artifacts.

## Discrimination — old step fails, new step passes, on identical input

Real msys binary-mode output (`HASH *name`) fed to both implementations:

    old:  shasum: *specsync-windows-x86_64.exe.zip: No such file or directory
    new:  specsync-windows-x86_64.exe.zip: OK

The old result reproduces the CI error verbatim, which is what establishes that the reproduction
is of the actual defect and not of a lookalike.

## Control — the failure is the mixed form, not the marker

Both single-mode forms verify, so the defect is specifically the mixed spelling the awk produced:

| input | result |
|---|---|
| `HASH  name` | OK |
| `HASH *name` | OK |
| `HASH  *name` | rejected |

## Byte-form evidence, against a real release

The target form was taken from the sidecar shipped with v5.2.0, not chosen:

    shipped: <HASH>  specsync-windows-x86_64.exe.zip\n
    fixed  : <HASH>  specsync-windows-x86_64.exe.zip\n
    SAME FORM

`unzip -l` on the shipped v5.2.0 zip confirms one top-level entry named
`specsync-windows-x86_64.exe`, which is the name `action.yml:112` moves into place — so adopting
`Compress-Archive` preserves the contract the consumer depends on.

## Sibling sweep

Every checksum generation and verification site in the repo was checked, not just the one the
failure pointed at:

| site | verdict |
|---|---|
| `rc-assets.yml` Windows | the defect |
| `rc-assets.yml` Unix | fine — verified OK for five targets in the failing run itself |
| `release.yml:794` Windows | the proven implementation, now the only one |
| `trust.yml:39` | Linux-only, runner-local mirror, never shipped |
| `ci.yml:725` | constructs its own line with `printf` |
| `action.yml:101` | compares field 1 only; never parses the filename |

## Not covered

The end-to-end run. The only proof that the lane attaches assets is dispatching it against
`v6.0.0-rc.2`, which is the open task.
