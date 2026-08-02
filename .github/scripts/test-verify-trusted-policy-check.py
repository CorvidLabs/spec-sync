#!/usr/bin/env python3

import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[2]
VERIFIER = ROOT / ".github/scripts/verify-trusted-policy-check.py"
WORKFLOW = ".github/workflows/lifecycle-policy-guard.yml"

publisher = (ROOT / WORKFLOW).read_text(encoding="utf-8")
assert "RUN_ID: ${{ github.run_id }}" in publisher
assert "RUN_ATTEMPT: ${{ github.run_attempt }}" in publisher
assert (
    "specsync-trusted-policy:${TRUSTED_WORKFLOW_SHA}:${HEAD_SHA}:"
    "${RUN_ID}:${RUN_ATTEMPT}"
) in publisher


def git(root: Path, *arguments: str) -> str:
    return subprocess.check_output(
        ["git", *arguments],
        cwd=root,
        text=True,
    ).strip()


def run_verifier(root: Path, base: str, head: str, fixture: dict) -> subprocess.CompletedProcess[str]:
    fixture_path = root / "fixture.json"
    fixture_path.write_text(json.dumps(fixture), encoding="utf-8")
    environment = os.environ.copy()
    environment.update(
        {
            "REPOSITORY": "CorvidLabs/spec-sync",
            "SERVER_URL": "https://github.com",
            "PR_NUMBER": "480",
            "BASE_SHA": base,
            "HEAD_SHA": head,
            "GIT_ROOT": str(root),
            "SPECSYNC_TRUSTED_POLICY_CHECK_FIXTURE": str(fixture_path),
        }
    )
    return subprocess.run(
        [sys.executable, str(VERIFIER)],
        env=environment,
        capture_output=True,
        text=True,
        check=False,
    )


with tempfile.TemporaryDirectory() as temporary:
    repository = Path(temporary)
    git(repository, "init", "-b", "main")
    git(repository, "config", "user.email", "test@example.com")
    git(repository, "config", "user.name", "Test")
    workflow = repository / WORKFLOW
    workflow.parent.mkdir(parents=True)
    workflow.write_text("name: trusted\n", encoding="utf-8")
    git(repository, "add", ".")
    git(repository, "commit", "-m", "trusted base")
    base = git(repository, "rev-parse", "HEAD")
    (repository / "README.md").write_text("candidate\n", encoding="utf-8")
    git(repository, "add", ".")
    git(repository, "commit", "-m", "candidate")
    head = git(repository, "rev-parse", "HEAD")

    check = {
        "id": 20,
        "name": "SpecSync trusted policy",
        "head_sha": head,
        "status": "completed",
        "conclusion": "success",
        "app": {"id": 15368, "slug": "github-actions"},
        "external_id": f"specsync-trusted-policy:{base}:{head}",
        "details_url": "https://github.com/CorvidLabs/spec-sync/runs/20",
    }
    runs_endpoint = (
        "repos/CorvidLabs/spec-sync/actions/runs?event=pull_request_target"
        f"&head_sha={head}&per_page=100"
    )
    run = {
        "id": 9001,
        "event": "pull_request_target",
        "status": "completed",
        "conclusion": "success",
        "path": WORKFLOW,
        "head_sha": head,
        "repository": {"full_name": "CorvidLabs/spec-sync"},
        "pull_requests": [
            {
                "number": 480,
                "base": {"sha": base},
                "head": {"sha": head},
            }
        ],
    }
    fixture = {
        "repos/CorvidLabs/spec-sync/commits/"
        f"{head}/check-runs?per_page=100": {"check_runs": [check]},
        "apps/github-actions": {
            "id": 15368,
            "slug": "github-actions",
            "name": "GitHub Actions",
            "owner": {"login": "github"},
        },
        runs_endpoint: {"total_count": 1, "workflow_runs": [run]},
    }

    passed = run_verifier(repository, base, head, fixture)
    assert passed.returncode == 0, passed.stderr

    attempt_endpoint = "repos/CorvidLabs/spec-sync/actions/runs/9001/attempts/1"
    attempt_bound = copy.deepcopy(fixture)
    attempt_bound[
        f"repos/CorvidLabs/spec-sync/commits/{head}/check-runs?per_page=100"
    ]["check_runs"][0]["external_id"] = (
        f"specsync-trusted-policy:{base}:{head}:9001:1"
    )
    attempt_bound[attempt_endpoint] = {**run, "run_attempt": 1}
    passed = run_verifier(repository, base, head, attempt_bound)
    assert passed.returncode == 0, passed.stderr

    same_run_failed_rerun = copy.deepcopy(attempt_bound)
    same_run_failed_rerun[runs_endpoint]["workflow_runs"][0].update(
        {"run_attempt": 2, "conclusion": "failure"}
    )
    passed = run_verifier(repository, base, head, same_run_failed_rerun)
    assert passed.returncode == 0, passed.stderr

    failed_bound_attempt = copy.deepcopy(attempt_bound)
    failed_bound_attempt[attempt_endpoint]["conclusion"] = "failure"
    rejected = run_verifier(repository, base, head, failed_bound_attempt)
    assert rejected.returncode != 0
    assert "workflow run attempt is not successful" in rejected.stderr

    wrong_bound_attempt = copy.deepcopy(attempt_bound)
    wrong_bound_attempt[attempt_endpoint]["run_attempt"] = 2
    rejected = run_verifier(repository, base, head, wrong_bound_attempt)
    assert rejected.returncode != 0
    assert "wrong run attempt" in rejected.stderr

    wrong_bound_run = copy.deepcopy(attempt_bound)
    wrong_bound_run[attempt_endpoint]["id"] = 9002
    rejected = run_verifier(repository, base, head, wrong_bound_run)
    assert rejected.returncode != 0
    assert "wrong workflow run" in rejected.stderr

    wrong_bound_workflow = copy.deepcopy(attempt_bound)
    wrong_bound_workflow[attempt_endpoint]["path"] = ".github/workflows/ci.yml"
    rejected = run_verifier(repository, base, head, wrong_bound_workflow)
    assert rejected.returncode != 0
    assert "different workflow" in rejected.stderr

    mismatched_bound_run = copy.deepcopy(attempt_bound)
    mismatched_bound_run[
        f"repos/CorvidLabs/spec-sync/commits/{head}/check-runs?per_page=100"
    ]["check_runs"][0]["details_url"] = (
        "https://github.com/CorvidLabs/spec-sync/actions/runs/9002"
    )
    rejected = run_verifier(repository, base, head, mismatched_bound_run)
    assert rejected.returncode != 0
    assert "different workflow run" in rejected.stderr

    workflow_details = copy.deepcopy(fixture)
    workflow_details[
        f"repos/CorvidLabs/spec-sync/commits/{head}/check-runs?per_page=100"
    ]["check_runs"][0]["details_url"] = (
        "https://github.com/CorvidLabs/spec-sync/actions/runs/9001"
    )
    passed = run_verifier(repository, base, head, workflow_details)
    assert passed.returncode == 0, passed.stderr

    wrong_workflow_details = copy.deepcopy(workflow_details)
    wrong_workflow_details[
        f"repos/CorvidLabs/spec-sync/commits/{head}/check-runs?per_page=100"
    ]["check_runs"][0]["details_url"] = (
        "https://github.com/CorvidLabs/spec-sync/actions/runs/9999"
    )
    rejected = run_verifier(repository, base, head, wrong_workflow_details)
    assert rejected.returncode != 0
    assert "missing or ambiguous" in rejected.stderr

    (repository / "README.md").write_text("archive child\n", encoding="utf-8")
    git(repository, "add", ".")
    git(repository, "commit", "-m", "archive child")
    descendant = git(repository, "rev-parse", "HEAD")
    moved_tip = copy.deepcopy(fixture)
    moved_tip[runs_endpoint]["workflow_runs"][0]["pull_requests"][0]["head"][
        "sha"
    ] = descendant
    passed = run_verifier(repository, base, head, moved_tip)
    assert passed.returncode == 0, passed.stderr

    wrong_event = copy.deepcopy(fixture)
    wrong_event[runs_endpoint]["workflow_runs"][0]["event"] = "pull_request"
    rejected = run_verifier(repository, base, head, wrong_event)
    assert rejected.returncode != 0
    assert "not base-controlled" in rejected.stderr

    wrong_details = copy.deepcopy(fixture)
    wrong_details[
        f"repos/CorvidLabs/spec-sync/commits/{head}/check-runs?per_page=100"
    ]["check_runs"][0]["details_url"] = "https://attacker.invalid/runs/20"
    rejected = run_verifier(repository, base, head, wrong_details)
    assert rejected.returncode != 0
    assert "not a recognized GitHub check or workflow run" in rejected.stderr

    wrong_app = copy.deepcopy(fixture)
    wrong_app[f"repos/CorvidLabs/spec-sync/commits/{head}/check-runs?per_page=100"][
        "check_runs"
    ][0]["app"]["id"] = 999
    rejected = run_verifier(repository, base, head, wrong_app)
    assert rejected.returncode != 0
    assert "not from GitHub Actions" in rejected.stderr

    wrong_path = copy.deepcopy(fixture)
    wrong_path[runs_endpoint]["workflow_runs"][0]["path"] = ".github/workflows/ci.yml"
    rejected = run_verifier(repository, base, head, wrong_path)
    assert rejected.returncode != 0
    assert "missing or ambiguous" in rejected.stderr

    wrong_repository = copy.deepcopy(fixture)
    wrong_repository[runs_endpoint]["workflow_runs"][0]["repository"][
        "full_name"
    ] = "CorvidLabs/other"
    rejected = run_verifier(repository, base, head, wrong_repository)
    assert rejected.returncode != 0
    assert "belongs to another repository" in rejected.stderr

    wrong_candidate = copy.deepcopy(fixture)
    wrong_candidate[runs_endpoint]["workflow_runs"][0]["head_sha"] = base
    rejected = run_verifier(repository, base, head, wrong_candidate)
    assert rejected.returncode != 0
    assert "exact candidate revision" in rejected.stderr

    wrong_pr = copy.deepcopy(fixture)
    wrong_pr[runs_endpoint]["workflow_runs"][0]["pull_requests"][0]["number"] = 481
    rejected = run_verifier(repository, base, head, wrong_pr)
    assert rejected.returncode != 0
    assert "exact PR and base revision" in rejected.stderr

    wrong_base = copy.deepcopy(fixture)
    wrong_base[runs_endpoint]["workflow_runs"][0]["pull_requests"][0]["base"][
        "sha"
    ] = head
    rejected = run_verifier(repository, base, head, wrong_base)
    assert rejected.returncode != 0
    assert "exact PR and base revision" in rejected.stderr

    unsuccessful_run = copy.deepcopy(fixture)
    unsuccessful_run[runs_endpoint]["workflow_runs"][0]["conclusion"] = "failure"
    rejected = run_verifier(repository, base, head, unsuccessful_run)
    assert rejected.returncode != 0
    assert "workflow run is not successful" in rejected.stderr

    malformed_selected_run = copy.deepcopy(fixture)
    malformed_selected_run[runs_endpoint]["workflow_runs"][0]["id"] = 0
    rejected = run_verifier(repository, base, head, malformed_selected_run)
    assert rejected.returncode != 0
    assert "valid GitHub identity" in rejected.stderr

    incomplete_lookup = copy.deepcopy(fixture)
    incomplete_lookup[runs_endpoint]["total_count"] = 2
    rejected = run_verifier(repository, base, head, incomplete_lookup)
    assert rejected.returncode != 0
    assert "incomplete or exceeds its bound" in rejected.stderr

    stale_success = copy.deepcopy(fixture)
    stale_success[
        f"repos/CorvidLabs/spec-sync/commits/{head}/check-runs?per_page=100"
    ]["check_runs"].append(
        {
            **check,
            "id": 21,
            "conclusion": "failure",
            "details_url": "https://github.com/CorvidLabs/spec-sync/actions/runs/9002",
        }
    )
    stale_success[runs_endpoint] = {
        "total_count": 2,
        "workflow_runs": [run, {**run, "id": 9002, "conclusion": "failure"}],
    }
    passed = run_verifier(repository, base, head, stale_success)
    assert passed.returncode == 0, passed.stderr

    newer_invalid_success = copy.deepcopy(fixture)
    newer_invalid_success[
        f"repos/CorvidLabs/spec-sync/commits/{head}/check-runs?per_page=100"
    ]["check_runs"].append(
        {
            **check,
            "id": 21,
            "external_id": f"specsync-trusted-policy:{'0' * 40}:{head}",
            "details_url": "https://github.com/CorvidLabs/spec-sync/runs/21",
        }
    )
    passed = run_verifier(repository, base, head, newer_invalid_success)
    assert passed.returncode == 0, passed.stderr

    excessive_successes = copy.deepcopy(fixture)
    excessive_successes[
        f"repos/CorvidLabs/spec-sync/commits/{head}/check-runs?per_page=100"
    ]["check_runs"] = [{**check, "id": identifier} for identifier in range(20, 29)]
    rejected = run_verifier(repository, base, head, excessive_successes)
    assert rejected.returncode != 0
    assert "bounded authentication limit" in rejected.stderr

    ambiguous_successes = copy.deepcopy(fixture)
    ambiguous_successes[runs_endpoint] = {
        "total_count": 2,
        "workflow_runs": [run, {**run, "id": 9002}],
    }
    rejected = run_verifier(repository, base, head, ambiguous_successes)
    assert rejected.returncode != 0
    assert "missing or ambiguous" in rejected.stderr

    unsuccessful_only = copy.deepcopy(fixture)
    unsuccessful_only[
        f"repos/CorvidLabs/spec-sync/commits/{head}/check-runs?per_page=100"
    ]["check_runs"][0]["conclusion"] = "cancelled"
    rejected = run_verifier(repository, base, head, unsuccessful_only)
    assert rejected.returncode != 0
    assert "no successful check exists" in rejected.stderr

    wrong_revision = copy.deepcopy(fixture)
    wrong_revision[
        f"repos/CorvidLabs/spec-sync/commits/{head}/check-runs?per_page=100"
    ]["check_runs"][0]["external_id"] = f"specsync-trusted-policy:{'0' * 40}:{head}"
    rejected = run_verifier(repository, base, head, wrong_revision)
    assert rejected.returncode != 0
    assert "latest SpecSync trusted policy result is invalid" in rejected.stderr

with tempfile.TemporaryDirectory() as temporary:
    repository = Path(temporary)
    git(repository, "init", "-b", "main")
    git(repository, "config", "user.email", "test@example.com")
    git(repository, "config", "user.name", "Test")
    (repository / "README.md").write_text("base\n", encoding="utf-8")
    git(repository, "add", ".")
    git(repository, "commit", "-m", "untrusted base")
    base = git(repository, "rev-parse", "HEAD")
    (repository / "README.md").write_text("candidate\n", encoding="utf-8")
    git(repository, "add", ".")
    git(repository, "commit", "-m", "candidate")
    head = git(repository, "rev-parse", "HEAD")
    rejected = run_verifier(repository, base, head, {})
    assert rejected.returncode != 0
    assert "restricted to the immutable SpecSync 6.0 PR/base identity" in rejected.stderr

print("trusted-policy provenance tests passed")
