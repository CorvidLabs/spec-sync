---
change: the-release-lane-must-be-able-to-read-the-tag-that-triggered-it
artifact: context
---

# Context

The release lane has never completed. #635 was one reason — it waited on a check run whose
producer had been deleted. This is the next reason underneath it, and it was only reachable
once #635 was fixed and a tag was actually pushed.

`v6.0.0-rc.1` was tagged twice. The first attempt used the GitHub releases UI, which creates a
lightweight tag, and the lane correctly refused it (#667). The second was created with
`git tag -a` and is annotated — confirmed against the API: `type=tag`, tagger `0xLeif`, target
`89886855`, message `spec-sync 6.0.0-rc.1`. The lane refused that one too, with the same message.

The tag was not the problem the second time. `resolve` could not see the annotation, because its
checkout never fetched the tag object.

That means the lane would have rejected any annotated tag anyone ever pushed. It is unexercised
code, and this is the second failure found in it in ten minutes; there are six jobs past
`resolve` that have still never run. The `dry_run` dispatch input is the right way to shake
those out, rather than one wrecked release per attempt.
