---
change: CHG-0131-dependency-analysis-must-not-report-a-clean-graph-built-from-imports-it-could-no
artifact: design
---

# Design

**Resolution now resolves.** The package map is built from two sources in order
of authority: each declared JVM source file's own `package` statement, then
directory suffixes for files that are missing, unreadable, or in the default
package. A file's own declaration wins over the directory guess, which is what
makes a layout that does not mirror its packages resolvable at all.

**"Unresolvable" became a distinct outcome rather than an absent one.** A
three-way `PackageOwner` replaces the `Option` that `filter_map` was silently
draining:

    Module(name)   an edge
    Foreign        outside every namespace the project occupies — no edge,
                   correctly silent (java.util, kotlinx.coroutines)
    Unattributed   inside the project's own namespace but no spec owns it —
                   the analysis came up short, and says so

The namespace test is written so that **silence is never the default**: if
nothing at all is known about the project's packages, an unowned import is
Unattributed, not Foreign. An ambiguous package owned by two specs is left
unowned and therefore disclosed, rather than guessed.

**The disclosure is advisory.** It adds no error or warning and never changes
the exit code, so it cannot become a backdoor gate. Making unresolved imports
fail `--strict` is a product decision, deliberately not taken here.

**Noise removed by exhaustive match.** `language_has_import_concept` is
exhaustive on purpose: YAML and shell are the only `false` arms, and adding a
new `Language` variant forces a decision rather than defaulting to silence.
