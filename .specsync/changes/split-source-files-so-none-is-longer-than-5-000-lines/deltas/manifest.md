## ADDED

### REQUIREMENT REQ-manifest-019
The manifest module SHALL keep the same behavior when its source is split across `include!` files, and none of those files SHALL exceed 5,000 lines.

Acceptance Criteria
- `src/manifest.rs` keeps its module docs and includes the rest in the original order.
- No manifest source file exceeds 5,000 lines.
