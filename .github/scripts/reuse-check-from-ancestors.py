#!/usr/bin/env python3
"""Locate reusable GitHub Actions check provenance on a first-parent chain.

Used by review_only / archive_only tips so agents need not push and wait on
every intermediate metadata tip. Starting at HEAD^, it skips only exact
scoped-review pairs or parent-bound workflow-v2 archive moves, then accepts
evidence only at the first product boundary when the check is:

- named exactly CHECK_NAME
- completed/success
- authored by the official GitHub Actions app
- bound to its exact successful job in a qualifying pull_request workflow run
- head_sha equal to the ancestor under consideration

Environment:
  REPOSITORY, SERVER_URL, PR_NUMBER, START_SHA, CHECK_NAME, WORKFLOW_PATH
  REQUIRE_RUN_SUCCESS (default true)
  MAX_ANCESTORS (default 32)
  GIT_ROOT (default cwd)
  GH_TOKEN (via gh)
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path


MAX_ANCESTORS = 32


def required(name: str) -> str:
    value = os.environ.get(name, "").strip()
    if not value:
        raise SystemExit(f"{name} is required")
    return value


def api(endpoint: str) -> dict | list:
    output = subprocess.check_output(
        [
            "gh",
            "api",
            "-H",
            "Accept: application/vnd.github+json",
            "-H",
            "X-GitHub-Api-Version: 2022-11-28",
            endpoint,
        ],
        text=True,
    )
    return json.loads(output)


def git(root: Path, *args: str) -> str:
    return subprocess.check_output(
        ["git", *args],
        cwd=root,
        text=True,
        timeout=30,
    ).strip()


def commit_parents(root: Path, commit: str) -> list[str]:
    fields = git(root, "rev-list", "--parents", "-n", "1", commit).split()
    return fields[1:]


def first_parent_chain(root: Path, start: str, limit: int) -> list[str]:
    chain: list[str] = []
    sha = start
    for _ in range(limit):
        if re.fullmatch(r"[0-9a-f]{40}", sha) is None:
            break
        chain.append(sha)
        parents = commit_parents(root, sha)
        if not parents:
            break
        sha = parents[0]
        if sha in chain:
            break
    return chain


def diff_records(
    root: Path, parent: str, child: str
) -> list[tuple[str, tuple[str, ...]]] | None:
    try:
        name_status = subprocess.check_output(
            ["git", "diff", "--name-status", "-z", "-M", parent, child],
            cwd=root,
            timeout=30,
        )
    except (OSError, subprocess.SubprocessError):
        return False

    try:
        fields = name_status.decode("utf-8").split("\0")
    except UnicodeDecodeError:
        return None
    if not fields or fields[-1] != "":
        return None
    fields.pop()
    records: list[tuple[str, tuple[str, ...]]] = []
    index = 0
    while index < len(fields):
        status = fields[index]
        index += 1
        path_count = 2 if status.startswith(("R", "C")) else 1
        if not status or index + path_count > len(fields):
            return None
        paths = tuple(fields[index : index + path_count])
        if any(not path for path in paths):
            return None
        records.append((status, paths))
        index += path_count
    return records


def review_metadata_only_edge(records: list[tuple[str, tuple[str, ...]]]) -> bool:
    if len(records) != 2 or any(
        status not in {"A", "M"} or len(paths) != 1 for status, paths in records
    ):
        return False

    pattern = re.compile(
        r"^\.specsync/changes/(CHG-[0-9]{4,}-.+)/"
        r"(review(?:-attempts)?\.json)$"
    )
    matched = [pattern.fullmatch(paths[0]) for _status, paths in records]
    if any(match is None for match in matched):
        return False
    change_ids = {match.group(1) for match in matched if match is not None}
    names = {match.group(2) for match in matched if match is not None}
    return len(change_ids) == 1 and names == {"review.json", "review-attempts.json"}


def git_object_exists(root: Path, revision: str, path: str) -> bool:
    return subprocess.run(
        ["git", "cat-file", "-e", f"{revision}:{path}"],
        cwd=root,
        capture_output=True,
        timeout=30,
        check=False,
    ).returncode == 0


def archive_metadata_only_edge(
    root: Path,
    parent: str,
    child: str,
    records: list[tuple[str, tuple[str, ...]]],
) -> bool:
    active_pattern = re.compile(
        r"^\.specsync/changes/(?P<change>CHG-[0-9]{4,}-.+)/(?P<relative>.+)$"
    )
    archive_pattern = re.compile(
        r"^\.specsync/archive/changes/"
        r"(?P<dated>[0-9]{4}-[0-9]{2}-[0-9]{2}-(?P<change>CHG-[0-9]{4,}-.+?))/"
        r"(?P<relative>.+)$"
    )
    change_id: str | None = None
    archive_dir: str | None = None
    active_seen = False
    archive_seen = False

    def bind(match: re.Match[str] | None, *, archive: bool) -> bool:
        nonlocal change_id, archive_dir, active_seen, archive_seen
        if match is None:
            return False
        candidate_id = match.group("change")
        candidate_dir = match.groupdict().get("dated")
        if change_id is not None and candidate_id != change_id:
            return False
        if archive and archive_dir is not None and candidate_dir != archive_dir:
            return False
        change_id = candidate_id
        if archive:
            archive_dir = candidate_dir
            archive_seen = True
        else:
            active_seen = True
        return True

    for status, paths in records:
        kind = status[:1]
        if kind == "R" and len(paths) == 2:
            active = active_pattern.fullmatch(paths[0])
            archive = archive_pattern.fullmatch(paths[1])
            if (
                not bind(active, archive=False)
                or not bind(archive, archive=True)
                or active is None
                or archive is None
                or active.group("relative") != archive.group("relative")
            ):
                return False
        elif kind == "D" and len(paths) == 1:
            if not bind(active_pattern.fullmatch(paths[0]), archive=False):
                return False
        elif kind == "A" and len(paths) == 1:
            if not bind(archive_pattern.fullmatch(paths[0]), archive=True):
                return False
        else:
            return False

    if not active_seen or not archive_seen or change_id is None or archive_dir is None:
        return False
    active_root = f".specsync/changes/{change_id}"
    archive_root = f".specsync/archive/changes/{archive_dir}"
    if (
        not git_object_exists(root, parent, active_root)
        or git_object_exists(root, parent, archive_root)
        or git_object_exists(root, child, active_root)
        or not git_object_exists(root, child, archive_root)
    ):
        return False
    try:
        state = json.loads(git(root, "show", f"{child}:{archive_root}/state.json"))
        finalization = json.loads(
            git(root, "show", f"{child}:{archive_root}/finalization.json")
        )
        parent_tree = git(root, "rev-parse", f"{parent}^{{tree}}")
    except (json.JSONDecodeError, subprocess.CalledProcessError):
        return False
    return (
        isinstance(state, dict)
        and state.get("workflow_version") == 2
        and state.get("id") == change_id
        and state.get("state") == "archived"
        and isinstance(finalization, dict)
        and finalization.get("schema_version") == 2
        and finalization.get("change_id") == change_id
        and finalization.get("implementation_commit") == parent
        and finalization.get("implementation_tree") == parent_tree
    )


def metadata_only_edge(root: Path, parent: str, child: str) -> bool:
    """Authenticate one historical review-only or workflow-v2 archive-only edge."""
    records = diff_records(root, parent, child)
    if records is None:
        return False
    return review_metadata_only_edge(records) or archive_metadata_only_edge(
        root, parent, child, records
    )


def metadata_parent(root: Path, child: str) -> str | None:
    parents = commit_parents(root, child)
    if not parents or not metadata_only_edge(root, parents[0], child):
        return None
    if len(parents) != 1:
        raise ValueError("lifecycle metadata child is a merge commit")
    return parents[0]


def check_metadata_edge_cli(root: Path, parent: str, child: str) -> None:
    try:
        resolved_parent = metadata_parent(root, child)
    except ValueError as error:
        raise SystemExit(str(error)) from error
    if resolved_parent != parent:
        raise SystemExit(f"{child} is not an exact lifecycle metadata child")
    print(f"Verified lifecycle metadata edge {parent}..{child}.")


def positive_bounded_limit(raw: str) -> int:
    try:
        limit = int(raw)
    except ValueError as error:
        raise SystemExit("MAX_ANCESTORS must be an integer from 1 through 32") from error
    if not 1 <= limit <= MAX_ANCESTORS:
        raise SystemExit("MAX_ANCESTORS must be an integer from 1 through 32")
    return limit


def main() -> None:
    repository = required("REPOSITORY")
    server_url = required("SERVER_URL").rstrip("/")
    pull_request = int(required("PR_NUMBER"))
    if pull_request <= 0:
        raise SystemExit("PR_NUMBER must be positive")
    start_sha = required("START_SHA")
    check_name = required("CHECK_NAME")
    workflow_path = required("WORKFLOW_PATH")
    require_run_success = os.environ.get("REQUIRE_RUN_SUCCESS", "true").strip().lower() in {
        "1",
        "true",
        "yes",
    }
    max_ancestors = positive_bounded_limit(os.environ.get("MAX_ANCESTORS", "32"))
    root = Path(os.environ.get("GIT_ROOT", ".")).resolve()
    child_kind = os.environ.get("CHILD_KIND", "child")

    github_actions_app = api("apps/github-actions")
    github_actions_owner = (
        github_actions_app.get("owner")
        if isinstance(github_actions_app, dict)
        else None
    )
    if (
        not isinstance(github_actions_app, dict)
        or github_actions_app.get("slug") != "github-actions"
        or github_actions_app.get("name") != "GitHub Actions"
        or not isinstance(github_actions_owner, dict)
        or github_actions_owner.get("login") != "github"
    ):
        raise SystemExit("could not resolve the official GitHub Actions app")

    errors: list[str] = []
    try:
        chain = first_parent_chain(root, start_sha, max_ancestors)
    except subprocess.CalledProcessError as error:
        raise SystemExit("START_SHA must name an exact available commit") from error
    if not chain or chain[0] != start_sha:
        raise SystemExit("START_SHA must name an exact available commit")
    for index, ancestor in enumerate(chain):
        try:
            parent = metadata_parent(root, ancestor)
        except ValueError as error:
            errors.append(f"stopped at {ancestor}: {error}")
            break
        if parent is not None:
            errors.append(f"skipped lifecycle metadata child {ancestor}")
            if index + 1 == len(chain) or chain[index + 1] != parent:
                errors.append("ancestor search limit exhausted before the product boundary")
                break
            continue

        # This is the nearest product boundary. It may use only its own exact
        # checks; never cross a product commit to borrow older green evidence.
        payload = api(f"repos/{repository}/commits/{ancestor}/check-runs?per_page=100")
        checks = payload.get("check_runs", []) if isinstance(payload, dict) else None
        total_count = payload.get("total_count") if isinstance(payload, dict) else None
        if (
            not isinstance(checks, list)
            or not isinstance(total_count, int)
            or isinstance(total_count, bool)
            or total_count != len(checks)
            or total_count > 100
        ):
            errors.append(f"{ancestor}: malformed check-run payload")
            break
        matches = sorted(
            (
                check
                for check in checks
                if isinstance(check, dict) and check.get("name") == check_name
            ),
            key=lambda check: int(check.get("id", 0))
            if str(check.get("id", "")).isdigit()
            else -1,
            reverse=True,
        )
        for check in matches:
            try:
                if check.get("head_sha") != ancestor:
                    raise ValueError("wrong head SHA")
                if check.get("status") != "completed" or check.get("conclusion") != "success":
                    raise ValueError(
                        f"check not successful: {check.get('status')}/{check.get('conclusion')}"
                    )
                app = check.get("app")
                if (
                    not isinstance(app, dict)
                    or app.get("id") != github_actions_app.get("id")
                    or app.get("slug") != github_actions_app.get("slug")
                ):
                    raise ValueError("check is not from GitHub Actions")
                check_id = check.get("id")
                if (
                    not isinstance(check_id, int)
                    or isinstance(check_id, bool)
                    or check_id <= 0
                ):
                    raise ValueError("check has no valid GitHub identity")
                details_url = str(check.get("details_url") or "")
                match = re.fullmatch(
                    rf"{re.escape(server_url)}/{re.escape(repository)}"
                    r"/actions/runs/([0-9]+)/job/([0-9]+)",
                    details_url,
                )
                if match is None:
                    raise ValueError("check does not name an exact GitHub Actions job")
                run_id = int(match.group(1))
                job_id = int(match.group(2))
                if run_id <= 0 or job_id <= 0:
                    raise ValueError("check has no valid workflow run or job identity")
                workflow_run = api(f"repos/{repository}/actions/runs/{run_id}")
                if not isinstance(workflow_run, dict):
                    raise ValueError("malformed workflow run")
                if workflow_run.get("id") != run_id:
                    raise ValueError("wrong workflow run ID")
                if workflow_run.get("head_sha") != ancestor:
                    raise ValueError("workflow run has the wrong head SHA")
                if workflow_run.get("event") != "pull_request":
                    raise ValueError("workflow run is not for a pull request")
                if workflow_run.get("status") != "completed":
                    raise ValueError("workflow run is not completed")
                if require_run_success and workflow_run.get("conclusion") != "success":
                    raise ValueError("workflow run is not successful")
                if str(workflow_run.get("path") or "").split("@", 1)[0] != workflow_path:
                    raise ValueError("workflow run has the wrong path")
                run_repository = workflow_run.get("repository")
                if (
                    not isinstance(run_repository, dict)
                    or run_repository.get("full_name") != repository
                ):
                    raise ValueError("workflow run belongs to another repository")
                pull_requests = workflow_run.get("pull_requests")
                if not isinstance(pull_requests, list) or any(
                    not isinstance(item, dict) for item in pull_requests
                ):
                    raise ValueError("workflow run has malformed pull-request bindings")
                if not any(item.get("number") == pull_request for item in pull_requests):
                    raise ValueError("workflow run is not bound to this PR")
                job = api(f"repos/{repository}/actions/jobs/{job_id}")
                if not isinstance(job, dict) or job.get("id") != job_id:
                    raise ValueError("malformed workflow job")
                if job.get("run_id") != run_id or job.get("head_sha") != ancestor:
                    raise ValueError("workflow job has the wrong run or head SHA")
                if job.get("name") != check_name:
                    raise ValueError("workflow job has the wrong name")
                if job.get("status") != "completed" or job.get("conclusion") != "success":
                    raise ValueError("workflow job is not successful")
                api_base = (
                    "https://api.github.com"
                    if server_url == "https://github.com"
                    else f"{server_url}/api/v3"
                )
                if job.get("check_run_url") != (
                    f"{api_base}/repos/{repository}/check-runs/{check_id}"
                ):
                    raise ValueError("workflow job is not bound to the selected check")
                if os.environ.get("OUTPUT_FORMAT", "human") == "env":
                    print(f"ancestor_sha={ancestor}")
                    print(f"workflow_run_id={run_id}")
                else:
                    print(
                        f"Reused {check_name} run {run_id} from ancestor {ancestor} "
                        f"(start {start_sha}) for the {child_kind} child."
                    )
                return
            except (
                json.JSONDecodeError,
                subprocess.CalledProcessError,
                TypeError,
                ValueError,
            ) as error:
                errors.append(f"{ancestor} check {check.get('id')}: {error}")
        break

    detail = "; ".join(errors) if errors else "no matching checks on first-parent chain"
    raise SystemExit(
        f"{check_name} provenance is not reusable from ancestors of {start_sha}: {detail}"
    )


if __name__ == "__main__":
    if len(sys.argv) == 4 and sys.argv[1] == "--check-metadata-edge":
        check_metadata_edge_cli(Path.cwd().resolve(), sys.argv[2], sys.argv[3])
    elif len(sys.argv) == 1:
        main()
    else:
        raise SystemExit(
            "usage: reuse-check-from-ancestors.py "
            "[--check-metadata-edge <parent-sha> <child-sha>]"
        )
