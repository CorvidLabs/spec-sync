## ADDED

### REQUIREMENT REQ-commands-change-audit-dispatch-001
The change command dispatcher SHALL route `Audit` to active-only project audit and `Check` to scoped verification without dual-wiring full archive integrity into check.

Acceptance Criteria
- Check path does not call full archive integrity.
- Audit path fails closed on active/living-spec errors only.
