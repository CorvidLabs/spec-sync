---
change: never-write-a-project-file-through-a-symbolic-link
artifact: context
---

# Context

A private sweep followed GHSA-7q5v-8jcc-rx54 (the hash cache written through a committed symbolic link). It found the same shape in commands a person runs on purpose inside a repository: a write to a fixed project path with plain `root.join(...)` and `fs::write`, which follows a link the repository committed. None of them runs on a plain `check`, so each needs the person to run the command in a clone.

- `hooks install` and `uninstall`: `CLAUDE.md`, `.cursorrules`, `.github/copilot-instructions.md`, `AGENTS.md`, `.claude/settings.json`.
- `agents install` and `uninstall`: the generated skill and command files and `.specsync/agent-artifacts.json`. A dangling link passed the "is it there" read and was created through.
- `mcp lock --write`: `.specsync/mcp-tools.lock.json`.
- `scaffold`: the registry append.
- `resolve --remote`: the cache under `.specsync-cache/remote-specs/`, whose file name the repository's own `depends_on` chooses.
- `migrate`: every fixed path it writes, and the spec files it rewrites, found by a walker that follows links. `.specsync/.gitignore` was guarded by `exists()`, which a dangling link passes.
- Found while reading, confirmed on released 6.3.0: `new`, `scaffold` and `add-spec` create the target of a dangling `specs/<name>/<name>.spec.md` link, and write the spec and four companions into a directory `specs/<name>` links to. `import` and the wizard share that code.

Not affected, checked on 6.3.0: `lifecycle promote` and `check --fix` stop at discovery on a linked spec file.

Human decisions, relayed by the planning session: "Fold into the same private fix"; for the siblings, "Yes, fold them in"; for refusing with an error and skipping the remote cache with a warning, "OK as proposed"; and, asked about `CLAUDE.md` linked to `AGENTS.md`, "Allow links that stay inside". Intent: SETUP-6, "A command I run in a repository somebody else wrote never writes outside it through a link it found there."

What a session picking this up needs to know:

- It stays in the advisory's private fork with the first change, on the same pull request.
- The hash cache keeps its stricter rule from the first change: any link is refused. The remote spec cache gets that strict rule too.
- Ruled out: refusing every link. `CLAUDE.md` as a link to `AGENTS.md` is a common setup and must keep working.
