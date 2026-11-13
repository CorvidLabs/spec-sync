---
change: the-persistent-read-cache-stays-bounded
artifact: design
---

# Design

- **No approvals bytes.** A cached introduction records `approvals: bool`, whether the
  introducing commit holds the package's `approvals.json`. In memory an `ArchiveIntroduction`
  carries `IntroducedApprovals::Read(bytes)` when this command read them, `Committed` when the
  cache answered, or `Absent`. `committed_ledger` reads a `Committed` ledger from
  `<commit>:<directory>/approvals.json` (through the command's batch reader where one is open).
  Ledgers are compared only for one change's own introductions, and only when that change
  entered history more than once, so a warm read of other archives costs nothing new.
- **At most 4096 introduction entries.** The cache keeps a `clock` that advances on each write;
  each entry has `last_used`. A write stamps every entry the scope read or stored with the new
  clock, then drops the least recently used entries beyond 4096 (ties by key). Recency is
  recorded only when a command writes the cache anyway, so a warm read still writes nothing.
- **8 MiB file bound.** A cache file over 8 MiB is not loaded; the scope starts from an empty
  cache, and its next write replaces the file.
- **Schema 2.** The entry shape changed, so a schema-1 file is not loaded and is rewritten fresh.
  The file name stays `read-cache-v1.json`.

Rejected: storing each ledger's Git object id. Getting the id costs a Git read per package on the
cold path, which the finalize bound of REQ-change-121 does not allow. Rejected: dropping
introductions of commits no longer reachable from any ref; that needs a ref walk on every write.
