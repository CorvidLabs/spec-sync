#!/usr/bin/env python3
"""Validate deterministic runtime pins in maintained GitHub workflows."""

from pathlib import Path
import sys

import yaml


EXPECTED_BUN_VERSION = "1.3.14"
EXPECTED_SETUP_BUN_JOBS = {
    (".github/workflows/ci.yml", "site"),
    (".github/workflows/ci.yml", "vscode-extension"),
    (".github/workflows/pages.yml", "build"),
}


def main() -> int:
    found: set[tuple[str, str]] = set()
    errors: list[str] = []

    for workflow_path in sorted({path for path, _ in EXPECTED_SETUP_BUN_JOBS}):
        document = yaml.safe_load(Path(workflow_path).read_text(encoding="utf-8")) or {}
        jobs = document.get("jobs", {}) if isinstance(document, dict) else {}
        for job_name, job in jobs.items():
            if not isinstance(job, dict):
                continue
            for step in job.get("steps", []):
                if not isinstance(step, dict):
                    continue
                if step.get("uses") != "oven-sh/setup-bun@v2":
                    continue

                location = (workflow_path, job_name)
                found.add(location)
                version = (step.get("with") or {}).get("bun-version")
                if str(version) != EXPECTED_BUN_VERSION:
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
