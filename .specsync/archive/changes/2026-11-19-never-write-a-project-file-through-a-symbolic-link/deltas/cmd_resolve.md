## ADDED

### REQUIREMENT REQ-cmd-resolve-002

The remote spec cache under `.specsync-cache/remote-specs/` SHALL never be read or written through a symbolic link, whether or not the link stays inside the project: a cache never needs one. When a cache file, or a directory on the way to it, is a symbolic link or not a regular file and a directory, `resolve --remote` SHALL skip the cache for that entry, SHALL say so once on stderr naming the path, and SHALL use the fetched content as it does with no cache.

Acceptance Criteria
- A symbolic link at the cache path of a dependency, to a file outside the project: storing the fetched spec leaves that file unchanged, and reading the cache returns nothing.
- A dangling link at that path creates nothing at its target.
- `.specsync-cache` linked to a directory outside the project: nothing is created there.
- A regular cache file is stored and read back as before.
