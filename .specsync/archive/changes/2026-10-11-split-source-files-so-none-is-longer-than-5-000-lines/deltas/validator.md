## ADDED

### REQUIREMENT REQ-validator-043
The validator SHALL keep the same behavior when its source is split across `include!` files, and none of those files SHALL exceed 5,000 lines.

Acceptance Criteria
- `src/validator.rs` includes its parts in the original order.
- The parts are the previous `src/validator.rs` bytes.
- No validator source file exceeds 5,000 lines.
