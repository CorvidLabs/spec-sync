---
change: release-specsync-6-0-1-with-the-git-lfs-object-id-evidence-fix
artifact: requirements
---

# Requirements

REQ-github-002 SHALL keep the Action omitted-input default on `main` equal to the promoted stable package version. For this release that version is `6.0.1`.

Acceptance Criteria

- `action.yml` default is `6.0.1`.
- README and site Action examples pin `@v6.0.1` and `version: '6.0.1'`.
- The immutable `@v6.0.0` tag is not rewritten.
