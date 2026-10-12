---
id: fix-custom-cargo-library-discovery-and-scoped-npm-generation-for-specsync-6-1-1
state: implementing
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

## Acceptance Criteria

- Custom Cargo library target paths bind to the package module in default, JSON and named generation; scoped npm package names generate a truthful spec and cannot silently return success with no output; regression tests preserve existing owners and confinement; prepare version 6.1.1 and qualify the same immutable candidate on Ubuntu and macOS before Cargo or Homebrew publication.

## No-spec Rationale

Not applicable
