#!/usr/bin/env python3
"""Focused tests for bounded first-parent GitHub check provenance reuse."""

import contextlib
import copy
import importlib.util
import io
import os
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / ".github/scripts/reuse-check-from-ancestors.py"
CI_WORKFLOW = ROOT / ".github/workflows/ci.yml"


def load_module():
    specification = importlib.util.spec_from_file_location("reuse_check", SCRIPT)
    assert specification and specification.loader
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def git(root: Path, *arguments: str) -> str:
    return subprocess.check_output(["git", *arguments], cwd=root, text=True).strip()


module = load_module()
ci_workflow = CI_WORKFLOW.read_text(encoding="utf-8")
assert "provenance_helpers" in ci_workflow
assert "CI provenance helpers may change only with the protected CI workflow" in ci_workflow
assert str(SCRIPT.relative_to(ROOT)) in ci_workflow
assert str(Path(__file__).resolve().relative_to(ROOT)) in ci_workflow

with tempfile.TemporaryDirectory() as temporary:
    repository = Path(temporary)
    git(repository, "init", "-b", "main")
    git(repository, "config", "user.email", "test@example.com")
    git(repository, "config", "user.name", "Test")
    (repository / "a.txt").write_text("a\n", encoding="utf-8")
    git(repository, "add", ".")
    git(repository, "commit", "-m", "a")
    first = git(repository, "rev-parse", "HEAD")
    (repository / "b.txt").write_text("b\n", encoding="utf-8")
    review_dir = repository / ".specsync/changes/CHG-0001-test"
    (review_dir / "deltas").mkdir(parents=True)
    (review_dir / "change.md").write_text("# Change\n", encoding="utf-8")
    (review_dir / "deltas/github.md").write_text("# Delta\n", encoding="utf-8")
    git(repository, "add", ".")
    git(repository, "commit", "-m", "b")
    product = git(repository, "rev-parse", "HEAD")
    (review_dir / "review.json").write_text("{}\n", encoding="utf-8")
    (review_dir / "review-attempts.json").write_text("{}\n", encoding="utf-8")
    git(repository, "add", ".")
    git(repository, "commit", "-m", "review metadata")
    metadata = git(repository, "rev-parse", "HEAD")

    archive_dir = (
        repository
        / ".specsync/archive/changes/2026-08-02-CHG-0001-test"
    )
    archive_dir.parent.mkdir(parents=True)
    git(repository, "mv", str(review_dir), str(archive_dir))
    metadata_tree = git(repository, "rev-parse", f"{metadata}^{{tree}}")
    (archive_dir / "state.json").write_text(
        '{"workflow_version":2,"id":"CHG-0001-test","state":"archived"}\n',
        encoding="utf-8",
    )
    (archive_dir / "finalization.json").write_text(
        (
            '{"schema_version":2,"change_id":"CHG-0001-test",'
            f'"implementation_commit":"{metadata}",'
            f'"implementation_tree":"{metadata_tree}"}}\n'
        ),
        encoding="utf-8",
    )
    git(repository, "add", ".")
    git(repository, "commit", "-m", "archive metadata")
    archive = git(repository, "rev-parse", "HEAD")

    chain = module.first_parent_chain(repository, metadata, 10)
    assert chain == [metadata, product, first]
    assert module.first_parent_chain(repository, metadata, 2) == [metadata, product]
    assert module.metadata_only_edge(repository, product, metadata)
    assert module.metadata_only_edge(repository, metadata, archive)
    assert module.metadata_parent(repository, archive) == metadata
    assert not module.metadata_only_edge(repository, first, product)

    git(repository, "switch", "-c", "bad-archive", metadata)
    bad_archive_dir = (
        repository
        / ".specsync/archive/changes/2026-08-02-CHG-0001-test"
    )
    bad_archive_dir.parent.mkdir(parents=True)
    git(repository, "mv", str(review_dir), str(bad_archive_dir))
    (bad_archive_dir / "state.json").write_text(
        '{"workflow_version":2,"id":"CHG-0001-test","state":"archived"}\n',
        encoding="utf-8",
    )
    (bad_archive_dir / "finalization.json").write_text(
        (
            '{"schema_version":2,"change_id":"CHG-0001-test",'
            f'"implementation_commit":"{product}",'
            f'"implementation_tree":"{metadata_tree}"}}\n'
        ),
        encoding="utf-8",
    )
    git(repository, "add", ".")
    git(repository, "commit", "-m", "bad archive metadata")
    bad_archive = git(repository, "rev-parse", "HEAD")
    assert not module.metadata_only_edge(repository, metadata, bad_archive)
    git(repository, "switch", "main")
    with contextlib.redirect_stdout(io.StringIO()):
        module.check_metadata_edge_cli(repository, product, metadata)

    git(repository, "switch", "-c", "side", first)
    (repository / "side.txt").write_text("side\n", encoding="utf-8")
    git(repository, "add", ".")
    git(repository, "commit", "-m", "second parent")
    second_parent = git(repository, "rev-parse", "HEAD")
    git(repository, "switch", "main")
    git(repository, "merge", "--no-ff", "side", "-m", "merge side")
    merge = git(repository, "rev-parse", "HEAD")
    merge_chain = module.first_parent_chain(repository, merge, 10)
    assert merge_chain[:2] == [merge, archive]
    assert second_parent not in merge_chain

    git(repository, "switch", "-c", "evil-review", product)
    evil_review_dir = repository / ".specsync/changes/CHG-0002-test"
    evil_review_dir.mkdir(parents=True)
    (evil_review_dir / "review.json").write_text("{}\n", encoding="utf-8")
    (evil_review_dir / "review-attempts.json").write_text("{}\n", encoding="utf-8")
    git(repository, "add", ".")
    git(repository, "commit", "-m", "review on second parent")
    git(repository, "switch", "-c", "evil-main", product)
    git(repository, "merge", "--no-ff", "evil-review", "-m", "evil metadata merge")
    evil_merge = git(repository, "rev-parse", "HEAD")
    assert module.metadata_only_edge(repository, product, evil_merge)
    try:
        module.metadata_parent(repository, evil_merge)
    except ValueError as error:
        assert "merge commit" in str(error)
    else:
        raise AssertionError("accepted a scoped-review merge commit")
    try:
        module.check_metadata_edge_cli(repository, product, evil_merge)
    except SystemExit as error:
        assert "merge commit" in str(error)
    else:
        raise AssertionError("CLI accepted a scoped-review merge commit")
    git(repository, "switch", "main")

    assert module.positive_bounded_limit("1") == 1
    assert module.positive_bounded_limit("32") == 32
    for invalid_limit in ("0", "33", "not-a-number"):
        try:
            module.positive_bounded_limit(invalid_limit)
        except SystemExit:
            pass
        else:
            raise AssertionError(f"accepted invalid limit {invalid_limit}")

    check = {
        "id": 20,
        "name": "trust",
        "head_sha": product,
        "status": "completed",
        "conclusion": "success",
        "app": {"id": 15368, "slug": "github-actions"},
        "details_url": "https://github.com/CorvidLabs/spec-sync/actions/runs/9001/job/10",
    }
    run = {
        "id": 9001,
        "event": "pull_request",
        "status": "completed",
        "conclusion": "success",
        "path": ".github/workflows/trust.yml",
        "head_sha": product,
        "repository": {"full_name": "CorvidLabs/spec-sync"},
        "pull_requests": [{"number": 42}],
    }
    job = {
        "id": 10,
        "run_id": 9001,
        "head_sha": product,
        "name": "trust",
        "status": "completed",
        "conclusion": "success",
        "check_run_url": "https://api.github.com/repos/CorvidLabs/spec-sync/check-runs/20",
    }
    metadata_check = {
        **check,
        "id": 21,
        "head_sha": metadata,
        "details_url": "https://github.com/CorvidLabs/spec-sync/actions/runs/9002/job/11",
    }
    metadata_run = {
        **run,
        "id": 9002,
        "head_sha": metadata,
    }
    checks_endpoint = (
        f"repos/CorvidLabs/spec-sync/commits/{product}/check-runs?per_page=100"
    )
    fixture = {
        "apps/github-actions": {
            "id": 15368,
            "slug": "github-actions",
            "name": "GitHub Actions",
            "owner": {"login": "github"},
        },
        f"repos/CorvidLabs/spec-sync/commits/{metadata}/check-runs?per_page=100": {
            "total_count": 1,
            "check_runs": [metadata_check],
        },
        checks_endpoint: {"total_count": 1, "check_runs": [check]},
        "repos/CorvidLabs/spec-sync/actions/runs/9001": run,
        "repos/CorvidLabs/spec-sync/actions/runs/9002": metadata_run,
        "repos/CorvidLabs/spec-sync/actions/jobs/10": job,
    }
    environment = {
        "REPOSITORY": "CorvidLabs/spec-sync",
        "SERVER_URL": "https://github.com",
        "PR_NUMBER": "42",
        "START_SHA": metadata,
        "CHECK_NAME": "trust",
        "WORKFLOW_PATH": ".github/workflows/trust.yml",
        "REQUIRE_RUN_SUCCESS": "true",
        "MAX_ANCESTORS": "3",
        "GIT_ROOT": str(repository),
        "CHILD_KIND": "review",
    }

    def run_case(
        candidate_fixture: dict,
        *,
        metadata_edge: bool = True,
        max_ancestors: str = "3",
        start_sha: str = metadata,
    ) -> tuple[int, str]:
        prior_environment = os.environ.copy()
        prior_api = module.api
        prior_edge = module.metadata_only_edge
        output = io.StringIO()
        try:
            os.environ.update(environment)
            os.environ["MAX_ANCESTORS"] = max_ancestors
            os.environ["START_SHA"] = start_sha
            def fixture_api(endpoint: str):
                if endpoint in candidate_fixture:
                    return candidate_fixture[endpoint]
                if endpoint.endswith("/check-runs?per_page=100"):
                    return {"total_count": 0, "check_runs": []}
                raise KeyError(endpoint)

            module.api = fixture_api
            module.metadata_only_edge = (
                prior_edge
                if metadata_edge
                else lambda _root, _parent, _child: False
            )
            with contextlib.redirect_stdout(output):
                try:
                    module.main()
                except SystemExit:
                    return 1, output.getvalue()
            return 0, output.getvalue()
        finally:
            os.environ.clear()
            os.environ.update(prior_environment)
            module.api = prior_api
            module.metadata_only_edge = prior_edge

    status, output = run_case(fixture)
    assert status == 0
    assert f"ancestor {product}" in output

    status, output = run_case(fixture, start_sha=archive)
    assert status == 0
    assert f"ancestor {product}" in output

    non_metadata = copy.deepcopy(fixture)
    non_metadata[
        f"repos/CorvidLabs/spec-sync/commits/{metadata}/check-runs?per_page=100"
    ] = {"total_count": 0, "check_runs": []}
    status, _ = run_case(non_metadata, metadata_edge=False)
    assert status == 1

    product_boundary = copy.deepcopy(fixture)
    product_boundary[checks_endpoint] = {"total_count": 0, "check_runs": []}
    product_boundary[
        f"repos/CorvidLabs/spec-sync/commits/{first}/check-runs?per_page=100"
    ] = {
        "total_count": 1,
        "check_runs": [{**check, "head_sha": first}],
    }
    status, _ = run_case(product_boundary)
    assert status == 1

    status, _ = run_case(fixture, max_ancestors="1")
    assert status == 1

    mutations = []
    wrong_app = copy.deepcopy(fixture)
    wrong_app[checks_endpoint]["check_runs"][0]["app"]["id"] = 999
    mutations.append(wrong_app)
    wrong_pr = copy.deepcopy(fixture)
    wrong_pr["repos/CorvidLabs/spec-sync/actions/runs/9001"]["pull_requests"][0][
        "number"
    ] = 99
    mutations.append(wrong_pr)
    wrong_repository = copy.deepcopy(fixture)
    wrong_repository["repos/CorvidLabs/spec-sync/actions/runs/9001"]["repository"][
        "full_name"
    ] = "CorvidLabs/other"
    mutations.append(wrong_repository)
    wrong_workflow = copy.deepcopy(fixture)
    wrong_workflow["repos/CorvidLabs/spec-sync/actions/runs/9001"][
        "path"
    ] = ".github/workflows/ci.yml"
    mutations.append(wrong_workflow)
    wrong_head = copy.deepcopy(fixture)
    wrong_head["repos/CorvidLabs/spec-sync/actions/runs/9001"]["head_sha"] = first
    mutations.append(wrong_head)
    wrong_event = copy.deepcopy(fixture)
    wrong_event["repos/CorvidLabs/spec-sync/actions/runs/9001"][
        "event"
    ] = "pull_request_target"
    mutations.append(wrong_event)
    wrong_details = copy.deepcopy(fixture)
    wrong_details[checks_endpoint]["check_runs"][0][
        "details_url"
    ] = "https://attacker.invalid/actions/runs/9001/job/10"
    mutations.append(wrong_details)
    run_level_details = copy.deepcopy(fixture)
    run_level_details[checks_endpoint]["check_runs"][0][
        "details_url"
    ] = "https://github.com/CorvidLabs/spec-sync/actions/runs/9001"
    mutations.append(run_level_details)
    wrong_job_name = copy.deepcopy(fixture)
    wrong_job_name["repos/CorvidLabs/spec-sync/actions/jobs/10"]["name"] = "other"
    mutations.append(wrong_job_name)
    wrong_job_check = copy.deepcopy(fixture)
    wrong_job_check["repos/CorvidLabs/spec-sync/actions/jobs/10"][
        "check_run_url"
    ] = "https://api.github.com/repos/CorvidLabs/spec-sync/check-runs/999"
    mutations.append(wrong_job_check)
    unsuccessful = copy.deepcopy(fixture)
    unsuccessful[checks_endpoint]["check_runs"][0]["conclusion"] = "cancelled"
    mutations.append(unsuccessful)
    incomplete = copy.deepcopy(fixture)
    incomplete[checks_endpoint]["total_count"] = 2
    mutations.append(incomplete)
    malformed = copy.deepcopy(fixture)
    malformed[checks_endpoint] = {"total_count": 1, "check_runs": "not-a-list"}
    mutations.append(malformed)

    for rejected_fixture in mutations:
        status, _ = run_case(rejected_fixture)
        assert status == 1

print("reuse-check-from-ancestors tests passed")
