---
change: CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r
artifact: research
---

# Research

Issue #334 reproduces a 4.8.0 regression introduced when both Rust scanners changed from treating `pub(crate)` as contract-visible to accepting only bare `pub`. Fledge multi-file specs intentionally document crate-visible module APIs, so the first/root file continues to match while documented symbols in sibling files become phantom exports. The 5.0 branch retained the 4.8.0 scanner unchanged and had no multi-file parity fixture.

The repository currently has two high CodeQL cleartext-logging alerts that trace provider API-key metadata into error output. The observed string is the environment-variable name rather than its value, but the embedded provider system also permits plaintext key configuration and source transmission. Removing the whole embedded inference boundary resolves the alert class and reduces core supply-chain and privacy risk.

Dependabot reports five Astro advisories against locked Astro 5.18.1; the highest patched floor is 6.4.6. The failing `corvid-pet` job receives the full validation subprocess output through `specsync comment`, growing the action input beyond the Linux argument limit even though every release gate passed.
