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
import sys
from pathlib import Path

root = Path(sys.argv[1])
changes = root / ".specsync" / "changes"
rows = []
if changes.is_dir():
    for directory in sorted(path for path in changes.iterdir() if path.is_dir()):
        state_file = directory / "state.json"
        if not state_file.is_file():
            continue
        change_id = directory.name
        try:
            payload = json.loads(state_file.read_text(encoding="utf-8"))
            state = payload.get("state") if isinstance(payload, dict) else None
            if not isinstance(state, str) or not state:
                state = "unreadable"
        except (OSError, json.JSONDecodeError):
            state = "unreadable"
        rank = {
            "verifying": 0,
            "approved": 1,
            "implementing": 2,
            "draft": 3,
        }.get(state, 4)
        ship = (
            state == "verifying"
            and (directory / "review.json").is_file()
            and (directory / "verification.json").is_file()
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
