---
change: fix-custom-cargo-library-discovery-and-scoped-npm-generation-for-specsync-6-1-1
artifact: requirements
---

# Requirements

- Custom Cargo library paths SHALL retain exact target ownership under the package name, including workspace prefixes.
- Scoped npm names SHALL preserve their identity while producing valid specs, companions and reported paths in default, JSON and named generation.
- Retained CLI write failures SHALL fail visibly instead of returning successful empty output.
- The prepared distribution SHALL be 6.1.1. Ubuntu and macOS SHALL qualify one immutable candidate SHA before Cargo or Homebrew publication.
