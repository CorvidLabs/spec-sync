---
change: fix-custom-cargo-library-discovery-and-scoped-npm-generation-for-specsync-6-1-1
artifact: requirements
---

# Requirements

- Custom Cargo library paths SHALL retain exact target ownership under the package name, including workspace prefixes.
- Scoped npm names SHALL preserve their identity while producing valid specs, companions and reported paths in default, JSON and named generation.
- Retained CLI write failures SHALL fail visibly instead of returning successful empty output.
- The prepared distribution SHALL be 6.1.1. Ubuntu and macOS SHALL qualify one immutable candidate SHA before Cargo or Homebrew publication.

- Root Cargo targets SHALL use exact-file scan selections without recursively scanning the project root; literal resolved workspace members SHALL retain ordinary package ownership.
- Validator exact-file reads SHALL retain checked target admission, no-follow regular-file identity, UTF-8 checks, existing budgets and explicit-directory authority.
- Release/configuration docs SHALL identify the minimum 6.1.1 reader and existing literal-member limitation.
