#!/usr/bin/env python3
"""Locate reusable GitHub Actions check provenance on a first-parent chain.

Used by review_only / archive_only tips so agents need not push and wait on
every intermediate metadata tip. Walks first-parent ancestors from a starting
SHA (usually HEAD^) and accepts the first successful check that is:

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


def first_parent_chain(root: Path, start: str, limit: int) -> list[str]:
    chain: list[str] = []
    sha = start
    for _ in range(limit):
        if re.fullmatch(r"[0-9a-f]{40}", sha) is None:
            break
        chain.append(sha)
        parents = git(root, "rev-list", "--parents", "-n", "1", sha).split()
        # rev-list --parents -n 1 => "<sha> <parent>..."
        if len(parents) < 2:
            break
        sha = parents[1]
        if sha in chain:
            break
    return chain


def main() -> None:
    repository = required("REPOSITORY")
    server_url = required("SERVER_URL").rstrip("/")
    pull_request = int(required("PR_NUMBER"))
    start_sha = required("START_SHA")
    check_name = required("CHECK_NAME")
    workflow_path = required("WORKFLOW_PATH")
    require_run_success = os.environ.get("REQUIRE_RUN_SUCCESS", "true").strip().lower() in {
        "1",
        "true",
        "yes",
    }
    max_ancestors = int(os.environ.get("MAX_ANCESTORS", "32"))
    root = Path(os.environ.get("GIT_ROOT", ".")).resolve()
    child_kind = os.environ.get("CHILD_KIND", "child")

    github_actions_app = api("apps/github-actions")
    if (
        github_actions_app.get("slug") != "github-actions"
        or github_actions_app.get("name") != "GitHub Actions"
        or (github_actions_app.get("owner") or {}).get("login") != "github"
    ):
        raise SystemExit("could not resolve the official GitHub Actions app")

    errors: list[str] = []
    for ancestor in first_parent_chain(root, start_sha, max_ancestors):
        payload = api(f"repos/{repository}/commits/{ancestor}/check-runs?per_page=100")
        checks = payload.get("check_runs", []) if isinstance(payload, dict) else []
        matches = sorted(
            (check for check in checks if check.get("name") == check_name),
            key=lambda check: int(check.get("id", 0)),
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
                app = check.get("app") or {}
                if (
                    app.get("id") != github_actions_app.get("id")
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
                if (workflow_run.get("repository") or {}).get("full_name") != repository:
                    raise ValueError("workflow run belongs to another repository")
                if not any(
                    item.get("number") == pull_request
                    for item in workflow_run.get("pull_requests") or []
                ):
                    raise ValueError("workflow run is not bound to this PR")
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

    detail = "; ".join(errors) if errors else "no matching checks on first-parent chain"
    raise SystemExit(
        f"{check_name} provenance is not reusable from ancestors of {start_sha}: {detail}"
    )


if __name__ == "__main__":
    main()
