## ADDED

### REQUIREMENT REQ-util-004

Inside a Git work tree, the project file functions SHALL confine a path to the work tree that holds the project root (`git rev-parse --show-toplevel`, asked once per root with `GIT_DIR` and `GIT_WORK_TREE` unset), so a monorepo subproject follows a link that stays inside its repository; outside a work tree, or when Git is not installed, the confinement SHALL stay the project root. Their messages SHALL name a path relative to the project root. An absolute symbolic link SHALL never be followed, and its refusal SHALL say it is an absolute link. A write to an existing file SHALL open it without blocking and without truncating, check on the open handle that it is a regular file, and only then empty and write it. `read_file_within` SHALL read a file within exactly a directory the person names, following a link that stays inside it and refusing one that leaves it, with the same bound. `create_new_project_dir` SHALL create a directory that must not exist yet, `is_project_file` SHALL say whether a path is a regular file the functions would read, `remove_empty_project_dir` SHALL remove an empty directory without following a link to it, and `find_project_files` SHALL list the files under a project directory through the same rule, starting at the project root itself when the directory is the root, without walking a link to a directory or the Git directory, and SHALL stop with the reason past an entry or a depth budget.

Acceptance Criteria
- A subproject at `packages/app` installs hooks through `CLAUDE.md -> ../../AGENTS.md` and reads its specs through `specs -> ../../specs/app`; a link that leaves the repository is still refused.
- A link to an absolute path inside the project is refused as an absolute link.
- A FIFO is refused without blocking; an outside directory's link out is refused by `read_file_within`; an empty directory reached through a link out is not removed.

### REQUIREMENT REQ-util-005

The project file functions SHALL refuse a path that resolves into the Git directory, directly or through links: a path with a `.git` component in any letter case or with trailing dots or spaces, a path under the work tree's Git directory (`git rev-parse --absolute-git-dir`, asked with the work tree's top folder), or a symbolic link whose canonical target lies under either, with the error `<path> leads into the Git directory`. Git runs hooks and reads its config from there, so a link that stays inside the project SHALL still never reach it. SpecSync's own writes into the Git directory, such as `hooks install` writing a Git hook, SHALL keep going through their own paths and are not project file writes.

Acceptance Criteria
- `hooks install` refuses `CLAUDE.md -> .git/hooks/post-checkout` and `AGENTS.md -> .GIT/config`, and both files keep their bytes.
- `change answer` refuses a committed `change.md -> ../../../.git/hooks/pre-commit`, and the hook keeps its bytes.
- With a Git directory named `gitstore` inside the work tree, a path under it, a link to a hook in it, and a path through a link to it are refused, the hook keeps its bytes, and a walk lists nothing from it.
