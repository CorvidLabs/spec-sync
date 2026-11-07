#!/usr/bin/env bash
# Say whether this pull request is finished.
#
# An open change workspace means the pull request is not finished. The message
# names one change to archive first and one command. No open workspace means
# the pull request is ready to merge when the other checks are green.
#
# The same sentences are produced by `pull_request_archive_sentence` in
# src/commands/change.rs. Keep the two in step.
set -euo pipefail

root="${1:-.}"
shopt -s nullglob

python3 - "$root" <<'PY'
import json
import re
import stat
import subprocess
import sys
from pathlib import Path

root = Path(sys.argv[1])
changes = root / ".specsync" / "changes"
rows = []


def holds_tracked_entry(directory: Path) -> bool:
    """True when any non-directory entry sits at any depth.

    A symlink is a file Git can track. Rust's husk check treats one the same
    way, so a symlink-only directory stays open here too.
    """
    try:
        children = list(directory.iterdir())
    except OSError:
        return False
    for child in children:
        try:
            kind = child.lstat()
        except OSError:
            continue
        if stat.S_ISDIR(kind.st_mode):
            if holds_tracked_entry(child):
                return True
        else:
            return True
    return False


def git(*args):
    try:
        return subprocess.run(
            ["git", *args], cwd=root, check=False, capture_output=True, text=True
        )
    except OSError:
        return None


def git_text(*args):
    completed = git(*args)
    if completed is None or completed.returncode != 0:
        return None
    return completed.stdout


def remote_default_ref():
    ref = (git_text("symbolic-ref", "--short", "refs/remotes/origin/HEAD") or "").strip()
    if ref:
        return ref
    for candidate in ("origin/main", "origin/master"):
        if git_text("rev-parse", "--verify", candidate) is not None:
            return candidate
    return None


def is_dated_package_for(package, change_id):
    suffix = "-" + change_id
    if not package.endswith(suffix):
        return False
    return re.fullmatch(r"\d{4}-\d{2}-\d{2}", package[: -len(suffix)]) is not None


def archived_upstream(change_id):
    """The test `upstream_archive_of` in src/change makes (#889, #902).

    True when the remote default holds a dated archive package of this change id,
    added by a commit HEAD does not contain. `ship` refuses such a copy, so it must
    not be told to run it. The script never fetches: without a remote default ref,
    as in a single-commit CI checkout, or when Git cannot answer, it says nothing.
    """
    reference = remote_default_ref()
    prefix = git_text("rev-parse", "--show-prefix")
    if reference is None or prefix is None:
        return False
    archive_root = prefix.strip() + ".specsync/archive/changes/"
    listing = git_text("ls-tree", "--full-tree", "--name-only", reference, "--", archive_root)
    for entry in (listing or "").splitlines():
        if not is_dated_package_for(entry.rsplit("/", 1)[-1], change_id):
            continue
        state_path = f"{entry}/state.json"
        state = git_text("show", f"{reference}:{state_path}", "--")
        try:
            archived = json.loads(state) if state is not None else None
        except json.JSONDecodeError:
            archived = None
        if not isinstance(archived, dict) or archived.get("id") != change_id:
            continue
        commit = (
            git_text(
                "log", "-1", "--format=%H", "--diff-filter=A", reference,
                "--", f":(top,literal){state_path}",
            )
            or ""
        ).strip()
        if not commit:
            continue
        ancestry = git("merge-base", "--is-ancestor", commit, "HEAD")
        if ancestry is not None and ancestry.returncode == 1:
            return True
    return False


if changes.is_dir():
    for directory in sorted(path for path in changes.iterdir() if path.is_dir()):
        state_file = directory / "state.json"
        # A husk is a directory tree with no regular file (often an empty
        # `deltas/` left after checkout). A directory that holds a file but
        # no state.json is damaged and still open: same rank as unreadable.
        missing_state = not state_file.is_file()
        if missing_state and not holds_tracked_entry(directory):
            continue
        change_id = directory.name
        if missing_state:
            payload = None
            state = "unreadable"
        else:
            try:
                payload = json.loads(state_file.read_text(encoding="utf-8"))
                state = payload.get("state") if isinstance(payload, dict) else None
                if not isinstance(state, str) or not state:
                    state = "unreadable"
            except (OSError, json.JSONDecodeError):
                payload = None
                state = "unreadable"
        rank = {
            "accepted": 0,
            "verifying": 1,
            "approved": 2,
            "implementing": 3,
            "draft": 4,
        }.get(state, 5)
        workflow = payload.get("workflow_version", 1) if isinstance(payload, dict) else 1
        if not isinstance(workflow, int):
            workflow = 1
        verdict = None
        reviewed_commit = None
        review_path = directory / "review.json"
        if review_path.is_file():
            try:
                review = json.loads(review_path.read_text(encoding="utf-8"))
            except (OSError, json.JSONDecodeError):
                review = None
            if isinstance(review, dict):
                verdict = review.get("verdict")
                reviewed_commit = review.get("implementation_commit")
        head = ""
        try:
            completed = subprocess.run(
                ["git", "rev-parse", "HEAD"],
                cwd=root,
                check=False,
                capture_output=True,
                text=True,
            )
            if completed.returncode == 0:
                head = completed.stdout.strip()
        except OSError:
            head = ""
        ship = (
            state == "verifying"
            and workflow >= 2
            and verdict == "pass"
            and isinstance(reviewed_commit, str)
            and reviewed_commit != ""
            and reviewed_commit == head
            and (directory / "verification.json").is_file()
            and not archived_upstream(change_id)
        )
        rows.append((rank, change_id, ship))

if not rows:
    print("All changes are archived. Ready to merge when the other checks are green.")
    sys.exit(0)

rows.sort()
first_rank, first_id, first_ship = rows[0]
command = (
    f"specsync change ship {first_id}"
    if first_ship
    else f"specsync change status {first_id}"
)
if len(rows) == 1:
    print(
        f"This pull request is not finished. `{first_id}` is still open. Run `{command}`."
    )
else:
    still_open = ", ".join(f"`{change_id}`" for _, change_id, _ in rows)
    print(
        "This pull request is not finished. "
        f"Archive `{first_id}` first. Still open: {still_open}. Run `{command}`."
    )
sys.exit(1)
PY
