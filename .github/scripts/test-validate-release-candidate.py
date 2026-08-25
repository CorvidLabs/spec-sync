#!/usr/bin/env python3
"""Fixture-driven tests for the immutable release-candidate validator."""

from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest
from typing import Any


sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
VALIDATOR = ROOT / ".github/scripts/validate-release-candidate.py"
SPEC = importlib.util.spec_from_file_location("validate_release_candidate", VALIDATOR)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("could not load release-candidate validator")
VALIDATOR_MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = VALIDATOR_MODULE
SPEC.loader.exec_module(VALIDATOR_MODULE)


def git(repository: Path, *arguments: str) -> str:
    """Run one local Git fixture command."""
    return subprocess.check_output(
        ["git", *arguments],
        cwd=repository,
        text=True,
    ).strip()


class ReleaseRepository:
    """A temporary repository with one package and immutable candidate commit."""

    def __init__(self, version: str = "1.2.3") -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        git(self.root, "init", "-b", "main")
        git(self.root, "config", "user.email", "release-test@specsync.dev")
        git(self.root, "config", "user.name", "SpecSync Release Test")
        (self.root / "Cargo.toml").write_text(
            f'[package]\nname = "fixture"\nversion = "{version}"\n',
            encoding="utf-8",
        )
        (self.root / "README.md").write_text("candidate\n", encoding="utf-8")
        git(self.root, "add", "Cargo.toml", "README.md")
        git(self.root, "commit", "-m", "candidate")
        self.candidate_sha = git(self.root, "rev-parse", "HEAD")
        self.rc_tag = f"v{version}-rc.1"
        self.final_tag = f"v{version}"

    def close(self) -> None:
        """Release the temporary repository."""
        self.temporary.cleanup()

    def annotate_rc(self, target: str | None = None) -> None:
        """Create the canonical annotated marker."""
        git(
            self.root,
            "tag",
            "-a",
            self.rc_tag,
            target or self.candidate_sha,
            "-m",
            "release candidate",
        )

    def write_evidence(self, overrides: dict[str, dict[str, Any]] | None = None) -> list[Path]:
        """Write one record per required platform and return their paths."""
        overrides = overrides or {}
        paths: list[Path] = []
        for platform in VALIDATOR_MODULE.REQUIRED_PLATFORMS:
            payload: dict[str, Any] = {
                "schema_version": 1,
                "platform": platform,
                "outcome": "success",
                "rc_tag": self.rc_tag,
                "candidate_sha": self.candidate_sha,
                "lane": "release-candidate",
            }
            payload.update(overrides.get(platform, {}))
            path = self.root / f"{platform}.json"
            path.write_text(json.dumps(payload), encoding="utf-8")
            paths.append(path)
        return paths

    def run_cli(self, *arguments: str) -> subprocess.CompletedProcess[str]:
        """Run the validator as an external CLI without network access."""
        return subprocess.run(
            [sys.executable, str(VALIDATOR), *arguments],
            cwd=self.root,
            capture_output=True,
            text=True,
            check=False,
        )

    def write_artifacts(self) -> Path:
        """Write the exact downloaded release-artifact topology."""
        artifact_root = self.root / "artifacts"
        artifact_root.mkdir(exist_ok=True)
        for artifact, archive_name in VALIDATOR_MODULE.EXPECTED_ARTIFACT_ARCHIVES.items():
            directory = artifact_root / artifact
            directory.mkdir(exist_ok=True)
            archive = directory / archive_name
            archive.write_bytes(f"packaged {artifact}\n".encode("utf-8"))
            digest = hashlib.sha256(archive.read_bytes()).hexdigest()
            (directory / f"{archive_name}.sha256").write_bytes(
                f"{digest}  {archive_name}\n".encode("ascii")
            )
            provenance = {
                "archive": archive_name,
                "artifact": artifact,
                "candidate_sha": self.candidate_sha,
                "rc_tag": self.rc_tag,
                "schema_version": 1,
                "sha256": digest,
            }
            (directory / f"{artifact}.provenance.json").write_text(
                json.dumps(provenance, indent=2, sort_keys=True) + "\n",
                encoding="utf-8",
            )
        return artifact_root

    def common_evidence_arguments(self, evidence: list[Path]) -> list[str]:
        """Return shared promote/release CLI arguments."""
        arguments = [
            "--repository",
            str(self.root),
            "--rc-tag",
            self.rc_tag,
            "--candidate-sha",
            self.candidate_sha,
            "--final-tag",
            self.final_tag,
            "--lane",
            "release-candidate",
        ]
        for path in evidence:
            arguments.extend(["--evidence", str(path)])
        return arguments


class ReleaseCandidateTagTests(unittest.TestCase):
    """Pure tag and package contract tests."""

    def test_parse_rc_tag_accepts_canonical_marker(self) -> None:
        tag = VALIDATOR_MODULE.parse_rc_tag("v6.0.0-rc.12")
        self.assertEqual(tag.version, "6.0.0")
        self.assertEqual(tag.iteration, 12)
        self.assertEqual(tag.final_tag, "v6.0.0")

    def test_parse_rc_tag_rejects_noncanonical_markers(self) -> None:
        malformed = (
            "6.0.0-rc.1",
            "v6.0.0",
            "v6.0.0-rc.0",
            "v6.0.0-rc.01",
            "v06.0.0-rc.1",
            "v6.0-rc.1",
            "v6.0.0-RC.1",
            "v6.0.0-rc.1-extra",
        )
        for value in malformed:
            with self.subTest(value=value):
                with self.assertRaises(VALIDATOR_MODULE.ValidationError):
                    VALIDATOR_MODULE.parse_rc_tag(value)

    def test_candidate_cli_resolves_annotated_tag_and_package(self) -> None:
        fixture = ReleaseRepository()
        self.addCleanup(fixture.close)
        fixture.annotate_rc()
        result = fixture.run_cli(
            "candidate",
            "--repository",
            str(fixture.root),
            "--rc-tag",
            fixture.rc_tag,
            "--candidate-sha",
            fixture.candidate_sha,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["candidate_sha"], fixture.candidate_sha)
        self.assertEqual(payload["final_tag"], fixture.final_tag)
        self.assertTrue(payload["valid"])

    def test_candidate_rejects_lightweight_or_missing_tag(self) -> None:
        fixture = ReleaseRepository()
        self.addCleanup(fixture.close)
        git(fixture.root, "tag", fixture.rc_tag, fixture.candidate_sha)
        lightweight = fixture.run_cli(
            "candidate",
            "--repository",
            str(fixture.root),
            "--rc-tag",
            fixture.rc_tag,
        )
        self.assertNotEqual(lightweight.returncode, 0)
        self.assertIn("must be an annotated tag", lightweight.stderr)
        git(fixture.root, "tag", "-d", fixture.rc_tag)
        missing = fixture.run_cli(
            "candidate",
            "--repository",
            str(fixture.root),
            "--rc-tag",
            fixture.rc_tag,
        )
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("does not exist", missing.stderr)

    def test_candidate_rejects_package_or_expected_sha_mismatch(self) -> None:
        fixture = ReleaseRepository(version="1.2.4")
        self.addCleanup(fixture.close)
        fixture.rc_tag = "v1.2.3-rc.1"
        fixture.annotate_rc()
        package = fixture.run_cli(
            "candidate",
            "--repository",
            str(fixture.root),
            "--rc-tag",
            fixture.rc_tag,
        )
        self.assertNotEqual(package.returncode, 0)
        self.assertIn("does not match package version", package.stderr)

        fixture.rc_tag = "v1.2.4-rc.1"
        fixture.annotate_rc()
        wrong_sha = fixture.run_cli(
            "candidate",
            "--repository",
            str(fixture.root),
            "--rc-tag",
            fixture.rc_tag,
            "--candidate-sha",
            "b" * 40,
        )
        self.assertNotEqual(wrong_sha.returncode, 0)
        self.assertIn("expected candidate", wrong_sha.stderr)


class PlatformEvidenceTests(unittest.TestCase):
    """Pure platform evidence binding and failure-mode tests."""

    def setUp(self) -> None:
        self.fixture = ReleaseRepository()
        self.addCleanup(self.fixture.close)
        self.fixture.annotate_rc()
        tag = VALIDATOR_MODULE.parse_rc_tag(self.fixture.rc_tag)
        self.identity = VALIDATOR_MODULE.CandidateIdentity(
            tag=tag,
            candidate_sha=self.fixture.candidate_sha,
            package_version="1.2.3",
        )

    def load(self, paths: list[Path]) -> list[Any]:
        """Load fixture records with the validator parser."""
        return [VALIDATOR_MODULE.load_platform_evidence(path) for path in paths]

    def test_all_platforms_green_for_same_identity_pass(self) -> None:
        revision = VALIDATOR_MODULE.validate_platform_evidence(
            self.load(self.fixture.write_evidence()),
            self.identity,
            "release-candidate",
        )
        self.assertIsNone(revision)

    def test_missing_and_duplicate_platforms_fail(self) -> None:
        records = self.load(self.fixture.write_evidence())
        with self.assertRaisesRegex(
            VALIDATOR_MODULE.ValidationError, "missing platform evidence: windows"
        ):
            VALIDATOR_MODULE.validate_platform_evidence(
                records[:-1], self.identity, "release-candidate"
            )
        with self.assertRaisesRegex(
            VALIDATOR_MODULE.ValidationError, "duplicate platform evidence for ubuntu"
        ):
            VALIDATOR_MODULE.validate_platform_evidence(
                [records[0], *records], self.identity, "release-candidate"
            )

    def test_failed_or_cancelled_platform_fails(self) -> None:
        for outcome in ("failure", "cancelled"):
            with self.subTest(outcome=outcome):
                records = self.load(
                    self.fixture.write_evidence({"windows": {"outcome": outcome}})
                )
                with self.assertRaisesRegex(
                    VALIDATOR_MODULE.ValidationError,
                    f"platform windows is not successful: outcome='{outcome}'",
                ):
                    VALIDATOR_MODULE.validate_platform_evidence(
                        records, self.identity, "release-candidate"
                    )

    def test_changed_candidate_and_mixed_tag_fail(self) -> None:
        records = self.load(
            self.fixture.write_evidence(
                {
                    "macos": {"candidate_sha": "b" * 40},
                }
            )
        )
        with self.assertRaisesRegex(
            VALIDATOR_MODULE.ValidationError, "platform macos evidence is bound to candidate"
        ):
            VALIDATOR_MODULE.validate_platform_evidence(
                records, self.identity, "release-candidate"
            )

        records = self.load(
            self.fixture.write_evidence({"windows": {"rc_tag": "v1.2.3-rc.2"}})
        )
        with self.assertRaisesRegex(
            VALIDATOR_MODULE.ValidationError, "platform windows evidence is bound to RC tag"
        ):
            VALIDATOR_MODULE.validate_platform_evidence(
                records, self.identity, "release-candidate"
            )

    def test_mixed_revision_package_or_lane_fails(self) -> None:
        cases = (
            (
                {
                    "ubuntu": {"workflow_revision": "b" * 40},
                    "macos": {"workflow_revision": "a" * 40},
                    "windows": {"workflow_revision": "a" * 40},
                },
                "mixed across workflow revisions",
            ),
            ({"macos": {"package_version": "1.2.4"}}, "package version"),
            ({"windows": {"lane": "different-lane"}}, "ran lane"),
        )
        for overrides, message in cases:
            with self.subTest(message=message):
                records = self.load(self.fixture.write_evidence(overrides))
                with self.assertRaisesRegex(VALIDATOR_MODULE.ValidationError, message):
                    VALIDATOR_MODULE.validate_platform_evidence(
                        records, self.identity, "release-candidate"
                    )

    def test_duplicate_json_key_is_rejected(self) -> None:
        path = self.fixture.root / "duplicate.json"
        path.write_text(
            '{"schema_version":1,"platform":"ubuntu","platform":"windows"}',
            encoding="utf-8",
        )
        with self.assertRaisesRegex(VALIDATOR_MODULE.ValidationError, "duplicate key 'platform'"):
            VALIDATOR_MODULE.load_platform_evidence(path)

    def test_unknown_fields_and_partial_optional_identity_fail(self) -> None:
        unknown = self.fixture.write_evidence({"ubuntu": {"unexpected": True}})
        with self.assertRaisesRegex(VALIDATOR_MODULE.ValidationError, "unknown fields: unexpected"):
            self.load(unknown)

        partial = self.load(
            self.fixture.write_evidence({"ubuntu": {"workflow_revision": "a" * 40}})
        )
        with self.assertRaisesRegex(
            VALIDATOR_MODULE.ValidationError,
            "mixes present and missing workflow revisions",
        ):
            VALIDATOR_MODULE.validate_platform_evidence(
                partial,
                self.identity,
                "release-candidate",
            )

    def test_stable_evidence_cli_contract_and_help(self) -> None:
        evidence = self.fixture.write_evidence()
        help_result = self.fixture.run_cli("evidence", "--help")
        self.assertEqual(help_result.returncode, 0, help_result.stderr)
        self.assertIn("--rc-tag", help_result.stdout)
        self.assertIn("--candidate-sha", help_result.stdout)
        self.assertIn("--evidence-dir", help_result.stdout)

        result = self.fixture.run_cli(
            "evidence",
            "--rc-tag",
            self.fixture.rc_tag,
            "--candidate-sha",
            self.fixture.candidate_sha,
            "--evidence-dir",
            str(self.fixture.root),
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["mode"], "evidence")
        self.assertEqual(payload["platforms"], ["ubuntu", "macos", "windows"])
        self.assertEqual(len(evidence), 3)

    def test_evidence_cli_refuses_missing_extra_failed_or_mixed_records(self) -> None:
        paths = self.fixture.write_evidence()
        paths[-1].unlink()
        missing = self.fixture.run_cli(
            "evidence",
            "--rc-tag",
            self.fixture.rc_tag,
            "--candidate-sha",
            self.fixture.candidate_sha,
            "--evidence-dir",
            str(self.fixture.root),
        )
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("missing windows.json", missing.stderr)

        self.fixture.write_evidence()
        (self.fixture.root / "linux.json").write_text("{}", encoding="utf-8")
        extra = self.fixture.run_cli(
            "evidence",
            "--rc-tag",
            self.fixture.rc_tag,
            "--candidate-sha",
            self.fixture.candidate_sha,
            "--evidence-dir",
            str(self.fixture.root),
        )
        self.assertNotEqual(extra.returncode, 0)
        self.assertIn("unexpected linux.json", extra.stderr)
        (self.fixture.root / "linux.json").unlink()

        self.fixture.write_evidence({"windows": {"outcome": "failure"}})
        failed = self.fixture.run_cli(
            "evidence",
            "--rc-tag",
            self.fixture.rc_tag,
            "--candidate-sha",
            self.fixture.candidate_sha,
            "--evidence-dir",
            str(self.fixture.root),
        )
        self.assertNotEqual(failed.returncode, 0)
        self.assertIn("platform windows is not successful", failed.stderr)

        self.fixture.write_evidence({"macos": {"candidate_sha": "b" * 40}})
        mixed = self.fixture.run_cli(
            "evidence",
            "--rc-tag",
            self.fixture.rc_tag,
            "--candidate-sha",
            self.fixture.candidate_sha,
            "--evidence-dir",
            str(self.fixture.root),
        )
        self.assertNotEqual(mixed.returncode, 0)
        self.assertIn("platform macos evidence is bound to candidate", mixed.stderr)

        wrong_lane = self.fixture.write_evidence(
            {
                "ubuntu": {"lane": "test"},
                "macos": {"lane": "test"},
                "windows": {"lane": "test"},
            }
        )
        self.assertEqual(len(wrong_lane), 3)
        lane = self.fixture.run_cli(
            "evidence",
            "--rc-tag",
            self.fixture.rc_tag,
            "--candidate-sha",
            self.fixture.candidate_sha,
            "--evidence-dir",
            str(self.fixture.root),
        )
        self.assertNotEqual(lane.returncode, 0)
        self.assertIn("ran lane 'test', expected 'release-candidate'", lane.stderr)


class RulesetValidationTests(unittest.TestCase):
    """Fail-closed tag-immutability ruleset contract tests."""

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.final_immutability_ruleset_path = self.root / "final-immutability-ruleset.json"
        self.rc_ruleset_path = self.root / "rc-ruleset.json"
        self.valid_final_immutability_ruleset: dict[str, Any] = {
            "id": 44,
            "node_id": "RRS_final_immutability_fixture",
            "name": "SpecSync immutable final tags",
            "target": "tag",
            "source_type": "Repository",
            "source": "CorvidLabs/spec-sync",
            "enforcement": "active",
            "current_user_can_bypass": "never",
            "bypass_actors": [],
            "conditions": {
                "ref_name": {
                    "include": ["refs/tags/v*.*.*"],
                    "exclude": ["refs/tags/v*.*.*-rc.*"],
                }
            },
            "rules": [
                {"type": "update"},
                {"type": "deletion"},
            ],
            "_links": {
                "self": {"href": "https://api.github.com/repos/CorvidLabs/spec-sync/rulesets/44"},
                "html": {"href": "https://github.com/CorvidLabs/spec-sync/rules/44"},
            },
            "created_at": "2026-08-01T00:00:00Z",
            "updated_at": "2026-08-01T00:00:00Z",
        }
        self.valid_rc_ruleset: dict[str, Any] = {
            "id": 43,
            "node_id": "RRS_rc_fixture",
            "name": "SpecSync immutable RC tags",
            "target": "tag",
            "source_type": "Repository",
            "source": "CorvidLabs/spec-sync",
            "enforcement": "active",
            "current_user_can_bypass": "never",
            "bypass_actors": [],
            "conditions": {
                "ref_name": {
                    "include": ["refs/tags/v*.*.*-rc.*"],
                    "exclude": [],
                }
            },
            "rules": [
                {"type": "update"},
                {"type": "deletion"},
            ],
            "_links": {
                "self": {"href": "https://api.github.com/repos/CorvidLabs/spec-sync/rulesets/43"},
                "html": {"href": "https://github.com/CorvidLabs/spec-sync/rules/43"},
            },
            "created_at": "2026-08-01T00:00:00Z",
            "updated_at": "2026-08-01T00:00:00Z",
        }

    def write_rulesets(
        self,
        final_immutability_payload: Any | None = None,
        rc_payload: Any | None = None,
    ) -> None:
        """Write both deterministic ruleset fixtures."""
        final_immutability_value = (
            self.valid_final_immutability_ruleset
            if final_immutability_payload is None
            else final_immutability_payload
        )
        rc_value = self.valid_rc_ruleset if rc_payload is None else rc_payload
        self.final_immutability_ruleset_path.write_text(
            json.dumps(final_immutability_value, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        self.rc_ruleset_path.write_text(
            json.dumps(rc_value, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

    def run_rulesets(self, *arguments: str) -> subprocess.CompletedProcess[str]:
        """Run the stable two-ruleset workflow command."""
        return subprocess.run(
            [
                sys.executable,
                str(VALIDATOR),
                "rulesets",
                "--final-immutability-ruleset-json",
                str(self.final_immutability_ruleset_path),
                "--rc-ruleset-json",
                str(self.rc_ruleset_path),
                *arguments,
            ],
            cwd=self.root,
            capture_output=True,
            text=True,
            check=False,
        )

    def assert_payload_rejected(
        self,
        policy: str,
        payload: dict[str, Any],
        message: str,
    ) -> None:
        """Require one pure ruleset fixture to fail with a useful diagnostic."""
        validator = {
            "final_immutability": VALIDATOR_MODULE.validate_final_tag_immutability_ruleset,
            "rc": VALIDATOR_MODULE.validate_rc_tag_ruleset,
        }[policy]
        with self.assertRaises(VALIDATOR_MODULE.ValidationError) as raised:
            validator(payload)
        self.assertIn(message, str(raised.exception))

    def test_rulesets_cli_accepts_exact_active_repository_policies(self) -> None:
        self.write_rulesets()
        help_result = subprocess.run(
            [sys.executable, str(VALIDATOR), "rulesets", "--help"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(help_result.returncode, 0, help_result.stderr)
        self.assertIn("--final-immutability-ruleset-json", help_result.stdout)
        self.assertIn("--rc-ruleset-json", help_result.stdout)
        self.assertNotIn("--final-creation-ruleset-json", help_result.stdout)
        self.assertNotIn("--release-app-id", help_result.stdout)

        result = self.run_rulesets()
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertTrue(payload["valid"])
        self.assertEqual(payload["mode"], "rulesets")
        self.assertEqual(payload["final_immutability"]["bypass_actors"], [])
        self.assertEqual(
            payload["final_immutability"]["rules"],
            ["update", "deletion"],
        )
        self.assertEqual(
            payload["final_immutability"]["ref_includes"],
            ["refs/tags/v*.*.*"],
        )
        self.assertEqual(
            payload["final_immutability"]["ref_excludes"],
            ["refs/tags/v*.*.*-rc.*"],
        )
        self.assertEqual(payload["rc"]["bypass_actors"], [])
        self.assertEqual(payload["rc"]["rules"], ["update", "deletion"])
        self.assertEqual(payload["rc"]["ref_includes"], ["refs/tags/v*.*.*-rc.*"])
        self.assertNotIn("final_creation", payload)

    def test_successful_rulesets_declare_every_unenforced_tag_protection(self) -> None:
        """A green ruleset check must still say what it does not verify.

        Dropping the App-only creation policy is allowed; dropping it silently is not. If this
        list is ever emptied, `release.yml` fails rather than reporting a clean bill of health it
        cannot back up.
        """
        self.write_rulesets()
        result = self.run_rulesets()
        self.assertEqual(result.returncode, 0, result.stderr)
        unenforced = json.loads(result.stdout)["unenforced"]
        self.assertIsInstance(unenforced, list)
        self.assertEqual(len(unenforced), 3)
        self.assertEqual(unenforced, list(VALIDATOR_MODULE.UNENFORCED_TAG_POLICIES))
        joined = "\n".join(unenforced)
        self.assertIn("SpecSync final tag creation", joined)
        self.assertIn("release GitHub App", joined)
        # Naming GITHUB_TOKEN is the point: the tag now comes from the same permission that runs
        # the workflow, and a reader who is told only "creation is unrestricted" would not learn
        # that dispatching a release IS the release authority.
        self.assertIn("GITHUB_TOKEN", joined)
        self.assertIn("deployment-environment gate", joined)
        for entry in unenforced:
            with self.subTest(entry=entry):
                self.assertIn("NOT", entry)

    def test_rulesets_cli_requires_both_inputs_and_rejects_retired_flags(self) -> None:
        self.write_rulesets()
        missing_final = subprocess.run(
            [
                sys.executable,
                str(VALIDATOR),
                "rulesets",
                "--rc-ruleset-json",
                str(self.rc_ruleset_path),
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertNotEqual(missing_final.returncode, 0)
        self.assertIn("--final-immutability-ruleset-json", missing_final.stderr)

        missing_rc = subprocess.run(
            [
                sys.executable,
                str(VALIDATOR),
                "rulesets",
                "--final-immutability-ruleset-json",
                str(self.final_immutability_ruleset_path),
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertNotEqual(missing_rc.returncode, 0)
        self.assertIn("--rc-ruleset-json", missing_rc.stderr)

        # The App-only creation policy is gone from the contract, not merely unused: passing its
        # retired flags must fail loudly so a stale caller cannot believe it is still checked.
        for flag, value in (
            ("--release-app-id", "1234567"),
            ("--final-creation-ruleset-json", str(self.final_immutability_ruleset_path)),
        ):
            with self.subTest(retired=flag):
                result = self.run_rulesets(flag, value)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("unrecognized arguments", result.stderr)

        legacy = subprocess.run(
            [sys.executable, str(VALIDATOR), "ruleset", "--ruleset-json", "ruleset.json"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertNotEqual(legacy.returncode, 0)
        self.assertIn("invalid choice", legacy.stderr)

        environment = subprocess.run(
            [sys.executable, str(VALIDATOR), "environment", "--default-branch", "main"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertNotEqual(environment.returncode, 0)
        self.assertIn("invalid choice", environment.stderr)

    def test_rulesets_reject_wrong_identity_or_inactive_enforcement(self) -> None:
        cases = (
            ("target", "branch", "target"),
            ("source_type", "Organization", "source_type"),
            ("enforcement", "evaluate", "enforcement"),
        )
        fixtures = (
            (
                "final_immutability",
                self.valid_final_immutability_ruleset,
                "SpecSync final tag creation",
            ),
            ("rc", self.valid_rc_ruleset, "SpecSync immutable final tags"),
        )
        for policy, fixture, wrong_name in fixtures:
            with self.subTest(policy=policy, field="name"):
                payload = copy.deepcopy(fixture)
                payload["name"] = wrong_name
                self.assert_payload_rejected(policy, payload, "name")
            for field, value, message in cases:
                with self.subTest(policy=policy, field=field):
                    payload = copy.deepcopy(fixture)
                    payload[field] = value
                    self.assert_payload_rejected(policy, payload, message)

        malformed_bypass = copy.deepcopy(self.valid_final_immutability_ruleset)
        malformed_bypass["current_user_can_bypass"] = True
        self.assert_payload_rejected(
            "final_immutability",
            malformed_bypass,
            "recognized GitHub bypass mode",
        )

    def test_rulesets_reject_broadened_overlapping_or_ambiguous_ref_conditions(self) -> None:
        final_cases = (
            ("include", ["refs/tags/v6.*"], "include patterns must be exactly"),
            (
                "include",
                ["refs/tags/v*.*.*", "refs/tags/v6.*"],
                "include patterns must be exactly",
            ),
            ("exclude", [], "exclude patterns must be exactly"),
            (
                "exclude",
                ["refs/tags/v*.*.*-rc.*", "refs/tags/v6.0.0"],
                "exclude patterns must be exactly",
            ),
        )
        for field, value, message in final_cases:
            with self.subTest(policy="final_immutability", field=field, value=value):
                payload = copy.deepcopy(self.valid_final_immutability_ruleset)
                payload["conditions"]["ref_name"][field] = value
                self.assert_payload_rejected("final_immutability", payload, message)

        rc_cases = (
            ("include", ["refs/tags/v*.*.*"], "include patterns must be exactly"),
            (
                "include",
                ["refs/tags/v*.*.*-rc.*", "refs/tags/v*.*.*"],
                "include patterns must be exactly",
            ),
            ("exclude", ["refs/tags/v6.0.0-rc.*"], "exclude patterns must be exactly"),
        )
        for field, value, message in rc_cases:
            with self.subTest(policy="rc", field=field, value=value):
                payload = copy.deepcopy(self.valid_rc_ruleset)
                payload["conditions"]["ref_name"][field] = value
                self.assert_payload_rejected("rc", payload, message)

        for policy, fixture in (
            ("final_immutability", self.valid_final_immutability_ruleset),
            ("rc", self.valid_rc_ruleset),
        ):
            duplicate = copy.deepcopy(fixture)
            duplicate["conditions"]["ref_name"]["include"] *= 2
            self.assert_payload_rejected(policy, duplicate, "must not contain duplicates")

            unknown = copy.deepcopy(fixture)
            unknown["conditions"]["repository_name"] = {"include": ["spec-sync"]}
            self.assert_payload_rejected(policy, unknown, "conditions has unknown fields")

    def test_rulesets_reject_missing_duplicate_unknown_or_parameterized_rules(self) -> None:
        immutability_cases = (
            ([{"type": "deletion"}], "must be exactly"),
            ([{"type": "update"}], "must be exactly"),
            (
                [{"type": "update"}, {"type": "deletion"}, {"type": "deletion"}],
                "must not contain duplicates",
            ),
            (
                [{"type": "creation"}, {"type": "update"}, {"type": "deletion"}],
                "must be exactly",
            ),
        )
        for policy, fixture in (
            ("final_immutability", self.valid_final_immutability_ruleset),
            ("rc", self.valid_rc_ruleset),
        ):
            for rules, message in immutability_cases:
                with self.subTest(policy=policy, rules=rules):
                    payload = copy.deepcopy(fixture)
                    payload["rules"] = rules
                    self.assert_payload_rejected(policy, payload, message)

            parameters = copy.deepcopy(fixture)
            parameters["rules"][0]["parameters"] = {"unexpected": True}
            self.assert_payload_rejected(policy, parameters, "has unknown fields")

    def test_immutability_rulesets_reject_every_bypass_actor(self) -> None:
        """No actor may move or delete a tag — the validator admits no exception at all.

        This is the protection that survived dropping the App-only creation policy, so it must not
        acquire a bypass escape hatch on the way through.
        """
        for policy, fixture in (
            ("final_immutability", self.valid_final_immutability_ruleset),
            ("rc", self.valid_rc_ruleset),
        ):
            for actor_type in ("Integration", "User", "RepositoryRole", "OrganizationAdmin"):
                with self.subTest(policy=policy, actor_type=actor_type):
                    payload = copy.deepcopy(fixture)
                    payload["bypass_actors"] = [
                        {
                            "actor_id": 1234567,
                            "actor_type": actor_type,
                            "bypass_mode": "always",
                        }
                    ]
                    self.assert_payload_rejected(
                        policy,
                        payload,
                        "must not grant bypass to any actor",
                    )

            malformed = copy.deepcopy(fixture)
            malformed["bypass_actors"] = {}
            self.assert_payload_rejected(policy, malformed, "must be an array")

    def test_rulesets_reject_unknown_duplicate_malformed_and_oversized_json(self) -> None:
        for policy, fixture in (
            ("final_immutability", self.valid_final_immutability_ruleset),
            ("rc", self.valid_rc_ruleset),
        ):
            unknown = copy.deepcopy(fixture)
            unknown["administrator_bypass"] = True
            self.assert_payload_rejected(policy, unknown, "has unknown fields")

            with self.subTest(policy=policy, shape="duplicate"):
                self.write_rulesets()
                raw = json.dumps(fixture)
                path = {
                    "final_immutability": self.final_immutability_ruleset_path,
                    "rc": self.rc_ruleset_path,
                }[policy]
                path.write_text('{"name":"duplicate",' + raw[1:], encoding="utf-8")
                duplicate = self.run_rulesets()
                self.assertNotEqual(duplicate.returncode, 0)
                self.assertIn("duplicate key 'name'", duplicate.stderr)

            with self.subTest(policy=policy, shape="non-object"):
                if policy == "final_immutability":
                    self.write_rulesets(final_immutability_payload=[])
                else:
                    self.write_rulesets(rc_payload=[])
                malformed = self.run_rulesets()
                self.assertNotEqual(malformed.returncode, 0)
                self.assertIn("must be a JSON object", malformed.stderr)

            with self.subTest(policy=policy, shape="oversized"):
                self.write_rulesets()
                path = {
                    "final_immutability": self.final_immutability_ruleset_path,
                    "rc": self.rc_ruleset_path,
                }[policy]
                path.write_bytes(b"x" * (VALIDATOR_MODULE.RULESET_MAX_BYTES + 1))
                oversized = self.run_rulesets()
                self.assertNotEqual(oversized.returncode, 0)
                self.assertIn("exceeds the", oversized.stderr)

    def test_rulesets_reject_duplicate_identity_or_symlinked_input(self) -> None:
        duplicate_id = copy.deepcopy(self.valid_rc_ruleset)
        duplicate_id["id"] = self.valid_final_immutability_ruleset["id"]
        self.write_rulesets(rc_payload=duplicate_id)
        result = self.run_rulesets()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("must have distinct ids", result.stderr)

        for policy, fixture in (
            ("final_immutability", self.valid_final_immutability_ruleset),
            ("rc", self.valid_rc_ruleset),
        ):
            with self.subTest(policy=policy):
                self.write_rulesets()
                path = {
                    "final_immutability": self.final_immutability_ruleset_path,
                    "rc": self.rc_ruleset_path,
                }[policy]
                target = self.root / f"{policy}-target.json"
                target.write_text(json.dumps(fixture), encoding="utf-8")
                path.unlink()
                try:
                    path.symlink_to(target)
                except OSError as error:
                    self.skipTest(f"symlinks unavailable for fixture: {error}")
                result = self.run_rulesets()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("regular, non-symlink file", result.stderr)
                path.unlink()


class ArtifactValidationTests(unittest.TestCase):
    """Downloaded release archive, checksum, and provenance contract tests."""

    def setUp(self) -> None:
        self.fixture = ReleaseRepository()
        self.addCleanup(self.fixture.close)
        self.artifact_root = self.fixture.write_artifacts()

    def run_artifacts(self) -> subprocess.CompletedProcess[str]:
        """Run the stable artifacts workflow command."""
        return self.fixture.run_cli(
            "artifacts",
            "--candidate-sha",
            self.fixture.candidate_sha,
            "--rc-tag",
            self.fixture.rc_tag,
            "--artifact-dir",
            str(self.artifact_root),
        )

    def paths(self, artifact: str) -> tuple[Path, Path, Path]:
        """Return archive, checksum, and provenance fixture paths."""
        archive_name = VALIDATOR_MODULE.EXPECTED_ARTIFACT_ARCHIVES[artifact]
        directory = self.artifact_root / artifact
        return (
            directory / archive_name,
            directory / f"{archive_name}.sha256",
            directory / f"{artifact}.provenance.json",
        )

    def test_artifacts_cli_accepts_exact_downloaded_release_payload(self) -> None:
        help_result = self.fixture.run_cli("artifacts", "--help")
        self.assertEqual(help_result.returncode, 0, help_result.stderr)
        self.assertIn("--candidate-sha", help_result.stdout)
        self.assertIn("--rc-tag", help_result.stdout)
        self.assertIn("--artifact-dir", help_result.stdout)

        result = self.run_artifacts()
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertTrue(payload["valid"])
        self.assertEqual(payload["candidate_sha"], self.fixture.candidate_sha)
        self.assertEqual(len(payload["artifacts"]), 6)

    def test_artifacts_cli_rejects_tampered_archive_and_mixed_candidate(self) -> None:
        archive, _, _ = self.paths("specsync-linux-x86_64")
        archive.write_bytes(archive.read_bytes() + b"tampered")
        tampered = self.run_artifacts()
        self.assertNotEqual(tampered.returncode, 0)
        self.assertIn("archive digest mismatch", tampered.stderr)

        self.fixture.write_artifacts()
        _, _, provenance = self.paths("specsync-macos-aarch64")
        payload = json.loads(provenance.read_text(encoding="utf-8"))
        payload["candidate_sha"] = "b" * 40
        provenance.write_text(json.dumps(payload), encoding="utf-8")
        mixed = self.run_artifacts()
        self.assertNotEqual(mixed.returncode, 0)
        self.assertIn("is bound to candidate", mixed.stderr)

        self.fixture.write_artifacts()
        payload = json.loads(provenance.read_text(encoding="utf-8"))
        payload["rc_tag"] = "v1.2.3-rc.2"
        provenance.write_text(json.dumps(payload), encoding="utf-8")
        mixed_tag = self.run_artifacts()
        self.assertNotEqual(mixed_tag.returncode, 0)
        self.assertIn("is bound to RC tag", mixed_tag.stderr)

    def test_artifacts_cli_rejects_missing_or_extra_payload_files(self) -> None:
        _, checksum, _ = self.paths("specsync-linux-aarch64")
        checksum.unlink()
        missing = self.run_artifacts()
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn(f"missing {checksum.name}", missing.stderr)

        self.fixture.write_artifacts()
        _, _, provenance = self.paths("specsync-linux-aarch64")
        provenance.unlink()
        missing_manifest = self.run_artifacts()
        self.assertNotEqual(missing_manifest.returncode, 0)
        self.assertIn(f"missing {provenance.name}", missing_manifest.stderr)

        self.fixture.write_artifacts()
        directory = self.artifact_root / "specsync-macos-x86_64"
        (directory / "duplicate.provenance.json").write_text("{}", encoding="utf-8")
        extra = self.run_artifacts()
        self.assertNotEqual(extra.returncode, 0)
        self.assertIn("unexpected duplicate.provenance.json", extra.stderr)

    def test_artifacts_cli_rejects_bad_checksum_bytes_and_wrong_names(self) -> None:
        artifact = "specsync-linux-x86_64-musl"
        archive, checksum, provenance = self.paths(artifact)
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        checksum.write_bytes(f"{digest}  {archive.name}\r\n".encode("ascii"))
        bad_checksum = self.run_artifacts()
        self.assertNotEqual(bad_checksum.returncode, 0)
        self.assertIn("exact LF-only SHA-256 record", bad_checksum.stderr)

        self.fixture.write_artifacts()
        payload = json.loads(provenance.read_text(encoding="utf-8"))
        payload["archive"] = "renamed.tar.gz"
        provenance.write_text(json.dumps(payload), encoding="utf-8")
        wrong_name = self.run_artifacts()
        self.assertNotEqual(wrong_name.returncode, 0)
        self.assertIn("names archive 'renamed.tar.gz'", wrong_name.stderr)

        self.fixture.write_artifacts()
        payload = json.loads(provenance.read_text(encoding="utf-8"))
        payload["artifact"] = "specsync-linux-renamed"
        provenance.write_text(json.dumps(payload), encoding="utf-8")
        wrong_artifact = self.run_artifacts()
        self.assertNotEqual(wrong_artifact.returncode, 0)
        self.assertIn("names artifact 'specsync-linux-renamed'", wrong_artifact.stderr)

    def test_artifacts_cli_rejects_unknown_and_duplicate_manifest_fields(self) -> None:
        artifact = "specsync-windows-x86_64.exe"
        _, _, provenance = self.paths(artifact)
        payload = json.loads(provenance.read_text(encoding="utf-8"))
        payload["unknown"] = True
        provenance.write_text(json.dumps(payload), encoding="utf-8")
        unknown = self.run_artifacts()
        self.assertNotEqual(unknown.returncode, 0)
        self.assertIn("unknown fields: unknown", unknown.stderr)

        self.fixture.write_artifacts()
        raw = provenance.read_text(encoding="utf-8").rstrip()
        duplicate = raw[:-1] + f', "sha256": {json.dumps(payload["sha256"])}' + "}\n"
        provenance.write_text(duplicate, encoding="utf-8")
        duplicated = self.run_artifacts()
        self.assertNotEqual(duplicated.returncode, 0)
        self.assertIn("duplicate key 'sha256'", duplicated.stderr)

    def test_artifacts_cli_rejects_symlinked_payload(self) -> None:
        archive, _, _ = self.paths("specsync-macos-x86_64")
        target = self.fixture.root / "outside-archive"
        target.write_bytes(archive.read_bytes())
        archive.unlink()
        try:
            archive.symlink_to(target)
        except OSError as error:
            self.skipTest(f"symlinks unavailable for fixture: {error}")
        result = self.run_artifacts()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("regular, non-symlink file", result.stderr)


class WorkflowSourceContractTests(unittest.TestCase):
    """Narrow regressions for the CHG-0075 workflow contracts."""

    # test_release_reconstruction_requires_actual_pull_request_event was removed
    # with the code it described. It pinned one property of `validate`'s
    # archive-binding reconstruction — that the workflow run behind the binding
    # check was reached via `pull_request` and never `pull_request_target`.
    # That reconstruction is gone (#635): its input, the `SpecSync archive
    # binding` check run, has had no producer since #499 deleted
    # post-merge-archive.yml, so the block it guarded was unreachable. A test
    # whose subject no longer exists cannot fail, and keeping it would only
    # assert something about a string that is not in the file.

    def test_development_and_release_platform_topology(self) -> None:
        continuous_integration = (ROOT / ".github/workflows/ci.yml").read_text(
            encoding="utf-8"
        )
        test_start = continuous_integration.index("\n  test:\n")
        test_end = continuous_integration.index("\n  fmt:\n", test_start)
        test_job = continuous_integration[test_start:test_end]
        self.assertRegex(test_job, r"(?m)^    runs-on: ubuntu-latest$")
        self.assertNotRegex(test_job, r"(?m)^\s+matrix:")

        release = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        qualify_start = release.index("\n  qualify:\n")
        qualify_end = release.index("\n  record-qualification:\n", qualify_start)
        qualify_job = release[qualify_start:qualify_end]
        for platform in VALIDATOR_MODULE.REQUIRED_PLATFORMS:
            with self.subTest(platform=platform):
                self.assertEqual(qualify_job.count(f"          - platform: {platform}\n"), 1)
        invocation = "fledge lanes run release-candidate"
        self.assertEqual(qualify_job.count(invocation), 1)
        self.assertEqual(release.count(invocation), 1)

    def test_release_entrypoint_is_rc_only_and_never_cancels_in_progress(self) -> None:
        release = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        trigger_start = release.index("\non:\n")
        trigger_end = release.index("\npermissions:\n", trigger_start)
        triggers = release[trigger_start:trigger_end]
        rc_pattern = "      - 'v[0-9]+.[0-9]+.[0-9]+-rc.[0-9]+'"
        stable_pattern = "      - 'v[0-9]+.[0-9]+.[0-9]+'"
        self.assertEqual(triggers.count(rc_pattern), 1)
        self.assertNotIn(stable_pattern, triggers)

        concurrency_start = release.index("\nconcurrency:\n")
        concurrency_end = release.index("\njobs:\n", concurrency_start)
        concurrency = release[concurrency_start:concurrency_end]
        self.assertEqual(concurrency.count("  cancel-in-progress: false"), 1)
        self.assertNotIn("cancel-in-progress: true", concurrency)

        resolve_start = release.index("\n  resolve:\n")
        resolve_end = release.index("\n  validate:\n", resolve_start)
        resolve_job = release[resolve_start:resolve_end]
        self.assertNotIn('mode="release"', resolve_job)
        self.assertNotIn("needs.resolve.outputs.mode == 'release'", release)
        self.assertEqual(resolve_job.count("Promotion must be dispatched from"), 1)
        self.assertEqual(resolve_job.count("github.workflow_ref"), 1)
        self.assertEqual(resolve_job.count("github.event.repository.default_branch"), 1)

    def test_release_queries_and_validates_exactly_the_two_immutability_rulesets(self) -> None:
        release = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        start = release.index('          rulesets_file="${RUNNER_TEMP}/specsync-rulesets.json"')
        end = release.index("\n\n          # A deleted/recreated", start)
        rulesets = release[start:end]

        self.assertEqual(
            rulesets.count('"repos/${REPOSITORY}/rulesets?includes_parents=true"'),
            1,
        )
        self.assertEqual(
            rulesets.count('"repos/${REPOSITORY}/rulesets/${ruleset_ids[0]}?includes_parents=true"'),
            1,
        )
        expected_queries = (
            'resolve_ruleset "SpecSync immutable final tags" "$final_immutability_ruleset_file"',
            'resolve_ruleset "SpecSync immutable RC tags" "$rc_ruleset_file"',
        )
        for query in expected_queries:
            with self.subTest(query=query):
                self.assertEqual(rulesets.count(query), 1)
        self.assertEqual(rulesets.count("          resolve_ruleset \""), 2)
        self.assertNotIn("SpecSync immutable release tags", rulesets)

        self.assertEqual(
            rulesets.count("validate-release-candidate.py rulesets \\\n"),
            1,
        )
        self.assertEqual(
            rulesets.count(
                '--final-immutability-ruleset-json "$final_immutability_ruleset_file"'
            ),
            1,
        )
        self.assertEqual(rulesets.count('--rc-ruleset-json "$rc_ruleset_file"'), 1)

        # The App-only creation policy and the `release` environment are gone from the whole
        # workflow, not merely from qualification. The owner decided against a release App, so
        # every reference to one is retired: a half-removed App reads as a policy that is only
        # temporarily off, and the next reader provisions a variable instead of learning that
        # promotion is now the workflow's own token. Demanding these is also what failed rc.1
        # through rc.6.
        for retired in (
            'resolve_ruleset "SpecSync final tag creation"',
            "--final-creation-ruleset-json",
            "--release-app-id",
            '"repos/${REPOSITORY}/environments/release"',
            "validate-release-candidate.py environment",
            "SPECSYNC_RELEASE_APP_ID",
            "SPECSYNC_RELEASE_APP_PRIVATE_KEY",
            "create-github-app-token",
        ):
            with self.subTest(retired=retired):
                self.assertNotIn(retired, release)

    def test_release_announces_every_unenforced_tag_protection_on_every_run(self) -> None:
        """A green release run must annotate what it did not verify.

        Without this, a passing `resolve` reads as proof that App-only final-tag creation is
        enforced. It is not enforced, and nobody may be allowed to infer that it is.
        """
        release = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        start = release.index('          rulesets_file="${RUNNER_TEMP}/specsync-rulesets.json"')
        end = release.index("\n\n          # A deleted/recreated", start)
        rulesets = release[start:end]

        self.assertEqual(
            rulesets.count('rulesets_result_file="${RUNNER_TEMP}/specsync-rulesets-result.json"'),
            1,
        )
        self.assertEqual(
            rulesets.count('--rc-ruleset-json "$rc_ruleset_file" > "$rulesets_result_file"'),
            1,
        )
        self.assertEqual(
            rulesets.count("""unenforced_count="$(jq -r '.unenforced | length' """),
            1,
        )
        self.assertEqual(
            rulesets.count('if [[ "$unenforced_count" -lt 1 ]]; then'),
            1,
        )
        self.assertEqual(
            rulesets.count(
                "::error::Ruleset validation must declare the protections it does not enforce"
            ),
            1,
        )
        self.assertEqual(
            rulesets.count(
                '::warning title=Release protection not enforced::${unenforced}'
            ),
            1,
        )
        self.assertEqual(
            rulesets.count("""done < <(jq -r '.unenforced[]' "$rulesets_result_file")"""),
            1,
        )
        self.assertEqual(rulesets.count('>> "$GITHUB_STEP_SUMMARY"'), 2)

    def test_release_actions_are_sha_pinned_and_evidence_upload_overwrites(self) -> None:
        release = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        action_references = re.findall(r"(?m)^\s*- uses:\s+(\S+)", release)
        self.assertTrue(action_references)
        for reference in action_references:
            with self.subTest(reference=reference):
                if reference.startswith("./"):
                    continue
                self.assertRegex(reference, r"^[^@\s]+@[0-9a-f]{40}$")

        upload_start = release.index("      - name: Upload platform evidence\n")
        upload_end = release.index("\n  record-qualification:\n", upload_start)
        upload = release[upload_start:upload_end]
        self.assertEqual(upload.count("          overwrite: true"), 1)

    def test_promotion_mints_the_final_tag_with_the_workflow_token_alone(self) -> None:
        """Promotion writes the final tag with GITHUB_TOKEN, scoped to this one job.

        The release GitHub App is gone by owner decision, and this pins what replaced it. Two
        properties matter and both are weaker than the design they replace, so both are asserted
        rather than assumed:

        1. Write access is granted on `promote` and nowhere else. A workflow-wide
           `contents: write` would hand every other job in this file the ability to move refs.
        2. No `environment:` is named. GitHub materializes a referenced environment on first use
           with no protection rules, so naming the `release` environment that this repository does
           not have would publish a deployment gate that gates nothing.
        """
        release = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        promote_start = release.index("\n  promote:\n")
        promote_end = release.index("\n  build:\n", promote_start)
        promote = release[promote_start:promote_end]

        self.assertEqual(promote.count("    permissions:\n"), 1)
        self.assertEqual(promote.count("      contents: write\n"), 1)
        self.assertNotRegex(promote, r"(?m)^      contents: read$")

        # Workflow-level defaults stay read-only; only `promote` and `release` may write, and each
        # asks for it on its own job.
        header = release[: release.index("\njobs:\n")]
        self.assertIn("permissions:\n  contents: read\n", header)
        self.assertNotIn("contents: write", header)
        self.assertEqual(release.count("      contents: write\n"), 2)

        self.assertNotRegex(promote, r"(?m)^    environment:$")
        self.assertNotIn("environment:\n      name: release", promote)

        for retired in (
            "actions/create-github-app-token",
            "SPECSYNC_RELEASE_APP_ID",
            "SPECSYNC_RELEASE_APP_PRIVATE_KEY",
            "permission-contents: write",
            "release-app-token",
        ):
            with self.subTest(retired=retired):
                self.assertNotIn(retired, promote)

        self.assertEqual(
            promote.count("RELEASE_TOKEN: ${{ github.token }}"),
            1,
        )
        self.assertEqual(promote.count("          persist-credentials: false\n"), 1)
        self.assertEqual(
            promote.count('release_remote="https://github.com/${REPOSITORY}.git"'),
            1,
        )
        self.assertEqual(
            promote.count('push "$release_remote" "refs/tags/${FINAL_TAG}"'),
            1,
        )
        for authenticated_command in (
            "git_release ls-remote",
            "git_release fetch",
            "git_release push",
        ):
            with self.subTest(authenticated_command=authenticated_command):
                self.assertEqual(promote.count(authenticated_command), 1)
        self.assertNotIn('git push origin "refs/tags/${FINAL_TAG}"', promote)
        self.assertNotIn("SPECSYNC_RELEASE_TAG_KEY", promote)
        self.assertNotIn("git@github.com", promote)

    def test_promotion_states_who_can_now_create_a_release_tag(self) -> None:
        """The authority that was given up must be readable at the job that gave it up.

        `resolve` annotates it on every run, but a reader auditing `promote` reads the job, not a
        run log. If this comment is deleted, the file stops saying that running the release lane
        and holding release authority are now the same permission.
        """
        release = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        promote_start = release.index("  # WHO CAN MINT A RELEASE TAG")
        promote_end = release.index("\n  build:\n", promote_start)
        promote = release[promote_start:promote_end]

        for stated in (
            "THE PROTECTION THAT WAS GIVEN UP",
            "NO `environment:` HERE, DELIBERATELY",
            "can run `release.yml` from the default branch",
        ):
            with self.subTest(stated=stated):
                self.assertIn(stated, promote)

    def test_validator_tests_are_wired_through_fledge_and_ci(self) -> None:
        test_path = ".github/scripts/test-validate-release-candidate.py"
        fledge = (ROOT / "fledge.toml").read_text(encoding="utf-8")
        task_pattern = re.compile(
            r"(?ms)^\[tasks\.([A-Za-z0-9_-]+)\]\n"
            r"((?:(?!^\[).)*?^cmd\s*=\s*\"[^\"]*"
            + re.escape(test_path)
            + r"[^\"]*\"\s*$)"
        )
        task = task_pattern.search(fledge)
        self.assertIsNotNone(task, "Fledge must define the release-candidate validator test task")
        if task is None:
            return
        task_name = task.group(1)
        continuous_integration = (ROOT / ".github/workflows/ci.yml").read_text(
            encoding="utf-8"
        )
        ci_commands = (
            f"fledge run {task_name}",
            f"python3 {test_path}",
        )
        self.assertTrue(
            any(command in continuous_integration for command in ci_commands),
            "CI must run the same validator tests exposed by Fledge",
        )

    def test_release_revalidates_fresh_tags_head_and_evidence_before_upload(self) -> None:
        release = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        release_start = release.rindex("\n  release:\n")
        release_job = release[release_start:]
        upload_start = release_job.index("      - name: Create release\n")
        pre_upload = release_job[:upload_start]
        self.assertIn("git fetch --force origin", pre_upload)
        self.assertIn("refs/tags/${RC_TAG}", pre_upload)
        self.assertIn("refs/tags/${FINAL_TAG}", pre_upload)
        self.assertIn("rc-evidence-*", pre_upload)
        self.assertIn("validate-release-candidate.py release", pre_upload)
        self.assertIn('--checkout-sha "$(git rev-parse HEAD)"', pre_upload)
        self.assertIn("--evidence", pre_upload)

    def test_authorization_uses_the_newest_exact_check_before_testing_success(self) -> None:
        release = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        authorize_start = release.index("\n  authorize-release:\n")
        authorize_end = release.index("\n  promote:\n", authorize_start)
        authorize = release[authorize_start:authorize_end]
        selection_start = authorize.index("              [\n")
        selection_end = authorize.index("            ' \"$checks_file\"", selection_start)
        selection = authorize[selection_start:selection_end]
        self.assertIn("| sort_by(.id)\n              | last", selection)
        self.assertNotIn(".conclusion == \"success\"", selection)
        newest_index = authorize.index("| last", selection_start)
        success_index = authorize.index(
            "'.status == \"completed\" and .conclusion == \"success\"'",
            newest_index,
        )
        self.assertLess(newest_index, success_index)
        self.assertIn("The newest exact RC gate is not successful", authorize)

    def test_check_writer_isolates_candidate_validation_from_the_write_token(self) -> None:
        release = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        record_start = release.index("\n  record-qualification:\n")
        record_end = release.index("\n  authorize-release:\n", record_start)
        record = release[record_start:record_end]
        publish_start = record.index("      - name: Publish immutable candidate gate\n")
        validation = record[:publish_start]
        publication = record[publish_start:]
        self.assertEqual(record.count("          persist-credentials: false\n"), 1)
        self.assertEqual(
            validation.count("PYTHONNOUSERSITE=1 python3 -I "),
            1,
        )
        self.assertNotIn("GH_TOKEN:", validation)
        self.assertEqual(publication.count("          GH_TOKEN: ${{ github.token }}\n"), 1)

    def test_release_reruns_refuse_asset_overwrite_and_isolate_validation(self) -> None:
        release = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
        release_start = release.rindex("\n  release:\n")
        release_job = release[release_start:]
        refuse_index = release_job.index("      - name: Refuse an existing release\n")
        create_index = release_job.index("      - name: Create release\n")
        self.assertLess(refuse_index, create_index)
        self.assertNotIn("gh release view", release_job)
        self.assertEqual(release_job.count("            gh api --include \\\n"), 1)
        self.assertIn('if [[ "$release_status" == "200" ]]', release_job)
        self.assertIn('if [[ "$release_status" == "404" ]]', release_job)
        self.assertIn("Cannot prove release ${FINAL_TAG} is absent", release_job)
        self.assertEqual(release_job.count("          overwrite_files: false\n"), 1)
        self.assertEqual(release_job.count("          fail_on_unmatched_files: true\n"), 1)
        self.assertEqual(release_job.count("          persist-credentials: false\n"), 1)
        self.assertGreaterEqual(
            release_job.count("PYTHONNOUSERSITE=1 python3 -I "),
            2,
        )
        self.assertEqual(release_job.count("          unset GH_TOKEN\n"), 1)


class PromotionAndReleaseTests(unittest.TestCase):
    """End-to-end promotion and final publication refusal tests."""

    def setUp(self) -> None:
        self.fixture = ReleaseRepository()
        self.addCleanup(self.fixture.close)
        self.fixture.annotate_rc()

    def test_promotion_passes_before_final_tag_creation(self) -> None:
        evidence = self.fixture.write_evidence()
        result = self.fixture.run_cli(
            "promote", *self.fixture.common_evidence_arguments(evidence)
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["mode"], "promotion")
        self.assertEqual(payload["candidate_sha"], self.fixture.candidate_sha)
        self.assertEqual(payload["platforms"], ["ubuntu", "macos", "windows"])

    def test_promotion_refuses_wrong_or_existing_final_tag(self) -> None:
        evidence = self.fixture.write_evidence()
        wrong_arguments = self.fixture.common_evidence_arguments(evidence)
        wrong_arguments[wrong_arguments.index(self.fixture.final_tag)] = "v1.2.4"
        wrong = self.fixture.run_cli("promote", *wrong_arguments)
        self.assertNotEqual(wrong.returncode, 0)
        self.assertIn("does not match RC promotion target", wrong.stderr)

        git(self.fixture.root, "tag", "-a", self.fixture.final_tag, "-m", "final")
        existing = self.fixture.run_cli(
            "promote", *self.fixture.common_evidence_arguments(evidence)
        )
        self.assertNotEqual(existing.returncode, 0)
        self.assertIn("already exists", existing.stderr)

    def test_moved_rc_marker_cannot_reuse_old_evidence(self) -> None:
        evidence = self.fixture.write_evidence()
        (self.fixture.root / "README.md").write_text("changed candidate\n", encoding="utf-8")
        git(self.fixture.root, "add", "README.md")
        git(self.fixture.root, "commit", "-m", "changed candidate")
        changed_sha = git(self.fixture.root, "rev-parse", "HEAD")
        git(self.fixture.root, "tag", "-d", self.fixture.rc_tag)
        self.fixture.annotate_rc(changed_sha)
        result = self.fixture.run_cli(
            "promote", *self.fixture.common_evidence_arguments(evidence)
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("expected candidate", result.stderr)

    def test_release_revalidates_final_tag_checkout_and_evidence(self) -> None:
        evidence = self.fixture.write_evidence()
        git(self.fixture.root, "tag", "-a", self.fixture.final_tag, "-m", "final")
        arguments = self.fixture.common_evidence_arguments(evidence)
        arguments.extend(["--checkout-sha", self.fixture.candidate_sha])
        result = self.fixture.run_cli("release", *arguments)
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["mode"], "release")
        self.assertEqual(payload["checkout_sha"], self.fixture.candidate_sha)

    def test_release_refuses_lightweight_final_tag(self) -> None:
        evidence = self.fixture.write_evidence()
        git(self.fixture.root, "tag", self.fixture.final_tag, self.fixture.candidate_sha)
        arguments = self.fixture.common_evidence_arguments(evidence)
        arguments.extend(["--checkout-sha", self.fixture.candidate_sha])
        result = self.fixture.run_cli("release", *arguments)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("must be an annotated tag", result.stderr)

    def test_release_refuses_changed_final_tag_or_checkout(self) -> None:
        evidence = self.fixture.write_evidence()
        (self.fixture.root / "README.md").write_text("different final\n", encoding="utf-8")
        git(self.fixture.root, "add", "README.md")
        git(self.fixture.root, "commit", "-m", "different final")
        changed_sha = git(self.fixture.root, "rev-parse", "HEAD")
        git(
            self.fixture.root,
            "tag",
            "-a",
            self.fixture.final_tag,
            changed_sha,
            "-m",
            "wrong final",
        )
        arguments = self.fixture.common_evidence_arguments(evidence)
        arguments.extend(["--checkout-sha", self.fixture.candidate_sha])
        wrong_tag = self.fixture.run_cli("release", *arguments)
        self.assertNotEqual(wrong_tag.returncode, 0)
        self.assertIn("expected candidate", wrong_tag.stderr)

        git(self.fixture.root, "tag", "-d", self.fixture.final_tag)
        git(
            self.fixture.root,
            "tag",
            "-a",
            self.fixture.final_tag,
            self.fixture.candidate_sha,
            "-m",
            "final",
        )
        checkout_arguments = self.fixture.common_evidence_arguments(evidence)
        checkout_arguments.extend(["--checkout-sha", changed_sha])
        wrong_checkout = self.fixture.run_cli("release", *checkout_arguments)
        self.assertNotEqual(wrong_checkout.returncode, 0)
        self.assertIn("release checkout", wrong_checkout.stderr)

    def test_release_refuses_missing_or_mixed_evidence(self) -> None:
        evidence = self.fixture.write_evidence()
        git(self.fixture.root, "tag", "-a", self.fixture.final_tag, "-m", "final")
        missing_arguments = self.fixture.common_evidence_arguments(evidence[:-1])
        missing_arguments.extend(["--checkout-sha", self.fixture.candidate_sha])
        missing = self.fixture.run_cli("release", *missing_arguments)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("missing platform evidence: windows", missing.stderr)

        mixed = self.fixture.write_evidence(
            {
                "ubuntu": {"workflow_revision": "a" * 40},
                "macos": {"workflow_revision": "b" * 40},
                "windows": {"workflow_revision": "a" * 40},
            }
        )
        mixed_arguments = self.fixture.common_evidence_arguments(mixed)
        mixed_arguments.extend(["--checkout-sha", self.fixture.candidate_sha])
        mixed_result = self.fixture.run_cli("release", *mixed_arguments)
        self.assertNotEqual(mixed_result.returncode, 0)
        self.assertIn("mixed across workflow revisions", mixed_result.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
