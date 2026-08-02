#!/usr/bin/env python3

import json
import os
from pathlib import Path
import re
import subprocess
import sys


WORKFLOW_PATH = ".github/workflows/lifecycle-policy-guard.yml"
WORKFLOW_V2_BASELINE_PATH = ".specsync/workflow-v2-baseline.json"
CHECK_NAME = "SpecSync trusted policy"
BOOTSTRAP_REPOSITORY = "CorvidLabs/spec-sync"
BOOTSTRAP_PULL_REQUEST = 480
BOOTSTRAP_BASE_SHA = "fc091c88f72a6d2fb2df168f4baa4370579ff8a2"
BOOTSTRAP_BASE_REF = "main"
BOOTSTRAP_HEAD_REF = "leif/specsync-6-stabilization"
FIXTURE_PATH = os.environ.get("SPECSYNC_TRUSTED_POLICY_CHECK_FIXTURE", "").strip()
FIXTURE = (
    json.loads(Path(FIXTURE_PATH).read_text(encoding="utf-8"))
    if FIXTURE_PATH
    else None
)


def required_env(name: str) -> str:
    value = os.environ.get(name, "").strip()
    if not value:
        raise SystemExit(f"{name} is required")
    return value


def api(endpoint: str) -> dict:
    if FIXTURE is not None:
        response = FIXTURE.get(endpoint)
        if not isinstance(response, dict):
            raise SystemExit(f"fixture has no object response for {endpoint}")
        return response
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


def git(root: Path, *arguments: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", *arguments],
        cwd=root,
        check=check,
        capture_output=True,
        text=True,
        timeout=30,
    )


def exact_sha(value: str, label: str) -> str:
    if re.fullmatch(r"[0-9a-f]{40}", value) is None:
        raise SystemExit(f"{label} is not an exact commit SHA")
    return value


repository = required_env("REPOSITORY")
server_url = required_env("SERVER_URL").rstrip("/")
pull_request = int(required_env("PR_NUMBER"))
candidate = exact_sha(required_env("HEAD_SHA"), "HEAD_SHA")
base = exact_sha(required_env("BASE_SHA"), "BASE_SHA")
root = Path(required_env("GIT_ROOT")).resolve()

base_guard = git(root, "cat-file", "-e", f"{base}:{WORKFLOW_PATH}", check=False)
if base_guard.returncode != 0:
    if (
        repository != BOOTSTRAP_REPOSITORY
        or pull_request != BOOTSTRAP_PULL_REQUEST
        or base != BOOTSTRAP_BASE_SHA
    ):
        raise SystemExit(
            "trusted-policy bootstrap is restricted to the immutable SpecSync 6.0 PR/base identity"
        )
    pull = api(f"repos/{repository}/pulls/{pull_request}")
    pull_head = (pull.get("head") or {}).get("sha")
    if (
        pull.get("number") != pull_request
        or (pull.get("base") or {}).get("sha") != base
        or (pull.get("base") or {}).get("ref") != BOOTSTRAP_BASE_REF
        or ((pull.get("base") or {}).get("repo") or {}).get("full_name") != repository
        or (pull.get("head") or {}).get("ref") != BOOTSTRAP_HEAD_REF
        or not isinstance(pull_head, str)
        or re.fullmatch(r"[0-9a-f]{40}", pull_head) is None
    ):
        raise SystemExit(
            "trusted-policy bootstrap PR metadata does not match its frozen repository/base/branch identity"
        )
    # Archive-only (and review-only) gates validate the exact parent SHA, which is an
    # ancestor of the PR head rather than equal to it. Accept both the tip and its
    # ancestors so same-PR finalization can bootstrap on PR #480.
    if pull_head != candidate and git(
        root, "merge-base", "--is-ancestor", candidate, pull_head, check=False
    ).returncode != 0:
        raise SystemExit(
            "trusted-policy bootstrap candidate is not the PR head or an ancestor of it"
        )
    if git(root, "merge-base", "--is-ancestor", base, candidate, check=False).returncode != 0:
        raise SystemExit("trusted-policy bootstrap candidate does not descend from its frozen base")
    required_paths = [
        WORKFLOW_PATH,
        ".github/scripts/verify-trusted-policy-check.py",
        ".github/scripts/test-verify-trusted-policy-check.py",
        ".github/scripts/verify-archive-introduction.py",
        ".github/scripts/test-lifecycle-workflows.sh",
        ".github/scripts/lifecycle-validation-limits.json",
        WORKFLOW_V2_BASELINE_PATH,
    ]
    added_paths = set(
        git(
            root,
            "diff",
            "--diff-filter=A",
            "--name-only",
            base,
            candidate,
            "--",
            *required_paths,
        ).stdout.splitlines()
    )
    for required in required_paths:
        if git(root, "cat-file", "-e", f"{candidate}:{required}", check=False).returncode != 0:
            raise SystemExit(
                f"bootstrap candidate is missing required trusted-policy file {required}"
            )
        if required not in added_paths:
            raise SystemExit(
                f"bootstrap candidate did not introduce required trusted-policy file {required}"
            )
    baseline_bytes = git(
        root,
        "show",
        f"{candidate}:{WORKFLOW_V2_BASELINE_PATH}",
    ).stdout
    expected_baseline = {
        "schema_version": 1,
        "domain": "specsync.workflow-v2-baseline.v1",
        "cutoff_commit": BOOTSTRAP_BASE_SHA,
    }
    if (
        json.loads(baseline_bytes) != expected_baseline
        or baseline_bytes != json.dumps(expected_baseline, indent=2) + "\n"
    ):
        raise SystemExit(
            "trusted-policy bootstrap baseline is not the canonical exact-base adoption anchor"
        )
    print(
        "Trusted-policy bootstrap: frozen CorvidLabs/spec-sync PR #480 on its exact base; "
        "full CI and independent review remain mandatory."
    )
    sys.exit(0)

checks = api(f"repos/{repository}/commits/{candidate}/check-runs?per_page=100").get(
    "check_runs", []
)
github_actions = api("apps/github-actions")
if (
    github_actions.get("slug") != "github-actions"
    or github_actions.get("name") != "GitHub Actions"
    or (github_actions.get("owner") or {}).get("login") != "github"
):
    raise SystemExit("could not authenticate the official GitHub Actions app")

matches = sorted(
    (check for check in checks if check.get("name") == CHECK_NAME),
    key=lambda check: int(check.get("id", 0)),
    reverse=True,
)
if not matches:
    raise SystemExit(f"{CHECK_NAME} has no check on the exact candidate")
# Prefer an authenticated success for this exact SHA. A newer cancelled or
# failed republication can be caused by a moved PR tip and must not poison an
# earlier successful result for the unchanged candidate.
successful_matches = [
    check
    for check in matches
    if check.get("status") == "completed" and check.get("conclusion") == "success"
]
check = successful_matches[0] if successful_matches else matches[0]
try:
    if check.get("head_sha") != candidate:
        raise ValueError("wrong candidate SHA")
    if check.get("status") != "completed" or check.get("conclusion") != "success":
        raise ValueError("no successful check exists for the exact candidate")
    app = check.get("app") or {}
    if (
        app.get("id") != github_actions.get("id")
        or app.get("slug") != github_actions.get("slug")
    ):
        raise ValueError("check is not from GitHub Actions")
    external = str(check.get("external_id") or "")
    external_match = re.fullmatch(
        rf"specsync-trusted-policy:([0-9a-f]{{40}}):{re.escape(candidate)}",
        external,
    )
    if external_match is None:
        raise ValueError("check lacks the exact trusted revision binding")
    trusted = external_match.group(1)
    if trusted != base:
        raise ValueError("check is not bound to the exact PR base revision")

    check_id = int(check.get("id", 0))
    if check_id <= 0:
        raise ValueError("check has no valid GitHub identity")
    details = str(check.get("details_url") or "")
    workflow_details = re.fullmatch(
        rf"{re.escape(server_url)}/{re.escape(repository)}/actions/runs/([0-9]+)",
        details,
    )
    canonical_check_details = f"{server_url}/{repository}/runs/{check_id}"
    if workflow_details is None and details != canonical_check_details:
        raise ValueError("check details URL is not a recognized GitHub check or workflow run")

    runs_endpoint = (
        f"repos/{repository}/actions/runs?event=pull_request_target"
        f"&head_sha={candidate}&per_page=100"
    )
    runs_payload = api(runs_endpoint)
    runs = runs_payload.get("workflow_runs")
    total_count = runs_payload.get("total_count")
    if (
        not isinstance(runs, list)
        or not isinstance(total_count, int)
        or isinstance(total_count, bool)
        or total_count != len(runs)
        or total_count > 100
    ):
        raise ValueError("workflow run lookup is incomplete or exceeds its bound")

    policy_runs = [
        run
        for run in runs
        if isinstance(run, dict)
        and str(run.get("path") or "").split("@", 1)[0] == WORKFLOW_PATH
    ]
    if len(policy_runs) != 1:
        raise ValueError("workflow run lookup is missing or ambiguous")
    run = policy_runs[0]
    run_id = int(run.get("id", 0))
    if run_id <= 0:
        raise ValueError("workflow run has no valid GitHub identity")
    if workflow_details is not None and int(workflow_details.group(1)) != run_id:
        raise ValueError("check details URL names a different workflow run")
    if run.get("event") != "pull_request_target":
        raise ValueError("workflow run is not base-controlled")
    if run.get("status") != "completed" or run.get("conclusion") != "success":
        raise ValueError("workflow run is not successful")
    if (run.get("repository") or {}).get("full_name") != repository:
        raise ValueError("workflow run belongs to another repository")
    if run.get("head_sha") != trusted and run.get("head_sha") != candidate:
        raise ValueError("workflow run is unrelated to the trusted or candidate revision")
    if run.get("head_sha") != candidate:
        raise ValueError("workflow run does not use the exact candidate revision")
    pull_requests = run.get("pull_requests") or []
    matching_prs = [
        item
        for item in pull_requests
        if item.get("number") == pull_request
        and (item.get("base") or {}).get("sha") == trusted
        and (
            (item.get("head") or {}).get("sha") == candidate
            or git(
                root,
                "merge-base",
                "--is-ancestor",
                candidate,
                str((item.get("head") or {}).get("sha") or ""),
                check=False,
            ).returncode
            == 0
        )
    ]
    if len(matching_prs) != 1:
        raise ValueError("workflow run is not bound to the exact PR and base revision")
    pull_head = str((matching_prs[0].get("head") or {}).get("sha") or "")
    if re.fullmatch(r"[0-9a-f]{40}", pull_head) is None:
        raise ValueError("workflow run has no exact PR head revision")
    if git(root, "rev-parse", "--verify", f"{trusted}^{{commit}}").stdout.strip() != trusted:
        raise ValueError("trusted workflow revision is unavailable")
    if git(root, "merge-base", "--is-ancestor", trusted, candidate, check=False).returncode != 0:
        raise ValueError("trusted workflow revision is not an ancestor of the candidate")
    if (
        git(root, "rev-parse", f"{trusted}:{WORKFLOW_PATH}").stdout.strip()
        != git(root, "rev-parse", f"{base}:{WORKFLOW_PATH}").stdout.strip()
    ):
        raise ValueError("trusted workflow does not match the exact PR base policy")
    print(
        f"Verified {CHECK_NAME} run {run_id} at trusted revision {trusted} "
        f"for candidate {candidate}."
    )
except (
    json.JSONDecodeError,
    subprocess.CalledProcessError,
    subprocess.TimeoutExpired,
    TypeError,
    ValueError,
) as error:
    raise SystemExit(f"latest {CHECK_NAME} result is invalid: {error}") from error
