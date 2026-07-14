---
change: CHG-0027-make-the-hosted-rustsec-audit-bootstrap-reproducible-on-specsync-rustc-1-89-by-i
artifact: research
---

# Research

Hosted job 87115582783 showed `cargo-audit 0.22.2` selecting `kstring 2.0.3`, followed by `rustc 1.89.0 is not supported` and Cargo's recommendation to retry with `--locked`. The failure occurred during tool installation, not while evaluating SpecSync's `Cargo.lock`.

The existing `fledge` audit task documents the intended exception as `cargo audit --ignore RUSTSEC-2024-0384`. Reusing that command avoids silently widening policy while correcting only the bootstrap failure.
