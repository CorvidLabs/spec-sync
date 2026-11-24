## ADDED

### REQUIREMENT REQ-change-175

The lost-requirements check of REQ-change-169 SHALL read at the reference commit only the requirements files that were canonical there: under the specs directory the reference's configuration named, or beside a spec its registry held. A requirements file the branch makes canonical, by changing `specs_dir` or registering a spec, SHALL count where a requirement is looked for in the working tree, but its contents at the reference SHALL NOT be read as canonical requirements (#1112).

Acceptance Criteria
- A branch that registers a spec beside a `requirements.md` that was already there, and rewrites that file under new ids, is not refused for the ids the file held before.
- A branch that changes `specs_dir` to a directory that already held a `requirements.md` is not refused for that file's old ids.
- A canonical requirement the branch moves into a newly canonical file is found there, and one the branch drops is still reported.

### REQUIREMENT REQ-change-176

`change audit` SHALL report the refusal of REQ-change-169 for each active change whether or not that change's own approval, delta files and approved delta bodies validate, beside any error of the change's own (#1116).

Acceptance Criteria
- An active change whose delta does not parse, on a branch that lost a requirement, gets both the delta's error and the lost requirement from `change audit`.
- The same holds for an invalid approval and for a delta body edited after approval.
- A change on a branch that lost nothing gets only its own error, as before.

### REQUIREMENT REQ-change-177

When the lost-requirements check of REQ-change-169 would report a requirement lost and a requirements directory or file it reads in the working tree could not be read, for a reason other than not existing, it SHALL instead refuse saying it cannot tell whether the branch lost canonical requirements, naming each path it could not read with the error, and that the way out is to make them readable and run `change check` again. A path that does not exist SHALL be read as empty, as before, and an unreadable path SHALL change nothing when no requirement would be reported lost (#1117).

Acceptance Criteria
- A requirement moved into a directory the check cannot list is not reported lost. `change check` and `change ship-status` say they cannot tell, naming the directory and the error.
- Once the directory is readable again, the check passes.
- A requirements file that cannot be read is named the same way.
- On a branch that lost nothing, an unreadable directory changes nothing.
