---
change: advance-the-action-default-and-current-guidance-to-published-specsync-6-3-0
artifact: design
---

# Design

This follows #867 exactly, one version later.

- **Moving to 6.3.0:**
  - the `action.yml` `version` default and its example;
  - `PUBLISHED_ACTION_DEFAULT`;
  - the docs `| version |` row;
  - current guidance in README, SECURITY.md, MIGRATION.md, ADOPTING.md, the site index and the Action docs.
- **The tag notes:** immutable `@v6.3.0` embeds default `6.2.0` (`action.yml` at `b87bbd88`), so the docs say to pass `version: '6.3.0'` with it.
- **The #886 comment fallback** is in `@v6.3.0`, and not in `@v6.2.0` or earlier. The Action docs now say so.
- **Unchanged:** historical lines (the glibc floor up to 6.2.0, `reconcile-history` available from 6.2.0, archive-gate and `@v6.1.2` notes), the Action shell scripts, and the CI and Trust candidate mirrors.
- **The github spec Purpose** changes through one SpecSection delta.
