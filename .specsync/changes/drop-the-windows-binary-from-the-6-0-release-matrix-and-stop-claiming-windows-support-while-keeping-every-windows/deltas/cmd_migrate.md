## ADDED

### REQUIREMENT REQ-cmd-migrate-003

Migration SHALL use only filesystem operations that behave identically on every host platform a repository may be checked out on, independently of which platforms SpecSync publishes binaries for.

Acceptance Criteria
- No migration step creates a symlink. A relocated file is moved or copied, because symlinks are fragile on Windows and confuse git.
- No step depends on Unix-specific semantics such as an explicit permission mode; created files and directories take the platform default.
- The constraint continues to hold for Windows even though 6.0 publishes no Windows binary, because a migrated repository is committed once and then read, re-checked, and re-migrated in clones on other hosts.
