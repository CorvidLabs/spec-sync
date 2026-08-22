---
change: name-the-lifecycle-you-are-on-and-record-the-archive-that-proves-it
artifact: context
---

# Context

Two findings from a real 6.0.0-rc.2 adoption. Same shape both times: the tool knew something the
user needed and did not say it.

## The anchor predicate asked a question a squash-merge makes unanswerable (#677, partial)

`accepted_change_is_recorded_in_ref` looked only at the ACTIVE workspace path:

    let state = format!("{CHANGES_PATH}/{}/state.json", record.id);   // .specsync/changes/<id>/

A workflow-v2 change is created, verified, and archived inside ONE pull request — `finalize`
accepts and archives atomically. Squash-merge that and the default branch receives a single commit
in which the workspace is ALREADY under `.specsync/archive/changes/`. The active path never appears
on the default branch at all.

Measured across this repository's own archives:

    ACTIVE path on origin/main:    83 / 172
    ARCHIVE path on origin/main:  172 / 172

### This nearly became a much larger change

The first reading was: archives are unanchored, so any anchor check on the archived path would turn
58% of history red, therefore the fix must re-derive content digests from the remote default. That
is a large change built on a false premise. Asking what the predicate was actually looking for
dissolved it — the 100 reds were a property of the QUESTION, not of the archives.

## A repository upgraded from 5.x stays on the legacy lifecycle silently (#678)

A fresh `init` writes policy `version: 2`; an upgraded repository carries `version: 1`, and
re-running `init` short-circuits without raising it. Every change created there is workflow-v1 and
nothing said so until `ship` refused several verbs later. It also loops: creating a change before
adopting makes `change adopt` refuse because of that change.

The reporter first said no verb existed and that v2 required hand-authoring the baseline. They
retracted both — `change adopt` is in `specsync change --help`, and their repository had run it
months earlier. That retraction moved the fix: discoverability was never the failure mode, so this
announces STATE, not a verb.

## Recorded because it cost a full lifecycle run

The first attempt at this change declared `--no-spec-change` with production source paths and NO
owning specs. It passed creation, approval, verification, and review, then failed at `ship`:

    error: acceptance input `src/change.rs` is production source without deterministic canonical ownership

That is the exact trap `docs/ADOPTING.md` warns about, written earlier in this same session. The
doc names the trap without naming the remedy: production source needs its owning specs declared
AND `--no-spec-change` when no spec text changes — the two flags coexist. Worth adding to the doc.
