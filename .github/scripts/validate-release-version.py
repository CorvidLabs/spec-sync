#!/usr/bin/env python3
"""Validate that current release surfaces match Cargo's package version."""

from pathlib import Path
import re
import subprocess
import sys
import tomllib


YAML_FILES = (
    "action.yml",
    ".github/workflows/ci.yml",
    ".github/workflows/pages.yml",
    ".github/workflows/trust.yml",
)


def yaml_scalar(raw: str) -> str:
    """Normalize the simple scalar forms used by maintained workflow files."""
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


def mapping_scalar(lines: list[str], key: str, indent: int) -> str | None:
    pattern = re.compile(rf"^ {{{indent}}}{re.escape(key)}:\s*(.*?)\s*$")
    values = [yaml_scalar(match.group(1)) for line in lines if (match := pattern.match(line))]
    return values[0] if len(values) == 1 else None


def step_input(lines: list[str], key: str) -> str | None:
    """Return a scalar only when it is nested in this step's `with` mapping."""
    with_block = mapping_block(lines, "with", 8)
    return mapping_scalar(with_block or [], key, 10)


def validate_yaml_syntax(errors: list[str]) -> None:
    """Parse maintained YAML with Ruby's standard-library Psych parser."""
    command = [
        "ruby",
        "-e",
        "require 'psych'; ARGV.each { |path| Psych.parse_file(path) }",
        *YAML_FILES,
    ]
    try:
        result = subprocess.run(
            command,
            capture_output=True,
            check=False,
            text=True,
            timeout=30,
        )
    except FileNotFoundError:
        errors.append("Ruby with standard-library Psych is required for YAML syntax validation")
        return
    except subprocess.TimeoutExpired:
        errors.append("YAML syntax validation timed out after 30 seconds")
        return

    if result.returncode != 0:
        detail = result.stderr.strip()[-2000:] or "unknown Psych parser error"
        errors.append(f"maintained YAML syntax validation failed: {detail}")


def workflow_uses_steps(path: str) -> list[tuple[str, str, list[str]]]:
    """Read workflow `uses` steps without requiring a third-party YAML package."""
    lines = Path(path).read_text(encoding="utf-8").splitlines()
    jobs = mapping_block(lines, "jobs", 0)
    if jobs is None:
        return []

    steps: list[tuple[str, str, list[str]]] = []
    current_job: str | None = None
    index = 0
    job_pattern = re.compile(r"^  ([A-Za-z0-9_-]+):\s*(?:#.*)?$")
    step_pattern = re.compile(r"^      -(?:\s+.*)?$")
    inline_uses_pattern = re.compile(r"^      - uses:\s*(.*?)\s*$")
    nested_uses_pattern = re.compile(r"^        uses:\s*(.*?)\s*$")
    while index < len(jobs):
        line = jobs[index]
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


def find_uses_step(
    steps: list[tuple[str, str, list[str]]], job: str, predicate
) -> tuple[str, str, list[str]] | None:
    matches = [step for step in steps if step[0] == job and predicate(step[1])]
    return matches[0] if len(matches) == 1 else None


def main() -> int:
    cargo = tomllib.loads(Path("Cargo.toml").read_text(encoding="utf-8"))
    version = cargo["package"]["version"]
    errors: list[str] = []
    validate_yaml_syntax(errors)

    lock = tomllib.loads(Path("Cargo.lock").read_text(encoding="utf-8"))
    lock_versions = [
        package["version"] for package in lock["package"] if package["name"] == "specsync"
    ]
    if lock_versions != [version]:
        errors.append(f"Cargo.lock specsync version must be {version}, found {lock_versions}")

    action_lines = Path("action.yml").read_text(encoding="utf-8").splitlines()
    inputs = mapping_block(action_lines, "inputs", 0)
    version_input = mapping_block(inputs or [], "version", 2)
    action_version = mapping_scalar(version_input or [], "default", 4)
    if action_version != version:
        errors.append(f"action.yml default must be {version}, found {action_version}")

    ci_steps = workflow_uses_steps(".github/workflows/ci.yml")
    consumer = find_uses_step(
        ci_steps, "action-consumer", lambda uses: uses == "./"
    )
    if consumer is None:
        errors.append("packaged Action consumer step not found in ci.yml")
    else:
        consumer_version = step_input(consumer[2], "version")
        if consumer_version != version:
            errors.append(
                f"packaged Action consumer must use {version}, found {consumer_version}"
            )

    trust_steps = workflow_uses_steps(".github/workflows/trust.yml")
    trust_step = find_uses_step(
        trust_steps, "trust", lambda uses: uses.startswith("CorvidLabs/trust@")
    )
    if trust_step is None:
        errors.append("Trust step not found in trust.yml")
    else:
        trust_version = step_input(trust_step[2], "specsync-version")
        if trust_version != version:
            errors.append(f"Trust candidate must use {version}, found {trust_version}")

    expected_ref = "${{ github.event_name == 'pull_request' && github.event.pull_request.head.sha || github.sha }}"
    for path, steps, job in (
        ("ci.yml", ci_steps, "spec-check"),
        ("trust.yml", trust_steps, "trust"),
    ):
        checkout = find_uses_step(steps, job, lambda uses: uses.startswith("actions/checkout@"))
        if checkout is None:
            errors.append(f"{path}:{job} checkout step not found")
            continue
        ref = step_input(checkout[2], "ref")
        fetch_depth = step_input(checkout[2], "fetch-depth")
        if ref != expected_ref or fetch_depth != "0":
            errors.append(
                f"{path}:{job} must check out the exact event head with full history"
            )

    for job in ("test", "fmt", "audit", "coverage", "action-consumer"):
        checkout = find_uses_step(
            ci_steps, job, lambda uses: uses.startswith("actions/checkout@")
        )
        if checkout is None:
            errors.append(f"ci.yml:{job} checkout step not found")
        elif step_input(checkout[2], "ref") is not None:
            errors.append(f"ci.yml:{job} checkout must not override the default ref")

    expected_action_ref = f"v{version}"
    action_ref_pattern = re.compile(
        r"CorvidLabs/spec-sync@(v[0-9][A-Za-z0-9._-]*)"
    )
    version_input_pattern = re.compile(
        r"^\s+version:\s*['\"]?([^'\"\s#]+)", re.MULTILINE
    )

    readme = Path("README.md").read_text(encoding="utf-8")
    readme_refs = action_ref_pattern.findall(readme)
    if expected_action_ref not in readme_refs:
        errors.append(f"README.md must contain immutable Action ref @v{version}")
    stale_readme_refs = sorted({ref for ref in readme_refs if ref != expected_action_ref})
    if stale_readme_refs:
        errors.append(
            f"README.md Action refs must all be @{expected_action_ref}, "
            f"found {stale_readme_refs}"
        )
    readme_versions = version_input_pattern.findall(readme)
    if version not in readme_versions:
        errors.append(f"README.md must pin Action binary version {version}")
    stale_readme_versions = sorted(
        {input_version for input_version in readme_versions if input_version != version}
    )
    if stale_readme_versions:
        errors.append(
            f"README.md Action version inputs must all be {version}, "
            f"found {stale_readme_versions}"
        )

    action_docs = Path("site/src/content/docs/integrations/github-action.md").read_text(
        encoding="utf-8"
    )
    docs_default = re.search(r"^\| `version` \| `([^`]+)` \|", action_docs, re.MULTILINE)
    if docs_default is None or docs_default.group(1) != version:
        found = docs_default.group(1) if docs_default else None
        errors.append(f"Action docs default must be {version}, found {found!r}")

    stale_site_refs: list[str] = []
    stale_site_versions: list[str] = []
    for path in Path("site/src/content").rglob("*"):
        if path.suffix not in {".md", ".mdx"}:
            continue
        content = path.read_text(encoding="utf-8")
        action_refs = action_ref_pattern.findall(content)
        for ref in action_refs:
            if ref != expected_action_ref:
                stale_site_refs.append(f"{path}: @{ref}")
        if action_refs:
            for input_version in version_input_pattern.findall(content):
                if input_version != version:
                    stale_site_versions.append(f"{path}: {input_version}")
    if stale_site_refs:
        errors.append(
            f"site Action refs must all be @{expected_action_ref}: "
            + ", ".join(sorted(stale_site_refs))
        )
    if stale_site_versions:
        errors.append(
            f"site Action version inputs must all be {version}: "
            + ", ".join(sorted(stale_site_versions))
        )

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
