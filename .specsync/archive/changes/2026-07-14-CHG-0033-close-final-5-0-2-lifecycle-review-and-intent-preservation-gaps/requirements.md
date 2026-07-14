---
change: CHG-0033-close-final-5-0-2-lifecycle-review-and-intent-preservation-gaps
artifact: requirements
---

# Requirements

## Recursive verification safety

- Detect `cargo run --manifest-path <path> -- ...` and `cargo run --manifest-path=<path> -- ...` when the selected manifest identifies SpecSync through its package name, `default-run`, or selected binary.
- Reject a recursive lifecycle verifier before changing attempt history or lifecycle state.
- Preserve the existing allowance for unrelated native Cargo verification commands.

## Precise implicit module coverage

- Registry-resolved module scope covers the exact canonical spec and only these standard companions in the same directory: `requirements.md`, `tasks.md`, `context.md`, `testing.md`, and `design.md`.
- Do not treat unrelated siblings or the entire canonical-spec directory as implicitly covered.
- Preserve explicit affected-path coverage and non-conventional registry mappings.

## Intent preservation

- Preserve a prose acceptance criterion as one exact trimmed value even when it contains commas or line breaks.
- Accept multiple criteria only through an explicit JSON array of strings.
- Retain comma/newline list parsing for identifier/path questions such as affected specs and affected paths.
- Persist and render the resulting criteria without punctuation loss or fragment creation.
