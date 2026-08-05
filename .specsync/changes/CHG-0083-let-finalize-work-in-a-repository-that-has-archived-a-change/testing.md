---
change: CHG-0083-let-finalize-work-in-a-repository-that-has-archived-a-change
artifact: testing
---

# Testing

`a_directory_candidate_admits_the_files_it_expands_to` builds a repository with
an archived change directory, supplies that directory as a candidate, and
asserts evidence collection succeeds. It fails on the unfixed code with the
exact production message.

Only the index guard was reachable from that test. Fixing it surfaced the
visibility guard, which is how the remaining two were found. A single-symptom
fix would have left three.

Drill 036 covers the lifecycle end to end but starts from an empty repository,
so it cannot reach this. It needs a pre-populated archive — tracked above.
