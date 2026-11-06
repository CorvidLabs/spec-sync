#!/usr/bin/env python3
"""Every pull request starts CI, and classify alone picks its lane (CI-6, REQ-github-026).

`Required CI gate` is the only required check on main. Before this change, `ci.yml` started
only when a changed path matched a glob in its `pull_request` or `push` `paths:` list, while
`classify-ci-paths.sh` sends every path it does not name to the full lane. A pull request that
changed only unlisted paths therefore never started CI, the required check never reported, and
the pull request could not merge. #941 hit this with a pull request that changed only
`.chough/` records; `scripts/`, the agent skill installs and several root files had the same gap.

These tests hold the workflow to two rules:

1. `ci.yml` has no path filter on `pull_request` or `push`, so every pull request starts CI.
2. A change that touches only Chough records, or only `scripts/`, is classified by the real
   classifier into a lane, and simulating that lane with the required-gate tests' own model
   (`test-required-ci-gate.py`) reports `Required CI gate` green when every selected job passes.

The workflow is read through Ruby's standard Psych parser, as the other workflow tests do.
"""

from __future__ import annotations

import importlib.util
import json
import re
import subprocess
import sys
import unittest
from pathlib import Path
from types import ModuleType
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ".github/scripts/test-ci-trigger-paths.py"
CLASSIFIER = ROOT / ".github/scripts/classify-ci-paths.sh"
TRIGGERS = ("pull_request", "push")
FILTER_KEYS = ("paths", "paths-ignore")
RECOVERY = (
    "remove the `paths:` / `paths-ignore:` filter from that trigger in .github/workflows/ci.yml "
    "(classify-ci-paths.sh picks the lane), then run `python3 .github/scripts/test-ci-trigger-paths.py`"
)

# The two cases #941 exposed. Every kind of record Chough writes is listed, tracked or not.
CHANGES = {
    "Chough records only": (
        ".chough/session.json",
        ".chough/sessions/882-linux-baseline.json",
        ".chough/sessions/archive/s5-881.json",
        ".chough/history/cycles/CYC-0/EVT-0.json",
        ".chough/evidence/OBS-0.json",
        ".chough/artifacts/OBS-0.log",
        ".chough/bugs/BUG-0.json",
        ".chough/cases/github.cases.toml",
    ),
    "scripts/ only": (
        "scripts/pre-push-gate.sh",
        "scripts/spec-check-fast.sh",
    ),
}


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

RUBY_TRIGGERS = r'''
require "json"
require "psych"
document = Psych.safe_load(File.read(ARGV.fetch(0), encoding: "UTF-8"), permitted_classes: [], aliases: false)
raise "workflow is not a mapping" unless document.is_a?(Hash)
# YAML 1.1 reads a bare `on:` key as true.
on = document.key?("on") ? document["on"] : document[true]
raise "workflow has no on: mapping" unless on.is_a?(Hash)
puts JSON.generate(on)
'''


def load_triggers() -> dict[str, Any]:
    """The workflow's `on:` mapping (the gate model's loader keeps only env and jobs)."""
    result = subprocess.run(
        ["ruby", "-e", RUBY_TRIGGERS, str(GATE.WORKFLOW)],
        capture_output=True,
        text=True,
        timeout=30,
        check=True,
    )
    return json.loads(result.stdout)


def path_filters(on: dict[str, Any], event: str) -> dict[str, Any]:
    trigger = on.get(event)
    if not isinstance(trigger, dict):
        return {}
    return {key: trigger[key] for key in FILTER_KEYS if key in trigger}


def classify(paths: tuple[str, ...]) -> dict[str, str]:
    result = subprocess.run(
        ["bash", str(CLASSIFIER), str(ROOT), "false", "names"],
        input="".join(f"{path}\0" for path in paths),
        capture_output=True,
        text=True,
        timeout=60,
        check=True,
    )
    return dict(line.split("=", 1) for line in result.stdout.splitlines() if "=" in line)


class TriggerTests(unittest.TestCase):
    workflow: dict[str, Any]
    on: dict[str, Any]

    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = GATE.load_workflow()
        cls.on = load_triggers()

    def test_ci_has_no_path_filter(self) -> None:
        for event in TRIGGERS:
            with self.subTest(event=event):
                self.assertIn(event, self.on, f"ci.yml must run on {event}")
                filters = path_filters(self.on, event)
                self.assertEqual(
                    filters,
                    {},
                    f"ci.yml filters {event} by {sorted(filters)}, so a pull request that changes only "
                    f"unlisted paths never starts CI and never gets the required check (CI-6); {RECOVERY}",
                )

    def test_chough_only_and_scripts_only_changes_report_the_required_gate(self) -> None:
        jobs = self.workflow["jobs"]
        outputs = set(jobs["classify"]["outputs"])
        for label, paths in CHANGES.items():
            for event, ref in (("pull_request", "refs/pull/1/merge"), ("push", "refs/heads/main")):
                with self.subTest(change=label, event=event):
                    self.assertEqual(
                        path_filters(self.on, event),
                        {},
                        f"a {label} change never starts CI on {event}; {RECOVERY}",
                    )
                    classified = classify(paths)
                    flags = tuple(key for key in sorted(outputs) if classified.get(key) == "true")
                    lanes = [lane for lane in ("full", "site", "vscode") if lane in flags]
                    self.assertTrue(lanes, f"classify chose no lane for a {label} change: {classified}")
                    results, transcript = GATE.simulate(self.workflow, GATE.Scenario(label, event, flags, ref))
                    self.assertEqual(results[GATE.REQUIRED_JOB], "success", transcript)
                    self.assertEqual(results[GATE.GATE_JOB], "success", transcript)

    def test_this_test_runs_through_fledge_and_ci(self) -> None:
        fledge = (ROOT / "fledge.toml").read_text(encoding="utf-8")
        task = re.search(
            r'(?ms)^\[tasks\.([A-Za-z0-9_-]+)\]\ncmd\s*=\s*"[^"]*' + re.escape(SCRIPT) + r'[^"]*"\s*$',
            fledge,
        )
        self.assertIsNotNone(task, f"fledge.toml must define a task that runs {SCRIPT}")
        if task is None:
            return
        for lane in ("verify", "ci"):
            body = re.search(rf'(?ms)^\[lanes\.{lane}\]\n(.*?)^\]', fledge)
            self.assertIsNotNone(body, f"fledge.toml has no {lane} lane")
            if body is not None:
                self.assertIn(f'"{task.group(1)}"', body.group(1), f"the {lane} lane must run {SCRIPT}")
        commands = [str(step.get("run", "")) for job in self.workflow["jobs"].values() for step in job["steps"]]
        self.assertTrue(any(SCRIPT in command for command in commands), f"ci.yml must run {SCRIPT}")


if __name__ == "__main__":
    unittest.main()
