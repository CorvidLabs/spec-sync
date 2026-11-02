---
change: acknowledge-anchored-historical-receipts-explicitly-without-weakening-normal-transition-proof
artifact: design
---

# Design

`change reconcile-history ID --actor --reason` requests a new definition-bound mode on an unaccepted v2 successor with existing exact obligations. The immutable existing base is the anchor; the command never approves or closes it. Fresh scope approval is mandatory. Historical schema-2 evidence binds sorted predecessor package and manifest digests, explicitly marks transition status unproven, and still proves each successor input at its current acceptance anchor. Original active accepted bytes must equal the committed anchored package; native archives may normalize only their authenticated accepted-state snapshot. Only regular file, missing and directory entries outside protected tool metadata are supported. Current ownership and every ordinary signature/digest/cycle/ancestry/resource check remain enforced.
