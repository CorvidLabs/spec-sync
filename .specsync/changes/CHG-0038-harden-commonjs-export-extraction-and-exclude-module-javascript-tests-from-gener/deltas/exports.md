## ADDED

### REQUIREMENT REQ-exports-004

The static CommonJS scanner SHALL preserve every statically named module export
without reporting assignment-like text from non-module scopes or literals.

Acceptance Criteria

- Chained property assignments report each exported name in source order.
- Function-like local `exports` or `module` aliases do not create module exports.
- Regular-expression literals cannot create property exports or corrupt object scanning.
- Type-level scans preserve local and inline classes exported through CommonJS.
- Regex and AST modes remain ordered, deduplicated, and compatible with ESM.
