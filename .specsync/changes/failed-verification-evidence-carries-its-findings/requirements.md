---
change: failed-verification-evidence-carries-its-findings
artifact: requirements
---

# Requirements


- A failed `CommandEvidence` carries `output` with the findings (bounded to the last ~4 KiB), so a failure is classifiable from verification.json without re-running.
- Passing records serialize exactly as before (field omitted), and old binaries parse new records (unknown field ignored).
