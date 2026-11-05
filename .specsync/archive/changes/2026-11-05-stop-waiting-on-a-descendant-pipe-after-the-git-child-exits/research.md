---
change: stop-waiting-on-a-descendant-pipe-after-the-git-child-exits
artifact: research
---

# Research

#854 and #855 name the same wait. The reproduction is a `git` wrapper that backgrounds `sleep` and then execs real Git. `Command::output` joins threads blocked in `read_to_end`. The bounded runner did the same in `read_bounded_pipe`. `.status()` returns when the child exits but leaves stdout inherited, so a grandchild can hold SpecSync's own stdout after SpecSync exits.

Dash keeps `$$` as the invoking shell even inside `( )`, so the holder pid is `$!`. A fifo the test never writes is what keeps the holder blocked. That is the event the test waits on, not a clock.
