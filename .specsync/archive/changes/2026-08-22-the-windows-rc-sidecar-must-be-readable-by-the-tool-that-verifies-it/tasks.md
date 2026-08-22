---
change: the-windows-rc-sidecar-must-be-readable-by-the-tool-that-verifies-it
artifact: tasks
---

# Tasks

- [x] Reproduce the failure locally from real msys binary-mode input, rather than inferring it
      from the error text
- [x] Confirm all three sidecar forms against `shasum -a 256 -c` (text OK, binary OK,
      mixed rejected)
- [x] Establish the target byte form from evidence: download the sidecar actually shipped with
      v5.2.0 and read its bytes
- [x] Confirm the zip entry name `action.yml:112` depends on, from the shipped v5.2.0 zip
- [x] Delete the bash Windows packaging step; copy `release.yml`'s pwsh step verbatim
- [x] Record in a comment why it is a verbatim copy, so a future reader does not "improve" it
      back into a second implementation
- [x] Sweep every other checksum site for the same defect
- [x] Re-run `RC assets` against `v6.0.0-rc.2` and confirm twelve assets attach — run
      32546250699, dispatched from this branch so the fixed workflow ran against the real tag.
      All six targets built, attach succeeded, 12 assets on the release
- [x] Prove an adopter can install it, rather than inferring that from a green build: all 12
      assets reachable, the Windows sidecar well formed and verifying its own zip, and the
      platform archive installed exactly as `action.yml` does, reporting `specsync 6.0.0`
