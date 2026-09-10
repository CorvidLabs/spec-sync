---
change: pin-the-trust-gate-to-stable-1-2-0
artifact: plan
---

# Plan

Single edit, one file, one line.

1. In `.github/workflows/trust.yml`, change the `uses:` ref on the
   `CorvidLabs Trust gate` step from
   `CorvidLabs/trust@e0272543ad5c88ec75cea1e606cc2efa552af25b # v1.2.0-rc.4`
   to
   `CorvidLabs/trust@fcc889f54d8b4892a81af463c5a0250e2be66fc5 # v1.2.0`.

   The SHA is the peeled commit of the annotated `v1.2.0` tag, not the tag
   object. The existing SHA-plus-comment pin style is preserved, and the pin
   stays immutable rather than moving to the `@v1` major channel.

2. Leave every other line of the step untouched, specifically
   `specsync-version: "6.0.0"` and
   `specsync-download-base-url: file://${{ runner.temp }}/specsync-trust-mirror`.

3. Leave the mirror packaging steps and the
   `specsync change audit --strict` preflight untouched.

## Explicitly out of scope

No spec text changes, which is why this change was opened with
`--no-spec-change`. No module contract, public API, or runtime behavior of the
published `specsync` binary or the published Action is affected. This is the
configuration of the gate that runs on this repository's own pull requests.
