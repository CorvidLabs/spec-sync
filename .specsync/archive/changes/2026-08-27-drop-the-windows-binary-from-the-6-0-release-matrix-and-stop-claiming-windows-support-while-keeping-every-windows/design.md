---
change: drop-the-windows-binary-from-the-6-0-release-matrix-and-stop-claiming-windows-support-while-keeping-every-windows
artifact: design
---

# Design

No UI, layout, component, or design-token surface is touched. The change is release
plumbing, documentation prose, and requirement wording.

The one design-shaped decision is the shape of the failure a Windows user now meets. There
were two options:

- Leave `action.yml`'s Windows branch in place and let it fail on an HTTP 404 from a URL
  that no longer resolves.
- Refuse at OS detection with a message naming the platform and the supported alternative.

The second is chosen. A 404 on `specsync-windows-x86_64.exe.zip` tells the reader that a
download broke, which is the wrong diagnosis and sends them looking for a network or
permissions problem. An explicit `Windows is not a supported target as of SpecSync 6.0; run
SpecSync under WSL` tells them what actually changed and what to do about it, at the first
step rather than the third.

The documentation follows the same rule: state the position plainly, in the place a reader
is already looking (the binaries table, the install note), rather than as a footnote. Nothing
in the repository promises a future Windows binary, so nothing is phrased as a deprecation
awaiting restoration.
