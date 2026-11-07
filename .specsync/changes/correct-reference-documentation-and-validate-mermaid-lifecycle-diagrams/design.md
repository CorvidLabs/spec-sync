# Documentation validation design

## Architecture

Add site/scripts/documentation-accuracy.test.ts to the existing Bun test discovery. It reads maintained Markdown/MDX/Astro documents from the repository and extracts all Mermaid blocks, reporting file/line on error. It uses the official Mermaid parser with pinned development dependencies; no browser rendering, external service or network request is part of normal execution. The parser API is documented at https://mermaid.js.org/config/usage.html#syntax-validation-without-rendering.

A separate assertion reads ChangeState and SEND_BACK_STATES from the Rust source and compares the HLD state diagram's nodes and send-back edges. Unknown or unreadable source structure must fail clearly rather than silently dropping expected states. Fixtures deliberately corrupt syntax/remove an edge/add a phantom state to prove rejection. Closeout sequence/staging assertions are tied to explicit code behavior and retained runtime fixture receipts; syntax success alone is not semantic correctness.

## Generated guidance

Edit only skill prose, its version metadata and focused tests in src/agents.rs. Increment the template version as the module lesson requires. Regenerate tracked integrations and the digest manifest with the current built binary. Preserve all installer safety, idempotency and custom-file protection. Modify the matching requirement so docs, template and tests agree about the automatic archive commit.

## Scope boundaries

No CLI behavior, runtime guard, supported-platform matrix, workflow selection, human gate, publication operation or timeout is changed. HLD diagrams remain explanatory views of existing behavior, not new behavior definitions. There is no product UI interaction or visual redesign to prototype.

## Failure behavior

Mermaid/parser failures, missing lifecycle nodes/edges and invalid source extracts fail the test with a location. Dependency installation failures are reported as environment failures, not parser passes. Test-only dependencies are locked; public external links are checked manually with bounded commands and recorded separately from CI.
