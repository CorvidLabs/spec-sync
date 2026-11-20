---
change: route-and-mark-the-listing-and-spawn-sites-the-6-4-1-scans-found-outside-the-hardening-change-s-paths
artifact: context
---

# Context

The 6.4.1 hardening change (`close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first`) widened the read, Git and spawn scans (X1) and routed SpecSync's record-folder listings through the confined listing (W1). Doing so touched 17 files outside its approved path list. SpecSync cannot add paths to an approved change without a delta for each owning module, so Leif chose a second change in the same PR that covers just those files, with no wording ('Yes, two changes (Recommended)', about 21:52Z on 2026-11-20, relayed by t4c). One review covers both changes.
