---
change: CHG-0142-watch-must-disclose-a-dropped-directory-and-not-claim-a-pass-over-an-empty-check
artifact: docs
---

# Docs

## User-visible change

`specsync watch` gains two disclosures. Neither adds a flag or changes an exit code.

**A configured directory that does not exist is now named.** Previously it was dropped from
the watch set silently and the banner listed only the survivors:

```
>>> Watching for changes in: src
```

Now, on stderr, before watching starts:

```
⊘ Warning: configured specs_dir does not exist and will not be watched: missing-specs
```

and under `--format json`:

```json
{"warning":"nonexistent_watch_directory","path":"missing-specs","role":"specs_dir",
 "message":"configured specs_dir does not exist and will not be watched: missing-specs"}
```

**A run that examined no specs no longer reports a pass.** Previously `All checks passed!`
appeared over a check that found nothing. Now:

```
No specs were examined — nothing was checked. (12ms)
```

`All checks passed!` still appears for a spec set that exists and passes.

## Unchanged

- A missing directory is not fatal while at least one configured directory exists.
- An empty watch set still exits 1 with `No directories to watch`.
- `check`, `check --strict`, and their exit codes are untouched.

## No migration

No configuration, schema, or on-disk format changes. A project whose configured directories
all exist sees identical output to before.
