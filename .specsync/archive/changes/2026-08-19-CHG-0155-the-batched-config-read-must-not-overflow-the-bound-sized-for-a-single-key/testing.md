---
change: CHG-0155-the-batched-config-read-must-not-overflow-the-bound-sized-for-a-single-key
artifact: testing
---

# Testing

## Discrimination

`the_batched_config_read_survives_two_config_scopes`, run against a binary built from a separate
checkout at `76ef32b1` — the commit carrying the regression:

```
76ef32b1   FAILED
           "`git config -z --get-regexp ^core[.](autocrlf|eol|symlinks|filemode)$`
            output exceeds deterministic bounds"
this change   ok
```

Measured in the fixture: `--get-regexp` returns **144 bytes** against the **128** bound.

## The assertion is the property, not a guess

The first version of this test asserted that the repository-local value overrides the included
one. Git resolved the other way here:

```
--get returns:  autocrlf=input  eol=lf  symlinks=true  filemode=false
regexp order:   filemode=true autocrlf=false eol=crlf symlinks=false
                autocrlf=input eol=lf symlinks=true filemode=false
last-of-each:   filemode=false autocrlf=input eol=lf symlinks=true    ← identical to --get
```

So last-wins matches `--get` exactly and CHG-0154's derivation was right; only my expectation
was wrong. The test now runs `git config --get <key>` in the same fixture and compares, which
holds whichever way git resolves.

## Unchanged behaviour

| test | result |
|---|---|
| `one_config_read_matches_four_for_every_case_git_distinguishes` | ok |
| `a_malformed_config_fails_loudly_rather_than_reading_as_unset` | ok |
| `checkout_autocrlf_resolution_honors_local_global_and_injected_values` | ok |

The malformed-config control is what shows the guard was not removed: a broken config still
fails loudly rather than reading as unset.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-077 | The two-scope fixture measures 144 bytes against the previous 128-byte bound and fails on `76ef32b1` with the exact deterministic-bounds error, passing after the bound is sized like the sibling `core.fsmonitor` read. Equivalence is asserted against `git config --get` run in the same fixture rather than against an assumption about scope precedence, and the malformed-config control confirms the guard itself is retained |
