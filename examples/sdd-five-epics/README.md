# Five-epic SpecSync 5.0 proof

This executable example builds a disposable Rust product and evolves it through five independently reviewed epics:

1. welcome message,
2. localization,
3. personalization,
4. audit events, and
5. health reporting.

Every epic uses the real SpecSync 5.0 lifecycle: deterministic interview, complete adaptive artifacts, semantic requirement/spec delta, definition approval, ordered dependency, implementation, Cargo tests, verification evidence, closing approval, simulated merge, and immutable archive. The project is preserved after the run so its Git history and evidence can be inspected.

```bash
SPECSYNC_BIN=/absolute/path/to/specsync ./run.sh
```

Set `DEMO_ROOT` to choose the output directory. The script refuses to reuse a non-empty directory.

The final `review-report.md` contains the strict validation, coverage, quality score, archive inventory, native agent installations, and Git timeline.

## Expected proof

A successful run ends with:

- SpecSync `5.0.0`,
- five accepted and archived epics with zero active changes,
- ten approval gates and five verification records,
- a six-version canonical spec with six permanent requirements,
- six passing product tests,
- 100% file and LOC coverage,
- an A/100 spec quality score, and
- a clean 16-commit implementation, acceptance, and archive timeline.

The installed Claude, Cursor, Codex, and Gemini files prove the local integration layout. Invoking remote agent models to prove live skill discovery is a separate, explicitly authorized network check because it can transmit project metadata to those services.
