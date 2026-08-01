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
        "details_url": "https://github.com/CorvidLabs/spec-sync/actions/runs/9001",
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
        "repos/CorvidLabs/spec-sync/actions/runs/9001": {
            "id": 9001,
            "event": "pull_request_target",
            "status": "completed",
            "conclusion": "success",
            "path": WORKFLOW,
            "head_sha": base,
            "repository": {"full_name": "CorvidLabs/spec-sync"},
            "pull_requests": [{"number": 480, "head": {"sha": head}}],
        },
    }

    passed = run_verifier(repository, base, head, fixture)
    assert passed.returncode == 0, passed.stderr

    wrong_event = copy.deepcopy(fixture)
    wrong_event["repos/CorvidLabs/spec-sync/actions/runs/9001"]["event"] = "pull_request"
    rejected = run_verifier(repository, base, head, wrong_event)
    assert rejected.returncode != 0
    assert "not base-controlled" in rejected.stderr

    # A later failed/cancelled republish must not poison a prior green result.
    stale_success = copy.deepcopy(fixture)
    stale_success[
        f"repos/CorvidLabs/spec-sync/commits/{head}/check-runs?per_page=100"
    ]["check_runs"].append(
        {
            **check,
            "id": 21,
            "conclusion": "failure",
        }
    )
    still_ok = run_verifier(repository, base, head, stale_success)
    assert still_ok.returncode == 0, still_ok.stderr

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
