---
change: pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user
artifact: design
---

# Design

- **Trigger:** only when an isolated call refuses with `detected dubious ownership`. Repository detection (`git_repository_present`) is the first isolated call a command makes, so it is the place the trust is established; the changed-path reads name the refusal too.
- **Probe:** one `git -C <root> rev-parse --show-toplevel` with the user's own environment and configuration (prompts off). Success means the user's Git trusts the repository; its output is the one path the setting names.
- **File:** a new private directory under the system temporary directory (`tempfile`, 0700) holding one file (0600, created new) with `[safe]` and `directory = <top level>` in Git's config quoting. Refused when the temporary directory lies inside the project or its Git directory. Held for the process and removed at its end.
- **Use:** `configured_git_command` passes that file as `GIT_CONFIG_GLOBAL` in place of `/dev/null` for this project; everything else stays as it is.
- **Refusal:** when the probe fails too, or the file cannot be made, the command refuses with `Git refuses this repository because it is owned by another user (detected dubious ownership at <path>). If you trust it, run \`git config --global --add safe.directory <path>\`, then run the command again.`
- **Test:** Git's `GIT_TEST_ASSUME_DIFFERENT_OWNER=1` reproduces dubious ownership for one user, on Linux and macOS.
