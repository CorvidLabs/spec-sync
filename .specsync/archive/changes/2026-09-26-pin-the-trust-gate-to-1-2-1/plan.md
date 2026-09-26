---
change: pin-the-trust-gate-to-1-2-1
artifact: plan
---

# Plan

Single edit, one file, one line.

1. In `.github/workflows/trust.yml`, change the `uses:` ref on the
   `CorvidLabs Trust gate` step from
   `CorvidLabs/trust@fcc889f54d8b4892a81af463c5a0250e2be66fc5 # v1.2.0`
   to
   `CorvidLabs/trust@dd52a7a90ffbc1d2b1030e37007c18274f3d96bc # v1.2.1`.

   The SHA is the peeled commit of the annotated `v1.2.1` tag, not the tag
   object. The SHA-plus-comment pin style is kept, and the pin stays immutable
   rather than moving to the `@v1` major channel.

2. Leave every other line of the step untouched, specifically
   `specsync-version: "6.0.0"` and
   `specsync-download-base-url: file://${{ runner.temp }}/specsync-trust-mirror`.

## Explicitly out of scope

No spec text changes, which is why this change was opened with
`--no-spec-change`. No module contract, public API, or runtime behavior of the
published `specsync` binary or the published Action is affected. This is the
configuration of the gate that runs on this repository's own pull requests.
