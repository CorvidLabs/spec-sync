---
change: CHG-0126-the-config-refusal-must-guard-both-loaders-because-load-config-is-a-second-door
artifact: context
---

# Context

#570 fixed "an unloadable config reports success" by installing
`refuse_unloadable_config` inside `load_and_discover`, with this comment:

    // Single choke point: every command that reads specs comes through here, so
    // none of them can report a verdict over rules that failed to load (#570).

It is not a single choke point. `config::load_config` is a second door, called
directly from roughly twenty files. Measured on a config file that exists and
cannot be read:

    rules    exit 0 — prints the full rule table with every rule `off`
    compact  exit 0 — "No changelogs need compaction (all within limit)."
    rehash   exit 0 — "Regenerated hash cache for 1 spec(s)"

`rules` is the clearest illustration: the user asks what rules are configured
and is shown the built-in defaults. That is an accurate description of a
configuration the project did not write.

**`rehash` is the one that matters.** It does not merely report — it persists.
It regenerated `.specsync/hashes.json` from specs interpreted under default
configuration, and that cache is what later `check` runs consult to decide which
specs are unchanged and can be skipped. A broken config therefore did not
produce one wrong answer; it wrote a stale-skip cache that silently shortened
every subsequent run.

Two corrections to the original report, both recorded because they cost time:

- The first fixture used a TOML **parse error**. That cannot reproduce this:
  `config.rs`'s reader is a hand-rolled line-by-line scanner that silently skips
  what it cannot parse, so `load_error` is never set on that path at all. Parse
  detection lives in a different loader reached only via `load_and_discover`.
  **Two config readers with different capabilities**, and only one can tell that
  a file is broken.
- `deps` was cited as the failing command. It was the wrong evidence — the
  valid-config control exited 0 there too, so the fixture had no discriminator.
  `deps` reaches `load_and_discover` on a tree with specs and refuses correctly.
