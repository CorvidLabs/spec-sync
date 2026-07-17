#!/usr/bin/env python3
"""Validate that current release surfaces match Cargo's package version."""

from pathlib import Path
import re
import sys
import tomllib

import yaml


def load_yaml(path: str) -> dict:
    document = yaml.safe_load(Path(path).read_text(encoding="utf-8")) or {}
    return document if isinstance(document, dict) else {}


def main() -> int:
    cargo = tomllib.loads(Path("Cargo.toml").read_text(encoding="utf-8"))
    version = cargo["package"]["version"]
    errors: list[str] = []

    lock = tomllib.loads(Path("Cargo.lock").read_text(encoding="utf-8"))
    lock_versions = [
        package["version"] for package in lock["package"] if package["name"] == "specsync"
    ]
    if lock_versions != [version]:
        errors.append(f"Cargo.lock specsync version must be {version}, found {lock_versions}")

    action = load_yaml("action.yml")
    action_version = str(
        ((action.get("inputs") or {}).get("version") or {}).get("default", "")
    )
    if action_version != version:
        errors.append(f"action.yml default must be {version}, found {action_version}")

    ci = load_yaml(".github/workflows/ci.yml")
    consumer_steps = ((ci.get("jobs") or {}).get("action-consumer") or {}).get("steps", [])
    consumer = next(
        (step for step in consumer_steps if isinstance(step, dict) and step.get("uses") == "./"),
        None,
    )
    if consumer is None:
        errors.append("packaged Action consumer step not found in ci.yml")
    else:
        consumer_version = str((consumer.get("with") or {}).get("version", ""))
        if consumer_version != version:
            errors.append(
                f"packaged Action consumer must use {version}, found {consumer_version}"
            )

    trust = load_yaml(".github/workflows/trust.yml")
    trust_steps = ((trust.get("jobs") or {}).get("trust") or {}).get("steps", [])
    trust_step = next(
        (
            step
            for step in trust_steps
            if isinstance(step, dict)
            and str(step.get("uses", "")).startswith("CorvidLabs/trust@")
        ),
        None,
    )
    if trust_step is None:
        errors.append("Trust step not found in trust.yml")
    else:
        trust_version = str((trust_step.get("with") or {}).get("specsync-version", ""))
        if trust_version != version:
            errors.append(f"Trust candidate must use {version}, found {trust_version}")

    readme = Path("README.md").read_text(encoding="utf-8")
    if f"CorvidLabs/spec-sync@v{version}" not in readme:
        errors.append(f"README.md must contain immutable Action ref @v{version}")
    if f"version: '{version}'" not in readme:
        errors.append(f"README.md must pin Action binary version {version}")

    action_docs = Path("site/src/content/docs/integrations/github-action.md").read_text(
        encoding="utf-8"
    )
    docs_default = re.search(r"^\| `version` \| `([^`]+)` \|", action_docs, re.MULTILINE)
    if docs_default is None or docs_default.group(1) != version:
        found = docs_default.group(1) if docs_default else None
        errors.append(f"Action docs default must be {version}, found {found!r}")

    changelog = Path("CHANGELOG.md").read_text(encoding="utf-8")
    if f"## [{version}]" not in changelog:
        errors.append(f"CHANGELOG.md must contain a {version} release section")
    if f"[Unreleased]: https://github.com/CorvidLabs/spec-sync/compare/v{version}...HEAD" not in changelog:
        errors.append(f"CHANGELOG.md Unreleased comparison must start at v{version}")

    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1

    print(f"Validated release version {version} across current distribution surfaces")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
