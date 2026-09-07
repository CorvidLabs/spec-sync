## ADDED

### REQUIREMENT REQ-change-096

Bounded Git timeout regression coverage SHALL observe child identity independently of child startup scheduling while checking deadline failure, child termination/reaping, and completion with blocked stdin.

Acceptance Criteria
- Obtain the spawned PID in the parent through test-only instrumentation.
- Retain the short deadline and a payload larger than pipe capacity.
- A delayed-start control does not require a child-written file.
- A cleanup defect is still detected by a negative control.
- Production deadlines, public API, and lifecycle policies remain unchanged.
