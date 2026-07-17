#!/usr/bin/env python3
"""Validate deterministic runtime pins in maintained GitHub workflows."""

import json
import subprocess
import sys

EXPECTED_BUN_VERSION = "1.3.14"
EXPECTED_SETUP_BUN_REF = "oven-sh/setup-bun@v2"
EXPECTED_SETUP_BUN_REPOSITORY = "oven-sh/setup-bun"
EXPECTED_SETUP_BUN_JOBS = {
    (".github/workflows/ci.yml", "site"),
    (".github/workflows/ci.yml", "vscode-extension"),
    (".github/workflows/pages.yml", "build"),
}


def split_action_reference(uses: str) -> tuple[str, str] | None:
    """Return a case-normalized action repository and its case-sensitive ref."""
    repository, separator, ref = uses.rpartition("@")
    if separator != "@" or not repository or not ref:
        return None
    return repository.casefold(), ref


def workflow_uses_steps(
    path: str, errors: list[str]
) -> list[tuple[str, str, dict[str, str]]]:
    """Read every workflow Action step through Ruby's standard-library Psych parser."""
    ruby = r'''\
require "json"
require "psych"

document = Psych.safe_load(File.read(ARGV.fetch(0), encoding: "UTF-8"), permitted_classes: [], aliases: false)
steps = []
jobs = document.is_a?(Hash) ? document["jobs"] : nil
if jobs.is_a?(Hash)
  jobs.each do |job_name, job|
    next unless job.is_a?(Hash) && job["steps"].is_a?(Array)
    job["steps"].each do |step|
      next unless step.is_a?(Hash) && step["uses"].is_a?(String)
      inputs = step["with"].is_a?(Hash) ? step["with"] : {}
      normalized_inputs = inputs.each_with_object({}) do |(key, value), result|
        result[key.to_s] = value.to_s unless value.is_a?(Hash) || value.is_a?(Array)
      end
      steps << { job: job_name.to_s, uses: step["uses"], inputs: normalized_inputs }
    end
  end
end
puts JSON.generate(steps)
'''
    try:
        result = subprocess.run(
            ["ruby", "-e", ruby, path],
            capture_output=True,
            check=False,
            text=True,
            timeout=30,
        )
    except FileNotFoundError:
        errors.append(f"{path}: Ruby with standard-library Psych is required")
        return []
    except subprocess.TimeoutExpired:
        errors.append(f"{path}: workflow parsing timed out after 30 seconds")
        return []

    if result.returncode != 0:
        detail = result.stderr.strip()[-2000:] or "unknown Psych parser error"
        errors.append(f"{path}: workflow parsing failed: {detail}")
        return []

    try:
        payload = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        errors.append(f"{path}: workflow parser returned invalid JSON: {error}")
        return []
    return [(step["job"], step["uses"], step["inputs"]) for step in payload]


def step_input(inputs: dict[str, str], key: str) -> str | None:
    return inputs.get(key)


def main() -> int:
    found_counts: dict[tuple[str, str], int] = {}
    errors: list[str] = []

    for workflow_path in sorted({path for path, _ in EXPECTED_SETUP_BUN_JOBS}):
        for job_name, uses, step in workflow_uses_steps(workflow_path, errors):
            reference = split_action_reference(uses)
            if reference is None or reference[0] != EXPECTED_SETUP_BUN_REPOSITORY:
                continue
            location = (workflow_path, job_name)
            found_counts[location] = found_counts.get(location, 0) + 1
            if reference[1] != "v2":
                errors.append(
                    f"{workflow_path}:{job_name} must use {EXPECTED_SETUP_BUN_REF}, "
                    f"found {uses!r}"
                )
            version = step_input(step, "bun-version")
            if version != EXPECTED_BUN_VERSION:
                errors.append(
                    f"{workflow_path}:{job_name} must pin bun-version "
                    f"{EXPECTED_BUN_VERSION}, found {version!r}"
                )

    found = set(found_counts)
    missing = EXPECTED_SETUP_BUN_JOBS - found
    unexpected = found - EXPECTED_SETUP_BUN_JOBS
    errors.extend(f"missing setup-bun step in {path}:{job}" for path, job in sorted(missing))
    errors.extend(f"unexpected setup-bun step in {path}:{job}" for path, job in sorted(unexpected))
    errors.extend(
        f"expected exactly one setup-bun step in {path}:{job}, found {count}"
        for (path, job), count in sorted(found_counts.items())
        if count != 1
    )

    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1

    print(
        f"Validated {EXPECTED_SETUP_BUN_REF} with Bun {EXPECTED_BUN_VERSION} "
        f"across {len(found)} workflow jobs"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
