## MODIFIED

### SPEC SECTION Invariants

Every path that can be merged can reach the required CI gate; a path the CI
workflow cannot trigger can never report the gate and blocks its pull request.
