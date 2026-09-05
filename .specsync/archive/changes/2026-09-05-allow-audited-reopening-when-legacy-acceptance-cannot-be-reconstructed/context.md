---
change: allow-audited-reopening-when-legacy-acceptance-cannot-be-reconstructed
artifact: context
---

# Context

<!-- What led here: the problem, and how it was noticed. -->

<!-- What a session picking this up mid-flight needs to know: constraints,
     prior attempts, anything already ruled out. -->

Issue #751 reports legacy accepted packages whose current raw input digest matches but whose acceptance-transition trees cannot reproduce that digest. Archive refuses reconstruction while reopen reports current evidence. Work is isolated on fix/751-legacy-reopen; the existing Trust-pin change is outside this scope.
