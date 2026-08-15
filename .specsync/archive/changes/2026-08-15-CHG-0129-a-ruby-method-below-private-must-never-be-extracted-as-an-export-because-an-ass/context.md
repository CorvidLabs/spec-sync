---
change: CHG-0129-a-ruby-method-below-private-must-never-be-extracted-as-an-export-because-an-ass
artifact: context
---

# Context

#479 was filed as "Ruby methods silently escape export extraction based on
class-body position". **That is not the defect**, and the difference is the
whole point.

Measured: `public_after`, a method following an `x = if … end` block, IS still
extracted. Nothing escapes. What actually happens is the inversion — methods
sitting BELOW `private` are wrongly ADDED to the export set:

    ⚠ Undocumented export 'leaked_after' from src/watch.rb

The root cause is in the Ruby extractor's block tracking. Its block-opener test
is anchored to a line's FIRST token, so an assignment-form multi-line
conditional

    coarse = if seconds < 3600
             ...
             end

never pushes a nesting entry, while its `end` still pops one. The stack
desyncs by one, the enclosing class's visibility-restore entry is popped early,
`public` flips back mid-body, and every method after that point leaks —
including the ones under `private`.

In the original report `display_stream` and `agent_mode?` genuinely are private,
so "no matching export found in source" was the CORRECT answer for them. The
real defect is that `attach_log_path`, `open_log` and `extract_log` — equally
private — are reported as undocumented exports because they follow the
desyncing method.

**Why this is dangerous rather than noisy.** The leaked method is a WARNING. The
obvious way to silence a warning is to document the symbol — and doing so makes
`check` accept it, publishing a private method as public contract. The bug
recruits the user into making it permanent.
