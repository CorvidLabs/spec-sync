#!/usr/bin/env python3
"""Tests for the Linux release baseline check and the files held to its table (#882).

Two kinds of test live here:

* Structural tests read `release.yml`, `rc-assets.yml` and `ci.yml` through Ruby's standard Psych
  parser (as the other workflow validators do) and fail when their Linux build rows, the check
  step, the CI job, the release artifact validator or the install docs drift from
  `LINUX_ARTIFACTS` in `linux-baseline.py`.
* Unit tests run `linux-baseline.py check` end to end against a fake `docker` that records its
  arguments and replays transcripts, so every verdict and refusal is exercised without Docker.

The real containers run in the `linux-baseline` CI job and in the release workflows.
"""

from __future__ import annotations

import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tarfile
import tempfile
import textwrap
import unittest
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / ".github/scripts/linux-baseline.py"


def load_module(name: str, path: Path) -> Any:
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"could not load {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


BASELINE = load_module("linux_baseline", SCRIPT)
VALIDATOR = load_module(
    "validate_release_candidate_for_baseline", ROOT / ".github/scripts/validate-release-candidate.py"
)

NEGATIVE_CONTROL_SHA256 = "86e081a5ef8084f1282d77d807e495b1dbdbc8ae89840594ed3e915439bcf35a"
DIGEST_REF = re.compile(r"^[a-z0-9][a-z0-9./_-]*:[A-Za-z0-9._-]+@sha256:[0-9a-f]{64}$")

RUBY_TO_JSON = r'''
require "json"
require "psych"
document = Psych.safe_load(File.read(ARGV.fetch(0), encoding: "UTF-8"), permitted_classes: [], aliases: false)
puts JSON.generate(document)
'''


def load_workflow(relative: str) -> dict[str, Any]:
    result = subprocess.run(
        ["ruby", "-e", RUBY_TO_JSON, str(ROOT / relative)],
        capture_output=True,
        text=True,
        timeout=30,
        check=True,
    )
    return json.loads(result.stdout)


def linux_rows(job: dict[str, Any]) -> set[tuple[str, str, str]]:
    rows = set()
    for entry in job.get("strategy", {}).get("matrix", {}).get("include", []):
        if "linux" in str(entry.get("target", "")):
            rows.add((entry.get("artifact"), entry.get("target"), entry.get("os")))
    return rows


def expected_rows() -> set[tuple[str, str, str]]:
    return {
        (artifact, entry["target"], entry["runner"])
        for artifact, entry in BASELINE.LINUX_ARTIFACTS.items()
    }


def step_index(steps: list[dict[str, Any]], predicate: Any) -> int:
    for index, step in enumerate(steps):
        if predicate(step):
            return index
    return -1


def runs(step: dict[str, Any], needle: str) -> bool:
    return needle in str(step.get("run", ""))


class WorkflowLockstepTests(unittest.TestCase):
    """The workflows build exactly the table's Linux rows and run the check before upload."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.release = load_workflow(".github/workflows/release.yml")
        cls.rc_assets = load_workflow(".github/workflows/rc-assets.yml")
        cls.ci = load_workflow(".github/workflows/ci.yml")

    def ci_job(self) -> dict[str, Any]:
        job = self.ci["jobs"].get("linux-baseline")
        if job is None:
            self.fail("ci.yml has no linux-baseline job")
        return job

    def release_jobs(self) -> list[tuple[str, dict[str, Any]]]:
        return [
            ("release.yml build", self.release["jobs"]["build"]),
            ("rc-assets.yml build", self.rc_assets["jobs"]["build"]),
        ]

    def test_release_workflows_build_the_table_rows(self) -> None:
        for label, job in self.release_jobs():
            with self.subTest(job=label):
                self.assertEqual(linux_rows(job), expected_rows())

    def test_ci_job_builds_the_same_rows(self) -> None:
        self.assertEqual(linux_rows(self.ci_job()), expected_rows())

    def test_glibc_rows_are_never_built_on_a_moving_runner(self) -> None:
        for artifact, entry in BASELINE.LINUX_ARTIFACTS.items():
            if entry["libc"] == "gnu":
                with self.subTest(artifact=artifact):
                    self.assertIn("22.04", entry["runner"])

    def test_release_workflows_check_after_packaging_and_before_upload(self) -> None:
        for label, job in self.release_jobs():
            with self.subTest(job=label):
                steps = job["steps"]
                package = step_index(steps, lambda step: step.get("name") == "Package")
                check = step_index(steps, lambda step: runs(step, ".github/scripts/linux-baseline.py check"))
                upload = step_index(steps, lambda step: "actions/upload-artifact" in str(step.get("uses", "")))
                self.assertGreaterEqual(package, 0, f"{label} has no Package step")
                self.assertGreater(check, package, f"{label} must run the baseline check after Package")
                self.assertGreater(upload, check, f"{label} must run the baseline check before upload")
                self.assertIn("runner.os == 'Linux'", str(steps[check].get("if", "")))

    def test_no_aarch64_cross_linker_remains(self) -> None:
        for label, job in self.release_jobs():
            with self.subTest(job=label):
                self.assertNotIn("gcc-aarch64-linux-gnu", json.dumps(job))

    def test_musl_toolchain_covers_both_musl_targets(self) -> None:
        for label, job in self.release_jobs() + [("ci.yml linux-baseline", self.ci_job())]:
            text = json.dumps(job)
            with self.subTest(job=label):
                self.assertIn("CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER", text)
                self.assertIn("CC_aarch64_unknown_linux_musl", text)
                self.assertIn("CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER", text)

    def test_ci_job_is_selected_on_full_runs_and_gated(self) -> None:
        jobs = self.ci["jobs"]
        job = self.ci_job()
        self.assertIn("classify", job.get("needs", []))
        self.assertEqual(job.get("if"), "needs.classify.outputs.full == 'true'")
        self.assertIn("linux-baseline", jobs["implementation-gate"]["needs"])
        self.assertIn("linux-baseline", jobs["corvid-pet"]["needs"])
        self.assertTrue(
            any(runs(step, ".github/scripts/linux-baseline.py check") for step in job["steps"]),
            "the CI job never runs the baseline check",
        )

    def test_ci_job_runs_the_pinned_v620_negative_control(self) -> None:
        steps = self.ci_job()["steps"]
        controls = [step for step in steps if runs(step, NEGATIVE_CONTROL_SHA256)]
        self.assertEqual(len(controls), 1, "expected one negative-control step pinned to the v6.2.0 digest")
        control = controls[0]
        self.assertEqual(control.get("if"), "matrix.artifact == 'specsync-linux-aarch64'")
        body = str(control.get("run", ""))
        self.assertIn("releases/download/v6.2.0/specsync-linux-aarch64.tar.gz", body)
        self.assertIn("GLIBC_2.39", body)
        self.assertIn(".github/scripts/linux-baseline.py check", body)

    def test_ci_runs_on_rc_assets_changes(self) -> None:
        triggers = self.ci.get("on") or self.ci.get("true") or {}
        for event in ("push", "pull_request"):
            with self.subTest(event=event):
                self.assertIn(".github/workflows/rc-assets.yml", triggers[event]["paths"])


class TableTests(unittest.TestCase):
    def test_images_are_pinned_by_digest(self) -> None:
        for name, image in BASELINE.IMAGES.items():
            with self.subTest(image=name):
                self.assertRegex(image["ref"], DIGEST_REF)

    def test_every_artifact_names_known_images(self) -> None:
        for artifact, entry in BASELINE.LINUX_ARTIFACTS.items():
            with self.subTest(artifact=artifact):
                self.assertTrue(set(entry["images"]) <= set(BASELINE.IMAGES))
                self.assertIn("ubuntu-22.04", entry["images"])
                self.assertIn("debian-12", entry["images"])

    def test_release_validator_expects_every_linux_archive(self) -> None:
        for artifact in BASELINE.LINUX_ARTIFACTS:
            with self.subTest(artifact=artifact):
                self.assertEqual(VALIDATOR.EXPECTED_ARTIFACT_ARCHIVES.get(artifact), f"{artifact}.tar.gz")


class InstallDocsTests(unittest.TestCase):
    """README and the Action docs name the floor, the fallback and every Linux download."""

    DOCS = ("README.md", "site/src/content/docs/integrations/github-action.md")

    def test_docs_name_the_floor_fallback_and_symptom(self) -> None:
        for relative in self.DOCS:
            text = (ROOT / relative).read_text(encoding="utf-8")
            with self.subTest(doc=relative):
                for phrase in ("glibc 2.35", "Ubuntu 22.04", "Debian 12", "GLIBC_"):
                    self.assertIn(phrase, text)
                for artifact in BASELINE.LINUX_ARTIFACTS:
                    self.assertIn(f"`{artifact}`", text)


FAKE_DOCKER = textwrap.dedent(
    """\
    #!/usr/bin/env python3
    import json, os, sys
    log = os.environ["FAKE_DOCKER_LOG"]
    with open(log, "a", encoding="utf-8") as handle:
        handle.write(json.dumps(sys.argv[1:]) + "\\n")
    if sys.argv[1] == "pull":
        sys.exit(int(os.environ.get("FAKE_PULL_EXIT", "0")))
    ref = sys.argv[-3]
    transcripts = json.loads(os.environ["FAKE_TRANSCRIPTS"])
    chosen = transcripts.get(ref.split("@")[0], transcripts["default"])
    sys.stdout.write(chosen)
    sys.exit(int(os.environ.get("FAKE_RUN_EXIT", "0")))
    """
)


def transcript(**overrides: tuple[int, str]) -> str:
    """A passing smoke transcript, with any step replaced by `(exit, output)`."""
    steps = {
        "version": (0, "specsync 6.2.0"),
        "help": (0, "Bidirectional spec-to-code validation\n\nUsage: specsync [OPTIONS] <COMMAND>"),
        "init": (0, "Created .specsync/config.toml"),
        "check": (0, "1 specs checked: 1 passed, 0 warning(s), 0 failed"),
        "drift": (1, "  ⚠ Undocumented export 'goodbye' from src/greeting.rs"),
    }
    steps.update(overrides)
    lines = []
    for name in BASELINE.STEPS:
        if name not in steps:
            continue
        code, output = steps[name]
        lines.append(f"@@step {name} {code}")
        lines.extend(f"@@out {line}" for line in output.splitlines())
    return "\n".join(lines) + "\n"


LOADER_FAILURE = transcript(
    version=(1, "/opt/specsync/specsync: /lib/aarch64-linux-gnu/libc.so.6: version `GLIBC_2.39' not found"),
)


class FakeDockerCheckTests(unittest.TestCase):
    """`linux-baseline.py check` end to end, with Docker replaced by a recording fake."""

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.docker = self.root / "docker"
        self.docker.write_text(FAKE_DOCKER, encoding="utf-8")
        self.docker.chmod(0o755)
        self.log = self.root / "docker.log"

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def archive(self, artifact: str, members: list[tuple[str, bytes]] | None = None, symlink: bool = False) -> Path:
        directory = self.root / "dist"
        directory.mkdir(exist_ok=True)
        path = directory / f"{artifact}.tar.gz"
        with tarfile.open(path, "w:gz") as bundle:
            if symlink:
                info = tarfile.TarInfo(artifact)
                info.type = tarfile.SYMTYPE
                info.linkname = "/bin/sh"
                bundle.addfile(info)
            for name, data in members if members is not None else [(artifact, b"\x7fELF fake")]:
                info = tarfile.TarInfo(name)
                info.size = len(data)
                info.mode = 0o755
                bundle.addfile(info, io.BytesIO(data))
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        (directory / f"{path.name}.sha256").write_bytes(f"{digest}  {path.name}\n".encode("ascii"))
        return path

    def run_check(
        self,
        archive: Path,
        transcripts: dict[str, str] | None = None,
        machine: str = "aarch64",
        pull_exit: int = 0,
        run_exit: int = 0,
    ) -> subprocess.CompletedProcess[str]:
        environment = dict(os.environ)
        environment.update(
            {
                "FAKE_DOCKER_LOG": str(self.log),
                "FAKE_TRANSCRIPTS": json.dumps(transcripts or {"default": transcript()}),
                "FAKE_PULL_EXIT": str(pull_exit),
                "FAKE_RUN_EXIT": str(run_exit),
            }
        )
        return subprocess.run(
            [
                sys.executable,
                "-I",
                str(SCRIPT),
                "check",
                "--archive",
                str(archive),
                "--expect-version",
                "6.2.0",
                "--docker",
                str(self.docker),
                "--machine",
                machine,
            ],
            capture_output=True,
            text=True,
            env=environment,
            timeout=60,
            check=False,
        )

    def docker_calls(self) -> list[list[str]]:
        if not self.log.exists():
            return []
        return [json.loads(line) for line in self.log.read_text(encoding="utf-8").splitlines()]

    def test_glibc_archive_passes_on_both_floor_images_in_isolation(self) -> None:
        result = self.run_check(self.archive("specsync-linux-aarch64"))
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(result.stdout)
        self.assertTrue(report["passed"])
        runs_ = [call for call in self.docker_calls() if call[0] == "run"]
        refs = [call[-3] for call in runs_]
        self.assertEqual(refs, [BASELINE.IMAGES[name]["ref"] for name in BASELINE.GLIBC_IMAGES])
        for call in runs_:
            joined = " ".join(call)
            for flag in ("--network none", "--read-only", "--user 65534:65534", "--cap-drop ALL", "--pull never"):
                self.assertIn(flag, joined)
            mounts = [call[index + 1] for index, value in enumerate(call) if value == "--mount"]
            self.assertEqual(len(mounts), 3)
            self.assertTrue(all(mount.endswith(",readonly") for mount in mounts))

    def test_musl_archive_runs_on_all_four_images(self) -> None:
        result = self.run_check(self.archive("specsync-linux-aarch64-musl"))
        self.assertEqual(result.returncode, 0, result.stderr)
        refs = [call[-3] for call in self.docker_calls() if call[0] == "run"]
        self.assertEqual(refs, [BASELINE.IMAGES[name]["ref"] for name in BASELINE.MUSL_IMAGES])

    def test_loader_failure_fails_and_names_the_rebuild(self) -> None:
        result = self.run_check(self.archive("specsync-linux-aarch64"), {"default": LOADER_FAILURE})
        self.assertEqual(result.returncode, 1)
        self.assertIn("does not start", result.stderr)
        self.assertIn("GLIBC_2.39", result.stderr)
        self.assertIn("cargo build --release --target aarch64-unknown-linux-gnu", result.stderr)
        self.assertIn("ubuntu-22.04-arm runner", result.stderr)
        self.assertFalse(json.loads(result.stdout)["passed"])

    def test_failure_on_one_image_fails_the_archive(self) -> None:
        result = self.run_check(
            self.archive("specsync-linux-aarch64"),
            {"default": transcript(), "debian:12-slim": LOADER_FAILURE},
        )
        self.assertEqual(result.returncode, 1)
        self.assertIn("debian:12-slim", result.stderr)
        self.assertNotIn("ubuntu:22.04 (", result.stderr)

    def test_wrong_version_fails(self) -> None:
        result = self.run_check(self.archive("specsync-linux-aarch64"), {"default": transcript(version=(0, "specsync 6.1.2"))})
        self.assertEqual(result.returncode, 1)
        self.assertIn("expected 'specsync 6.2.0'", result.stderr)

    def test_help_failure_fails(self) -> None:
        result = self.run_check(self.archive("specsync-linux-aarch64"), {"default": transcript(help=(2, "error"))})
        self.assertEqual(result.returncode, 1)
        self.assertIn("--help exited 2", result.stderr)

    def test_failed_check_on_documented_project_fails(self) -> None:
        result = self.run_check(self.archive("specsync-linux-aarch64"), {"default": transcript(check=(1, "1 failed"))})
        self.assertEqual(result.returncode, 1)
        self.assertIn("on a documented project", result.stderr)

    def test_undetected_drift_fails(self) -> None:
        result = self.run_check(self.archive("specsync-linux-aarch64"), {"default": transcript(drift=(0, "1 passed"))})
        self.assertEqual(result.returncode, 1)
        self.assertIn("must fail and name 'goodbye'", result.stderr)

    def test_truncated_run_fails(self) -> None:
        truncated = "@@step version 0\n@@out specsync 6.2.0\n"
        result = self.run_check(self.archive("specsync-linux-aarch64"), {"default": truncated}, run_exit=137)
        self.assertEqual(result.returncode, 1)
        self.assertIn("stopped before help, init, check, drift", result.stderr)

    def test_unpullable_image_fails_after_three_attempts(self) -> None:
        result = self.run_check(self.archive("specsync-linux-aarch64"), pull_exit=1)
        self.assertEqual(result.returncode, 1)
        self.assertIn("after 3 attempts", result.stderr)
        pulls = [call for call in self.docker_calls() if call[0] == "pull"]
        self.assertEqual(len(pulls), 3 * len(BASELINE.GLIBC_IMAGES))

    def test_architecture_mismatch_is_refused_with_the_runner(self) -> None:
        result = self.run_check(self.archive("specsync-linux-aarch64"), machine="x86_64")
        self.assertEqual(result.returncode, 2)
        self.assertIn("ubuntu-22.04-arm", result.stderr)
        self.assertEqual(self.docker_calls(), [])

    def test_x86_64_archive_accepts_amd64_host(self) -> None:
        result = self.run_check(self.archive("specsync-linux-x86_64"), machine="AMD64")
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_checksum_mismatch_is_refused(self) -> None:
        archive = self.archive("specsync-linux-aarch64")
        sidecar = archive.with_name(archive.name + ".sha256")
        sidecar.write_bytes(f"{'0' * 64}  {archive.name}\n".encode("ascii"))
        result = self.run_check(archive)
        self.assertEqual(result.returncode, 2)
        self.assertIn("rebuild or re-download", result.stderr)
        self.assertEqual(self.docker_calls(), [])

    def test_crlf_or_misnamed_sidecar_is_refused(self) -> None:
        archive = self.archive("specsync-linux-aarch64")
        sidecar = archive.with_name(archive.name + ".sha256")
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        for content in (f"{digest}  {archive.name}\r\n", f"{digest}  other.tar.gz\n", f"{digest} {archive.name}\n"):
            with self.subTest(content=content):
                sidecar.write_bytes(content.encode("ascii"))
                self.assertEqual(self.run_check(archive).returncode, 2)

    def test_missing_sidecar_is_refused(self) -> None:
        archive = self.archive("specsync-linux-aarch64")
        archive.with_name(archive.name + ".sha256").unlink()
        result = self.run_check(archive)
        self.assertEqual(result.returncode, 2)
        self.assertIn("missing", result.stderr)

    def test_missing_archive_is_refused(self) -> None:
        result = self.run_check(self.root / "dist" / "specsync-linux-aarch64.tar.gz")
        self.assertEqual(result.returncode, 2)
        self.assertIn("does not exist", result.stderr)

    def test_archive_with_extra_or_wrong_members_is_refused(self) -> None:
        cases = {
            "extra": [("specsync-linux-aarch64", b"x"), ("README", b"y")],
            "wrong-name": [("specsync", b"x")],
            "nested": [("./specsync-linux-aarch64", b"x")],
            "empty": [],
        }
        for label, members in cases.items():
            with self.subTest(case=label):
                result = self.run_check(self.archive("specsync-linux-aarch64", members=members))
                self.assertEqual(result.returncode, 2, result.stderr)

    def test_symlink_member_is_refused(self) -> None:
        result = self.run_check(self.archive("specsync-linux-aarch64", members=[], symlink=True))
        self.assertEqual(result.returncode, 2)
        self.assertIn("regular file", result.stderr)

    def test_unknown_artifact_is_refused(self) -> None:
        for name in ("specsync-macos-aarch64", "specsync-linux-riscv64"):
            with self.subTest(name=name):
                self.assertEqual(self.run_check(self.archive(name)).returncode, 2)


class TranscriptParserTests(unittest.TestCase):
    def test_outputs_are_grouped_per_step(self) -> None:
        steps = BASELINE.parse_transcript(transcript())
        self.assertEqual(list(steps), list(BASELINE.STEPS))
        self.assertEqual(steps["version"]["output"], "specsync 6.2.0")
        self.assertEqual(steps["drift"]["exit"], 1)

    def test_noise_before_the_first_marker_is_ignored(self) -> None:
        steps = BASELINE.parse_transcript("@@out stray\nnoise\n" + transcript())
        self.assertEqual(steps["version"]["output"], "specsync 6.2.0")


if __name__ == "__main__":
    unittest.main(verbosity=2)
