#!/usr/bin/env python3
"""Every job in every workflow declares a time limit of at most 30 minutes (REQ-github-032).

A lane that gets slower fails at its limit, and the limit is not made longer (SPEED-2.b.4). This
test is what refuses the longer limit: it reads every workflow file and fails on a job that
declares no `timeout-minutes`, one above 30, or one it cannot read as a plain number.
"""

from __future__ import annotations

from pathlib import Path
import re
import sys
import unittest


sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = ROOT / ".github/workflows"
MAXIMUM_MINUTES = 30
JOB = re.compile(r"^  ([A-Za-z_][A-Za-z0-9_-]*):\s*(?:#.*)?$")
LIMIT = re.compile(r"^    timeout-minutes:\s*(.*?)\s*(?:#.*)?$")


def job_limits(workflow: str) -> dict[str, str | None]:
    """Return each job's declared `timeout-minutes` text, or None when it declares none."""
    limits: dict[str, str | None] = {}
    in_jobs = False
    job: str | None = None
    for line in workflow.splitlines():
        if re.match(r"^\S", line):
            in_jobs = line.rstrip() == "jobs:"
            job = None
            continue
        if not in_jobs:
            continue
        named = JOB.match(line)
        if named:
            job = named.group(1)
            if job in limits:
                raise ValueError(f"job {job!r} is defined twice")
            limits[job] = None
            continue
        declared = LIMIT.match(line)
        if declared and job is not None:
            if limits[job] is not None:
                raise ValueError(f"job {job!r} declares two limits")
            limits[job] = declared.group(1)
    return limits


def problems(name: str, workflow: str) -> list[str]:
    """Return one sentence for each job whose limit is absent, unreadable or above the maximum."""
    found: list[str] = []
    for job, limit in job_limits(workflow).items():
        if limit is None:
            found.append(f"{name}: job {job} declares no timeout-minutes")
        elif not re.fullmatch(r"[1-9][0-9]*", limit):
            found.append(f"{name}: job {job} declares timeout-minutes {limit!r}, not a plain number")
        elif int(limit) > MAXIMUM_MINUTES:
            found.append(
                f"{name}: job {job} declares {limit} minutes, above the {MAXIMUM_MINUTES}-minute limit"
            )
    return found


def workflow_files() -> list[Path]:
    return sorted(path for path in WORKFLOWS.iterdir() if path.suffix in (".yml", ".yaml"))


class WorkflowTimeLimitTests(unittest.TestCase):
    def test_every_job_declares_a_limit_of_at_most_thirty_minutes(self) -> None:
        found: list[str] = []
        for path in workflow_files():
            found.extend(problems(path.name, path.read_text(encoding="utf-8")))
        self.assertEqual(found, [], "shorten the job; the limit is not raised past 30 minutes")

    def test_the_reader_sees_every_job(self) -> None:
        self.assertGreaterEqual(len(workflow_files()), 6)
        for path in workflow_files():
            with self.subTest(workflow=path.name):
                text = path.read_text(encoding="utf-8")
                jobs = job_limits(text)
                self.assertGreater(len(jobs), 0)
                # No job here calls a reusable workflow, so each one names its runner once.
                self.assertEqual(len(jobs), len(re.findall(r"(?m)^    runs-on:", text)))
                self.assertEqual(len(re.findall(r"(?m)^    uses:", text)), 0)

    def test_the_release_candidate_parts_declare_thirty(self) -> None:
        release = job_limits((WORKFLOWS / "release.yml").read_text(encoding="utf-8"))
        rehearsal = job_limits((WORKFLOWS / "rc-rehearsal.yml").read_text(encoding="utf-8"))
        self.assertEqual(release["qualify"], "30")
        self.assertEqual(rehearsal["rehearse"], "30")

    def test_a_missing_raised_or_computed_limit_is_refused(self) -> None:
        header = "name: Example\non:\n  push:\njobs:\n"
        job = "  build:\n    runs-on: ubuntu-latest\n{limit}    steps:\n      - run: 'true'\n"
        cases = {
            "": ["example.yml: job build declares no timeout-minutes"],
            "    timeout-minutes: 30\n": [],
            "    timeout-minutes: 5 # quick\n": [],
            "    timeout-minutes: 31\n": [
                "example.yml: job build declares 31 minutes, above the 30-minute limit"
            ],
            "    timeout-minutes: 120\n": [
                "example.yml: job build declares 120 minutes, above the 30-minute limit"
            ],
            "    timeout-minutes: ${{ matrix.limit }}\n": [
                "example.yml: job build declares timeout-minutes '${{ matrix.limit }}', not a plain number"
            ],
            "    timeout-minutes: 0\n": [
                "example.yml: job build declares timeout-minutes '0', not a plain number"
            ],
            "    timeout-minutes: 30.5\n": [
                "example.yml: job build declares timeout-minutes '30.5', not a plain number"
            ],
        }
        for limit, expected in cases.items():
            with self.subTest(limit=limit):
                self.assertEqual(problems("example.yml", header + job.format(limit=limit)), expected)

    def test_a_step_limit_does_not_stand_in_for_the_job_limit(self) -> None:
        workflow = (
            "jobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n"
            "      - run: 'true'\n        timeout-minutes: 5\n"
            "  test:\n    runs-on: ubuntu-latest\n    timeout-minutes: 10\n    steps:\n      - run: 'true'\n"
        )
        self.assertEqual(job_limits(workflow), {"build": None, "test": "10"})
        self.assertEqual(problems("x.yml", workflow), ["x.yml: job build declares no timeout-minutes"])

    def test_this_test_runs_in_ci_and_in_the_local_lanes(self) -> None:
        script = ".github/scripts/test-workflow-time-limits.py"
        fledge = (ROOT / "fledge.toml").read_text(encoding="utf-8")
        self.assertIn(f'[tasks.workflow-time-limits-test]\ncmd = "python3 {script}"\n', fledge)
        for lane in ("ci", "verify", "repo"):
            with self.subTest(lane=lane):
                found = re.search(rf"(?ms)^\[lanes\.{lane}\]\n(?:(?!^\[).)*", fledge)
                self.assertIsNotNone(found)
                self.assertIn('"workflow-time-limits-test"', found.group(0) if found else "")
        self.assertIn(f"python3 {script}", (WORKFLOWS / "ci.yml").read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
