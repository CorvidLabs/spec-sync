---
change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
artifact: design
---

# Design

- Hash cache: every `.3md` entry in the spec's directory, listed through `read_project_dir`, joins the spec's companion inputs. That one list feeds the snapshot input digest, the changed-spec classification and the per-file hashes, so no second mechanism is added. The digest also frames each entry's kind, as it does for history fragments, so a file swapped for a directory re-validates.
- All `.3md` entries rather than one name: the cache has the spec's path, not its parsed module, and a superset only costs an extra re-validation.
- Validator: the lesson file is `<module>.3md` from the spec's frontmatter module, falling back to the spec file's stem when the frontmatter has none, which is how the lifecycle names it. The existence test no longer follows the entry: anything of that name goes to the bounded project read, whose refusal becomes the warning.
- Unchanged: the warning's words, the bounded read, coverage discovery's refusal of links, and `change check --strict`.
