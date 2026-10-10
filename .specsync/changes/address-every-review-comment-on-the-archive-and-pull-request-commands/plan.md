---
change: address-every-review-comment-on-the-archive-and-pull-request-commands
artifact: plan
---

# Plan

## Approach

Fix each review finding on pull request 804 in the product, keep the Archive check and the Rust sentence on the same rule, and reply on every thread.

## Out of scope

Merging the pull request. Copying lessons into the principles file. Changing the required check away from Required CI gate.

## Steps

Fix the code and the published specs. Approve, verify, review, and archive this change. Reply on each review thread and resolve it.

## Risks

A stale review can still be named for ship when its commit matches HEAD but its digest does not. Ship refuses that review.

## Constraints consulted

The archive sentence must not say to merge while a change is open. A selected plan stays substantive. SpecSync does not copy lessons into principles.
