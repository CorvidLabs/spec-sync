## ADDED

### REQUIREMENT REQ-change-109
The change module SHALL keep the same behavior when its source is split across `include!` files, and none of those files SHALL exceed 5,000 lines.

Acceptance Criteria
- `src/change.rs` includes its parts in the original order.
- Every part is a whole-item slice of the previous file.
- No change source file exceeds 5,000 lines.
