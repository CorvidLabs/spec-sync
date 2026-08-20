---
id: CHG-0161-a-slug-must-be-a-legal-directory-name-on-every-platform-we-ship
state: implementing
type: bug_fix
base_commit: ac17bfbc31fbc2a78062af02f3f97dddfdc0c7b5
---

# A slug must be a legal directory name on every platform we ship

## Intent

a slug must be a legal directory name on every platform we ship

## Affected Canonical Specs

- `change`
- `commands`

## Acceptance Criteria

- slugify mints the directory name a change lives in, and three of its properties do not survive the slug becoming the whole path component. It caps at 80 INPUT characters rather than output bytes, so the cap does not actually bound the component and 43 of this repository's 159 archived slugs land at exactly 80 only because separator runs collapsed. It truncates mid-word, leaving 52 of 159 reading like preserved-audited-guara. And it can emit a Windows reserved device name: slugify of NUL is nul, which Windows cannot create or open and matches case-insensitively, while the empty-input fallback is literally change, itself reserved because it collides with the workspace layout. Done when: the cap bounds emitted bytes and is sized against Windows MAX_PATH rather than the looser 255-byte component limit; truncation stops at a word boundary; a description that slugifies to a reserved name does not become one; and an ordinary description slugifies exactly as it did before.

## No-spec Rationale

Not applicable
