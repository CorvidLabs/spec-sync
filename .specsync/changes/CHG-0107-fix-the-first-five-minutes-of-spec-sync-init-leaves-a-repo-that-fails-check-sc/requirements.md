---
change: CHG-0107-fix-the-first-five-minutes-of-spec-sync-init-leaves-a-repo-that-fails-check-sc
artifact: requirements
---

# Requirements

## REQ-cmd-init-005 — initialization leaves a checkable repository

`specsync init` MUST record the protected SDD paths it creates in
`.specsync/bootstrap.json`, so that `specsync check` immediately afterwards reports no
uncovered meaningful delivery for files initialization itself wrote.

Failure to write the record MUST be reported as a warning and MUST NOT fail
initialization.

## REQ-change-060 — bootstrap exemption is narrow and revocable

A path recorded in a bootstrap record MUST be exempt from lifecycle path coverage only
when all of the following hold:

1. the path is a protected SDD path;
2. the path is absent at the delivery comparison base;
3. the recorded base commit is an ancestor of `HEAD`; and
4. the file's current content matches the recorded digest.

Editing a bootstrapped file MUST revoke its exemption. A bootstrap record MUST NOT
exempt any path that is not a protected SDD path.

Bootstrap records written in the earlier single-path `bootstrap_policy` shape MUST
continue to be honored.

## REQ-change-061 — the bootstrap digest pins the enforcement surface

The digest recorded for `.specsync/sdd.json` MUST cover every field that determines
whether the coverage gate applies, and MUST NOT cover `verification_commands`.

A policy file that cannot be parsed MUST fall back to a digest of its bytes.

## REQ-change-062 — delivery comparison base resolves in a one-commit repository

Resolution of the delivery comparison base MUST succeed in a repository containing a
single commit. It MUST reduce both a `<ref>...HEAD` range and a bare commit to a single
commit via its merge base with `HEAD`.

## REQ-change-063 — generated sections gate on authorship, not on shape

An unfinished spec section MUST NOT produce a fatal effective-contract finding when no
active change authored that section.

An unfinished spec section that an active change authored MUST remain fatal.

Ignore configuration MUST be applied through the project's `IgnoreRules`.

## REQ-validator-010 — a directory in `files:` is an error, never silent success

A `files:` entry that resolves to a directory inside the project root MUST produce a
validation error. It MUST NOT be reported as resolving outside the project root, and it
MUST NOT pass validation.

The error MUST carry a fix naming the source files beneath the directory, expanded by
the same rule module generation applies, excluding configured exclude directories, and
truncated with a remainder count beyond five entries.

Snapshot-based validation MUST report the same error without enumerating the ambient
filesystem.

## REQ-cmd-issues-002 — snapshot validation distinguishes directories from escapes

`SourceSnapshot` MUST represent a confined directory distinctly from a rejected path,
so a directory mapping is reported as a mapping-shape error rather than a security
escape.

Symbolic links and reparse points MUST continue to be rejected.
