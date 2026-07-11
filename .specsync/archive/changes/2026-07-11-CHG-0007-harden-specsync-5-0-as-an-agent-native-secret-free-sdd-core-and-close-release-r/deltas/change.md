## ADDED

### REQUIREMENT REQ-change-015
Unified lifecycle checking SHALL support a protocol-clean reporting mode without weakening verification.

Acceptance Criteria
- Reporting mode still executes every configured verification command and records failures.
- Reporting mode suppresses child command stdout and stderr so the caller can emit one machine-consumable document.
- Normal check and explicit change verification retain their diagnostic output.
