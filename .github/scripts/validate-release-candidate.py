#!/usr/bin/env python3
"""Validate immutable SpecSync release-candidate identity and promotion evidence."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
from typing import Any, Iterable, Sequence


# Windows is neither built nor qualified as of 6.0 (#735). The retained `#[cfg(windows)]`
# content code is best-effort and unverified — see docs/ci-confidence.md. Re-adding a
# platform here without also adding it to the `qualify` matrix fails every candidate.
REQUIRED_PLATFORMS = ("ubuntu", "macos")
RELEASE_CANDIDATE_LANE = "release-candidate"
RC_TAG_PATTERN = re.compile(
    r"^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)-rc\.([1-9][0-9]*)$"
)
VERSION_PATTERN = re.compile(r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$")
SHA_PATTERN = re.compile(r"^[0-9a-f]{40}$")
LANE_PATTERN = re.compile(r"^[A-Za-z0-9._/-]{1,128}$")
REQUIRED_EVIDENCE_FIELDS = (
    "schema_version",
    "platform",
    "outcome",
    "rc_tag",
    "candidate_sha",
    "lane",
)
OPTIONAL_EVIDENCE_FIELDS = (
    "package_version",
    "workflow_revision",
)
EXPECTED_ARTIFACT_ARCHIVES = {
    "specsync-linux-x86_64": "specsync-linux-x86_64.tar.gz",
    "specsync-linux-x86_64-musl": "specsync-linux-x86_64-musl.tar.gz",
    "specsync-linux-aarch64": "specsync-linux-aarch64.tar.gz",
    "specsync-macos-x86_64": "specsync-macos-x86_64.tar.gz",
    "specsync-macos-aarch64": "specsync-macos-aarch64.tar.gz",
}
PROVENANCE_FIELDS = (
    "schema_version",
    "artifact",
    "archive",
    "candidate_sha",
    "rc_tag",
    "sha256",
)
DIGEST_PATTERN = re.compile(r"^[0-9a-f]{64}$")
RULESET_MAX_BYTES = 1024 * 1024
FINAL_IMMUTABILITY_RULESET_NAME = "SpecSync immutable final tags"
FINAL_RULESET_REF_PATTERN = "refs/tags/v*.*.*"
FINAL_RULESET_EXCLUDE_PATTERN = "refs/tags/v*.*.*-rc.*"
RC_RULESET_NAME = "SpecSync immutable RC tags"
RC_RULESET_REF_PATTERN = "refs/tags/v*.*.*-rc.*"
REQUIRED_RULESET_FIELDS = (
    "name",
    "target",
    "source_type",
    "enforcement",
    "conditions",
    "rules",
)
OPTIONAL_RULESET_FIELDS = (
    # GitHub returns `bypass_actors` only to a caller with admin access to repository settings.
    # A workflow using GITHUB_TOKEN is not one, so the field is ABSENT rather than empty there —
    # a distinction that cost a release lane its green run, because requiring it made the gate
    # impossible to satisfy from inside CI. Absent means UNOBSERVED, not "no bypass actors": it is
    # validated when visible and reported as unverified when not.
    "bypass_actors",
    "id",
    "node_id",
    "source",
    "current_user_can_bypass",
    "_links",
    "created_at",
    "updated_at",
)
FINAL_IMMUTABILITY_RULES = ("update", "deletion")
RC_TAG_RULES = ("update", "deletion")

# Protections this repository does NOT have, stated by `rulesets` on every successful run.
#
# The design in REQ-github-007 also called for a `SpecSync final tag creation` ruleset naming a
# dedicated release GitHub App as its only bypass actor, and a protected `release` deployment
# environment holding that App's private key. Neither was ever provisioned: no App was created, the
# App id variable and private-key secret were never set, and there is no `release` environment.
# Demanding them failed `release.yml` on every RC tag from v6.0.0-rc.1 through rc.6, which is the
# same as having no release gate at all — the two rulesets that DO exist were never checked because
# the job died before reaching them.
#
# The owner then decided against a release App altogether. `promote` now creates the final tag with
# the workflow's own GITHUB_TOKEN under a job-scoped `contents: write`, and no longer names a
# deployment environment, so both halves of the original creation policy are gone rather than
# pending. Dropping a protection is allowed; dropping it quietly is not. Every green `rulesets` run
# therefore carries this list, and `release.yml` prints each entry as a warning annotation and into
# the step summary, so a passing release can never be read as proof of an authority nobody enforces.
def unobserved_bypass_notices(results: dict[str, Any]) -> list[str]:
    """Name every ruleset whose bypass list this token could not see.

    GitHub returns `bypass_actors` only to a caller with admin access to repository settings, and
    a workflow using GITHUB_TOKEN is not one. Absence therefore means UNOBSERVED, never "no bypass
    actors" — reading it as the latter would let a ruleset that grants bypass pass a green run.
    """
    return [
        f"Bypass actors on the {label} ruleset were NOT verified: this token cannot read "
        f"`bypass_actors`, which GitHub returns only to repository administrators. A green run "
        f"is not evidence that the ruleset grants bypass to nobody. Check it in repository "
        f"settings, or run this validation with a credential that can see the field."
        for label, result in sorted(results.items())
        if isinstance(result, dict) and result.get("bypass_actors") is None
    ]


UNENFORCED_TAG_POLICIES = (
    "Final-tag creation is NOT restricted to a release GitHub App. This repository has no "
    "'SpecSync final tag creation' ruleset, so any actor with tag-write access can create "
    "refs/tags/vX.Y.Z directly, without a qualified candidate and without this workflow. "
    "A green release run is not evidence that a final tag came from qualification.",
    "The final tag is minted by this workflow's own GITHUB_TOKEN, NOT by a separate release "
    "identity. There is no App whose key a workflow author cannot reach, so anyone able to run "
    "release.yml from the default branch can cause refs/tags/vX.Y.Z to be created. Running the "
    "release lane and holding release authority are the same permission here.",
    "Promotion is NOT behind a deployment-environment gate. This workflow names no environment, "
    "so no required reviewer, wait timer, or branch policy stands between dispatching a promotion "
    "and the final tag being written. Nothing in a green run proves that a human other than the "
    "dispatcher approved it.",
)


class ValidationError(ValueError):
    """A deterministic, user-correctable release-candidate validation error."""


@dataclass(frozen=True)
class ReleaseCandidateTag:
    """The parsed immutable RC marker."""

    raw: str
    version: str
    iteration: int

    @property
    def final_tag(self) -> str:
        """Return the stable tag produced by promotion."""
        return f"v{self.version}"


@dataclass(frozen=True)
class CandidateIdentity:
    """The exact commit and package identity captured by an annotated RC tag."""

    tag: ReleaseCandidateTag
    candidate_sha: str
    package_version: str


@dataclass(frozen=True)
class PlatformEvidence:
    """One platform's release-candidate qualification result."""

    platform: str
    outcome: str
    rc_tag: str
    candidate_sha: str
    lane: str
    package_version: str | None
    workflow_revision: str | None


@dataclass(frozen=True)
class ArtifactProvenance:
    """The immutable identity and digest for one packaged release archive."""

    artifact: str
    archive: str
    candidate_sha: str
    rc_tag: str
    sha256: str


def parse_rc_tag(value: str) -> ReleaseCandidateTag:
    """Parse a canonical vX.Y.Z-rc.N tag without accepting aliases or leading zeroes."""
    match = RC_TAG_PATTERN.fullmatch(value)
    if match is None:
        raise ValidationError(
            f"invalid RC tag {value!r}; expected canonical vX.Y.Z-rc.N with N >= 1"
        )
    major, minor, patch, iteration = match.groups()
    return ReleaseCandidateTag(
        raw=value,
        version=f"{major}.{minor}.{patch}",
        iteration=int(iteration),
    )


def validate_full_sha(value: str, label: str) -> str:
    """Require a lowercase, full Git SHA-1 commit identity."""
    if SHA_PATTERN.fullmatch(value) is None:
        raise ValidationError(f"{label} must be an exact lowercase 40-character Git SHA")
    return value


def read_package_version(path: Path) -> str:
    """Read the root [package] version without requiring a third-party TOML parser."""
    try:
        content = path.read_text(encoding="utf-8")
    except OSError as error:
        raise ValidationError(f"cannot read package manifest {path}: {error}") from error
    package = re.search(r"(?ms)^\[package\]\s*$\n(.*?)(?=^\[|\Z)", content)
    if package is None:
        raise ValidationError(f"package manifest {path} has no [package] table")
    versions = re.findall(
        r'^version\s*=\s*["\']([^"\']+)["\']\s*(?:#.*)?$',
        package.group(1),
        flags=re.MULTILINE,
    )
    if len(versions) != 1:
        raise ValidationError(
            f"package manifest {path} must contain exactly one [package] version"
        )
    version = versions[0]
    if VERSION_PATTERN.fullmatch(version) is None:
        raise ValidationError(
            f"package version {version!r} must be a canonical X.Y.Z release version"
        )
    return version


def run_git(repository: Path, *arguments: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    """Run one bounded local Git query without consulting the network."""
    try:
        result = subprocess.run(
            ["git", *arguments],
            cwd=repository,
            capture_output=True,
            text=True,
            check=False,
            timeout=30,
        )
    except FileNotFoundError as error:
        raise ValidationError("git is required for release-candidate validation") from error
    except subprocess.TimeoutExpired as error:
        raise ValidationError(f"git {' '.join(arguments)} timed out") from error
    if check and result.returncode != 0:
        detail = result.stderr.strip().splitlines()
        suffix = f": {detail[-1]}" if detail else ""
        raise ValidationError(f"git {' '.join(arguments)} failed{suffix}")
    return result


def annotated_tag_headers(repository: Path, reference: str) -> dict[str, str]:
    """Return unique annotated-tag headers, rejecting ambiguous tag objects."""
    payload = run_git(repository, "cat-file", "-p", reference).stdout
    headers: dict[str, str] = {}
    for line in payload.splitlines():
        if not line:
            break
        key, separator, value = line.partition(" ")
        if separator != " " or key not in {"object", "type", "tag", "tagger"}:
            continue
        if key in headers:
            raise ValidationError(f"annotated tag {reference} contains duplicate {key!r} headers")
        headers[key] = value
    return headers


def resolve_annotated_rc_tag(repository: Path, value: str) -> tuple[ReleaseCandidateTag, str]:
    """Resolve an exact annotated RC tag that points directly to one commit."""
    tag = parse_rc_tag(value)
    reference = f"refs/tags/{tag.raw}"
    object_type = run_git(repository, "cat-file", "-t", reference, check=False)
    if object_type.returncode != 0:
        raise ValidationError(f"RC tag {tag.raw!r} does not exist")
    if object_type.stdout.strip() != "tag":
        raise ValidationError(f"RC tag {tag.raw!r} must be an annotated tag, not a lightweight tag")
    headers = annotated_tag_headers(repository, reference)
    if headers.get("type") != "commit":
        raise ValidationError(f"RC tag {tag.raw!r} must point directly to a commit")
    if headers.get("tag") != tag.raw:
        raise ValidationError(f"annotated tag header does not match RC tag {tag.raw!r}")
    target = validate_full_sha(headers.get("object", ""), "annotated RC tag target")
    peeled = run_git(repository, "rev-parse", "--verify", f"{reference}^{{commit}}").stdout.strip()
    validate_full_sha(peeled, "resolved RC candidate")
    if peeled != target:
        raise ValidationError(f"RC tag {tag.raw!r} does not resolve directly to its commit target")
    return tag, peeled


def resolve_tag_commit(repository: Path, value: str, label: str) -> str:
    """Resolve an exact annotated tag that points directly to one commit."""
    reference = f"refs/tags/{value}"
    object_type = run_git(repository, "cat-file", "-t", reference, check=False)
    if object_type.returncode != 0:
        raise ValidationError(f"{label} {value!r} does not exist")
    if object_type.stdout.strip() != "tag":
        raise ValidationError(f"{label} {value!r} must be an annotated tag")
    headers = annotated_tag_headers(repository, reference)
    if headers.get("type") != "commit":
        raise ValidationError(f"{label} {value!r} must point directly to a commit")
    if headers.get("tag") != value:
        raise ValidationError(f"annotated tag header does not match {label} {value!r}")
    target = validate_full_sha(headers.get("object", ""), f"annotated {label} target")
    peeled = run_git(repository, "rev-parse", "--verify", f"{reference}^{{commit}}").stdout.strip()
    validate_full_sha(peeled, f"resolved {label}")
    if peeled != target:
        raise ValidationError(f"{label} {value!r} does not resolve directly to its commit target")
    return peeled


def tag_exists(repository: Path, value: str) -> bool:
    """Return whether one exact tag ref exists, failing on unexpected Git errors."""
    result = run_git(
        repository,
        "show-ref",
        "--verify",
        "--quiet",
        f"refs/tags/{value}",
        check=False,
    )
    if result.returncode not in {0, 1}:
        raise ValidationError(f"could not determine whether final tag {value!r} exists")
    return result.returncode == 0


def reject_duplicate_json_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    """Build a JSON object while rejecting last-key-wins ambiguity."""
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ValidationError(f"evidence JSON contains duplicate key {key!r}")
        result[key] = value
    return result


def read_bounded_json_object(path: Path, maximum_bytes: int, label: str) -> dict[str, Any]:
    """Read one regular JSON object without following symlinks or exceeding a byte cap."""
    if path.is_symlink() or not path.is_file():
        raise ValidationError(f"{label} {path} must be a regular, non-symlink file")
    try:
        with path.open("rb") as source:
            raw = source.read(maximum_bytes + 1)
    except OSError as error:
        raise ValidationError(f"cannot read {label} {path}: {error}") from error
    if len(raw) > maximum_bytes:
        raise ValidationError(f"{label} {path} exceeds the {maximum_bytes}-byte limit")
    try:
        payload = json.loads(raw.decode("utf-8"), object_pairs_hook=reject_duplicate_json_keys)
    except UnicodeDecodeError as error:
        raise ValidationError(f"{label} {path} is not UTF-8") from error
    except json.JSONDecodeError as error:
        raise ValidationError(f"{label} {path} is invalid JSON: {error.msg}") from error
    if not isinstance(payload, dict):
        raise ValidationError(f"{label} {path} must be a JSON object")
    return payload


def require_exact_object_fields(
    payload: dict[str, Any],
    required: Sequence[str],
    optional: Sequence[str],
    label: str,
) -> None:
    """Reject missing and unknown object fields deterministically."""
    missing = sorted(set(required) - set(payload))
    unknown = sorted(set(payload) - set(required) - set(optional))
    if missing:
        raise ValidationError(f"{label} is missing fields: {', '.join(missing)}")
    if unknown:
        raise ValidationError(f"{label} has unknown fields: {', '.join(unknown)}")


def require_unique_string_list(value: Any, label: str) -> list[str]:
    """Require a duplicate-free JSON array of strings."""
    if not isinstance(value, list) or any(not isinstance(item, str) for item in value):
        raise ValidationError(f"{label} must be an array of strings")
    if len(value) != len(set(value)):
        raise ValidationError(f"{label} must not contain duplicates")
    return value


def validate_ruleset_metadata(payload: dict[str, Any], label: str) -> None:
    """Validate optional fields emitted by GitHub's repository-ruleset endpoint."""
    if "id" in payload and (type(payload["id"]) is not int or payload["id"] <= 0):
        raise ValidationError(f"{label} id must be a positive integer")
    for key in ("node_id", "source", "created_at", "updated_at"):
        if key in payload and (not isinstance(payload[key], str) or not payload[key]):
            raise ValidationError(f"{label} {key} must be a non-empty string")
    if "current_user_can_bypass" in payload and payload["current_user_can_bypass"] not in {
        "always",
        "pull_requests_only",
        "never",
        "exempt",
    }:
        raise ValidationError(
            f"{label} current_user_can_bypass must be a recognized GitHub bypass mode"
        )
    if "_links" not in payload:
        return
    links = payload["_links"]
    if not isinstance(links, dict):
        raise ValidationError(f"{label} _links must be an object")
    require_exact_object_fields(links, ("self", "html"), (), f"{label} _links")
    for key in ("self", "html"):
        link = links[key]
        if not isinstance(link, dict):
            raise ValidationError(f"{label} _links.{key} must be an object")
        require_exact_object_fields(link, ("href",), (), f"{label} _links.{key}")
        if not isinstance(link["href"], str) or not link["href"]:
            raise ValidationError(f"{label} _links.{key}.href must be a non-empty string")


def validate_tag_ruleset(
    payload: dict[str, Any],
    *,
    label: str,
    expected_name: str,
    expected_includes: Sequence[str],
    expected_excludes: Sequence[str],
    expected_rules: Sequence[str],
) -> dict[str, Any]:
    """Validate one exact repository tag-ruleset policy without accepting broadening."""
    require_exact_object_fields(
        payload,
        REQUIRED_RULESET_FIELDS,
        OPTIONAL_RULESET_FIELDS,
        label,
    )
    validate_ruleset_metadata(payload, label)
    expected_scalars = {
        "name": expected_name,
        "target": "tag",
        "source_type": "Repository",
        "enforcement": "active",
    }
    for key, expected in expected_scalars.items():
        if payload[key] != expected:
            raise ValidationError(
                f"{label} {key} must be {expected!r}, found {payload[key]!r}"
            )

    conditions = payload["conditions"]
    if not isinstance(conditions, dict):
        raise ValidationError(f"{label} conditions must be an object")
    require_exact_object_fields(conditions, ("ref_name",), (), f"{label} conditions")
    ref_name = conditions["ref_name"]
    if not isinstance(ref_name, dict):
        raise ValidationError(f"{label} conditions.ref_name must be an object")
    require_exact_object_fields(
        ref_name,
        ("include", "exclude"),
        (),
        f"{label} conditions.ref_name",
    )
    includes = require_unique_string_list(
        ref_name["include"],
        f"{label} conditions.ref_name.include",
    )
    excludes = require_unique_string_list(
        ref_name["exclude"],
        f"{label} conditions.ref_name.exclude",
    )
    if includes != list(expected_includes):
        raise ValidationError(
            f"{label} include patterns must be exactly {list(expected_includes)!r}"
        )
    if excludes != list(expected_excludes):
        raise ValidationError(
            f"{label} exclude patterns must be exactly {list(expected_excludes)!r}"
        )

    rules = payload["rules"]
    if not isinstance(rules, list):
        raise ValidationError(f"{label} rules must be an array")
    rule_types: list[str] = []
    for index, rule in enumerate(rules):
        if not isinstance(rule, dict):
            raise ValidationError(f"{label} rules[{index}] must be an object")
        require_exact_object_fields(rule, ("type",), (), f"{label} rules[{index}]")
        rule_type = rule["type"]
        if not isinstance(rule_type, str):
            raise ValidationError(f"{label} rules[{index}].type must be a string")
        rule_types.append(rule_type)
    if len(rule_types) != len(set(rule_types)):
        raise ValidationError(f"{label} rule types must not contain duplicates")
    if set(rule_types) != set(expected_rules):
        raise ValidationError(
            f"{label} rule types must be exactly {list(expected_rules)!r}"
        )

    # Both surviving rulesets are immutability rulesets, and immutability that someone may bypass
    # is not immutability. There is deliberately no parameter here to admit an exception: the
    # App-only creation policy was the one place a bypass actor was ever expected, and it is gone.
    if "bypass_actors" in payload:
        bypass_actors = payload["bypass_actors"]
        if not isinstance(bypass_actors, list):
            raise ValidationError(f"{label} bypass_actors must be an array")
        if bypass_actors:
            raise ValidationError(f"{label} must not grant bypass to any actor")
        bypass_observed = []
    else:
        # Unobservable under this token. Do NOT infer emptiness from absence — that is the
        # failure this release has fixed repeatedly. Report it as unverified instead.
        bypass_observed = None

    return {
        "bypass_actors": bypass_observed,
        "name": expected_name,
        "ref_excludes": list(expected_excludes),
        "ref_includes": list(expected_includes),
        "rules": list(expected_rules),
        "target": "tag",
        "valid": True,
    }


def validate_final_tag_immutability_ruleset(payload: dict[str, Any]) -> dict[str, Any]:
    """Require update/deletion protection on final tags that no actor may bypass."""
    return validate_tag_ruleset(
        payload,
        label="final tag immutability ruleset",
        expected_name=FINAL_IMMUTABILITY_RULESET_NAME,
        expected_includes=(FINAL_RULESET_REF_PATTERN,),
        expected_excludes=(FINAL_RULESET_EXCLUDE_PATTERN,),
        expected_rules=FINAL_IMMUTABILITY_RULES,
    )


def validate_rc_tag_ruleset(payload: dict[str, Any]) -> dict[str, Any]:
    """Require immutable RC tags while deliberately allowing their initial creation."""
    return validate_tag_ruleset(
        payload,
        label="RC tag ruleset",
        expected_name=RC_RULESET_NAME,
        expected_includes=(RC_RULESET_REF_PATTERN,),
        expected_excludes=(),
        expected_rules=RC_TAG_RULES,
    )


def rulesets_result(
    final_immutability_path: Path,
    rc_path: Path,
) -> dict[str, Any]:
    """Load and validate the two bounded repository tag-ruleset API responses."""
    final_immutability_payload = read_bounded_json_object(
        final_immutability_path,
        RULESET_MAX_BYTES,
        "final immutability ruleset JSON",
    )
    rc_payload = read_bounded_json_object(
        rc_path,
        RULESET_MAX_BYTES,
        "RC ruleset JSON",
    )
    ruleset_ids = [
        payload.get("id")
        for payload in (final_immutability_payload, rc_payload)
        if "id" in payload
    ]
    if len(ruleset_ids) != len(set(ruleset_ids)):
        raise ValidationError("immutability and RC rulesets must have distinct ids")
    results = {
        "final_immutability": validate_final_tag_immutability_ruleset(
            final_immutability_payload
        ),
        "rc": validate_rc_tag_ruleset(rc_payload),
    }
    return {
        "final_immutability": results["final_immutability"],
        "mode": "rulesets",
        "rc": results["rc"],
        "unenforced": list(UNENFORCED_TAG_POLICIES) + unobserved_bypass_notices(results),
        "valid": True,
    }


def required_string(payload: dict[str, Any], key: str, source: Path) -> str:
    """Read one required, non-empty string from an evidence object."""
    value = payload.get(key)
    if not isinstance(value, str) or not value:
        raise ValidationError(f"evidence {source} field {key!r} must be a non-empty string")
    return value


def load_platform_evidence(path: Path) -> PlatformEvidence:
    """Load one deterministic platform evidence record from JSON."""
    try:
        raw = path.read_bytes()
    except OSError as error:
        raise ValidationError(f"cannot read platform evidence {path}: {error}") from error
    if len(raw) > 1024 * 1024:
        raise ValidationError(f"platform evidence {path} exceeds the 1 MiB limit")
    try:
        payload = json.loads(raw.decode("utf-8"), object_pairs_hook=reject_duplicate_json_keys)
    except UnicodeDecodeError as error:
        raise ValidationError(f"platform evidence {path} is not UTF-8") from error
    except json.JSONDecodeError as error:
        raise ValidationError(f"platform evidence {path} is invalid JSON: {error.msg}") from error
    if not isinstance(payload, dict):
        raise ValidationError(f"platform evidence {path} must be a JSON object")
    missing = [field for field in REQUIRED_EVIDENCE_FIELDS if field not in payload]
    if missing:
        raise ValidationError(f"platform evidence {path} is missing fields: {', '.join(missing)}")
    allowed = set(REQUIRED_EVIDENCE_FIELDS) | set(OPTIONAL_EVIDENCE_FIELDS)
    unknown = sorted(set(payload) - allowed)
    if unknown:
        raise ValidationError(
            f"platform evidence {path} has unknown fields: {', '.join(unknown)}"
        )
    if type(payload["schema_version"]) is not int or payload["schema_version"] != 1:
        raise ValidationError(f"platform evidence {path} must use schema_version 1")
    platform = required_string(payload, "platform", path)
    outcome = required_string(payload, "outcome", path)
    rc_tag = required_string(payload, "rc_tag", path)
    candidate_sha = validate_full_sha(
        required_string(payload, "candidate_sha", path),
        f"evidence {path} candidate_sha",
    )
    lane = required_string(payload, "lane", path)
    if LANE_PATTERN.fullmatch(lane) is None:
        raise ValidationError(f"evidence {path} lane has unsupported characters")
    package_version = payload.get("package_version")
    if package_version is not None:
        if not isinstance(package_version, str) or VERSION_PATTERN.fullmatch(package_version) is None:
            raise ValidationError(
                f"evidence {path} package_version must be a canonical X.Y.Z version"
            )
    workflow_revision = payload.get("workflow_revision")
    if workflow_revision is not None:
        if not isinstance(workflow_revision, str):
            raise ValidationError(
                f"evidence {path} workflow_revision must be a full Git SHA"
            )
        workflow_revision = validate_full_sha(
            workflow_revision,
            f"evidence {path} workflow_revision",
        )
    return PlatformEvidence(
        platform=platform,
        outcome=outcome,
        rc_tag=rc_tag,
        candidate_sha=candidate_sha,
        lane=lane,
        package_version=package_version,
        workflow_revision=workflow_revision,
    )


def load_artifact_provenance(path: Path) -> ArtifactProvenance:
    """Load one exact schema-1 artifact provenance record."""
    try:
        raw = path.read_bytes()
    except OSError as error:
        raise ValidationError(f"cannot read artifact provenance {path}: {error}") from error
    if len(raw) > 1024 * 1024:
        raise ValidationError(f"artifact provenance {path} exceeds the 1 MiB limit")
    try:
        payload = json.loads(raw.decode("utf-8"), object_pairs_hook=reject_duplicate_json_keys)
    except UnicodeDecodeError as error:
        raise ValidationError(f"artifact provenance {path} is not UTF-8") from error
    except json.JSONDecodeError as error:
        raise ValidationError(f"artifact provenance {path} is invalid JSON: {error.msg}") from error
    if not isinstance(payload, dict):
        raise ValidationError(f"artifact provenance {path} must be a JSON object")
    missing = sorted(set(PROVENANCE_FIELDS) - set(payload))
    unknown = sorted(set(payload) - set(PROVENANCE_FIELDS))
    if missing:
        raise ValidationError(
            f"artifact provenance {path} is missing fields: {', '.join(missing)}"
        )
    if unknown:
        raise ValidationError(
            f"artifact provenance {path} has unknown fields: {', '.join(unknown)}"
        )
    if type(payload["schema_version"]) is not int or payload["schema_version"] != 1:
        raise ValidationError(f"artifact provenance {path} must use schema_version 1")
    artifact = required_string(payload, "artifact", path)
    archive = required_string(payload, "archive", path)
    candidate_sha = validate_full_sha(
        required_string(payload, "candidate_sha", path),
        f"artifact provenance {path} candidate_sha",
    )
    rc_tag = required_string(payload, "rc_tag", path)
    parse_rc_tag(rc_tag)
    digest = required_string(payload, "sha256", path)
    if DIGEST_PATTERN.fullmatch(digest) is None:
        raise ValidationError(
            f"artifact provenance {path} sha256 must be a lowercase 64-character digest"
        )
    return ArtifactProvenance(
        artifact=artifact,
        archive=archive,
        candidate_sha=candidate_sha,
        rc_tag=rc_tag,
        sha256=digest,
    )


def sha256_file(path: Path) -> str:
    """Hash one local archive with bounded memory and no network access."""
    digest = hashlib.sha256()
    try:
        with path.open("rb") as archive:
            while True:
                chunk = archive.read(1024 * 1024)
                if not chunk:
                    break
                digest.update(chunk)
    except OSError as error:
        raise ValidationError(f"cannot hash release archive {path}: {error}") from error
    return digest.hexdigest()


def exact_entry_names(directory: Path) -> list[str]:
    """List one directory without following its entries."""
    try:
        return sorted(path.name for path in directory.iterdir())
    except OSError as error:
        raise ValidationError(f"cannot inspect release artifact directory {directory}: {error}") from error


def require_exact_entries(directory: Path, expected: Sequence[str], label: str) -> None:
    """Require one exact, duplicate-free filesystem entry set."""
    actual = exact_entry_names(directory)
    wanted = sorted(expected)
    if actual == wanted:
        return
    missing = sorted(set(wanted) - set(actual))
    extra = sorted(set(actual) - set(wanted))
    details: list[str] = []
    if missing:
        details.append(f"missing {', '.join(missing)}")
    if extra:
        details.append(f"unexpected {', '.join(extra)}")
    raise ValidationError(f"{label} has the wrong entries ({'; '.join(details)})")


def artifact_directory_result(
    candidate_sha: str,
    rc_tag: str,
    artifact_dir: Path,
) -> dict[str, Any]:
    """Revalidate downloaded release archives, checksums, and provenance."""
    candidate = validate_full_sha(candidate_sha, "candidate SHA")
    tag = parse_rc_tag(rc_tag)
    if artifact_dir.is_symlink() or not artifact_dir.is_dir():
        raise ValidationError(
            f"artifact directory {artifact_dir} must be a real, non-symlink directory"
        )
    expected_artifacts = list(EXPECTED_ARTIFACT_ARCHIVES)
    require_exact_entries(artifact_dir, expected_artifacts, "artifact directory")

    validated: list[dict[str, str]] = []
    for artifact, archive_name in EXPECTED_ARTIFACT_ARCHIVES.items():
        artifact_path = artifact_dir / artifact
        if artifact_path.is_symlink() or not artifact_path.is_dir():
            raise ValidationError(
                f"artifact entry {artifact_path} must be a real, non-symlink directory"
            )
        checksum_name = f"{archive_name}.sha256"
        provenance_name = f"{artifact}.provenance.json"
        require_exact_entries(
            artifact_path,
            (archive_name, checksum_name, provenance_name),
            f"artifact {artifact!r}",
        )
        archive_path = artifact_path / archive_name
        checksum_path = artifact_path / checksum_name
        provenance_path = artifact_path / provenance_name
        for path in (archive_path, checksum_path, provenance_path):
            if path.is_symlink() or not path.is_file():
                raise ValidationError(
                    f"release artifact file {path} must be a regular, non-symlink file"
                )

        provenance = load_artifact_provenance(provenance_path)
        if provenance.artifact != artifact:
            raise ValidationError(
                f"provenance {provenance_path} names artifact {provenance.artifact!r}, "
                f"expected {artifact!r}"
            )
        if provenance.archive != archive_name:
            raise ValidationError(
                f"provenance {provenance_path} names archive {provenance.archive!r}, "
                f"expected {archive_name!r}"
            )
        if provenance.candidate_sha != candidate:
            raise ValidationError(
                f"artifact {artifact!r} is bound to candidate "
                f"{provenance.candidate_sha}, expected {candidate}"
            )
        if provenance.rc_tag != tag.raw:
            raise ValidationError(
                f"artifact {artifact!r} is bound to RC tag {provenance.rc_tag!r}, "
                f"expected {tag.raw!r}"
            )

        actual_digest = sha256_file(archive_path)
        if provenance.sha256 != actual_digest:
            raise ValidationError(
                f"archive digest mismatch for {archive_name}: provenance has "
                f"{provenance.sha256}, downloaded archive has {actual_digest}"
            )
        expected_checksum = f"{actual_digest}  {archive_name}\n".encode("ascii")
        try:
            checksum_bytes = checksum_path.read_bytes()
        except OSError as error:
            raise ValidationError(f"cannot read checksum file {checksum_path}: {error}") from error
        if checksum_bytes != expected_checksum:
            raise ValidationError(
                f"checksum file {checksum_path} must contain the exact LF-only SHA-256 record"
            )
        validated.append(
            {
                "archive": archive_name,
                "artifact": artifact,
                "sha256": actual_digest,
            }
        )

    return {
        "artifacts": validated,
        "candidate_sha": candidate,
        "mode": "artifacts",
        "rc_tag": tag.raw,
        "valid": True,
    }


def validate_platform_evidence(
    records: Iterable[PlatformEvidence],
    identity: CandidateIdentity,
    lane: str,
) -> str | None:
    """Require exactly one green, coherently bound record for every platform."""
    if LANE_PATTERN.fullmatch(lane) is None:
        raise ValidationError("expected lane has unsupported characters")
    by_platform: dict[str, PlatformEvidence] = {}
    for record in records:
        if record.platform not in REQUIRED_PLATFORMS:
            raise ValidationError(f"unexpected platform evidence {record.platform!r}")
        if record.platform in by_platform:
            raise ValidationError(f"duplicate platform evidence for {record.platform}")
        by_platform[record.platform] = record
    missing = [platform for platform in REQUIRED_PLATFORMS if platform not in by_platform]
    if missing:
        raise ValidationError(f"missing platform evidence: {', '.join(missing)}")
    package_versions: set[str | None] = set()
    revisions: set[str | None] = set()
    for platform in REQUIRED_PLATFORMS:
        record = by_platform[platform]
        if record.outcome != "success":
            raise ValidationError(
                f"platform {platform} is not successful: outcome={record.outcome!r}"
            )
        if record.rc_tag != identity.tag.raw:
            raise ValidationError(
                f"platform {platform} evidence is bound to RC tag {record.rc_tag!r}, "
                f"expected {identity.tag.raw!r}"
            )
        if record.candidate_sha != identity.candidate_sha:
            raise ValidationError(
                f"platform {platform} evidence is bound to candidate "
                f"{record.candidate_sha}, expected {identity.candidate_sha}"
            )
        if (
            record.package_version is not None
            and record.package_version != identity.package_version
        ):
            raise ValidationError(
                f"platform {platform} evidence package version {record.package_version!r} "
                f"does not match {identity.package_version!r}"
            )
        if record.lane != lane:
            raise ValidationError(
                f"platform {platform} ran lane {record.lane!r}, expected {lane!r}"
            )
        package_versions.add(record.package_version)
        revisions.add(record.workflow_revision)
    if None in package_versions and len(package_versions) != 1:
        raise ValidationError("platform evidence mixes present and missing package versions")
    if None in revisions and len(revisions) != 1:
        raise ValidationError("platform evidence mixes present and missing workflow revisions")
    if len(revisions) != 1:
        raise ValidationError("platform evidence is mixed across workflow revisions")
    return next(iter(revisions))


def build_candidate_identity(
    repository: Path,
    cargo_toml: Path,
    rc_tag: str,
    candidate_sha: str | None,
) -> CandidateIdentity:
    """Resolve and cross-check the RC marker, package version, and optional expected SHA."""
    parsed, resolved_sha = resolve_annotated_rc_tag(repository, rc_tag)
    package_version = read_package_version(cargo_toml)
    if package_version != parsed.version:
        raise ValidationError(
            f"RC tag version {parsed.version!r} does not match package version "
            f"{package_version!r}"
        )
    if candidate_sha is not None:
        expected = validate_full_sha(candidate_sha, "candidate SHA")
        if resolved_sha != expected:
            raise ValidationError(
                f"RC tag {parsed.raw!r} resolves to {resolved_sha}, expected candidate {expected}"
            )
    return CandidateIdentity(
        tag=parsed,
        candidate_sha=resolved_sha,
        package_version=package_version,
    )


def load_and_validate_evidence(
    paths: Sequence[Path], identity: CandidateIdentity, lane: str
) -> str | None:
    """Load evidence files and return their one shared workflow revision."""
    records = [load_platform_evidence(path) for path in paths]
    return validate_platform_evidence(records, identity, lane)


def evidence_directory_result(
    rc_tag: str,
    candidate_sha: str,
    evidence_dir: Path,
) -> dict[str, Any]:
    """Validate the stable workflow-facing three-file platform evidence contract."""
    tag = parse_rc_tag(rc_tag)
    candidate = validate_full_sha(candidate_sha, "candidate SHA")
    if not evidence_dir.is_dir():
        raise ValidationError(f"evidence directory {evidence_dir} is not a directory")
    expected_names = [f"{platform}.json" for platform in REQUIRED_PLATFORMS]
    try:
        actual_names = sorted(
            path.name for path in evidence_dir.iterdir() if path.suffix == ".json"
        )
    except OSError as error:
        raise ValidationError(f"cannot inspect evidence directory {evidence_dir}: {error}") from error
    if actual_names != sorted(expected_names):
        missing = sorted(set(expected_names) - set(actual_names))
        extra = sorted(set(actual_names) - set(expected_names))
        details: list[str] = []
        if missing:
            details.append(f"missing {', '.join(missing)}")
        if extra:
            details.append(f"unexpected {', '.join(extra)}")
        raise ValidationError(
            "evidence directory must contain exactly ubuntu.json, macos.json, and "
            f"windows.json ({'; '.join(details)})"
        )
    evidence_paths = [evidence_dir / name for name in expected_names]
    for evidence_path in evidence_paths:
        if not evidence_path.is_file() or evidence_path.is_symlink():
            raise ValidationError(
                f"evidence file {evidence_path} must be a regular, non-symlink file"
            )
    records = [load_platform_evidence(path) for path in evidence_paths]
    for expected_platform, record in zip(REQUIRED_PLATFORMS, records):
        if record.platform != expected_platform:
            raise ValidationError(
                f"evidence file {expected_platform}.json declares platform "
                f"{record.platform!r}"
            )
    identity = CandidateIdentity(
        tag=tag,
        candidate_sha=candidate,
        package_version=tag.version,
    )
    workflow_revision = validate_platform_evidence(
        records,
        identity,
        RELEASE_CANDIDATE_LANE,
    )
    result: dict[str, Any] = {
        "candidate_sha": candidate,
        "lane": RELEASE_CANDIDATE_LANE,
        "mode": "evidence",
        "platforms": list(REQUIRED_PLATFORMS),
        "rc_tag": tag.raw,
        "valid": True,
    }
    if workflow_revision is not None:
        result["workflow_revision"] = workflow_revision
    return result


def candidate_result(identity: CandidateIdentity) -> dict[str, Any]:
    """Return the stable machine-readable candidate result."""
    return {
        "candidate_sha": identity.candidate_sha,
        "final_tag": identity.tag.final_tag,
        "mode": "candidate",
        "package_version": identity.package_version,
        "rc_iteration": identity.tag.iteration,
        "rc_tag": identity.tag.raw,
        "valid": True,
    }


def promotion_result(
    repository: Path,
    identity: CandidateIdentity,
    final_tag: str,
    lane: str,
    evidence_paths: Sequence[Path],
) -> dict[str, Any]:
    """Validate pre-tag promotion and return the immutable publication identity."""
    if final_tag != identity.tag.final_tag:
        raise ValidationError(
            f"final tag {final_tag!r} does not match RC promotion target "
            f"{identity.tag.final_tag!r}"
        )
    if tag_exists(repository, final_tag):
        raise ValidationError(
            f"final tag {final_tag!r} already exists; promotion refuses to move or reuse it"
        )
    workflow_revision = load_and_validate_evidence(evidence_paths, identity, lane)
    return {
        "candidate_sha": identity.candidate_sha,
        "final_tag": final_tag,
        "lane": lane,
        "mode": "promotion",
        "package_version": identity.package_version,
        "platforms": list(REQUIRED_PLATFORMS),
        "rc_tag": identity.tag.raw,
        "valid": True,
        "workflow_revision": workflow_revision,
    }


def release_result(
    repository: Path,
    identity: CandidateIdentity,
    final_tag: str,
    checkout_sha: str,
    lane: str,
    evidence_paths: Sequence[Path],
) -> dict[str, Any]:
    """Independently revalidate final-tag, checkout, and platform evidence identity."""
    if final_tag != identity.tag.final_tag:
        raise ValidationError(
            f"final tag {final_tag!r} does not match RC promotion target "
            f"{identity.tag.final_tag!r}"
        )
    resolved_final = resolve_tag_commit(repository, final_tag, "final tag")
    if resolved_final != identity.candidate_sha:
        raise ValidationError(
            f"final tag {final_tag!r} resolves to {resolved_final}, "
            f"expected candidate {identity.candidate_sha}"
        )
    checkout = validate_full_sha(checkout_sha, "checkout SHA")
    if checkout != identity.candidate_sha:
        raise ValidationError(
            f"release checkout {checkout} does not match candidate {identity.candidate_sha}"
        )
    workflow_revision = load_and_validate_evidence(evidence_paths, identity, lane)
    return {
        "candidate_sha": identity.candidate_sha,
        "checkout_sha": checkout,
        "final_tag": final_tag,
        "lane": lane,
        "mode": "release",
        "package_version": identity.package_version,
        "platforms": list(REQUIRED_PLATFORMS),
        "rc_tag": identity.tag.raw,
        "valid": True,
        "workflow_revision": workflow_revision,
    }


def add_identity_arguments(parser: argparse.ArgumentParser, require_candidate: bool) -> None:
    """Add the shared repository, tag, package, and expected-SHA arguments."""
    parser.add_argument("--repository", type=Path, default=Path("."))
    parser.add_argument("--cargo-toml", type=Path, default=None)
    parser.add_argument("--rc-tag", required=True)
    parser.add_argument("--candidate-sha", required=require_candidate)


def add_evidence_arguments(parser: argparse.ArgumentParser) -> None:
    """Add the shared promotion/release evidence contract."""
    parser.add_argument("--final-tag", required=True)
    parser.add_argument("--lane", required=True)
    parser.add_argument(
        "--evidence",
        type=Path,
        action="append",
        required=True,
        help="Path to one platform evidence JSON record; repeat for all required platforms",
    )


def parse_arguments(arguments: Sequence[str] | None = None) -> argparse.Namespace:
    """Parse the deterministic validator command line."""
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)

    evidence = commands.add_parser(
        "evidence",
        help="validate exactly ubuntu/macos/windows evidence for one RC tag and SHA",
    )
    evidence.add_argument("--rc-tag", required=True)
    evidence.add_argument("--candidate-sha", required=True)
    evidence.add_argument("--evidence-dir", type=Path, required=True)

    artifacts = commands.add_parser(
        "artifacts",
        help="validate downloaded release archives, checksums, and provenance",
    )
    artifacts.add_argument("--candidate-sha", required=True)
    artifacts.add_argument("--rc-tag", required=True)
    artifacts.add_argument("--artifact-dir", type=Path, required=True)

    rulesets = commands.add_parser(
        "rulesets",
        help=(
            "validate the two active tag-immutability rulesets and report the tag protections "
            "this repository does not enforce"
        ),
    )
    rulesets.add_argument("--final-immutability-ruleset-json", type=Path, required=True)
    rulesets.add_argument("--rc-ruleset-json", type=Path, required=True)

    candidate = commands.add_parser("candidate", help="validate an annotated RC marker")
    add_identity_arguments(candidate, require_candidate=False)

    promote = commands.add_parser("promote", help="validate evidence before final-tag creation")
    add_identity_arguments(promote, require_candidate=True)
    add_evidence_arguments(promote)

    release = commands.add_parser("release", help="revalidate an existing promoted release tag")
    add_identity_arguments(release, require_candidate=True)
    add_evidence_arguments(release)
    release.add_argument("--checkout-sha", required=True)

    return parser.parse_args(arguments)


def resolve_manifest_path(repository: Path, cargo_toml: Path | None) -> Path:
    """Resolve the Cargo manifest relative to the selected repository by default."""
    if cargo_toml is None:
        return repository / "Cargo.toml"
    return cargo_toml if cargo_toml.is_absolute() else repository / cargo_toml


def execute(arguments: argparse.Namespace) -> dict[str, Any]:
    """Execute one parsed command through the shared pure validation surfaces."""
    if arguments.command == "evidence":
        return evidence_directory_result(
            arguments.rc_tag,
            arguments.candidate_sha,
            arguments.evidence_dir,
        )
    if arguments.command == "artifacts":
        return artifact_directory_result(
            arguments.candidate_sha,
            arguments.rc_tag,
            arguments.artifact_dir,
        )
    if arguments.command == "rulesets":
        return rulesets_result(
            arguments.final_immutability_ruleset_json,
            arguments.rc_ruleset_json,
        )
    repository = arguments.repository.resolve()
    if not repository.is_dir():
        raise ValidationError(f"repository path {repository} is not a directory")
    identity = build_candidate_identity(
        repository,
        resolve_manifest_path(repository, arguments.cargo_toml),
        arguments.rc_tag,
        arguments.candidate_sha,
    )
    if arguments.command == "candidate":
        return candidate_result(identity)
    if arguments.command == "promote":
        return promotion_result(
            repository,
            identity,
            arguments.final_tag,
            arguments.lane,
            arguments.evidence,
        )
    if arguments.command == "release":
        return release_result(
            repository,
            identity,
            arguments.final_tag,
            arguments.checkout_sha,
            arguments.lane,
            arguments.evidence,
        )
    raise ValidationError(f"unsupported command {arguments.command!r}")


def main(arguments: Sequence[str] | None = None) -> int:
    """Run the CLI and emit one deterministic JSON result or concise error."""
    try:
        result = execute(parse_arguments(arguments))
    except ValidationError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
