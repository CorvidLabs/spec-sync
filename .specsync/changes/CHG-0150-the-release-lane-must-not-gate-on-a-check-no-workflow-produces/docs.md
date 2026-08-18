# Documentation

No user-facing documentation changes: `release.yml` is internal release infrastructure and is
not part of any published surface.

The reasoning is recorded where it is enforced. A comment above the surviving step names the
deleted check, the commit that removed its producer, why restoring it is the wrong direction,
and where the binding guarantee now lives. A second comment explains why `dry_run` rejects
unrecognized values rather than defaulting.
