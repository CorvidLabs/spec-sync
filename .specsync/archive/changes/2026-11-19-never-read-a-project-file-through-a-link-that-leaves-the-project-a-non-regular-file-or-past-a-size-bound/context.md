---
change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
artifact: context
---

# Context

A tester found that SpecSync 6.3.0 reads `.specsync/sdd.json` through a symbolic link with no size bound. A committed link to `/dev/zero` made `specsync change status --json` reach 227 GB on macOS and get killed by the OOM killer on Linux, and Chough's `next` and `status` call that command. `.specsync/change-transaction.json` is read the same way. A FIFO at either path makes the command wait forever. On this branch before the change, a link from `sdd.json` to a file outside the project was read, a FIFO blocked `change status --json` past 8 seconds, and a 5 MiB file was read whole.

The same shape is in about 140 reads: `std::fs::read` or `read_to_string` on a project path, which follows any link, waits on a FIFO and reads without a bound.

Human decisions, relayed by the planning session: "Yes, a 3rd change" (same advisory and release; every SpecSync read of a project file refuses a link that leaves the project and a non-regular file, with a size bound), then "Approve" on the written scope (2026-11-19, about 14:21Z).

What a session picking this up needs to know:

- It is the third change on the private advisory PR, after the hash cache and the project file writes.
- The approved scope, in short:
  - every read of a project file goes through one bounded helper on the project root, opened without blocking, type-checked on the handle, and read to at most the bound plus one byte;
  - a link that stays inside the project is followed; a link out, a dangling link, a non-regular file and a file over its bound are refused, naming the path and the reason;
  - a refused read is handled like that read's other read errors today;
  - the caches stay strict.
- The bounds:
  - 4 MiB for policy, config, state and change records;
  - 8 MiB for specs, companions, templates, hook files, source and schema files;
  - 64 MiB for the transaction journal and `hashes.json`.
- Out of scope:
  - paths named on the command line (`import --from-dir`, `change correct-owner --manifest`);
  - the repository's own `.git`, including the pre-commit hook in the Git hooks directory;
  - the files Git hands the merge driver;
  - Git object reads and MCP stdin, which are already bounded;
  - reads already inside the confined, bounded discovery.
- One read is not named in the scope's table: comparing a delivery file's digest when a change is reconciled. It gets the 64 MiB project input bound, since that is what it reads.
- Never use `/dev/zero` in a test: a run on the unfixed code would exhaust memory on a shared machine.
