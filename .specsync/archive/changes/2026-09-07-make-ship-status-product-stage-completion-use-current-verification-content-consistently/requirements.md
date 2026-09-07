---
change: make-ship-status-product-stage-completion-use-current-verification-content-consistently
artifact: requirements
---

# Requirements

### REQ-cmd-change-016

`change ship-status` SHALL derive product-stage completion from recorded verification content currency, consistently with its readiness calculation, rather than treating commit ancestry as proof of freshness.

Acceptance Criteria
- Current verified content with rewritten ancestry reports the product stage as done.
- Changed verification inputs report the product stage as incomplete even if the recorded commit remains an ancestor.
- Product-stage explanatory text describes content currency truthfully; ancestry remains separately observable diagnostic data.
- Missing or malformed evidence does not become a completed product stage.
- Existing scoped-review currency and finalization gates retain their behavior; this change does not decide #694's unavailable-review policy.
