#!/usr/bin/env python3
"""The lint gate covers the tests, not only the code that ships (CI-8, REQ-github-028, #608).

`cargo clippy` without `--all-targets` compiles only the library and binary targets, so it never
looks at `#[cfg(test)]` modules or `tests/`. CI and the fledge `lint` task ran exactly that, and
for the whole of 6.x a clean lint run meant "the shipped code lints clean" while it read as "the
code lints clean". Test code could, and did, accumulate lints nobody saw.

These tests hold every place the project runs clippy to three rules:

1. Every `cargo clippy` command runs with `--all-targets`.
2. Every one denies warnings (`-- -D warnings`) and none passes an allow or cap that would let a
   lint through (`-A`, `--allow`, `--cap-lints`).
3. CI's lint step and the fledge `lint` task exist, and the `check`, `pre-commit`, `verify`, `ci`
   and `repo` lanes all run that task, so the gate cannot disappear and leave these checks with
   nothing to read.

The workflow is read through the required-gate tests' own loader (`test-required-ci-gate.py`),
which parses ci.yml with Ruby's standard Psych parser, as the other workflow tests do.
"""

from __future__ import annotations

import importlib.util
import re
import shlex
import sys
import unittest
from pathlib import Path
from types import ModuleType

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ".github/scripts/test-clippy-all-targets.py"
FLEDGE = ROOT / "fledge.toml"
WEAKENING = ("-A", "--allow", "--cap-lints")
RECOVERY = (
    "run clippy as `cargo clippy --all-targets -- -D warnings` there, clear what it reports in "
    "the code (never with an allow), then run `python3 .github/scripts/test-clippy-all-targets.py`"
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


def clippy_commands(text: str) -> list[list[str]]:
    """Every `cargo clippy` invocation in a shell snippet, split into words.

    Shell control operators end a command: tokens after `&&`, `||`, `;` or `|` belong to
    the next command and must never count toward this invocation's flags.
    """
    commands = []
    for line in text.replace("\\\n", " ").splitlines():
        for segment in re.split(r"&&|\|\||[;|]", line):
            if not re.search(r"\bcargo\s+clippy\b", segment):
                continue
            try:
                words = shlex.split(segment, comments=True)
            except ValueError:
                # A quote opened on an earlier line of a multi-line script; plain words are enough here.
                words = segment.split()
            for index in range(len(words) - 1):
                if words[index] == "cargo" and words[index + 1] == "clippy":
                    commands.append(words[index:])
    return commands


def fledge_tasks() -> dict[str, str]:
    text = FLEDGE.read_text(encoding="utf-8")
    return dict(re.findall(r'(?m)^\[tasks\.([A-Za-z0-9_-]+)\]\ncmd\s*=\s*"([^"]*)"\s*$', text))


def fledge_lane(name: str) -> str:
    text = FLEDGE.read_text(encoding="utf-8")
    body = re.search(rf"(?ms)^\[lanes\.{re.escape(name)}\]\n(.*?)(?=^\[|\Z)", text)
    return body.group(1) if body else ""


def problems(command: list[str]) -> list[str]:
    """What is wrong with one clippy command; empty when it lints every target and denies warnings."""
    found = []
    if "--all-targets" not in command:
        found.append("it has no --all-targets, so test code is never compiled or linted")
    separator = command.index("--") if "--" in command else len(command)
    lint_args = command[separator + 1 :]
    if not any(lint_args[i : i + 2] == ["-D", "warnings"] for i in range(len(lint_args))) and "-Dwarnings" not in lint_args:
        found.append("it does not deny warnings (`-- -D warnings`)")
    for word in command:
        if (
            word.startswith("-A")
            or word in WEAKENING
            or any(word.startswith(flag + "=") for flag in WEAKENING if flag.startswith("--"))
        ):
            found.append(f"it passes `{word}`, which lets a lint through")
    return found


class ClippyAllTargetsTests(unittest.TestCase):
    def test_every_ci_clippy_step_lints_every_target_and_denies_warnings(self) -> None:
        workflow = GATE.load_workflow()
        seen = []
        for job_name, job in workflow["jobs"].items():
            for step in job.get("steps", []):
                for command in clippy_commands(str(step.get("run", ""))):
                    seen.append(job_name)
                    with self.subTest(job=job_name, command=" ".join(command)):
                        self.assertEqual(
                            problems(command),
                            [],
                            f"ci.yml job `{job_name}` runs `{' '.join(command)}`; {RECOVERY}",
                        )
        self.assertIn("test", seen, f"ci.yml's `test` job must run clippy; {RECOVERY}")

    def test_every_fledge_clippy_task_lints_every_target_and_denies_warnings(self) -> None:
        tasks = fledge_tasks()
        clippy_tasks = {name: cmd for name, cmd in tasks.items() if clippy_commands(cmd)}
        self.assertIn("lint", clippy_tasks, "fledge.toml's `lint` task must run cargo clippy")
        for name, cmd in clippy_tasks.items():
            for command in clippy_commands(cmd):
                with self.subTest(task=name, command=cmd):
                    self.assertEqual(problems(command), [], f"fledge task `{name}` runs `{cmd}`; {RECOVERY}")

    def test_every_lane_that_must_lint_runs_the_lint_task(self) -> None:
        for lane in ("check", "pre-commit", "verify", "ci", "repo"):
            with self.subTest(lane=lane):
                self.assertIn('"lint"', fledge_lane(lane), f"the fledge `{lane}` lane must run the `lint` task")

    def test_the_problem_check_catches_each_weakening(self) -> None:
        good = shlex.split("cargo clippy --all-targets -- -D warnings")
        self.assertEqual(problems(good), [])
        for bad in (
            "cargo clippy -- -D warnings",
            "cargo clippy --all-targets",
            "cargo clippy --all-targets -- -D warnings -A clippy::all",
            "cargo clippy --all-targets -- -D warnings --cap-lints=warn",
            "cargo clippy --all-targets -- --allow=clippy::all -D warnings",
        ):
            with self.subTest(command=bad):
                self.assertNotEqual(problems(shlex.split(bad)), [])
        # A later command's flags never count toward the clippy invocation.
        for chained in (
            "cargo clippy -- -D warnings && cargo test --all-targets",
            "cargo test --all-targets; cargo clippy --all-targets",
        ):
            for command in clippy_commands(chained):
                with self.subTest(command=" ".join(command)):
                    self.assertNotEqual(problems(command), [], chained)
        good_chained = clippy_commands("cargo clippy --all-targets -- -D warnings && cargo test")
        self.assertEqual(len(good_chained), 1)
        self.assertEqual(problems(good_chained[0]), [])

    def test_this_test_runs_through_fledge_and_ci(self) -> None:
        tasks = fledge_tasks()
        task = next((name for name, cmd in tasks.items() if SCRIPT in cmd), None)
        self.assertIsNotNone(task, f"fledge.toml must define a task that runs {SCRIPT}")
        for lane in ("verify", "ci"):
            self.assertIn(f'"{task}"', fledge_lane(lane), f"the fledge `{lane}` lane must run {SCRIPT}")
        workflow = GATE.load_workflow()
        commands = [str(step.get("run", "")) for job in workflow["jobs"].values() for step in job.get("steps", [])]
        self.assertTrue(any(SCRIPT in command for command in commands), f"ci.yml must run {SCRIPT}")


if __name__ == "__main__":
    unittest.main()
