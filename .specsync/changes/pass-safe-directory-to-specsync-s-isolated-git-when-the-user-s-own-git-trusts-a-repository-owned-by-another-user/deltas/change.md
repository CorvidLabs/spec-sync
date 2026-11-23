## ADDED

### REQUIREMENT REQ-change-171

When SpecSync's isolated Git refuses the repository for dubious ownership and the user's own Git, run once with the user's environment and configuration, accepts it, SpecSync SHALL pass `safe.directory` to its isolated Git calls naming exactly the work tree top level that probe reports, through a global configuration file that holds nothing else, and SHALL never pass `*` or a value read from the repository's configuration. That file SHALL be created exclusively, with owner-only permissions, in a new private directory under the system temporary directory, never inside the project or its Git directory and never through a symlink, and SHALL be removed when the process ends; when it cannot be, SpecSync SHALL refuse as for an untrusted repository. When the user's Git refuses the repository too, SpecSync SHALL refuse, naming dubious ownership, the path and `git config --global --add safe.directory <path>`.

Acceptance Criteria
- A repository owned by another user, which the user's global configuration lists in `safe.directory`, passes `change audit`, `change status` and `change check` as it would for its owner, including from a linked worktree.
- The same repository without that entry is refused, naming dubious ownership, the path and the `git config --global --add safe.directory` command, not "unable to inspect changed paths" or "Git repository detection failed".
- A repository owned by the user runs every isolated Git call with the global configuration switched off, as before.
- A user configuration of `safe.directory = *` passes the repository's own top level, never `*`.
- It holds on Git 2.36 and on 2.39.
- The file is owner-only, outside the project and its Git directory, and gone when the command ends. A temporary directory that cannot take it, or that lies inside the project, makes the command refuse with the dubious-ownership message and the reason, writing nothing in the project.
