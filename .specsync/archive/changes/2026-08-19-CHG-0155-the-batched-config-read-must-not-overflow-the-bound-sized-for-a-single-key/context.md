# Context

CHG-0154 replaced four `git config --get` calls with one `git config -z --get-regexp`. It kept
the `128` byte stdout bound the four calls had shared.

That bound was sized for one key's answer — about six bytes. The batched query returns **every
occurrence of all four keys across every configuration scope**. Four keys in two scopes is
**144 bytes**, and `run_git_command_bounded` treats overflow as a hard error:

```
error: `git config -z --get-regexp ^core[.](autocrlf|eol|symlinks|filemode)$`
       output exceeds deterministic bounds
```

## Blast radius

`effective_checkout_overrides` feeds `inspect_git_candidates`, which feeds the git-evidence and
workspace-digest capture. On an affected machine every lifecycle command that captures evidence
fails outright.

Two scopes is not exotic — it is `~/.gitconfig` plus a repository-local override, or an
`include.path` team file plus local settings. This was shipped to `main` in `76ef32b1`.

## Why the tests did not catch it

The equivalence test written alongside CHG-0154 checks six git behaviours: multi-valued keys,
valueless keys, mixed-case sections, whitespace, unset, malformed config. **Total output volume
is not one of them**, and that test uses a single scope, so it could not see this.

The development machine has only `core.filemode` set — 20 bytes — so the full suite passed
there too. The defect is invisible on any machine whose git config is sparse, and immediate on
one whose config is normal.

Found by an external adversarial review of the merged diff, not by the suite.
