---
change: a-change-approved-in-two-clones-is-not-valid-until-one-fresh-approval-is-recorded-after-the-fork
artifact: requirements
---

# Requirements

The `change` delta adds REQ-change-169: definition-approval validation compares every committed version of `approvals.json` that `HEAD` reaches, both sides of every merge included; a fork counts until one fresh definition approval recorded after it is the one that counts; check, review, finalize, ship and audit refuse and status reports it; archived changes are not judged again. It is the approved text, word for word, under the next number that stays free once #1132 (REQ-change-167 and 168) merges.
