#!/usr/bin/env python3
"""Attestation on main, and a pull-request summary that cannot look green over a failure.

CI-9, REQ-github-029 (#798): `attest` ("Record attestation") must run on every push to main
whose `Required CI gate` passed. It had no `always()`, so GitHub's implicit job-level
`success()` looked at every job upstream of `ci-gate`. `corvid-pet` is one of them and is
skipped on every push, so `attest` was skipped on every push.

CI-10, REQ-github-030 (#799): `corvid-pet`'s summary table had no row for `preflight` or
`lifecycle-gate` and called every skipped job "Not selected". When the lifecycle gate failed,
the jobs gated on it were skipped and the summary came out green. Each row now carries the job's
own selection, the same rule `implementation-gate` uses: a skipped job is "Not selected" only
when classify deselected it, and a selected job that was skipped was blocked by a failure.

Both are checked with the required-gate tests' own workflow model (`test-required-ci-gate.py`),
which simulates GitHub's job-level status functions, and by running the summary step's real
script.
"""

from __future__ import annotations

import importlib.util
import os
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from types import ModuleType
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ".github/scripts/test-ci-attest-and-summary.py"
ATTEST = "attest"
SUMMARY = "corvid-pet"
SUMMARY_STEP = "status"
ROW = re.compile(
    r"(?P<label>[^=]+)=(?P<selected>true|\$\{\{ .+? \}\})=\$\{\{ needs\.(?P<job>[A-Za-z0-9_-]+)\.result \}\}"
)


def load_gate_model() -> ModuleType:
    """The required-gate tests' workflow model, imported rather than copied."""
    path = ROOT / ".github/scripts/test-required-ci-gate.py"
    spec = importlib.util.spec_from_file_location("required_ci_gate_model", path)
    if spec is None or spec.loader is None:
        raise ImportError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


GATE = load_gate_model()


def summary_step(workflow: dict[str, Any]) -> dict[str, Any]:
    steps = [step for step in workflow["jobs"][SUMMARY]["steps"] if step.get("id") == SUMMARY_STEP]
    if len(steps) != 1:
        raise AssertionError(f"{SUMMARY} must have exactly one step with id: {SUMMARY_STEP}")
    return steps[0]


def check_rows(workflow: dict[str, Any]) -> dict[str, str]:
    """The summary step's CHECK_* rows, by variable name, unevaluated."""
    env = summary_step(workflow).get("env") or {}
    return {name: str(value) for name, value in env.items() if name.startswith("CHECK_")}


def run_summary(script: str, environment: dict[str, str]) -> tuple[str, str, str]:
    """Run the summary script the way the runner does (`bash -e`); return (result, context, log)."""
    with tempfile.TemporaryDirectory() as directory:
        output = Path(directory) / "github_output"
        output.write_text("", encoding="utf-8")
        path = Path(directory) / "step.sh"
        path.write_text(script, encoding="utf-8")
        env = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"), "GITHUB_OUTPUT": str(output), **environment}
        completed = subprocess.run(
            ["bash", "-e", str(path)], env=env, capture_output=True, text=True, timeout=60, check=False
        )
        log = completed.stdout + completed.stderr
        if completed.returncode != 0:
            raise AssertionError(f"summary step exited {completed.returncode}:\n{log}")
        text = output.read_text(encoding="utf-8")
    result = re.search(r"(?m)^result=(.*)$", text)
    context = re.search(r"(?ms)^context<<(\S+)\n(.*?)\n\1$", text)
    return (result.group(1) if result else ""), (context.group(2) if context else ""), log


def simulated_summary(
    workflow: dict[str, Any], scenario: Any, forced: dict[str, str] | None = None
) -> tuple[dict[str, str], str, str]:
    """Simulate one run, then run the summary step over those results: (results, result, context)."""
    jobs = workflow["jobs"]
    results, _ = GATE.simulate(workflow, scenario, forced)
    if results[SUMMARY] == "skipped":
        return results, "skipped", ""
    outputs = scenario.outputs(jobs)
    evaluator = GATE.Evaluator(
        GATE.expression_context(jobs, SUMMARY, results, scenario, outputs),
        GATE.job_functions(jobs, SUMMARY, results),
    )
    step = summary_step(workflow)
    environment = {
        name: GATE.interpolate(value, evaluator.evaluate) for name, value in (step.get("env") or {}).items()
    }
    result, context, _ = run_summary(str(step["run"]), environment)
    return results, result, context


def scenario(label: str) -> Any:
    return next(case for case in GATE.SCENARIOS if case.label == label)


FULL_PR = "Full PR (src/, tests/, workflows, docs/, *.md)"


class AttestationTests(unittest.TestCase):
    workflow: dict[str, Any]

    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = GATE.load_workflow()

    def test_attest_runs_on_every_main_push_whose_required_gate_passed(self) -> None:
        pushes = [case for case in GATE.SCENARIOS if case.event == "push" and case.ref == "refs/heads/main"]
        self.assertTrue(pushes, "the gate model has no push-to-main scenario")
        for case in pushes:
            with self.subTest(scenario=case.label):
                results, transcript = GATE.simulate(self.workflow, case)
                self.assertEqual(results[GATE.REQUIRED_JOB], "success", transcript)
                self.assertEqual(
                    results[ATTEST],
                    "success",
                    f"`{ATTEST}` was {results[ATTEST]} on a green push to main; give it `if: always() && "
                    f"needs.{GATE.REQUIRED_JOB}.result == 'success' && …` so the implicit success() over "
                    f"every upstream job (corvid-pet is skipped on pushes) cannot skip it (CI-9)",
                )

    def test_attest_does_not_run_when_the_required_gate_fails(self) -> None:
        results, transcript = GATE.simulate(self.workflow, scenario("Push to main (full)"), {"test": "failure"})
        self.assertEqual(results[GATE.REQUIRED_JOB], "failure", transcript)
        self.assertEqual(results[ATTEST], "skipped")

    def test_attest_never_runs_off_a_push_to_main(self) -> None:
        for case in GATE.SCENARIOS:
            if case.event == "push" and case.ref == "refs/heads/main":
                continue
            with self.subTest(scenario=case.label):
                results, _ = GATE.simulate(self.workflow, case)
                self.assertEqual(results[ATTEST], "skipped")


class SummaryTests(unittest.TestCase):
    workflow: dict[str, Any]

    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = GATE.load_workflow()

    def test_the_summary_has_one_row_for_every_job_it_waits_on(self) -> None:
        jobs = self.workflow["jobs"]
        waited = [job for job in jobs[SUMMARY]["needs"] if job != "classify"]
        rows = check_rows(self.workflow)
        named = []
        for name, value in rows.items():
            match = ROW.fullmatch(value)
            self.assertIsNotNone(match, f"{name}={value!r} is not `<label>=<selected>=${{{{ needs.<job>.result }}}}`")
            if match:
                named.append(match.group("job"))
        for job in waited:
            with self.subTest(job=job):
                self.assertEqual(named.count(job), 1, f"the {SUMMARY} summary needs exactly one row for {job} (CI-10)")
        self.assertEqual(sorted(named), sorted(waited), "every row must be for a job the summary waits on")

    def test_each_row_selects_on_its_jobs_own_condition(self) -> None:
        jobs = self.workflow["jobs"]
        for name, value in check_rows(self.workflow).items():
            match = ROW.fullmatch(value)
            if match is None:
                self.fail(f"{name}={value!r} is not `<label>=<selected>=${{{{ needs.<job>.result }}}}`")
            job = match.group("job")
            with self.subTest(job=job):
                selected = match.group("selected")
                actual = "true" if selected == "true" else " ".join(GATE.strip_wrapper(selected).split())
                self.assertEqual(actual, GATE.selection_expression(jobs[job]))

    def test_a_green_run_has_a_green_summary(self) -> None:
        results, result, context = simulated_summary(self.workflow, scenario(FULL_PR))
        self.assertEqual(result, "success", context)
        self.assertNotIn("❌", context)
        self.assertNotIn("⛔", context)

    def test_a_failed_lifecycle_gate_or_preflight_turns_the_summary_red(self) -> None:
        for failed in ("lifecycle-gate", "preflight"):
            with self.subTest(failed=failed):
                results, result, context = simulated_summary(self.workflow, scenario(FULL_PR), {failed: "failure"})
                self.assertEqual(results[GATE.REQUIRED_JOB], "failure")
                self.assertEqual(result, "failure", f"a failed {failed} produced a green summary:\n{context}")
                blocked = [job for job, value in results.items() if value == "skipped"]
                self.assertTrue(blocked, "the failure should have skipped the jobs gated on it")
                self.assertNotIn("Not selected", context, f"blocked jobs read as deselected:\n{context}")

    def test_deselected_reads_not_selected_and_blocked_does_not(self) -> None:
        script = str(summary_step(self.workflow)["run"])
        result, context, _ = run_summary(script, {"CHECK_A": "Deselected job=false=skipped"})
        self.assertEqual(result, "success", context)
        self.assertIn("Not selected", context)
        result, context, _ = run_summary(
            script, {"CHECK_A": "Deselected job=false=skipped", "CHECK_B": "Blocked job=true=skipped"}
        )
        self.assertEqual(result, "failure", context)
        self.assertRegex(context, r"\*\*Blocked job\*\* \| ⛔ Blocked")

    def test_this_test_runs_through_fledge_and_ci(self) -> None:
        fledge = (ROOT / "fledge.toml").read_text(encoding="utf-8")
        task = re.search(
            r'(?ms)^\[tasks\.([A-Za-z0-9_-]+)\]\ncmd\s*=\s*"[^"]*' + re.escape(SCRIPT) + r'[^"]*"\s*$', fledge
        )
        self.assertIsNotNone(task, f"fledge.toml must define a task that runs {SCRIPT}")
        if task is None:
            return
        for lane in ("verify", "ci"):
            body = re.search(rf"(?ms)^\[lanes\.{lane}\]\n(.*?)^\]", fledge)
            self.assertIsNotNone(body, f"fledge.toml has no {lane} lane")
            if body is not None:
                self.assertIn(f'"{task.group(1)}"', body.group(1), f"the {lane} lane must run {SCRIPT}")
        commands = [str(step.get("run", "")) for job in self.workflow["jobs"].values() for step in job["steps"]]
        self.assertTrue(any(SCRIPT in command for command in commands), f"ci.yml must run {SCRIPT}")


if __name__ == "__main__":
    unittest.main()
