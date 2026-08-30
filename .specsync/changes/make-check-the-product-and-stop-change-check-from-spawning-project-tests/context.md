---
change: make-check-the-product-and-stop-change-check-from-spawning-project-tests
artifact: context
---

# Context

`specsync check` is the product: look at specs, look at code, report drift. The SDD change
workflow is opt-in (`specsync change adopt`). `change check` stole the word "check" and then
spawned `sdd.json` `verification_commands` (`cargo test` on this repo), so a spec-code tool
spent 15–20 minutes running the project's tests.

This change is already implemented on the branch. The workspace exists so this repo's still-on
SDD path coverage can see the dirty files.

Constraints: quiet 6.0 candidate. Do not merge to main. Do not tag. Do not cut rc.12.
This repo's `.specsync/sdd.json` stays `enabled: true` so `change` still works here.
