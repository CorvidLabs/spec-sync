#!/usr/bin/env python3
"""Validate deterministic runtime pins in maintained GitHub workflows."""

from pathlib import Path
import re
import sys

EXPECTED_BUN_VERSION = "1.3.14"
EXPECTED_SETUP_BUN_JOBS = {
    (".github/workflows/ci.yml", "site"),
    (".github/workflows/ci.yml", "vscode-extension"),
    (".github/workflows/pages.yml", "build"),
}


def yaml_scalar(raw: str) -> str:
    value = raw.split(" #", 1)[0].strip()
    if len(value) >= 2 and value[0] == value[-1] and value[0] in {"'", '"'}:
        return value[1:-1]
    return value


def mapping_block(lines: list[str], key: str, indent: int) -> list[str] | None:
    """Return one indentation-bounded YAML mapping block, failing closed on ambiguity."""
    pattern = re.compile(rf"^ {{{indent}}}{re.escape(key)}:\s*(?:#.*)?$")
    starts = [index for index, line in enumerate(lines) if pattern.match(line)]
    if len(starts) != 1:
        return None

    start = starts[0] + 1
    end = len(lines)
    for index in range(start, len(lines)):
        line = lines[index]
        stripped = line.lstrip()
        if not stripped or stripped.startswith("#"):
            continue
        leading = len(line) - len(stripped)
        if leading <= indent:
            end = index
            break
    return lines[start:end]


def workflow_uses_steps(path: str) -> list[tuple[str, str, list[str]]]:
    """Read workflow `uses` steps using only the Python standard library."""
    lines = Path(path).read_text(encoding="utf-8").splitlines()
    jobs_starts = [index for index, line in enumerate(lines) if re.match(r"^jobs:\s*(?:#.*)?$", line)]
    if len(jobs_starts) != 1:
        return []

    jobs = lines[jobs_starts[0] + 1 :]
    current_job: str | None = None
    steps: list[tuple[str, str, list[str]]] = []
    job_pattern = re.compile(r"^  ([A-Za-z0-9_-]+):\s*(?:#.*)?$")
    step_pattern = re.compile(r"^      -(?:\s+.*)?$")
    inline_uses_pattern = re.compile(r"^      - uses:\s*(.*?)\s*$")
    nested_uses_pattern = re.compile(r"^        uses:\s*(.*?)\s*$")
    index = 0
    while index < len(jobs):
        line = jobs[index]
        if line and not line.startswith((" ", "#")):
            break
        if job_match := job_pattern.match(line):
            current_job = job_match.group(1)
        if step_pattern.match(line):
            end = index + 1
            while end < len(jobs):
                candidate = jobs[end]
                stripped = candidate.lstrip()
                if stripped and not stripped.startswith("#"):
                    leading = len(candidate) - len(stripped)
                    if leading <= 6:
                        break
                end += 1
            step_lines = jobs[index:end]
            uses_values = [
                yaml_scalar(match.group(1))
                for step_line in step_lines
                if (match := inline_uses_pattern.match(step_line))
                or (match := nested_uses_pattern.match(step_line))
            ]
            if current_job is not None and len(uses_values) == 1:
                steps.append((current_job, uses_values[0], step_lines))
            index = end
            continue
        index += 1
    return steps


def step_input(lines: list[str], key: str) -> str | None:
    with_block = mapping_block(lines, "with", 8)
    pattern = re.compile(rf"^          {re.escape(key)}:\s*(.*?)\s*$")
    values = [
        yaml_scalar(match.group(1))
        for line in with_block or []
        if (match := pattern.match(line))
    ]
    return values[0] if len(values) == 1 else None


def main() -> int:
    found: set[tuple[str, str]] = set()
    errors: list[str] = []

    for workflow_path in sorted({path for path, _ in EXPECTED_SETUP_BUN_JOBS}):
        for job_name, uses, step in workflow_uses_steps(workflow_path):
            if uses != "oven-sh/setup-bun@v2":
                continue
            location = (workflow_path, job_name)
            found.add(location)
            version = step_input(step, "bun-version")
            if version != EXPECTED_BUN_VERSION:
                errors.append(
                    f"{workflow_path}:{job_name} must pin bun-version "
                    f"{EXPECTED_BUN_VERSION}, found {version!r}"
                )

    missing = EXPECTED_SETUP_BUN_JOBS - found
    unexpected = found - EXPECTED_SETUP_BUN_JOBS
    errors.extend(f"missing setup-bun step in {path}:{job}" for path, job in sorted(missing))
    errors.extend(f"unexpected setup-bun step in {path}:{job}" for path, job in sorted(unexpected))

    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1

    print(f"Validated Bun {EXPECTED_BUN_VERSION} across {len(found)} workflow jobs")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
