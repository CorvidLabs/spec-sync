#!/usr/bin/env python3
"""Validate that current release surfaces match Cargo's package version."""

from pathlib import Path
import json
import re
import subprocess
import sys


YAML_FILES = (
    "action.yml",
    ".github/workflows/ci.yml",
    ".github/workflows/pages.yml",
    ".github/workflows/trust.yml",
)

ACTION_DOCS = (
    "README.md",
    *sorted(
        str(path)
        for path in Path("site/src/content").rglob("*")
        if path.suffix in {".md", ".mdx"}
    ),
)


def cargo_package_version(path: str) -> str | None:
    """Read the package version without requiring Python 3.11's tomllib."""
    content = Path(path).read_text(encoding="utf-8")
    package = re.search(r"(?ms)^\[package\]\s*$\n(.*?)(?=^\[|\Z)", content)
    if package is None:
        return None
    version = re.search(r'^version\s*=\s*["\']([^"\']+)["\']\s*$', package.group(1), re.MULTILINE)
    return version.group(1) if version else None


def lock_package_versions(path: str, name: str) -> list[str]:
    """Read matching Cargo.lock package versions using its repeated table boundaries."""
    content = Path(path).read_text(encoding="utf-8")
    versions: list[str] = []
    for package in re.findall(r"(?ms)^\[\[package\]\]\s*$\n(.*?)(?=^\[\[package\]\]|\Z)", content):
        package_name = re.search(r'^name\s*=\s*"([^"]+)"\s*$', package, re.MULTILINE)
        version = re.search(r'^version\s*=\s*"([^"]+)"\s*$', package, re.MULTILINE)
        if package_name and package_name.group(1) == name and version:
            versions.append(version.group(1))
    return versions


def split_action_reference(uses: str) -> tuple[str, str] | None:
    """Return a case-normalized action repository and its case-sensitive ref."""
    repository, separator, ref = uses.rpartition("@")
    if separator != "@" or not repository or not ref:
        return None
    return repository.casefold(), ref


def uses_action(uses: str, repository: str) -> bool:
    reference = split_action_reference(uses)
    return reference is not None and reference[0] == repository.casefold()


def documented_action_steps(errors: list[str]) -> list[dict[str, str | None]]:
    """Parse YAML documentation examples and return every spec-sync Action step."""
    ruby = r'''\
require "json"
require "psych"

steps = []
errors = []
ARGV.each do |path|
  content = File.read(path, encoding: "UTF-8")
  content.scan(/^```(?:yaml|yml)(?:[ \t]+[^\r\n]*)?[ \t]*\r?\n(.*?)^```[ \t]*\r?$/m).each_with_index do |match, index|
    begin
      document = Psych.safe_load(match.first, permitted_classes: [], aliases: false)
    rescue Psych::Exception => error
      errors << { path: path, block: index + 1, error: error.message }
      next
    end

    walk = lambda do |value|
      case value
      when Array
        value.each { |item| walk.call(item) }
      when Hash
        uses = value["uses"]
        if uses.is_a?(String)
          repository, separator, ref = uses.rpartition("@")
        end
        if uses.is_a?(String) && separator == "@" &&
           repository.downcase == "corvidlabs/spec-sync" && !ref.empty?
          inputs = value["with"]
          version = inputs.is_a?(Hash) ? inputs["version"] : nil
          steps << {
            path: path,
            block: index + 1,
            ref: ref,
            version: version.nil? ? nil : version.to_s,
          }
        end
        value.each_value { |item| walk.call(item) }
      end
    end
    walk.call(document)
  end
end
puts JSON.generate({ steps: steps, errors: errors })
'''
    try:
        result = subprocess.run(
            ["ruby", "-e", ruby, *ACTION_DOCS],
            capture_output=True,
            check=False,
            text=True,
            timeout=30,
        )
    except FileNotFoundError:
        errors.append("Ruby with standard-library Psych is required for Action docs validation")
        return []
    except subprocess.TimeoutExpired:
        errors.append("Action docs YAML validation timed out after 30 seconds")
        return []

    if result.returncode != 0:
        detail = result.stderr.strip()[-2000:] or "unknown Psych parser error"
        errors.append(f"Action docs YAML validation failed: {detail}")
        return []

    try:
        payload = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        errors.append(f"Action docs validator returned invalid JSON: {error}")
        return []
    for parse_error in payload["errors"]:
        errors.append(
            f"{parse_error['path']} YAML block {parse_error['block']} is invalid: "
            f"{parse_error['error']}"
        )
    return payload["steps"]


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
    errors: list[str] = []
    version = cargo_package_version("Cargo.toml")
    if version is None:
        errors.append("Cargo.toml package version not found")
        version = "<missing>"
    validate_yaml_syntax(errors)

    lock_versions = lock_package_versions("Cargo.lock", "specsync")
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
        trust_steps, "trust", lambda uses: uses_action(uses, "corvidlabs/trust")
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
        checkout = find_uses_step(
            steps, job, lambda uses: uses_action(uses, "actions/checkout")
        )
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
            ci_steps, job, lambda uses: uses_action(uses, "actions/checkout")
        )
        if checkout is None:
            errors.append(f"ci.yml:{job} checkout step not found")
        elif step_input(checkout[2], "ref") is not None:
            errors.append(f"ci.yml:{job} checkout must not override the default ref")

    expected_action_ref = f"v{version}"
    documented_steps = documented_action_steps(errors)
    readme_steps = [step for step in documented_steps if step["path"] == "README.md"]
    readme_refs = [step["ref"] for step in readme_steps]
    if expected_action_ref not in readme_refs:
        errors.append(f"README.md must contain immutable Action ref @v{version}")
    stale_readme_refs = sorted({ref for ref in readme_refs if ref != expected_action_ref})
    if stale_readme_refs:
        errors.append(
            f"README.md Action refs must all be @{expected_action_ref}, "
            f"found {stale_readme_refs}"
        )
    readme_versions = [step["version"] for step in readme_steps if step["version"] is not None]
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

    stale_site_refs = [
        f"{step['path']} YAML block {step['block']}: @{step['ref']}"
        for step in documented_steps
        if step["path"] != "README.md" and step["ref"] != expected_action_ref
    ]
    stale_site_versions = [
        f"{step['path']} YAML block {step['block']}: {step['version']}"
        for step in documented_steps
        if step["path"] != "README.md"
        and step["version"] is not None
        and step["version"] != version
    ]
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
