#!/usr/bin/env python3
"""Regression tests for the required CI gate in .github/workflows/ci.yml (#796).

`Required CI gate` is the only required check on main. It passes when
`implementation-gate` ("SpecSync implementation ready") passes, and that job
reads the results of the jobs it needs. GitHub reports `skipped` for a job
classify deselected and for a job whose own dependency failed, so the gate has
to tell the two apart. Before #796 it could not: the lifecycle gate failed,
test, audit, coverage and spec-check were skipped, and both gates went green.

These tests hold the workflow to three rules:

1. Every job that can finish before implementation-gate is in its `needs`,
   `preflight` and `lifecycle-gate` included.
2. The gate's row for each job evaluates that job's own `if:` again, so
   `skipped` passes only where classify deselected the job.
3. Simulated over every classify path, the required gate is green when every
   selected job succeeds, and red when any one of them fails or is cancelled.

The workflow is parsed with Ruby's standard-library Psych, like the other
workflow validators here, and the gate's own shell steps are executed under
GitHub's default bash invocation. `--truth-table` prints the per-path table.
"""

from __future__ import annotations

import copy
from concurrent.futures import ThreadPoolExecutor
import functools
import itertools
import json
import math
import os
from pathlib import Path
import re
import shlex
import subprocess
import sys
import tempfile
import unittest
from dataclasses import dataclass, field
from typing import Any, Callable


sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github/workflows/ci.yml"
GATE_JOB = "implementation-gate"
REQUIRED_JOB = "ci-gate"
ALWAYS_REQUIRED = ("classify", "preflight", "lifecycle-gate")
CLASSIFY_FLAGS = (
    "full",
    "site",
    "vscode",
    "archive_only",
    "legacy_archive_only",
    "review_only",
    "review_required",
)
STATUS_FUNCTIONS = frozenset({"always", "success", "failure", "cancelled"})


# MARK: - Workflow loading


@functools.lru_cache(maxsize=None)
def load_workflow(path: Path = WORKFLOW) -> dict[str, Any]:
    """Parse a workflow through Ruby's standard-library Psych parser (once; never mutate)."""
    ruby = r"""
require "json"
require "psych"

document = Psych.safe_load(File.read(ARGV.fetch(0), encoding: "UTF-8"), permitted_classes: [], aliases: false)
raise "workflow is not a mapping" unless document.is_a?(Hash)
raise "workflow has no jobs mapping" unless document["jobs"].is_a?(Hash)
puts JSON.generate({ "env" => document["env"] || {}, "jobs" => document["jobs"] })
"""
    try:
        result = subprocess.run(
            ["ruby", "-e", ruby, str(path)],
            capture_output=True,
            check=False,
            text=True,
            timeout=30,
        )
    except FileNotFoundError as error:
        raise RuntimeError("Ruby with standard-library Psych is required") from error
    if result.returncode != 0:
        raise RuntimeError(f"{path}: workflow parsing failed: {result.stderr.strip()[-2000:]}")
    document = json.loads(result.stdout)
    jobs = {name: normalize_job(name, job) for name, job in document["jobs"].items()}
    return {"env": document["env"], "jobs": jobs}


def normalize_job(name: str, job: Any) -> dict[str, Any]:
    """Return one job with list `needs`, optional string `if`, and a step list."""
    if not isinstance(job, dict):
        raise RuntimeError(f"job {name} is not a mapping")
    needs = job.get("needs", [])
    if isinstance(needs, str):
        needs = [needs]
    condition = job.get("if")
    if isinstance(condition, bool):
        condition = "true" if condition else "false"
    return {
        "needs": list(needs),
        "if": condition,
        "outputs": job.get("outputs") or {},
        "env": job.get("env") or {},
        "steps": job.get("steps") or [],
    }


def ancestors(jobs: dict[str, Any], name: str) -> set[str]:
    """Every job `name` needs, directly or transitively."""
    found: set[str] = set()
    pending = list(jobs[name]["needs"])
    while pending:
        current = pending.pop()
        if current in found:
            continue
        if current not in jobs:
            raise RuntimeError(f"{name} needs unknown job {current}")
        found.add(current)
        pending.extend(jobs[current]["needs"])
    return found


def topological_order(jobs: dict[str, Any]) -> list[str]:
    """Jobs in an order where each follows everything it needs."""
    order: list[str] = []
    placed: set[str] = set()
    remaining = dict(jobs)
    while remaining:
        ready = sorted(name for name, job in remaining.items() if set(job["needs"]) <= placed)
        if not ready:
            raise RuntimeError(f"workflow needs contain a cycle among {sorted(remaining)}")
        for name in ready:
            order.append(name)
            placed.add(name)
            del remaining[name]
    return order


# MARK: - GitHub expressions


TOKEN = re.compile(
    r"""\s*(?:
        (?P<string>'(?:[^']|'')*')
      | (?P<number>\d+(?:\.\d+)?)
      | (?P<op>==|!=|&&|\|\||!|\(|\)|,|\.)
      | (?P<star>\*)
      | (?P<name>[A-Za-z_][A-Za-z0-9_-]*)
    )""",
    re.VERBOSE,
)


def tokenize(source: str) -> list[tuple[str, str]]:
    """Split one expression into (kind, text) tokens, failing on anything unsupported."""
    tokens: list[tuple[str, str]] = []
    position = 0
    source = source.rstrip()
    while position < len(source):
        match = TOKEN.match(source, position)
        if match is None or match.end() == position:
            raise ValueError(f"unsupported expression syntax at {source[position:]!r} in {source!r}")
        kind = match.lastgroup or ""
        tokens.append((kind, match.group(kind)))
        position = match.end()
    return tokens


def called_functions(source: str) -> set[str]:
    """Names called as functions anywhere in an expression."""
    tokens = tokenize(source)
    return {
        text.lower()
        for (kind, text), (_, following) in zip(tokens, tokens[1:] + [("", "")])
        if kind == "name" and following == "("
    }


def to_number(value: Any) -> float:
    if value is None:
        return 0.0
    if isinstance(value, bool):
        return 1.0 if value else 0.0
    if isinstance(value, float):
        return value
    if isinstance(value, str):
        stripped = value.strip()
        if not stripped:
            return 0.0
        try:
            return float(stripped)
        except ValueError:
            return math.nan
    return math.nan


def truthy(value: Any) -> bool:
    if value is None or value is False:
        return False
    if isinstance(value, float):
        return value != 0 and not math.isnan(value)
    if isinstance(value, str):
        return value != ""
    return True


def loose_equal(left: Any, right: Any) -> bool:
    """GitHub's `==`: strings compare case-insensitively, mixed types as numbers."""
    if isinstance(left, str) and isinstance(right, str):
        return left.casefold() == right.casefold()
    if type(left) is type(right) and not isinstance(left, (list, dict)):
        return left == right
    return to_number(left) == to_number(right)


def to_text(value: Any) -> str:
    """How GitHub renders an expression value inside a string."""
    if value is None:
        return ""
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, float):
        return str(int(value)) if value.is_integer() else str(value)
    if isinstance(value, str):
        return value
    return json.dumps(value)


def lookup(container: Any, key: str) -> Any:
    if isinstance(container, list):
        return [lookup(item, key) for item in container]
    if not isinstance(container, dict):
        return None
    if key in container:
        return container[key]
    folded = {str(name).casefold(): value for name, value in container.items()}
    return folded.get(key.casefold())


@dataclass
class Evaluator:
    """Recursive-descent evaluator for the expression subset ci.yml uses."""

    context: dict[str, Any]
    functions: dict[str, Callable[..., Any]]
    tokens: list[tuple[str, str]] = field(default_factory=list)
    index: int = 0

    def evaluate(self, source: str) -> Any:
        self.tokens = tokenize(source)
        self.index = 0
        value = self.either()
        if self.index != len(self.tokens):
            raise ValueError(f"unexpected {self.tokens[self.index][1]!r} in {source!r}")
        return value

    def peek(self) -> str:
        return self.tokens[self.index][1] if self.index < len(self.tokens) else ""

    def take(self, expected: str | None = None) -> tuple[str, str]:
        if self.index >= len(self.tokens):
            raise ValueError("expression ended early")
        token = self.tokens[self.index]
        if expected is not None and token[1] != expected:
            raise ValueError(f"expected {expected!r}, found {token[1]!r}")
        self.index += 1
        return token

    def either(self) -> Any:
        value = self.both()
        while self.peek() == "||":
            self.take()
            right = self.both()
            value = value if truthy(value) else right
        return value

    def both(self) -> Any:
        value = self.comparison()
        while self.peek() == "&&":
            self.take()
            right = self.comparison()
            value = right if truthy(value) else value
        return value

    def comparison(self) -> Any:
        value = self.unary()
        while self.peek() in ("==", "!="):
            operator = self.take()[1]
            right = self.unary()
            equal = loose_equal(value, right)
            value = equal if operator == "==" else not equal
        return value

    def unary(self) -> Any:
        if self.peek() == "!":
            self.take()
            return not truthy(self.unary())
        return self.primary()

    def primary(self) -> Any:
        kind, text = self.take()
        if text == "(" and kind == "op":
            value = self.either()
            self.take(")")
            return value
        if kind == "string":
            return text[1:-1].replace("''", "'")
        if kind == "number":
            return float(text)
        if kind != "name":
            raise ValueError(f"unexpected {text!r}")
        lowered = text.lower()
        if lowered in ("true", "false"):
            return lowered == "true"
        if lowered == "null":
            return None
        if self.peek() == "(":
            self.take("(")
            arguments: list[Any] = []
            while self.peek() != ")":
                arguments.append(self.either())
                if self.peek() == ",":
                    self.take(",")
            self.take(")")
            function = self.functions.get(lowered)
            if function is None:
                raise ValueError(f"unsupported function {text}()")
            return function(*arguments)
        value = lookup(self.context, text)
        while self.peek() == ".":
            self.take(".")
            kind, key = self.take()
            if kind == "star":
                value = list(value.values()) if isinstance(value, dict) else None
            elif kind == "name":
                value = lookup(value, key)
            else:
                raise ValueError(f"unsupported property {key!r}")
        return value


def join(values: Any, separator: Any = ",") -> str:
    if isinstance(values, list):
        return str(separator).join(to_text(value) for value in values)
    return to_text(values)


def strip_wrapper(source: str) -> str:
    """Drop one `${{ ... }}` wrapping a whole condition."""
    text = source.strip()
    match = re.fullmatch(r"\$\{\{(.*)\}\}", text, re.DOTALL)
    return match.group(1).strip() if match else text


def interpolate(template: Any, evaluate: Callable[[str], Any]) -> str:
    """Replace every `${{ expression }}` in a string the way the runner does."""
    return re.sub(r"\$\{\{(.*?)\}\}", lambda match: to_text(evaluate(match.group(1))), str(template))


# MARK: - Job selection


def selection_expression(job: dict[str, Any]) -> str:
    """The job's own `if:` with any leading `always() &&` removed; `true` when unconditional."""
    condition = job["if"]
    if condition is None:
        return "true"
    text = " ".join(strip_wrapper(condition).split())
    text = re.sub(r"^always\(\)\s*(&&\s*|$)", "", text)
    return text or "true"


def gate_rows(jobs: dict[str, Any]) -> tuple[dict[str, tuple[str, str]], list[str]]:
    """Parse the gate's GATES rows into {job: (selection, result)} plus parse errors."""
    errors: list[str] = []
    rows: dict[str, tuple[str, str]] = {}
    tables = [
        step["env"]["GATES"]
        for step in jobs[GATE_JOB]["steps"]
        if isinstance(step.get("env"), dict) and "GATES" in step["env"]
    ]
    if len(tables) != 1:
        return rows, [f"{GATE_JOB} must have exactly one step with a GATES table, found {len(tables)}"]
    row_pattern = re.compile(
        r"(?P<job>[A-Za-z_][A-Za-z0-9_-]*) "
        r"(?P<selected>true|\$\{\{ .+? \}\}) "
        r"(?P<result>\$\{\{ needs\.[A-Za-z0-9_-]+\.result \}\})"
    )
    for line in str(tables[0]).splitlines():
        if not line.strip():
            continue
        match = row_pattern.fullmatch(line.strip())
        if match is None:
            errors.append(f"{GATE_JOB} GATES row is malformed: {line!r}")
            continue
        job = match.group("job")
        if job in rows:
            errors.append(f"{GATE_JOB} GATES lists {job} twice")
        rows[job] = (match.group("selected"), match.group("result"))
    return rows, errors


def gate_contract_errors(jobs: dict[str, Any]) -> list[str]:
    """Structural rules for implementation-gate; empty when the workflow holds to them."""
    errors: list[str] = []
    for name in (GATE_JOB, REQUIRED_JOB):
        if name not in jobs:
            return [f"workflow has no {name} job"]
    gate = jobs[GATE_JOB]
    needs = set(gate["needs"])
    if GATE_JOB not in ancestors(jobs, REQUIRED_JOB):
        errors.append(f"{REQUIRED_JOB} must need {GATE_JOB}")
    if selection_expression(gate) != "true" or "always" not in called_functions(strip_wrapper(gate["if"] or "")):
        errors.append(f"{GATE_JOB} must run with if: always() so a failure upstream cannot skip it")

    for name in ALWAYS_REQUIRED:
        if name not in needs:
            errors.append(f"{GATE_JOB}.needs must include {name}")
    downstream = {name for name in jobs if GATE_JOB in ancestors(jobs, name)} | {GATE_JOB}
    for name in sorted(set(jobs) - downstream):
        if name in needs or name in ALWAYS_REQUIRED:
            continue
        reason = (
            "gates on lifecycle-gate"
            if "lifecycle-gate" in ancestors(jobs, name)
            else "can finish before the gate"
        )
        errors.append(f"{name} {reason} but is missing from {GATE_JOB}.needs")
    for name in sorted(needs & downstream):
        errors.append(f"{GATE_JOB}.needs contains {name}, which runs after the gate")

    rows, row_errors = gate_rows(jobs)
    errors.extend(row_errors)
    if row_errors and not rows:
        return errors
    for name in sorted(needs - set(rows)):
        errors.append(f"{name} is in {GATE_JOB}.needs but has no GATES row, so the gate ignores it")
    for name in sorted(set(rows) - needs):
        errors.append(f"{GATE_JOB} GATES has a row for {name}, which is not in its needs")
    for name in sorted(needs & set(rows)):
        selected, result = rows[name]
        if result != f"${{{{ needs.{name}.result }}}}":
            errors.append(f"{name} GATES row reads {result}, not its own result")
        expected = selection_expression(jobs[name])
        leftover = called_functions(expected) & STATUS_FUNCTIONS
        if leftover:
            errors.append(
                f"{name} if: calls {sorted(leftover)}; the gate cannot tell when it is selected"
            )
            continue
        actual = "true" if selected == "true" else " ".join(strip_wrapper(selected).split())
        if actual != expected:
            errors.append(
                f"{name} GATES row selects on {actual!r} but the job runs on {expected!r}"
            )
    return errors


# MARK: - Simulation


@dataclass(frozen=True)
class Scenario:
    """One classify outcome for one event."""

    label: str
    event: str
    flags: tuple[str, ...] = ()
    ref: str = "refs/pull/1/merge"

    def outputs(self, jobs: dict[str, Any]) -> dict[str, str]:
        keys = jobs["classify"]["outputs"].keys()
        values = {key: ("false" if key in CLASSIFY_FLAGS else "") for key in keys}
        for flag in self.flags:
            if flag not in values:
                raise KeyError(f"classify has no output {flag}")
            values[flag] = "true"
        return values


# Every lane classify-ci-paths.sh and select-ci-lane.sh can choose today.
SCENARIOS = (
    Scenario("Full PR (src/, tests/, workflows, docs/, *.md)", "pull_request", ("full",)),
    Scenario("Full PR, change awaiting scoped review", "pull_request", ("full", "review_required")),
    Scenario("Site-only PR (site/**)", "pull_request", ("site",)),
    Scenario("VS Code-only PR (vscode-extension/**)", "pull_request", ("vscode",)),
    Scenario("Site and VS Code PR", "pull_request", ("site", "vscode")),
    Scenario("Specs/lifecycle-only PR (specs/**, .specsync/changes/**)", "pull_request"),
    Scenario("Specs/lifecycle-only PR, awaiting scoped review", "pull_request", ("review_required",)),
    Scenario("Archive-only PR (workflow-v2 archive move)", "pull_request", ("archive_only",)),
    Scenario(
        "Legacy archive-only PR (workflow-v1)",
        "pull_request",
        ("legacy_archive_only", "full"),
    ),
    Scenario("Review-only PR (review.json + review-attempts.json)", "pull_request", ("review_only",)),
    Scenario("Push to main (full)", "push", ("full",), "refs/heads/main"),
    Scenario("Push to main, verifying change present", "push", ("full", "review_required"), "refs/heads/main"),
    Scenario("Push to main (site-only)", "push", ("site",), "refs/heads/main"),
    Scenario("Push to main (specs/lifecycle-only)", "push", (), "refs/heads/main"),
    Scenario("workflow_dispatch (forced full)", "workflow_dispatch", ("full",), "refs/heads/main"),
)


def expression_context(
    jobs: dict[str, Any],
    name: str,
    results: dict[str, str],
    scenario: Scenario,
    outputs: dict[str, str],
) -> dict[str, Any]:
    needs = {}
    for dependency in jobs[name]["needs"]:
        needs[dependency] = {
            "result": results[dependency],
            "outputs": outputs if dependency == "classify" and results[dependency] == "success" else {},
        }
    return {
        "needs": needs,
        "github": {"event_name": scenario.event, "ref": scenario.ref},
    }


def job_functions(jobs: dict[str, Any], name: str, results: dict[str, str]) -> dict[str, Callable[..., Any]]:
    # Job-level status functions look at every transitive dependency, so a job
    # whose grandparent was skipped is skipped too. That is observable here:
    # `attest` is skipped on every push to main, where `corvid-pet`, upstream of
    # `ci-gate`, is skipped.
    upstream = [results[job] for job in ancestors(jobs, name)]
    return {
        "always": lambda: True,
        "success": lambda: all(result == "success" for result in upstream),
        "failure": lambda: any(result == "failure" for result in upstream),
        "cancelled": lambda: False,
        "join": join,
    }


def job_runs(
    jobs: dict[str, Any],
    name: str,
    results: dict[str, str],
    scenario: Scenario,
    outputs: dict[str, str],
) -> bool:
    functions = job_functions(jobs, name, results)
    evaluator = Evaluator(expression_context(jobs, name, results, scenario, outputs), functions)
    condition = jobs[name]["if"]
    source = "true" if condition is None else strip_wrapper(condition)
    decision = truthy(evaluator.evaluate(source))
    if not called_functions(source) & STATUS_FUNCTIONS:
        decision = decision and functions["success"]()
    return decision


def run_gate_steps(
    workflow: dict[str, Any],
    name: str,
    results: dict[str, str],
    scenario: Scenario,
    outputs: dict[str, str],
) -> tuple[str, str]:
    """Execute a gate job's `run` steps; return (result, combined output)."""
    jobs = workflow["jobs"]
    evaluator = Evaluator(
        expression_context(jobs, name, results, scenario, outputs),
        job_functions(jobs, name, results),
    )
    transcript: list[str] = []
    for step in jobs[name]["steps"]:
        if "run" not in step:
            continue
        if "if" in step:
            raise NotImplementedError(f"{name} step conditions are not simulated")
        environment = {"PATH": os.environ.get("PATH", "/usr/bin:/bin")}
        for scope in (workflow["env"], jobs[name]["env"], step.get("env") or {}):
            for key, value in scope.items():
                environment[key] = interpolate(value, evaluator.evaluate)
        script = interpolate(step["run"], evaluator.evaluate)
        returncode, output = run_step(
            step.get("shell", "bash"), script, tuple(sorted(environment.items()))
        )
        transcript.append(output)
        if returncode != 0:
            return "failure", "".join(transcript)
    return "success", "".join(transcript)


@functools.lru_cache(maxsize=None)
def run_step(shell: str, script: str, environment: tuple[tuple[str, str], ...]) -> tuple[int, str]:
    """Run one step script under GitHub's invocation for its shell; same input, same answer."""
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "step.sh"
        path.write_text(script, encoding="utf-8")
        if shell == "bash":
            command = ["bash", "--noprofile", "--norc", "-eo", "pipefail", str(path)]
        elif "{0}" in shell:
            command = shlex.split(shell.replace("{0}", str(path)))
        else:
            raise NotImplementedError(f"unsupported shell {shell!r}")
        completed = subprocess.run(
            command,
            cwd=ROOT,
            env=dict(environment),
            capture_output=True,
            text=True,
            timeout=30,
            check=False,
        )
    return completed.returncode, completed.stdout + completed.stderr


def simulate(
    workflow: dict[str, Any],
    scenario: Scenario,
    forced: dict[str, str] | None = None,
) -> tuple[dict[str, str], str]:
    """Every job's result for one scenario; `forced` sets a job's result if it runs."""
    jobs = workflow["jobs"]
    forced = forced or {}
    outputs = scenario.outputs(jobs)
    results: dict[str, str] = {}
    transcript = ""
    for name in topological_order(jobs):
        if not job_runs(jobs, name, results, scenario, outputs):
            results[name] = "skipped"
        elif name in forced:
            results[name] = forced[name]
        elif name in (GATE_JOB, REQUIRED_JOB):
            results[name], output = run_gate_steps(workflow, name, results, scenario, outputs)
            transcript += output
        else:
            results[name] = "success"
    return results, transcript


def simulate_many(
    workflow: dict[str, Any],
    cases: list[tuple[Scenario, dict[str, str]]],
) -> list[tuple[dict[str, str], str]]:
    """`simulate` for many independent cases, in parallel (each spawns real shells)."""
    with ThreadPoolExecutor(max_workers=max(4, os.cpu_count() or 4)) as pool:
        return list(pool.map(lambda case: simulate(workflow, case[0], case[1]), cases))


def product_jobs(workflow: dict[str, Any]) -> list[str]:
    """Jobs that finish before the gate, in workflow order."""
    jobs = workflow["jobs"]
    downstream = {name for name in jobs if GATE_JOB in ancestors(jobs, name)} | {GATE_JOB}
    return [name for name in jobs if name not in downstream]


# The gate as it stood when #796 was found, kept to prove the harness sees the bug.
PRE_796_GATE_NEEDS = [
    "classify", "test", "fmt", "hi-check", "validate-action", "action-consumer",
    "spec-check", "audit", "coverage", "site", "vscode-extension", "corvid-pet",
]
PRE_796_GATE_STEP = {
    "name": "Require every selected gate",
    "env": {"RESULTS": "${{ join(needs.*.result, ' ') }}"},
    "run": (
        'for result in $RESULTS; do\n'
        '  case "$result" in\n'
        '    success|skipped) ;;\n'
        '    *) echo "Selected CI gate ended with: $result" >&2; exit 1 ;;\n'
        '  esac\n'
        'done\n'
    ),
}


def pre_796_workflow(workflow: dict[str, Any]) -> dict[str, Any]:
    before = copy.deepcopy(workflow)
    before["jobs"][GATE_JOB]["needs"] = list(PRE_796_GATE_NEEDS)
    before["jobs"][GATE_JOB]["steps"] = [copy.deepcopy(PRE_796_GATE_STEP)]
    return before


# MARK: - Tests


class RequiredGateContractTests(unittest.TestCase):
    """The workflow on disk holds to the gate's structural rules."""

    workflow: dict[str, Any]

    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = load_workflow()

    def test_workflow_holds_to_the_gate_contract(self) -> None:
        self.assertEqual(gate_contract_errors(self.workflow["jobs"]), [])

    def test_gate_needs_preflight_and_the_lifecycle_gate(self) -> None:
        needs = self.workflow["jobs"][GATE_JOB]["needs"]
        for name in ALWAYS_REQUIRED:
            with self.subTest(job=name):
                self.assertIn(name, needs)

    def test_every_job_gated_on_the_lifecycle_gate_is_required(self) -> None:
        jobs = self.workflow["jobs"]
        needs = set(jobs[GATE_JOB]["needs"])
        gated = [
            name
            for name in product_jobs(self.workflow)
            if "lifecycle-gate" in ancestors(jobs, name)
        ]
        self.assertTrue({"test", "audit", "coverage", "spec-check"} <= set(gated))
        for name in gated:
            with self.subTest(job=name):
                self.assertIn(name, needs)

    def test_required_gate_depends_on_the_implementation_gate(self) -> None:
        self.assertIn(GATE_JOB, self.workflow["jobs"][REQUIRED_JOB]["needs"])

    def test_these_tests_run_through_fledge_and_ci(self) -> None:
        script = ".github/scripts/test-required-ci-gate.py"
        fledge = (ROOT / "fledge.toml").read_text(encoding="utf-8")
        task = re.search(
            r'(?ms)^\[tasks\.([A-Za-z0-9_-]+)\]\ncmd\s*=\s*"[^"]*' + re.escape(script) + r'[^"]*"\s*$',
            fledge,
        )
        self.assertIsNotNone(task, "Fledge must define a task that runs the required-gate tests")
        if task is None:
            return
        verify = re.search(r'(?ms)^\[lanes\.verify\]\n(.*?)^\]', fledge)
        self.assertIsNotNone(verify)
        if verify is not None:
            self.assertIn(f'"{task.group(1)}"', verify.group(1))
        commands = [
            str(step.get("run", ""))
            for job in self.workflow["jobs"].values()
            for step in job["steps"]
        ]
        self.assertTrue(
            any(script in command for command in commands),
            "CI must run the required-gate tests",
        )


class GuardMutationTests(unittest.TestCase):
    """The structural guard fails on each way the gate can drift."""

    workflow: dict[str, Any]

    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = load_workflow()

    def mutated(self) -> dict[str, Any]:
        return copy.deepcopy(self.workflow["jobs"])

    def assert_guard_reports(self, jobs: dict[str, Any], fragment: str) -> None:
        errors = gate_contract_errors(jobs)
        self.assertTrue(
            any(fragment in error for error in errors),
            f"expected an error containing {fragment!r}, got {errors}",
        )

    def test_dropping_the_lifecycle_gate_from_needs_fails(self) -> None:
        jobs = self.mutated()
        jobs[GATE_JOB]["needs"].remove("lifecycle-gate")
        self.assert_guard_reports(jobs, f"{GATE_JOB}.needs must include lifecycle-gate")

    def test_dropping_preflight_from_needs_fails(self) -> None:
        jobs = self.mutated()
        jobs[GATE_JOB]["needs"].remove("preflight")
        self.assert_guard_reports(jobs, f"{GATE_JOB}.needs must include preflight")

    def test_a_new_job_gated_on_the_lifecycle_gate_must_be_required(self) -> None:
        jobs = self.mutated()
        jobs["new-product-check"] = normalize_job(
            "new-product-check",
            {
                "needs": ["classify", "lifecycle-gate"],
                "if": "needs.classify.outputs.full == 'true'",
                "steps": [{"run": "true"}],
            },
        )
        self.assert_guard_reports(jobs, "new-product-check gates on lifecycle-gate but is missing")

    def test_a_new_selected_job_must_be_required(self) -> None:
        jobs = self.mutated()
        jobs["new-site-check"] = normalize_job(
            "new-site-check",
            {"needs": "classify", "if": "needs.classify.outputs.site == 'true'", "steps": []},
        )
        self.assert_guard_reports(jobs, "new-site-check can finish before the gate but is missing")

    def test_a_needed_job_without_a_row_fails(self) -> None:
        jobs = self.mutated()
        jobs["new-site-check"] = normalize_job("new-site-check", {"needs": "classify", "steps": []})
        jobs[GATE_JOB]["needs"].append("new-site-check")
        self.assert_guard_reports(jobs, "new-site-check is in implementation-gate.needs but has no GATES row")

    def test_a_row_that_no_longer_matches_its_job_condition_fails(self) -> None:
        jobs = self.mutated()
        jobs["test"]["if"] = "needs.classify.outputs.site == 'true'"
        self.assert_guard_reports(jobs, "test GATES row selects on")

    def test_a_condition_the_gate_cannot_mirror_fails(self) -> None:
        jobs = self.mutated()
        jobs["test"]["if"] = "failure() && needs.classify.outputs.full == 'true'"
        self.assert_guard_reports(jobs, "test if: calls ['failure']")


class RequiredGateSimulationTests(unittest.TestCase):
    """Simulated over every classify path, the required gate matches reality."""

    workflow: dict[str, Any]

    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = load_workflow()

    def test_issue_796_lifecycle_gate_failure_turns_the_required_gate_red(self) -> None:
        scenario = SCENARIOS[0]
        results, transcript = simulate(self.workflow, scenario, {"lifecycle-gate": "failure"})
        for name in ("test", "audit", "coverage", "spec-check"):
            self.assertEqual(results[name], "skipped", name)
        self.assertEqual(results[GATE_JOB], "failure")
        self.assertEqual(results[REQUIRED_JOB], "failure")
        self.assertIn("::error::lifecycle-gate was selected and ended with: failure", transcript)
        self.assertIn("::error::test was selected but skipped", transcript)

    def test_the_pre_796_gate_reproduces_the_bug(self) -> None:
        results, _ = simulate(
            pre_796_workflow(self.workflow), SCENARIOS[0], {"lifecycle-gate": "failure"}
        )
        self.assertEqual(results["lifecycle-gate"], "failure")
        self.assertEqual(results[GATE_JOB], "success")
        self.assertEqual(results[REQUIRED_JOB], "success")

    def test_every_path_is_green_when_every_selected_job_succeeds(self) -> None:
        for scenario in SCENARIOS:
            with self.subTest(path=scenario.label):
                results, transcript = simulate(self.workflow, scenario)
                self.assertEqual(results[GATE_JOB], "success", transcript)
                self.assertEqual(results[REQUIRED_JOB], "success", transcript)

    def test_every_classify_output_combination_is_green_when_everything_succeeds(self) -> None:
        referenced = ("full", "site", "vscode", "archive_only", "review_only", "review_required")
        scenarios = [
            Scenario(f"{event} {flags}", event, flags, ref)
            for event, ref in (
                ("pull_request", "refs/pull/1/merge"),
                ("push", "refs/heads/main"),
                ("workflow_dispatch", "refs/heads/main"),
            )
            for bits in itertools.product((False, True), repeat=len(referenced))
            for flags in [tuple(flag for flag, bit in zip(referenced, bits) if bit)]
        ]
        outcomes = simulate_many(self.workflow, [(scenario, {}) for scenario in scenarios])
        for scenario, (results, transcript) in zip(scenarios, outcomes):
            with self.subTest(path=scenario.label):
                self.assertEqual(results[REQUIRED_JOB], "success", transcript)

    def test_any_selected_job_failing_or_cancelled_turns_the_required_gate_red(self) -> None:
        cases: list[tuple[Scenario, dict[str, str]]] = []
        for scenario, (green, _) in zip(
            SCENARIOS, simulate_many(self.workflow, [(scenario, {}) for scenario in SCENARIOS])
        ):
            ran = [name for name in product_jobs(self.workflow) if green[name] != "skipped"]
            self.assertTrue(set(ALWAYS_REQUIRED) <= set(ran), scenario.label)
            cases.extend(
                (scenario, {name: outcome})
                for name, outcome in itertools.product(ran, ("failure", "cancelled"))
            )
        for (scenario, forced), (results, transcript) in zip(cases, simulate_many(self.workflow, cases)):
            with self.subTest(path=scenario.label, forced=forced):
                self.assertEqual(results[GATE_JOB], "failure", transcript)
                self.assertEqual(results[REQUIRED_JOB], "failure", transcript)

    def test_deselected_jobs_are_skipped_and_do_not_block(self) -> None:
        expectations = {
            "Archive-only PR (workflow-v2 archive move)": {
                "test", "fmt", "hi-check", "validate-action", "action-consumer",
                "spec-check", "audit", "coverage", "site", "vscode-extension", "corvid-pet",
            },
            "Review-only PR (review.json + review-attempts.json)": {
                "test", "fmt", "hi-check", "validate-action", "action-consumer",
                "spec-check", "audit", "coverage", "site", "vscode-extension", "corvid-pet",
            },
            "Site-only PR (site/**)": {
                "test", "fmt", "hi-check", "action-consumer", "audit", "coverage",
                "vscode-extension", "corvid-pet",
            },
        }
        by_label = {scenario.label: scenario for scenario in SCENARIOS}
        for label, skipped in expectations.items():
            with self.subTest(path=label):
                results, _ = simulate(self.workflow, by_label[label])
                self.assertEqual(
                    {name for name in product_jobs(self.workflow) if results[name] == "skipped"},
                    skipped,
                )
                for name in ALWAYS_REQUIRED:
                    self.assertEqual(results[name], "success")
                self.assertEqual(results[REQUIRED_JOB], "success")


class GateScriptTests(unittest.TestCase):
    """The gate's own step fails closed on inputs a consistent run never produces."""

    def run_gate(self, gates: str, needs_results: str) -> tuple[int, str]:
        steps = [step for step in load_workflow()["jobs"][GATE_JOB]["steps"] if "run" in step]
        self.assertEqual(len(steps), 1)
        environment = (
            ("GATES", gates),
            ("NEEDS_RESULTS", needs_results),
            ("PATH", os.environ.get("PATH", "/usr/bin:/bin")),
        )
        return run_step(steps[0].get("shell", "bash"), steps[0]["run"], environment)

    def test_consistent_rows_pass(self) -> None:
        code, output = self.run_gate("lifecycle-gate true success\nsite false skipped\n", "success skipped")
        self.assertEqual(code, 0, output)

    def test_a_selected_job_that_was_skipped_fails(self) -> None:
        code, output = self.run_gate("lifecycle-gate true failure\ntest true skipped\n", "failure skipped")
        self.assertNotEqual(code, 0)
        self.assertIn("::error::test was selected but skipped", output)

    def test_a_deselected_job_that_ran_fails(self) -> None:
        code, output = self.run_gate("site false success\n", "success")
        self.assertNotEqual(code, 0)
        self.assertIn("its row no longer matches its if:", output)

    def test_a_malformed_selection_fails(self) -> None:
        code, output = self.run_gate("site  skipped\n", "skipped")
        self.assertNotEqual(code, 0)
        self.assertIn("has no usable selection", output)

    def test_rows_that_do_not_cover_needs_fail(self) -> None:
        code, output = self.run_gate("site false skipped\n", "skipped skipped")
        self.assertNotEqual(code, 0)
        self.assertIn("1 rows for 2 jobs in needs", output)

    def test_a_failure_in_needs_fails_whatever_the_rows_say(self) -> None:
        code, output = self.run_gate("site false skipped\n", "failure")
        self.assertNotEqual(code, 0)
        self.assertIn("a job this gate needs ended with: failure", output)


class ExpressionTests(unittest.TestCase):
    """The evaluator agrees with GitHub on the operators ci.yml uses."""

    def evaluate(self, source: str, **context: Any) -> Any:
        return Evaluator(context, {"always": lambda: True, "join": join}).evaluate(source)

    def test_string_equality_is_case_insensitive(self) -> None:
        self.assertIs(self.evaluate("'TRUE' == 'true'"), True)

    def test_logical_operators_return_operands(self) -> None:
        self.assertEqual(self.evaluate("'' || 'fallback'"), "fallback")
        self.assertEqual(self.evaluate("'a' && 'b'"), "b")

    def test_missing_outputs_compare_unequal_to_true(self) -> None:
        self.assertIs(self.evaluate("needs.classify.outputs.full == 'true'", needs={}), False)
        self.assertIs(self.evaluate("needs.classify.outputs.full != 'true'", needs={}), True)

    def test_join_over_a_wildcard(self) -> None:
        needs = {"a": {"result": "success"}, "b": {"result": "skipped"}}
        self.assertEqual(self.evaluate("join(needs.*.result, ' ')", needs=needs), "success skipped")

    def test_unsupported_syntax_fails_loudly(self) -> None:
        with self.assertRaises(ValueError):
            self.evaluate("contains(github.ref, 'main')")
        with self.assertRaises(ValueError):
            self.evaluate("1 < 2")


# MARK: - Truth table


def truth_table() -> str:
    """Markdown table of the required gate's verdict per classify path."""
    workflow = load_workflow()
    before = pre_796_workflow(workflow)
    lines = [
        "| Classify path | Event | Selected, must succeed | Deselected, `skipped` passes "
        "| All selected green | `lifecycle-gate` fails | `preflight` fails "
        "| Any one selected job fails or is cancelled | `lifecycle-gate` fails, before this fix |",
        "|---|---|---|---|---|---|---|---|---|",
    ]

    def verdict(results: dict[str, str]) -> str:
        return "green" if results[REQUIRED_JOB] == "success" else "**red**"

    for scenario in SCENARIOS:
        green, _ = simulate(workflow, scenario)
        ran = [name for name in product_jobs(workflow) if green[name] != "skipped"]
        skipped = [name for name in product_jobs(workflow) if green[name] == "skipped"]
        injected = [
            simulate(workflow, scenario, {name: outcome})[0][REQUIRED_JOB]
            for name, outcome in itertools.product(ran, ("failure", "cancelled"))
        ]
        red = sum(result != "success" for result in injected)
        lines.append(
            "| {label} | `{event}` | {ran} | {skipped} | {green} | {lifecycle} | {preflight} "
            "| **red** in {red}/{total} | {before} |".format(
                label=scenario.label,
                event=scenario.event,
                ran=", ".join(f"`{name}`" for name in ran),
                skipped=", ".join(f"`{name}`" for name in skipped) or "none",
                green=verdict(green),
                lifecycle=verdict(simulate(workflow, scenario, {"lifecycle-gate": "failure"})[0]),
                preflight=verdict(simulate(workflow, scenario, {"preflight": "failure"})[0]),
                red=red,
                total=len(injected),
                before=verdict(simulate(before, scenario, {"lifecycle-gate": "failure"})[0]),
            )
        )
    return "\n".join(lines)


if __name__ == "__main__":
    if sys.argv[1:] == ["--truth-table"]:
        print(truth_table())
        raise SystemExit(0)
    unittest.main()
