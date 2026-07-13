---
change: CHG-0015-adopt-trust-1-and-specsync-5
artifact: docs
---

# Docs

Contributor guidance and generated agent instructions describe the SpecSync 5 change lifecycle and unified Trust 1 gate.

SpecSync is the sole self-hosting exception to Trust's released-binary contract component. The immutable Trust 1 action invokes released SpecSync 5.0.1, which cannot validate pull requests that evolve SpecSync's own lifecycle record schema. This repository therefore keeps contract enforcement blocking in the lifecycle lane with `cargo run -- check --strict --require-coverage 100 --force` and disables only the duplicate released-binary component. Consumer repositories must keep Trust's contract component enabled.
