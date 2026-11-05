---
change: stop-waiting-on-a-descendant-pipe-after-the-git-child-exits
artifact: context
---

# Context

`std::process::Command::output` and the bounded Git runner both read stdout until end-of-file. End-of-file arrives only when every process that inherited the pipe has closed it. A grandchild can keep that pipe after Git itself has exited: a hook, a credential helper, `git gc --auto`, or a wrapper that backgrounds a process and then execs Git.

That is #855: `specsync change status` does not return. Chough bounds its read of SpecSync at 120 seconds, so `chough status` reports timed-out (#854). Both issues are this one wait.

The fix does not add a timeout and does not kill the grandchild. It stops reading once the direct child has been reaped, and it does not hand the caller's stdout or stderr to a status child. The existing deadline, cap, and stdin-writer join stay in place.

Leif's bar, captured as SPEED-1.e: SpecSync must never wait on a descendant's open pipe after its child exits.
