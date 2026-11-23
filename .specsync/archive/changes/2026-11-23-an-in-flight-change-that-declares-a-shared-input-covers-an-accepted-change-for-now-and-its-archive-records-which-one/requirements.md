---
change: an-in-flight-change-that-declares-a-shared-input-covers-an-accepted-change-for-now-and-its-archive-records-which-one
artifact: requirements
---

# Requirements

The `change` delta adds REQ-change-167 (an in-flight change that declares a changed input's module and path covers it for now; the accepted change reads `provisionally-covered`, audits with a warning, archives with `coverage.json`, and its archive warns when the covering change is sent back or closed) and REQ-change-168 (each of the two archive refusals over one stale input names the other change and says it is blocked too). Both are the approved text, word for word.
