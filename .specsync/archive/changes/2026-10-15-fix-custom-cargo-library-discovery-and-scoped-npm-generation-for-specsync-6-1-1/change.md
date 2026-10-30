---
id: fix-custom-cargo-library-discovery-and-scoped-npm-generation-for-specsync-6-1-1
state: archived
type: feature
base_commit: 8754ae17048742cc4150d1d98122b802ddc32e88
---

# Fix custom Cargo library discovery and scoped npm generation for SpecSync 6.1.1

## Intent

Fix custom Cargo library discovery and scoped npm generation for SpecSync 6.1.1

## Affected Canonical Specs

- `manifest`
- `generator`
- `cli`
- `github`
- `validator`

## Acceptance Criteria

- Custom Cargo library target paths bind to the package module in default, JSON and named generation; scoped npm package names generate a truthful spec and cannot silently return success with no output; regression tests preserve existing owners and confinement; prepare version 6.1.1 and qualify the same immutable candidate on Ubuntu and macOS before Cargo or Homebrew publication.
- Declared project-root Cargo lib.rs/main.rs targets are bounded exact-file scan selections, not recursive scans of the project root. Literal resolved workspace members preserve whole-member boundaries and exact cross-member target ownership. Checked validator reads preserve no-follow file identity, confinement, filters and existing input budgets. Explicit directory selections retain their stated scope; workspace glob expansion is not added. Configuration and release documentation require a 6.1.1 reader for exact-file selections because 6.1.0 skips them even when its reduced coverage reports 100%.

## No-spec Rationale

Not applicable
