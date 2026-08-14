---
change: CHG-0116-asking-to-view-a-module-that-does-not-exist-must-fail-not-print-nothing-and-suc
artifact: context
---

# Context

## What led here

CorvidLabs/spec-sync#551.

```
$ specsync view --role dev --spec no_such_module
(no output)
exit 0
```

Zero bytes, in text and in JSON alike. Indistinguishable from a module that exists and
renders empty.

## Root cause

The filter loop skipped every non-matching spec with `continue`. A filter matching nothing
therefore left the loop body unexecuted, and the function returned normally — success by
omission.

## The second defect, same shape

Found while fixing the first: a spec that failed to render printed its error to stderr and
was then **ignored by the exit code**, so a caller could not distinguish a rendered spec
from an unrenderable one. Both directions now count and both gate.

## Why it matters

`view` exists to feed spec context to a person or an agent. Returning success with an empty
payload for a typo is the failure mode most likely to be acted on silently: a caller has
nothing to retry against and no reason to think anything went wrong.

## Ninth instance of one class

Same as #546, #547, #548, #549, #550, #553, #558, #560: something that would have
contradicted the success was discarded rather than examined — here, the fact that the filter
matched nothing.
