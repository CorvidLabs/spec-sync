---
change: correct-the-6-0-init-version-stamp-quickstart-example-and-release-notes
artifact: design
---

# Design

Smallest change that makes each surface true, with nothing that alters runtime behaviour for an
existing project:

- The stamp is a one-token constant change. Existing `.specsync/version` files are never
  rewritten by 6.0, so no repository observes a difference until it runs `init` fresh.
- The quickstart spec keeps its prose and gains the canonical Public API table shape so the
  export validator can match `greet`; `status: active` is what the binary itself recommends in
  its draft notice. Paths are relative to the example root because that is where the README
  runs it.
- The example is pinned by integration tests rather than a CI step so it is exercised by every
  `cargo test`, including the release-candidate qualification lane.
- Release notes are corrected in place with a parenthetical naming what was wrong, in keeping
  with how this changelog records its own errata, rather than silently rewritten.
