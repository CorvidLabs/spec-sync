#!/usr/bin/env python3
"""Each expensive confidence signal has one owner (REQ-github-006, #493).

GitHub CI owns formatting, linting and the full test suite. The Trust action's lifecycle
step runs `fledge lanes run trust-lifecycle`, and that lane must not run the suite again:
no test, lint, clippy or verify step. `lanes.verify` stays the full local completion suite.
The guide that explains this is published on the site, and docs/ci-confidence.md points
there.
"""

from __future__ import annotations

import copy
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TRUST = ROOT / ".trust.toml"
FLEDGE = ROOT / "fledge.toml"
SITE_GUIDE = ROOT / "site/src/content/docs/ci-confidence.md"
DOCS_POINTER = ROOT / "docs/ci-confidence.md"

LIFECYCLE_COMMAND = ["fledge", "lanes", "run", "trust-lifecycle"]
FORBIDDEN_IN_TRUST_LIFECYCLE = {"test", "test-nextest", "lint", "verify"}
VERIFY_MUST_RUN = {"fmt", "lint", "check-types", "test", "build", "spec-check"}


def lane_steps(lane: dict) -> list[str]:
    """Every task a lane runs, with parallel groups flattened."""
    names: list[str] = []
    for step in lane.get("steps", []):
        if isinstance(step, str):
            names.append(step)
        elif isinstance(step, dict):
            names.extend(step.get("parallel", []))
    return names


def ownership_errors(trust: dict, fledge: dict) -> list[str]:
    errors: list[str] = []
    command = trust.get("lifecycle", {}).get("command")
    if command != LIFECYCLE_COMMAND:
        errors.append(f".trust.toml lifecycle command is {command!r}, not {LIFECYCLE_COMMAND!r}")
    lanes = fledge.get("lanes", {})
    tasks = fledge.get("tasks", {})
    lifecycle = lanes.get("trust-lifecycle")
    if lifecycle is None:
        errors.append("fledge.toml has no trust-lifecycle lane")
    else:
        for name in lane_steps(lifecycle):
            command_text = str(tasks.get(name, {}).get("cmd", ""))
            if (
                name in FORBIDDEN_IN_TRUST_LIFECYCLE
                or "cargo test" in command_text
                or "clippy" in command_text
                or "lanes run verify" in command_text
            ):
                errors.append(
                    f"trust-lifecycle runs `{name}`, which repeats a suite CI owns"
                )
    verify = lanes.get("verify")
    if verify is None:
        errors.append("fledge.toml has no verify lane")
    else:
        missing = sorted(VERIFY_MUST_RUN - set(lane_steps(verify)))
        if missing:
            errors.append(f"the verify lane no longer runs {', '.join(missing)}")
    return errors


def load() -> tuple[dict, dict]:
    return (
        tomllib.loads(TRUST.read_text(encoding="utf-8")),
        tomllib.loads(FLEDGE.read_text(encoding="utf-8")),
    )


class LaneOwnershipTests(unittest.TestCase):
    def test_the_repository_holds_to_one_owner_per_suite(self) -> None:
        trust, fledge = load()
        self.assertEqual(ownership_errors(trust, fledge), [])

    def test_a_suite_added_to_trust_lifecycle_fails(self) -> None:
        trust, fledge = load()
        for duplicate in ("test", "lint", "verify"):
            mutated = copy.deepcopy(fledge)
            mutated["lanes"]["trust-lifecycle"]["steps"].append(duplicate)
            self.assertTrue(
                any(f"`{duplicate}`" in error for error in ownership_errors(trust, mutated)),
                duplicate,
            )

    def test_a_task_that_runs_cargo_test_in_trust_lifecycle_fails(self) -> None:
        trust, fledge = load()
        mutated = copy.deepcopy(fledge)
        mutated["tasks"]["sneaky"] = {"cmd": "cargo test --workspace"}
        mutated["lanes"]["trust-lifecycle"]["steps"].append("sneaky")
        self.assertTrue(any("`sneaky`" in error for error in ownership_errors(trust, mutated)))

    def test_the_trust_lifecycle_must_run_its_own_lane(self) -> None:
        trust, fledge = load()
        mutated = copy.deepcopy(trust)
        mutated["lifecycle"]["command"] = ["fledge", "lanes", "run", "verify"]
        self.assertTrue(any("lifecycle command" in error for error in ownership_errors(mutated, fledge)))

    def test_dropping_the_tests_from_verify_fails(self) -> None:
        trust, fledge = load()
        mutated = copy.deepcopy(fledge)
        mutated["lanes"]["verify"]["steps"].remove("test")
        self.assertTrue(any("no longer runs test" in error for error in ownership_errors(trust, mutated)))

    def test_the_guide_is_published_on_the_site(self) -> None:
        self.assertTrue(SITE_GUIDE.is_file(), f"{SITE_GUIDE.relative_to(ROOT)} is missing")
        text = SITE_GUIDE.read_text(encoding="utf-8")
        self.assertTrue(text.startswith("---\ntitle: "), "the site page has front matter")
        self.assertIn("trust-lifecycle", text)
        pointer = DOCS_POINTER.read_text(encoding="utf-8")
        self.assertIn("site/src/content/docs/ci-confidence.md", pointer)


if __name__ == "__main__":
    unittest.main()
