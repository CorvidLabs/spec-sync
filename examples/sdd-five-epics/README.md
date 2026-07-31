# Five-epic SpecSync 6.0 proof

This executable example builds a disposable Rust product and evolves it through five independently reviewed epics:

1. welcome message,
2. localization,
3. personalization,
4. audit events, and
5. health reporting.

Every epic uses the real SpecSync 6.0 lifecycle: deterministic interview, complete adaptive artifacts, semantic requirement/spec delta, one human scope approval, ordered dependency, implementation, Cargo tests, scoped `change check` evidence, independent scoped review, and same-PR `change finalize` into the immutable dated archive. The project is preserved after the run so its Git history and evidence can be inspected.

```bash
SPECSYNC_BIN=/absolute/path/to/specsync ./run.sh
```

Set `DEMO_ROOT` to choose the output directory. The script refuses to reuse a non-empty directory.

The final `review-report.md` contains the strict validation, coverage, quality score, archive inventory, native agent installations, and Git timeline.

## Expected proof

A successful run ends with:

- SpecSync `6.0.0`,
- five finalized and archived epics with zero active changes,
- five definition approvals and five scoped-review records,
- five verification records,
- a six-version canonical spec with six permanent requirements,
- six passing product tests,
- 100% file and LOC coverage,
- an A/100 spec quality score, and
- a clean implementation-plus-finalize Git timeline.

The installed Claude, Cursor, Codex, and Gemini files prove the local integration layout. Invoking remote agent models to prove live skill discovery is a separate, explicitly authorized network check because it can transmit project metadata to those services.
