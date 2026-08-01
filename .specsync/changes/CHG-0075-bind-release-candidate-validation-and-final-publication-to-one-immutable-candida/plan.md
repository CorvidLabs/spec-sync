---
change: CHG-0075-bind-release-candidate-validation-and-final-publication-to-one-immutable-candida
artifact: plan
---

# Plan

1. Add one named Fledge release-candidate lane shared by local and GitHub execution.
2. Make ordinary development/product pull requests run the authoritative Ubuntu integration lane
   without macOS or Windows jobs.
3. Define an RC branch convention and immutable `vX.Y.Z-rc.N` marker that captures the candidate
   commit.
4. Run the RC lane on Ubuntu, macOS, and Windows against the marker's exact commit and emit
   commit-bound platform results.
5. Add a fail-closed promotion validator that creates the final `vX.Y.Z` tag only after every
   required platform result is green for the same candidate commit.
6. Refuse release uploads when the final tag, RC marker, platform evidence, or checked-out commit
   disagree.
7. Land the protected workflow update through the separately pinned policy process, then dogfood one
   failing RC and one successful RC before release.
