# Quickstart example

The committed reference for the [5-Minute Start](../../README.md#get-started-in-5-minutes) walk-through in the main README.

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
specsync check
```

Expected output:

```text
✓ greeter (v1, draft) — 1 source file, 7/7 sections
1 spec checked, 0 errors, 0 warnings
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
# Warning: greeter — undocumented export `farewell` in src/lib.rs

# Add it to the spec's Public API section, run again, green.
```

This is the loop: code + spec stay in sync, or CI catches it.
