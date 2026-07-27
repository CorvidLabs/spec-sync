## ADDED

### REQUIREMENT REQ-generator-002

The CLI generator SHALL keep every project filesystem side effect beneath one retained
project-root capability.

Acceptance Criteria

- Template reads, module-directory creation, spec publication, and companion publication are
  relative to the retained capability.
- Configured specs paths and module destinations reject absolute, rooted, prefix, and parent
  traversal components before use.
- Redirecting the caller-visible root after checked coverage cannot redirect an output write.
- Existing files remain no-overwrite destinations.

## MODIFIED

### SPEC SECTION Invariants

- CLI generation confines template reads, directory creation, and no-overwrite publication to one
  retained project-root capability so public-root replacement cannot redirect output.
