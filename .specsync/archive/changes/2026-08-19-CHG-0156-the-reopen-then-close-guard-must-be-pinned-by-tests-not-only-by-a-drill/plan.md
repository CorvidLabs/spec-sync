# Plan

Three unit tests against `validate_scoped_review_history_transition`, which takes its inputs
directly and needs no repository fixture.

| test | pins |
|---|---|
| `..._may_move_between_a_change_s_two_homes_in_either_direction` | both directions, so removing archive-to-active fails |
| `..._moved_to_a_third_location_is_refused` | the refusal itself, in both directions |
| `..._may_not_be_deleted` | committed evidence may not vanish |

## The pair has to fail differently

Two removals matter and they are not the same removal:

- dropping the archive-to-active term reintroduces #540
- deleting the guard entirely passes drill 049

A single test cannot catch both — the first makes the guard stricter, the second makes it
absent. So the direction test and the third-location test have to be separate, and each must
fail on the removal the other misses. That is the property this change is really buying.
