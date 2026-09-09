# Quickstart example

The committed reference for the [Quick start](../../README.md#quick-start) walk-through in the main README.

This is a tiny Rust library with a single public function and one
matching SpecSync spec. Use it as:

- A copy-paste starting point for your own first spec
- A reference for what a clean `.spec.md` + `src/` pair looks like
- A check that `specsync check` runs green against a known-good repo

## Layout

```
quickstart/
├── README.md                       # this file
├── Cargo.toml                      # minimal Rust crate
├── .specsync/
│   ├── config.toml                 # SpecSync project config
│   ├── hashes.json                 # hash cache (normally gitignored; committed here)
│   └── registry.toml               # spec name → file mapping
├── specs/
│   └── greeter/
│       └── greeter.spec.md         # the spec for our `greet` function
└── src/
    └── lib.rs                      # one pub fn that matches the spec
```

## Try it

From the repo root:

```bash
cd examples/quickstart
specsync check --force
```

(`--force` skips the committed hash cache; a plain `specsync check` on an untouched
clone reports the spec as unchanged and validates nothing. CI passes `--force` too.)

Expected output:

```text
specs/greeter/greeter.spec.md
  ✓ Frontmatter valid
  ✓ All source files exist
  ✓ All required sections present
  ✓ 1/1 exports documented
  ✓ All dependency specs exist

1 specs checked: 1 passed, 0 warning(s), 0 failed
File coverage: 1/1 (100%)
LOC coverage:  13/13 (100%)
```

## Make it fail

Now break the contract to see SpecSync earn its keep:

```bash
# Add a new public function NOT in the spec
echo '
pub fn farewell(name: &str) -> String {
    format!("bye, {name}!")
}
' >> src/lib.rs

specsync check
#   ⚠ Undocumented export 'farewell' from src/lib.rs

# `specsync check --strict` (the CI form) turns that warning into a
# failing exit. Add `farewell` to the spec's Public API table, run
# again, green.
```

This is the loop: code + spec stay in sync, or CI catches it.
