# Plan

Replace four `git config --get <key>` invocations with one
`git config -z --get-regexp '^core[.](autocrlf|eol|symlinks|filemode)$'`, parse the records into
a snapshot, and derive all four values from it.

## Equivalence was verified against git, not assumed

Each case is one behaviour where `--get-regexp` could plausibly differ from `--get`. Checked
against git 2.50.1 before any code was written:

| case | `--get` | `--get-regexp -z` |
|---|---|---|
| multi-valued key | returns the last value | lists in order; last-wins matches |
| valueless key | rc=0, empty value | record with no `\n` → empty value |
| mixed-case `[CORE] FileMode` | key lowercased | key lowercased |
| surrounding whitespace | trimmed | trimmed |
| nothing set | rc=1, empty stdout and stderr | rc=1, empty stdout and stderr |
| malformed config | rc=128 with stderr | rc=128 with stderr |

The last row is the one that matters: "no matching key" and "config is broken" must stay
distinguishable, or a broken repository reads as a default one.

## What is deliberately excluded

`core.fsmonitor` stays on its own path. It is read through `configured_git_command`, which
scrubs system, global and injected configuration, while this query is built on
`rooted_git_command` and must be — the callers depend on that precedence. Folding fsmonitor in
would silently change how it resolves.

## Structure

`checkout_autocrlf_from_command` becomes `#[cfg(test)]`. Production now reads all four keys
together, and that helper's remaining job is to let the existing local/global/injected
precedence tests drive the same snapshot path production uses, rather than a parallel one that
could drift.
