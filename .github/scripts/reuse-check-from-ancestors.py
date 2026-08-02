#!/usr/bin/env python3
"""Locate reusable GitHub Actions check provenance on a first-parent chain.

Used by review_only / archive_only tips so agents need not push and wait on
every intermediate metadata tip. Starting at HEAD^, it skips only exact
scoped-review metadata pairs, then accepts evidence only at the first product
boundary when the check is:

- named exactly CHECK_NAME
- completed/success
- authored by the official GitHub Actions app
- bound to a successful (when required) pull_request workflow run on this PR
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


def metadata_only_edge(root: Path, parent: str, child: str) -> bool:
    """Return whether parent..child is exactly one scoped-review metadata update.

    The current review/archive child is classified by classify-ci-paths.sh before
    this helper runs. Only earlier review children need to be crossed here; use
    the historical diff itself so classification does not depend on checkout HEAD.
    """
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
        return False
    if fields and fields[-1] == "":
        fields.pop()
    if len(fields) != 4:
        return False
    records = [(fields[0], fields[1]), (fields[2], fields[3])]
    if any(status not in {"A", "M"} for status, _path in records):
        return False

    pattern = re.compile(
        r"^\.specsync/changes/(CHG-[0-9]{4,}-.+)/"
        r"(review(?:-attempts)?\.json)$"
    )
    matched = [pattern.fullmatch(path) for _status, path in records]
    if any(match is None for match in matched):
        return False
    change_ids = {match.group(1) for match in matched if match is not None}
    names = {match.group(2) for match in matched if match is not None}
    return len(change_ids) == 1 and names == {"review.json", "review-attempts.json"}


def metadata_parent(root: Path, child: str) -> str | None:
    parents = commit_parents(root, child)
    if not parents or not metadata_only_edge(root, parents[0], child):
        return None
    if len(parents) != 1:
        raise ValueError("scoped-review metadata child is a merge commit")
    return parents[0]


def check_metadata_edge_cli(root: Path, parent: str, child: str) -> None:
    try:
        resolved_parent = metadata_parent(root, child)
    except ValueError as error:
        raise SystemExit(str(error)) from error
    if resolved_parent != parent:
        raise SystemExit(f"{child} is not an exact scoped-review metadata child")
    print(f"Verified scoped-review metadata edge {parent}..{child}.")


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
            errors.append(f"skipped scoped-review metadata child {ancestor}")
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
                details_url = str(check.get("details_url") or "")
                # trust jobs use .../runs/ID/job/JOB; policy uses .../runs/ID
                match = re.fullmatch(
                    rf"{re.escape(server_url)}/{re.escape(repository)}"
                    r"/actions/runs/([0-9]+)(?:/job/[0-9]+)?",
                    details_url,
                )
                if match is None:
                    raise ValueError("invalid GitHub Actions details URL")
                run_id = int(match.group(1))
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
