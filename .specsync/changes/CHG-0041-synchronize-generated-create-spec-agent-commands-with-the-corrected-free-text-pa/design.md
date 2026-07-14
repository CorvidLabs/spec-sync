---
change: CHG-0041-synchronize-generated-create-spec-agent-commands-with-the-corrected-free-text-pa
artifact: design
---

# Design

## Source-of-truth flow

```text
CREATE_SPEC_STEPS_MD / CREATE_SPEC_STEPS_TOML
  -> Claude/Cursor/Gemini renderer
  -> install_agent temporary output
  -> exact checked-in asset comparison
```

The shared Markdown body continues serving Claude and Cursor, while the TOML-safe body serves
Gemini. The examples carry the same semantics with format-appropriate punctuation. Checked-in files
are generated outputs, not an independent fourth copy of the behavior.

## Prompt sequence

1. Read the complete native argument placeholder.
2. Remove standalone `--minimal` tokens in any position.
3. If the remaining input is empty, ask for it.
4. Classify the complete remainder as one identifier or free text.
5. Preserve an identifier or derive/confirm a kebab-case slug from the full description.
6. Select `new` for minimal mode or `scaffold` otherwise.

Four concrete examples immediately follow classification so models cannot infer position-dependent
behavior. They are assertions in the generator test as well as user-facing guidance.

## Drift prevention

The parity test uses the existing public installer in a temporary root, then compares bytes with
compile-time repository assets. Any future template edit must intentionally regenerate the checked-in
commands in the same change, while line endings and each tool's native placeholder remain exact.
