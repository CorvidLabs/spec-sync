#!/usr/bin/env python3
"""Start a packaged SpecSync Linux release archive on its documented baseline (#882).

The prebuilt Linux downloads promise to start on glibc 2.35 or newer (Ubuntu 22.04, Debian 12 and
newer). The static musl downloads promise to start on any Linux of their architecture. This
script is the one check behind both promises:

    linux-baseline.py check --archive <dir>/<artifact>.tar.gz --expect-version <version>

It verifies the archive against its `.sha256` sidecar, extracts the single binary, and runs it in
digest-pinned containers with no network, as an unprivileged user on a read-only root filesystem:
`--version`, `--help`, `init`, a strict `check` on a one-module project that must pass, and the
same check after an undocumented export is added, which must fail.

`release.yml` and `rc-assets.yml` run it on every packaged Linux archive before upload, so a
download that would not start is never released. `ci.yml` builds the same archives the same way
and runs it on every full CI run.

Running the glibc download on the floor image is the authoritative proof that the floor holds:
the dynamic loader checks every symbol version the binary needs, weak symbols included. v6.2.0
failed exactly there, because two weak Rust std symbols (`pidfd_spawnp`, `pidfd_getpid`) carried
`GLIBC_2.39` after being linked on `ubuntu-latest`.

`LINUX_ARTIFACTS` is the single table the workflows and docs are held to by
`test-linux-baseline.py`. Change a runner, a target or the floor here and in those files together.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path
from typing import Any, Sequence

GLIBC_FLOOR = "2.35"
FLOOR_DESCRIPTION = "glibc 2.35 or newer (Ubuntu 22.04, Debian 12 and newer)"

# Multi-arch index digests: each carries linux/amd64 and linux/arm64, so one pin serves both
# runner architectures and Docker selects the host's.
IMAGES: dict[str, dict[str, str]] = {
    "ubuntu-22.04": {
        "ref": "ubuntu:22.04@sha256:076e7bf82319619f28dd0aa9c02ab34b055ad37100395b4f49a2ffba04fb5e11",
        "libc": "glibc 2.35, the documented floor",
    },
    "debian-12": {
        "ref": "debian:12-slim@sha256:734a72b3d6142a71993b4bc580ca01d8ca687f77a2c422d96a961d3376b179bc",
        "libc": "glibc 2.36, the system reported in #882",
    },
    "debian-11": {
        "ref": "debian:11-slim@sha256:e5b6442dd2e9684cf5e87d8338b5968f3b348636fc0be6d7850a381e3731a2bd",
        "libc": "glibc 2.31, below the floor",
    },
    "alpine-3.22": {
        "ref": "alpine:3.22@sha256:bfe7ceb9f09ec0566c48fb93356e45a47a00be5a1dd14fa5b909e7709c66b034",
        "libc": "musl, no glibc at all",
    },
}

GLIBC_IMAGES = ("ubuntu-22.04", "debian-12")
MUSL_IMAGES = ("ubuntu-22.04", "debian-12", "debian-11", "alpine-3.22")

LINUX_ARTIFACTS: dict[str, dict[str, Any]] = {
    "specsync-linux-x86_64": {
        "target": "x86_64-unknown-linux-gnu",
        "runner": "ubuntu-22.04",
        "arch": "x86_64",
        "libc": "gnu",
        "images": GLIBC_IMAGES,
    },
    "specsync-linux-x86_64-musl": {
        "target": "x86_64-unknown-linux-musl",
        "runner": "ubuntu-latest",
        "arch": "x86_64",
        "libc": "musl",
        "images": MUSL_IMAGES,
    },
    "specsync-linux-aarch64": {
        "target": "aarch64-unknown-linux-gnu",
        "runner": "ubuntu-22.04-arm",
        "arch": "aarch64",
        "libc": "gnu",
        "images": GLIBC_IMAGES,
    },
    "specsync-linux-aarch64-musl": {
        "target": "aarch64-unknown-linux-musl",
        "runner": "ubuntu-24.04-arm",
        "arch": "aarch64",
        "libc": "musl",
        "images": MUSL_IMAGES,
    },
}

HOST_ARCHES = {
    "x86_64": "x86_64",
    "amd64": "x86_64",
    "aarch64": "aarch64",
    "arm64": "aarch64",
}

ARCHIVE_SUFFIX = ".tar.gz"
CHECKSUM_PATTERN = re.compile(rb"^([0-9a-f]{64})  ([^\s/]+)\n$")
PULL_ATTEMPTS = 3
DOCKER_TIMEOUT_SECONDS = 180

FIXTURE_SOURCE = "pub fn hello() {}\n"
FIXTURE_DRIFT = "pub fn goodbye() {}\n"
FIXTURE_SPEC = """---
module: greeting
version: 1
status: stable
files:
  - src/greeting.rs
---

# Greeting

## Purpose

Provides a greeting.

## Public API

| Name | Description |
|------|-------------|
| `hello` | Return a greeting |

## Invariants

1. Greeting behavior remains stable.

## Behavioral Examples

Calling hello succeeds.

## Error Cases

None.

## Dependencies

None.

## Change Log

| Date | Change |
|------|--------|
| 2026-11-05 | Baseline fixture for the Linux release check |
"""

# POSIX sh, so it runs on Ubuntu, Debian and Alpine's BusyBox alike. Every step prints a marker
# and then its own output, prefixed, so the verdict never depends on interleaved streams.
SMOKE_SCRIPT = """set -u
bin=/opt/specsync/specsync
work="$(mktemp -d)"
run() {
    step="$1"
    shift
    "$@" >"$work/$step.log" 2>&1
    code=$?
    echo "@@step $step $code"
    sed 's/^/@@out /' "$work/$step.log"
}
run version "$bin" --version
run help "$bin" --help
mkdir "$work/project"
cd "$work/project" || exit 90
run init "$bin" init
cp -R /opt/fixture/. .
run check "$bin" check --strict --require-coverage 100
printf '%s' "$DRIFT" >> src/greeting.rs
run drift "$bin" check --strict --require-coverage 100
"""

STEPS = ("version", "help", "init", "check", "drift")


class BaselineError(Exception):
    """A refusal that stops the check before any image runs."""


def artifact_for(archive: Path) -> str:
    """Return the artifact name an archive file name declares, or refuse."""
    name = archive.name
    if not name.endswith(ARCHIVE_SUFFIX):
        raise BaselineError(f"{name} is not a .tar.gz release archive")
    artifact = name[: -len(ARCHIVE_SUFFIX)]
    if artifact not in LINUX_ARTIFACTS:
        known = ", ".join(sorted(LINUX_ARTIFACTS))
        raise BaselineError(f"{artifact} is not a Linux release artifact; expected one of {known}")
    return artifact


def require_host_arch(artifact: str, machine: str) -> None:
    """Refuse to run an artifact on a host of another architecture."""
    wanted = LINUX_ARTIFACTS[artifact]["arch"]
    host = HOST_ARCHES.get(machine.lower())
    if host != wanted:
        runner = LINUX_ARTIFACTS[artifact]["runner"]
        raise BaselineError(
            f"{artifact} is a {wanted} binary but this host is {machine or 'unknown'}; "
            f"run this check on a {wanted} Linux runner such as {runner}, "
            "as the linux-baseline job in ci.yml does"
        )


def verify_checksum(archive: Path) -> str:
    """Verify `<archive>.sha256` exactly as the release writes it; return the digest."""
    if not archive.is_file():
        raise BaselineError(f"{archive} does not exist; build and package the archive first")
    sidecar = archive.with_name(archive.name + ".sha256")
    if not sidecar.is_file():
        raise BaselineError(f"{sidecar.name} is missing; package the archive with its checksum first")
    match = CHECKSUM_PATTERN.match(sidecar.read_bytes())
    if match is None:
        raise BaselineError(
            f"{sidecar.name} is not one LF-terminated '<sha256>  <archive>' record"
        )
    expected, named = match.group(1).decode("ascii"), match.group(2).decode("utf-8")
    if named != archive.name:
        raise BaselineError(f"{sidecar.name} names {named}, not {archive.name}")
    actual = hashlib.sha256(archive.read_bytes()).hexdigest()
    if actual != expected:
        raise BaselineError(
            f"{archive.name} has SHA-256 {actual} but {sidecar.name} records {expected}; "
            "rebuild or re-download the archive"
        )
    return actual


def extract_binary(archive: Path, artifact: str, destination: Path) -> Path:
    """Extract the archive's one regular file, named after the artifact, refusing anything else."""
    try:
        with tarfile.open(archive, "r:gz") as bundle:
            members = bundle.getmembers()
            if len(members) != 1:
                names = ", ".join(member.name for member in members) or "nothing"
                raise BaselineError(f"{archive.name} must hold exactly {artifact}; it holds {names}")
            member = members[0]
            if member.name != artifact or not member.isreg():
                raise BaselineError(
                    f"{archive.name} must hold one regular file named {artifact}; "
                    f"it holds {member.name!r}"
                )
            source = bundle.extractfile(member)
            if source is None:
                raise BaselineError(f"{archive.name}: {artifact} could not be read")
            target = destination / "specsync"
            with target.open("wb") as handle:
                shutil.copyfileobj(source, handle)
    except tarfile.TarError as error:
        raise BaselineError(f"{archive.name} is not a readable gzip tar archive: {error}") from error
    target.chmod(0o755)
    return target


def write_fixture(root: Path) -> tuple[Path, Path]:
    """Write the one-module project and the smoke script, readable by any container user."""
    fixture = root / "fixture"
    (fixture / "src").mkdir(parents=True)
    (fixture / "specs/greeting").mkdir(parents=True)
    (fixture / "src/greeting.rs").write_text(FIXTURE_SOURCE, encoding="utf-8")
    (fixture / "specs/greeting/greeting.spec.md").write_text(FIXTURE_SPEC, encoding="utf-8")
    script = root / "smoke.sh"
    script.write_text(SMOKE_SCRIPT, encoding="utf-8")
    for path in [root, *root.rglob("*")]:
        path.chmod(0o755 if path.is_dir() else 0o644)
    return fixture, script


def parse_transcript(stdout: str) -> dict[str, dict[str, Any]]:
    """Group `@@step`/`@@out` lines into {step: {exit, output}}."""
    steps: dict[str, dict[str, Any]] = {}
    current: dict[str, Any] | None = None
    for line in stdout.splitlines():
        if line.startswith("@@step "):
            parts = line.split()
            if len(parts) == 3 and parts[2].lstrip("-").isdigit():
                current = {"exit": int(parts[2]), "output": []}
                steps[parts[1]] = current
                continue
        if line.startswith("@@out ") and current is not None:
            current["output"].append(line[len("@@out ") :])
        elif line == "@@out" and current is not None:
            current["output"].append("")
    return {name: {"exit": step["exit"], "output": "\n".join(step["output"])} for name, step in steps.items()}


def excerpt(text: str, lines: int = 6) -> str:
    """The first few lines of some output, for an error message."""
    kept = [line for line in text.splitlines() if line.strip()][:lines]
    return " | ".join(kept) if kept else "(no output)"


def judge(artifact: str, image: str, expected_version: str, steps: dict[str, dict[str, Any]]) -> list[str]:
    """Return every reason this image's transcript fails the baseline; empty when it passes."""
    entry = LINUX_ARTIFACTS[artifact]
    where = f"{artifact} on {IMAGES[image]['ref'].split('@')[0]} ({IMAGES[image]['libc']})"
    errors: list[str] = []
    missing = [step for step in STEPS if step not in steps]
    version = steps.get("version")
    if version is not None and version["exit"] != 0:
        output = version["output"]
        message = f"{where} does not start: --version exited {version['exit']}: {excerpt(output)}"
        if "GLIBC_" in output and "not found" in output:
            message += (
                f". The {entry['libc']} download must be linked on the documented floor, "
                f"{FLOOR_DESCRIPTION}: build it with `cargo build --release --target "
                f"{entry['target']}` on the {entry['runner']} runner, as release.yml does, or raise "
                "the documented floor in README.md, the Action docs and LINUX_ARTIFACTS together"
            )
        return [message]
    if missing:
        return [f"{where}: the smoke run stopped before {', '.join(missing)}"]
    first_line = steps["version"]["output"].splitlines()[0] if steps["version"]["output"] else ""
    if first_line != f"specsync {expected_version}":
        errors.append(f"{where}: --version printed {first_line!r}, expected 'specsync {expected_version}'")
    if steps["help"]["exit"] != 0 or "Usage:" not in steps["help"]["output"]:
        errors.append(f"{where}: --help exited {steps['help']['exit']}: {excerpt(steps['help']['output'])}")
    if steps["init"]["exit"] != 0:
        errors.append(f"{where}: init exited {steps['init']['exit']}: {excerpt(steps['init']['output'])}")
    if steps["check"]["exit"] != 0:
        errors.append(
            f"{where}: check --strict --require-coverage 100 exited {steps['check']['exit']} "
            f"on a documented project: {excerpt(steps['check']['output'])}"
        )
    drift = steps["drift"]
    if drift["exit"] == 0 or "goodbye" not in drift["output"]:
        errors.append(
            f"{where}: check exited {drift['exit']} after an undocumented export was added; "
            f"it must fail and name 'goodbye': {excerpt(drift['output'])}"
        )
    return errors


def pull(docker: str, ref: str) -> str | None:
    """Pull one pinned image, retrying; return an error message or None."""
    last = ""
    for attempt in range(1, PULL_ATTEMPTS + 1):
        try:
            result = subprocess.run(
                [docker, "pull", "--quiet", ref],
                capture_output=True,
                text=True,
                timeout=DOCKER_TIMEOUT_SECONDS,
                check=False,
            )
        except (OSError, subprocess.TimeoutExpired) as error:
            last = str(error)
            continue
        if result.returncode == 0:
            return None
        last = excerpt(result.stderr or result.stdout)
        print(f"pull {ref} attempt {attempt}/{PULL_ATTEMPTS} failed: {last}", file=sys.stderr)
    return f"could not pull {ref} after {PULL_ATTEMPTS} attempts: {last}"


def run_image(docker: str, ref: str, binary_dir: Path, fixture: Path, script: Path) -> tuple[int, str, str]:
    """Run the smoke script in one image with no network and no privileges."""
    command = [
        docker,
        "run",
        "--rm",
        "--pull",
        "never",
        "--network",
        "none",
        "--read-only",
        "--tmpfs",
        "/tmp",
        "--cap-drop",
        "ALL",
        "--security-opt",
        "no-new-privileges",
        "--user",
        "65534:65534",
        "--env",
        "HOME=/tmp",
        "--env",
        f"DRIFT={FIXTURE_DRIFT}",
        "--mount",
        f"type=bind,src={binary_dir},dst=/opt/specsync,readonly",
        "--mount",
        f"type=bind,src={fixture},dst=/opt/fixture,readonly",
        "--mount",
        f"type=bind,src={script},dst=/opt/smoke.sh,readonly",
        ref,
        "sh",
        "/opt/smoke.sh",
    ]
    try:
        result = subprocess.run(
            command,
            capture_output=True,
            text=True,
            timeout=DOCKER_TIMEOUT_SECONDS,
            check=False,
        )
    except subprocess.TimeoutExpired:
        return 124, "", f"docker run timed out after {DOCKER_TIMEOUT_SECONDS}s"
    except OSError as error:
        return 127, "", str(error)
    return result.returncode, result.stdout, result.stderr


def check(archive: Path, expected_version: str, docker: str = "docker", machine: str | None = None) -> tuple[dict[str, Any], list[str]]:
    """Run the whole check; return a JSON-ready report and every failure."""
    artifact = artifact_for(archive)
    require_host_arch(artifact, platform.machine() if machine is None else machine)
    digest = verify_checksum(archive)
    report: dict[str, Any] = {
        "artifact": artifact,
        "archive": archive.name,
        "sha256": digest,
        "expected_version": expected_version,
        "floor": FLOOR_DESCRIPTION,
        "images": [],
    }
    errors: list[str] = []
    with tempfile.TemporaryDirectory(prefix="specsync-baseline-") as temporary:
        root = Path(temporary)
        binary_dir = root / "bin"
        binary_dir.mkdir()
        extract_binary(archive, artifact, binary_dir)
        fixture, script = write_fixture(root / "project")
        binary_dir.chmod(0o755)
        root.chmod(0o755)
        for image in LINUX_ARTIFACTS[artifact]["images"]:
            ref = IMAGES[image]["ref"]
            entry: dict[str, Any] = {"image": ref, "libc": IMAGES[image]["libc"]}
            report["images"].append(entry)
            pull_error = pull(docker, ref)
            if pull_error is not None:
                entry["passed"] = False
                errors.append(f"{artifact}: {pull_error}")
                continue
            code, stdout, stderr = run_image(docker, ref, binary_dir, fixture, script)
            steps = parse_transcript(stdout)
            entry["container_exit"] = code
            entry["steps"] = {name: step["exit"] for name, step in steps.items()}
            image_errors = judge(artifact, image, expected_version, steps)
            if code != 0 and not image_errors:
                image_errors = [f"{artifact} on {ref}: the container exited {code}: {excerpt(stderr)}"]
            if not steps and stderr.strip():
                image_errors.append(f"{artifact} on {ref}: docker reported: {excerpt(stderr)}")
            entry["passed"] = not image_errors
            errors.extend(image_errors)
    report["passed"] = not errors
    return report, errors


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    commands = parser.add_subparsers(dest="command", required=True)
    run = commands.add_parser("check", help="start one packaged Linux archive on its baseline")
    run.add_argument("--archive", required=True, type=Path, help="<artifact>.tar.gz beside its .sha256")
    run.add_argument("--expect-version", required=True, help="the package version --version must print")
    run.add_argument("--docker", default="docker", help="docker executable (tests pass a fake)")
    run.add_argument("--machine", default=None, help=argparse.SUPPRESS)
    arguments = parser.parse_args(argv)
    try:
        report, errors = check(arguments.archive, arguments.expect_version, arguments.docker, arguments.machine)
    except BaselineError as error:
        print(f"::error::{error}", file=sys.stderr)
        return 2
    print(json.dumps(report, indent=2, sort_keys=True))
    for error in errors:
        print(f"::error::{error}", file=sys.stderr)
    if errors:
        print(f"{report['artifact']} does not meet its documented Linux baseline.", file=sys.stderr)
        return 1
    images = ", ".join(entry["image"].split("@")[0] for entry in report["images"])
    print(f"{report['artifact']} started and checked a project on {images}.", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
