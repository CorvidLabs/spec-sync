#!/usr/bin/env python3
"""Focused tests for bounded first-parent GitHub check provenance reuse."""

import contextlib
import copy
import importlib.util
import io
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path


sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / ".github/scripts/reuse-check-from-ancestors.py"
CI_WORKFLOW = ROOT / ".github/workflows/ci.yml"


def load_module():
    specification = importlib.util.spec_from_file_location("reuse_check", SCRIPT)
    assert specification and specification.loader
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def git(root: Path, *arguments: str) -> str:
    return subprocess.check_output(["git", *arguments], cwd=root, text=True).strip()


module = load_module()
ci_workflow = CI_WORKFLOW.read_text(encoding="utf-8")
assert "provenance_helpers" in ci_workflow
assert "CI provenance helpers may change only with the protected CI workflow" in ci_workflow
assert str(SCRIPT.relative_to(ROOT)) in ci_workflow
assert str(Path(__file__).resolve().relative_to(ROOT)) in ci_workflow


def signed_entry(
    path: str,
    kind: str,
    mode: int,
    payload: bytes,
    owners: list[str] | None = None,
) -> dict:
    entry = {
        "path": path,
        "kind": kind,
        "mode": mode,
        "payload_digest": __import__("hashlib").sha256(payload).hexdigest(),
        "entry_digest": "",
        "owners": owners or ["@exact:delivery"],
    }
    entry["entry_digest"] = module.acceptance_entry_digest(entry)
    return entry


empty_manifest = {"schema_version": 1, "entries": []}
assert module.acceptance_manifest_digest(empty_manifest) is not None
assert module.u64_json_integer(0)
assert module.u64_json_integer(2**64 - 1)
assert not module.u64_json_integer(True)
assert not module.u64_json_integer(-1)
assert not module.u64_json_integer(2**64)
unsupported_non_file = {
    "schema_version": 1,
    "entries": [signed_entry("commands", "non-file", 0, b"")],
}
assert module.acceptance_manifest_digest(unsupported_non_file) is None


with tempfile.TemporaryDirectory() as temporary:
    ownership_repository = Path(temporary)
    git(ownership_repository, "init", "-b", "main")
    git(ownership_repository, "config", "user.email", "test@example.com")
    git(ownership_repository, "config", "user.name", "Test")
    (ownership_repository / ".specsync").mkdir()
    (ownership_repository / ".specsync/config.toml").write_text(
        'specs_dir = "custom-specs"\nsource_dirs = ["src"]\n', encoding="utf-8"
    )
    (ownership_repository / ".specsync/registry.toml").write_text(
        '[registry]\nname = "ownership-fixture"\n\n'
        '[specs]\nfoo = "specs/foo/foo.spec.md"\nbar = "specs/bar/bar.spec.md"\n'
        'baz = "specs/baz/baz.spec.md"\n'
        'linkowner = "specs/linkowner/linkowner.spec.md"\n',
        encoding="utf-8",
    )
    (ownership_repository / "specs/foo").mkdir(parents=True)
    (ownership_repository / "specs/bar").mkdir(parents=True)
    (ownership_repository / "specs/baz").mkdir(parents=True)
    (ownership_repository / "specs/linkowner").mkdir(parents=True)
    (ownership_repository / "specs/flow").mkdir(parents=True)
    (ownership_repository / "specs/scalar").mkdir(parents=True)
    (ownership_repository / "specs/commented").mkdir(parents=True)
    (ownership_repository / "specs/interrupted").mkdir(parents=True)
    (ownership_repository / "custom-specs/fallback").mkdir(parents=True)
    (ownership_repository / "src").mkdir()
    foo_spec = (
        "---\nmodule: foo\nversion: 1\nstatus: stable\nfiles:\n"
        "  - src/foo.rs\n  - docs/foo.rs\n  - tests/foo.rs\n"
        "  - specsync-registry.toml\n---\n\n# Foo\n"
    )
    bar_spec = (
        "---\nmodule: bar\nversion: 1\nstatus: stable\nfiles:\n"
        "  - src/foo.rs\n  - src/link.rs\n  - docs/foo.rs\n---\n\n# Bar\n"
    )
    baz_spec = (
        "---\nmodule: baz\nversion: 1\nstatus: stable\nfiles:\n"
        "  - src/baz.rs\n---\n\n# Baz\n"
    )
    (ownership_repository / "specs/foo/foo.spec.md").write_text(
        foo_spec, encoding="utf-8"
    )
    (ownership_repository / "specs/foo/requirements.md").write_text(
        "# Requirements\n", encoding="utf-8"
    )
    (ownership_repository / "specs/foo/retired.md").write_text(
        "# Retired\n", encoding="utf-8"
    )
    (ownership_repository / "specs/bar/bar.spec.md").write_text(
        bar_spec, encoding="utf-8"
    )
    (ownership_repository / "specs/baz/baz.spec.md").write_text(
        baz_spec, encoding="utf-8"
    )
    (ownership_repository / "specs/linkowner/linkowner.spec.md").symlink_to(
        "../bar/bar.spec.md"
    )
    (ownership_repository / "specs/flow/flow.spec.md").write_text(
        "---\nmodule: flow\nversion: 1\nstatus: stable\n"
        "files: [src/foo.rs, 'src/baz.rs']\n---\n\n# Flow\n",
        encoding="utf-8",
    )
    (ownership_repository / "specs/scalar/scalar.spec.md").write_text(
        "---\nmodule: scalar\nversion: 1\nstatus: stable\n"
        "files: src/foo.rs\n---\n\n# Scalar\n",
        encoding="utf-8",
    )
    (ownership_repository / "specs/commented/commented.spec.md").write_text(
        "---\nmodule: commented\nversion: 1\nstatus: stable\nfiles:\n"
        "  - src/foo.rs # canonical source\n---\n\n# Commented\n",
        encoding="utf-8",
    )
    (ownership_repository / "specs/interrupted/interrupted.spec.md").write_text(
        "---\nmodule: interrupted\nversion: 1\nstatus: stable\nfiles:\n"
        "  - src/foo.rs\n  # list terminator\n  - src/baz.rs\n---\n\n# Interrupted\n",
        encoding="utf-8",
    )
    (ownership_repository / "custom-specs/fallback/fallback.spec.md").write_text(
        "---\nmodule: fallback\nversion: 1\nstatus: stable\nfiles:\n"
        "  - src/foo.rs\n---\n\n# Fallback\n",
        encoding="utf-8",
    )
    (ownership_repository / "src/foo.rs").write_text(
        "pub fn foo() {}\n", encoding="utf-8"
    )
    (ownership_repository / "src/link.rs").symlink_to("foo.rs")
    (ownership_repository / "src/baz.rs").write_text(
        "pub fn baz() {}\n", encoding="utf-8"
    )
    (ownership_repository / "docs").mkdir()
    (ownership_repository / "docs/foo.rs").write_text(
        "pub fn documented_foo() {}\n", encoding="utf-8"
    )
    (ownership_repository / "docs/good-link").symlink_to("foo.rs")
    (ownership_repository / "docs/bad-link").symlink_to("/tmp/outside")
    (ownership_repository / "tests").mkdir()
    (ownership_repository / "tests/foo.rs").write_text(
        "#[test]\nfn foo_works() {}\n", encoding="utf-8"
    )
    (ownership_repository / "specsync-registry.toml").write_text(
        '[registry]\nname = "secondary"\n', encoding="utf-8"
    )
    git(ownership_repository, "add", ".")
    git(ownership_repository, "commit", "-m", "ownership fixture")
    ownership_commit = git(ownership_repository, "rev-parse", "HEAD")
    assert module.spec_source_paths(
        ownership_repository, ownership_commit, "specs/flow/flow.spec.md"
    ) == {"src/foo.rs", "src/baz.rs"}
    assert module.spec_source_paths(
        ownership_repository, ownership_commit, "specs/scalar/scalar.spec.md"
    ) == {"src/foo.rs"}
    assert module.spec_source_paths(
        ownership_repository, ownership_commit, "specs/commented/commented.spec.md"
    ) == {"src/foo.rs"}
    assert module.spec_source_paths(
        ownership_repository, ownership_commit, "specs/interrupted/interrupted.spec.md"
    ) == {"src/foo.rs"}
    ownership_state = {
        "id": "CHG-0001-ownership",
        "affected_specs": ["foo"],
        "affected_paths": ["specs/foo", "src/foo.rs"],
        "acceptance_owner_corrections": [
            {
                "schema_version": 1,
                "sequence": 1,
                "path": "src/foo.rs",
                "module": "bar",
                "actor": "Independent reviewer",
                "reason": "Restore the audited canonical co-owner",
                "timestamp": 1,
            }
        ],
    }
    ownership_manifest = {
        "schema_version": 1,
        "entries": [
            signed_entry("specs/foo", "non_file", 0, b""),
            signed_entry(
                "specs/foo/foo.spec.md",
                "file",
                0o100644,
                foo_spec.encode(),
                ["foo"],
            ),
            signed_entry(
                "specs/foo/requirements.md",
                "file",
                0o100644,
                b"# Requirements\n",
                ["foo"],
            ),
            signed_entry(
                "specs/foo/retired.md",
                "file",
                0o100644,
                b"# Retired\n",
            ),
            signed_entry(
                "src/foo.rs",
                "file",
                0o100644,
                b"pub fn foo() {}\n",
                ["bar", "foo"],
            ),
        ],
    }

    def ownership_manifest_with(entry: dict) -> dict:
        candidate = copy.deepcopy(ownership_manifest)
        candidate["entries"].append(entry)
        candidate["entries"].sort(key=lambda item: item["path"])
        return candidate
    assert module.acceptance_manifest_matches_commit(
        ownership_repository,
        ownership_commit,
        ownership_manifest,
        ownership_state,
    )
    fallback_state = copy.deepcopy(ownership_state)
    fallback_state["acceptance_owner_corrections"][0]["module"] = "fallback"
    fallback_manifest = copy.deepcopy(ownership_manifest)
    fallback_source = next(
        entry for entry in fallback_manifest["entries"] if entry["path"] == "src/foo.rs"
    )
    fallback_source["owners"] = ["fallback", "foo"]
    fallback_source["entry_digest"] = module.acceptance_entry_digest(fallback_source)
    assert module.acceptance_manifest_matches_commit(
        ownership_repository,
        ownership_commit,
        fallback_manifest,
        fallback_state,
    )
    forged_extra_owner = copy.deepcopy(ownership_manifest)
    retired = next(
        entry
        for entry in forged_extra_owner["entries"]
        if entry["path"] == "specs/foo/retired.md"
    )
    retired["owners"] = ["foo"]
    retired["entry_digest"] = module.acceptance_entry_digest(retired)
    assert not module.acceptance_manifest_matches_commit(
        ownership_repository,
        ownership_commit,
        forged_extra_owner,
        ownership_state,
    )
    missing_companion_state = copy.deepcopy(ownership_state)
    missing_companion_state["affected_paths"].append("specs/foo/tasks.md")
    missing_companion_manifest = ownership_manifest_with(
        signed_entry(
            "specs/foo/tasks.md",
            "missing",
            0,
            b"",
            ["foo"],
        )
    )
    assert module.acceptance_manifest_matches_commit(
        ownership_repository,
        ownership_commit,
        missing_companion_manifest,
        missing_companion_state,
    )
    forged_missing_owner = copy.deepcopy(missing_companion_manifest)
    missing_entry = next(
        entry
        for entry in forged_missing_owner["entries"]
        if entry["path"] == "specs/foo/tasks.md"
    )
    missing_entry["owners"] = ["@exact:delivery"]
    missing_entry["entry_digest"] = module.acceptance_entry_digest(missing_entry)
    assert not module.acceptance_manifest_matches_commit(
        ownership_repository,
        ownership_commit,
        forged_missing_owner,
        missing_companion_state,
    )
    test_state = copy.deepcopy(ownership_state)
    test_state["affected_paths"].append("tests/foo.rs")
    test_manifest = ownership_manifest_with(
        signed_entry(
            "tests/foo.rs",
            "file",
            0o100644,
            b"#[test]\nfn foo_works() {}\n",
            ["@exact:test"],
        )
    )
    assert module.acceptance_manifest_matches_commit(
        ownership_repository, ownership_commit, test_manifest, test_state
    )
    forged_test_owner = copy.deepcopy(test_manifest)
    forged_test_entry = next(
        entry for entry in forged_test_owner["entries"] if entry["path"] == "tests/foo.rs"
    )
    forged_test_entry["owners"] = ["@exact:delivery"]
    forged_test_entry["entry_digest"] = module.acceptance_entry_digest(forged_test_entry)
    assert not module.acceptance_manifest_matches_commit(
        ownership_repository, ownership_commit, forged_test_owner, test_state
    )
    delivery_state = copy.deepcopy(ownership_state)
    delivery_state["affected_paths"].append("docs/foo.rs")
    delivery_manifest = ownership_manifest_with(
        signed_entry(
            "docs/foo.rs",
            "file",
            0o100644,
            b"pub fn documented_foo() {}\n",
            ["@exact:delivery"],
        )
    )
    assert module.acceptance_manifest_matches_commit(
        ownership_repository, ownership_commit, delivery_manifest, delivery_state
    )
    forged_delivery_owner = copy.deepcopy(delivery_manifest)
    forged_delivery_entry = next(
        entry
        for entry in forged_delivery_owner["entries"]
        if entry["path"] == "docs/foo.rs"
    )
    forged_delivery_entry["owners"] = ["foo"]
    forged_delivery_entry["entry_digest"] = module.acceptance_entry_digest(
        forged_delivery_entry
    )
    assert not module.acceptance_manifest_matches_commit(
        ownership_repository,
        ownership_commit,
        forged_delivery_owner,
        delivery_state,
    )
    protected_registry_state = copy.deepcopy(ownership_state)
    protected_registry_state["affected_paths"].append("specsync-registry.toml")
    protected_registry_manifest = ownership_manifest_with(
        signed_entry(
            "specsync-registry.toml",
            "file",
            0o100644,
            b'[registry]\nname = "secondary"\n',
            ["@exact:delivery"],
        )
    )
    assert module.acceptance_manifest_matches_commit(
        ownership_repository,
        ownership_commit,
        protected_registry_manifest,
        protected_registry_state,
    )
    forged_registry_owner = copy.deepcopy(protected_registry_manifest)
    forged_registry_entry = next(
        entry
        for entry in forged_registry_owner["entries"]
        if entry["path"] == "specsync-registry.toml"
    )
    forged_registry_entry["owners"] = ["foo"]
    forged_registry_entry["entry_digest"] = module.acceptance_entry_digest(
        forged_registry_entry
    )
    assert not module.acceptance_manifest_matches_commit(
        ownership_repository,
        ownership_commit,
        forged_registry_owner,
        protected_registry_state,
    )
    unowned_source_state = copy.deepcopy(ownership_state)
    unowned_source_state["affected_paths"].append("src/baz.rs")
    unowned_source_manifest = ownership_manifest_with(
        signed_entry(
            "src/baz.rs",
            "file",
            0o100644,
            b"pub fn baz() {}\n",
            ["@exact:delivery"],
        )
    )
    assert not module.acceptance_manifest_matches_commit(
        ownership_repository,
        ownership_commit,
        unowned_source_manifest,
        unowned_source_state,
    )
    good_link_state = copy.deepcopy(ownership_state)
    good_link_state["affected_paths"].append("docs/good-link")
    good_link_manifest = ownership_manifest_with(
        signed_entry("docs/good-link", "symlink", 0o120000, b"foo.rs")
    )
    assert module.acceptance_manifest_matches_commit(
        ownership_repository, ownership_commit, good_link_manifest, good_link_state
    )
    bad_link_state = copy.deepcopy(ownership_state)
    bad_link_state["affected_paths"].append("docs/bad-link")
    bad_link_manifest = ownership_manifest_with(
        signed_entry("docs/bad-link", "symlink", 0o120000, b"/tmp/outside")
    )
    assert not module.acceptance_manifest_matches_commit(
        ownership_repository, ownership_commit, bad_link_manifest, bad_link_state
    )

    (ownership_repository / ".specsync/config.toml").write_text(
        'specs_dir = "custom-specs"\n', encoding="utf-8"
    )
    (ownership_repository / "Cargo.toml").write_text(
        '[package]\nname = "ownership-fixture"\nversion = "0.1.0"\n',
        encoding="utf-8",
    )
    (ownership_repository / "Package.swift").write_text(
        '// swift-tools-version: 6.0\n'
        'let package = Package(targets: [.target('
        'name: "Nested", dependencies: [.product(name: "Dep", package: "dep")], '
        'path: "CustomSources")])\n',
        encoding="utf-8",
    )
    (ownership_repository / "CustomSources").mkdir()
    (ownership_repository / "CustomSources/main.swift").write_text(
        "public func nested() {}\n", encoding="utf-8"
    )
    (ownership_repository / "build.gradle.kts").write_text(
        'plugins { kotlin("jvm") version "2.0.0" }\n', encoding="utf-8"
    )
    (ownership_repository / "settings.gradle.kts").write_text(
        '// include(":ignored")\n'
        '"""project("app").projectDir = file("triple-ignored")"""\n'
        '/* outer /* project("app").projectDir = file("nested-ignored") */ */\n'
        'include("app")\n'
        'project("app").projectDir = file("custom-app")\n',
        encoding="utf-8",
    )
    (ownership_repository / "custom-app/src/main/kotlin").mkdir(parents=True)
    (ownership_repository / "custom-app/src/main/kotlin/App.kt").write_text(
        "class App\n", encoding="utf-8"
    )
    (ownership_repository / "tools").mkdir()
    (ownership_repository / "tools/helper.rs").write_text(
        "pub fn helper() {}\n", encoding="utf-8"
    )
    predecessor_state = copy.deepcopy(ownership_state)
    predecessor_state["id"] = "CHG-0000-predecessor"
    predecessor_state["state"] = "accepted"
    predecessor_root = ownership_repository / ".specsync/changes/CHG-0000-predecessor"
    predecessor_root.mkdir(parents=True)
    (predecessor_root / "state.json").write_text(
        json.dumps(predecessor_state), encoding="utf-8"
    )
    (predecessor_root / "verification.json").write_text(
        json.dumps(
            {
                "acceptance_input_digest": module.acceptance_manifest_digest(
                    ownership_manifest
                )
            }
        ),
        encoding="utf-8",
    )
    git(ownership_repository, "add", ".")
    git(ownership_repository, "commit", "-m", "accepted predecessor base")
    predecessor_base = git(ownership_repository, "rev-parse", "HEAD")
    detected_roots = {"src", "CustomSources", "custom-app/src/main/kotlin"}
    assert module.configured_source_dirs(
        ownership_repository, predecessor_base
    ) == detected_roots
    expected_symlink = signed_entry(
        "docs/good-link", "symlink", 0o120000, b"foo.rs"
    )["entry_digest"]
    assert module.predecessor_entry_digest_at_base(
        ownership_repository,
        predecessor_base,
        "CHG-0000-predecessor",
        "docs/good-link",
    ) == expected_symlink
    auto_state = copy.deepcopy(ownership_state)
    auto_state["affected_paths"].append("tools/helper.rs")
    auto_manifest = ownership_manifest_with(
        signed_entry(
            "tools/helper.rs",
            "file",
            0o100644,
            b"pub fn helper() {}\n",
            ["@exact:delivery"],
        )
    )
    assert module.acceptance_manifest_matches_commit(
        ownership_repository, predecessor_base, auto_manifest, auto_state
    )

    successor_id = "CHG-0001-ownership"
    successor_delta = ownership_repository / f".specsync/changes/{successor_id}/deltas"
    successor_delta.mkdir(parents=True)
    (successor_delta / "foo.md").write_text(
        "## MODIFIED\n\n### REQUIREMENT REQ-foo-001\n\nThe successor changes the signed contract.\n",
        encoding="utf-8",
    )
    git(ownership_repository, "add", ".")
    git(ownership_repository, "commit", "-m", "successor semantic delta")
    successor_revision = git(ownership_repository, "rev-parse", "HEAD")

    successor_entry = next(
        entry for entry in ownership_manifest["entries"] if entry["path"] == "src/foo.rs"
    )
    predecessor_digest = successor_entry["entry_digest"]
    succession_state = copy.deepcopy(ownership_state)
    succession_state["base_commit"] = predecessor_base
    succession_state["supersedes"] = [
        {
            "predecessor_id": "CHG-0000-predecessor",
            "obligations": [
                {
                    "path": "src/foo.rs",
                    "module": "foo",
                    "predecessor_entry_digest": predecessor_digest,
                }
            ],
        }
    ]
    succession = {
        "schema_version": 1,
        "tuples": [
            {
                "predecessor_id": "CHG-0000-predecessor",
                "path": "src/foo.rs",
                "module": "foo",
                "predecessor_entry_digest": predecessor_digest,
                "successor_entry_digest": successor_entry["entry_digest"],
            }
        ],
    }
    changed_successor_manifest = copy.deepcopy(ownership_manifest)
    changed_successor = next(
        entry for entry in changed_successor_manifest["entries"] if entry["path"] == "src/foo.rs"
    )
    changed_successor["payload_digest"] = "b" * 64
    changed_successor["entry_digest"] = module.acceptance_entry_digest(changed_successor)
    succession["tuples"][0]["successor_entry_digest"] = changed_successor["entry_digest"]
    assert module.semantic_succession_matches_state(
        ownership_repository,
        successor_revision,
        succession,
        succession_state,
        changed_successor_manifest,
    )
    assert not module.semantic_succession_matches_state(
        ownership_repository,
        successor_revision,
        None,
        succession_state,
        changed_successor_manifest,
    )
    forged_succession = copy.deepcopy(succession)
    forged_succession["tuples"] = []
    assert not module.semantic_succession_matches_state(
        ownership_repository,
        successor_revision,
        forged_succession,
        succession_state,
        changed_successor_manifest,
    )
    forged_predecessor = copy.deepcopy(succession)
    forged_predecessor["tuples"][0]["predecessor_entry_digest"] = "a" * 64
    succession_state_forged = copy.deepcopy(succession_state)
    succession_state_forged["supersedes"][0]["obligations"][0][
        "predecessor_entry_digest"
    ] = "a" * 64
    assert not module.semantic_succession_matches_state(
        ownership_repository,
        successor_revision,
        forged_predecessor,
        succession_state_forged,
        changed_successor_manifest,
    )
    assert module.semantic_succession_matches_state(
        ownership_repository,
        successor_revision,
        None,
        ownership_state,
        ownership_manifest,
    )
    malformed_correction = copy.deepcopy(ownership_state)
    malformed_correction["acceptance_owner_corrections"][0]["timestamp"] = -1
    assert not module.acceptance_manifest_matches_commit(
        ownership_repository,
        ownership_commit,
        ownership_manifest,
        malformed_correction,
    )
    invalid_corrections = []

    duplicate_correction = copy.deepcopy(ownership_state)
    duplicate_correction["acceptance_owner_corrections"].append(
        {
            **duplicate_correction["acceptance_owner_corrections"][0],
            "sequence": 2,
        }
    )
    invalid_corrections.append((duplicate_correction, ownership_manifest))

    out_of_scope = copy.deepcopy(ownership_state)
    out_of_scope["acceptance_owner_corrections"][0]["path"] = "src/baz.rs"
    out_of_scope["acceptance_owner_corrections"][0]["module"] = "baz"
    invalid_corrections.append((out_of_scope, ownership_manifest))

    affected_module = copy.deepcopy(ownership_state)
    affected_module["acceptance_owner_corrections"][0]["module"] = "foo"
    invalid_corrections.append((affected_module, ownership_manifest))

    non_owning_module = copy.deepcopy(ownership_state)
    non_owning_module["acceptance_owner_corrections"][0]["module"] = "baz"
    invalid_corrections.append((non_owning_module, ownership_manifest))

    noncanonical_actor = copy.deepcopy(ownership_state)
    noncanonical_actor["acceptance_owner_corrections"][0]["actor"] += " "
    invalid_corrections.append((noncanonical_actor, ownership_manifest))

    reserved_owner = copy.deepcopy(ownership_state)
    reserved_owner["acceptance_owner_corrections"][0]["module"] = "@exact:delivery"
    invalid_corrections.append((reserved_owner, ownership_manifest))

    malformed_path = copy.deepcopy(ownership_state)
    malformed_path["acceptance_owner_corrections"][0]["path"] = "src/../foo.rs"
    invalid_corrections.append((malformed_path, ownership_manifest))

    symlink_source = copy.deepcopy(ownership_state)
    symlink_source["affected_paths"].append("src/link.rs")
    symlink_source["acceptance_owner_corrections"][0]["path"] = "src/link.rs"
    symlink_manifest = ownership_manifest_with(
        signed_entry("src/link.rs", "symlink", 0o120000, b"foo.rs", ["bar"])
    )
    invalid_corrections.append((symlink_source, symlink_manifest))

    nonproduction_source = copy.deepcopy(ownership_state)
    nonproduction_source["affected_paths"].append("docs/foo.rs")
    nonproduction_source["acceptance_owner_corrections"][0]["path"] = "docs/foo.rs"
    nonproduction_manifest = ownership_manifest_with(
        signed_entry(
            "docs/foo.rs",
            "file",
            0o100644,
            b"pub fn documented_foo() {}\n",
            ["bar"],
        )
    )
    invalid_corrections.append((nonproduction_source, nonproduction_manifest))

    symlink_spec = copy.deepcopy(ownership_state)
    symlink_spec["acceptance_owner_corrections"][0]["module"] = "linkowner"
    invalid_corrections.append((symlink_spec, ownership_manifest))

    oversized_ledger = copy.deepcopy(ownership_state)
    oversized_ledger["acceptance_owner_corrections"] = [
        {
            **ownership_state["acceptance_owner_corrections"][0],
            "sequence": sequence,
            "module": f"owner-{sequence}",
        }
        for sequence in range(1, module.MAX_ACCEPTANCE_OWNER_CORRECTIONS + 2)
    ]
    invalid_corrections.append((oversized_ledger, ownership_manifest))

    for invalid_state, candidate_manifest in invalid_corrections:
        assert not module.acceptance_manifest_matches_commit(
            ownership_repository,
            ownership_commit,
            candidate_manifest,
            invalid_state,
        )

    (ownership_repository / ".specsync/config.toml").write_text(
        'sourceDirs = ["docs"]\n', encoding="utf-8"
    )
    git(ownership_repository, "add", ".specsync/config.toml")
    git(ownership_repository, "commit", "-m", "wrong TOML source-dir key")
    wrong_key_commit = git(ownership_repository, "rev-parse", "HEAD")
    assert module.configured_source_dirs(
        ownership_repository, wrong_key_commit
    ) == detected_roots

    (ownership_repository / ".specsync/config.toml").unlink()
    (ownership_repository / ".specsync/config.toml").symlink_to(
        'source_dirs = ["docs"]'
    )
    git(ownership_repository, "add", ".specsync/config.toml")
    git(ownership_repository, "commit", "-m", "symlinked source-dir config")
    symlink_config_commit = git(ownership_repository, "rev-parse", "HEAD")
    assert module.configured_source_dirs(ownership_repository, symlink_config_commit) is None

    (ownership_repository / ".specsync/config.toml").unlink()
    git(ownership_repository, "add", ".specsync/config.toml")
    git(ownership_repository, "commit", "-m", "missing source-dir config")
    missing_config_commit = git(ownership_repository, "rev-parse", "HEAD")
    assert module.configured_source_dirs(
        ownership_repository, missing_config_commit
    ) == detected_roots

    (ownership_repository / ".specsync/registry.toml").write_text(
        '[specs]\nbar = "specs/bar/bar.spec.md"\n', encoding="utf-8"
    )
    git(ownership_repository, "add", ".specsync/registry.toml")
    git(ownership_repository, "commit", "-m", "nameless mapped registry")
    nameless_registry_commit = git(ownership_repository, "rev-parse", "HEAD")
    nameless_tree = module.revision_entries(
        ownership_repository, nameless_registry_commit
    )
    assert (
        module.registry_specs_at_revision(
            ownership_repository, nameless_registry_commit, nameless_tree
        )
        is None
    )

    (ownership_repository / ".specsync/registry.toml").unlink()
    (ownership_repository / ".specsync/registry.toml").symlink_to(
        'name = "fixture"'
    )
    git(ownership_repository, "add", ".specsync/registry.toml")
    git(ownership_repository, "commit", "-m", "symlinked registry")
    symlink_registry_commit = git(ownership_repository, "rev-parse", "HEAD")
    symlink_registry_tree = module.revision_entries(
        ownership_repository, symlink_registry_commit
    )
    assert (
        module.registry_specs_at_revision(
            ownership_repository, symlink_registry_commit, symlink_registry_tree
        )
        is None
    )

with tempfile.TemporaryDirectory() as temporary:
    repository = Path(temporary)
    git(repository, "init", "-b", "main")
    git(repository, "config", "user.email", "test@example.com")
    git(repository, "config", "user.name", "Test")
    (repository / "a.txt").write_text("a\n", encoding="utf-8")
    git(repository, "add", ".")
    git(repository, "commit", "-m", "a")
    first = git(repository, "rev-parse", "HEAD")
    (repository / "b.txt").write_text("b\n", encoding="utf-8")
    commands_dir = repository / "commands"
    commands_dir.mkdir()
    (commands_dir / "hello.txt").write_text("hello\n", encoding="utf-8")
    review_dir = repository / ".specsync/changes/CHG-0001-test"
    (review_dir / "deltas").mkdir(parents=True)
    legacy_baseline = repository / ".specsync/archive/legacy-baseline.json"
    legacy_baseline.parent.mkdir(parents=True)
    legacy_baseline.write_text("{}\n", encoding="utf-8")
    archived_noise = repository / ".specsync/archive/changes/old/state.json"
    archived_noise.parent.mkdir(parents=True)
    archived_noise.write_text("{}\n", encoding="utf-8")
    (repository / ".specsync/sdd.json").write_text("{}\n", encoding="utf-8")
    (review_dir / "change.md").write_text(
        "---\nid: CHG-0001-test\nstate: verifying\n---\n# Change\n",
        encoding="utf-8",
    )
    (review_dir / "deltas/github.md").write_text("# Delta\n", encoding="utf-8")
    state = {
        "workflow_version": 2,
        "id": "CHG-0001-test",
        "state": "verifying",
        "updated_at": 1,
        "affected_paths": [".specsync", "a.txt", "commands"],
    }
    (review_dir / "state.json").write_text(
        json.dumps(state) + "\n", encoding="utf-8"
    )
    (review_dir / "approvals.json").write_text(
        json.dumps(
            {
                "approvals": [
                    {
                        "gate": "definition",
                        "actor": "Scope owner",
                        "digest": "1" * 64,
                    }
                ],
                "reopenings": [],
            }
        )
        + "\n",
        encoding="utf-8",
    )
    verification = {
        "timestamp": 1,
        "commit": "0" * 40,
        "contract_digest": "1" * 64,
        "execution_digest": "2" * 64,
        "workspace_digest": "3" * 64,
        "passed": True,
        "commands": [],
        "requirement_ids": [],
    }
    (review_dir / "verification.json").write_text(
        json.dumps(verification) + "\n", encoding="utf-8"
    )
    (review_dir / "verification-attempts.json").write_text(
        json.dumps({"schema_version": 1, "attempts": []}) + "\n",
        encoding="utf-8",
    )
    git(repository, "add", ".")
    git(repository, "commit", "-m", "b")
    product = git(repository, "rev-parse", "HEAD")
    volatile_only_state = {
        **state,
        "affected_paths": [".specsync/hashes.json"],
        "affected_specs": [],
    }
    assert module.acceptance_manifest_matches_commit(
        repository, product, empty_manifest, volatile_only_state
    )
    review = {
        "schema_version": 2,
        "change_id": "CHG-0001-test",
        "reviewer": "Independent reviewer",
        "provenance": {
            "schema_version": 1,
            "provider": "github_actions_check",
            "required_check": "SpecSync scoped review",
        },
        "verdict": "pass",
        "implementation_commit": product,
        "contract_digest": "1" * 64,
        "execution_digest": "2" * 64,
        "workspace_digest": "3" * 64,
        "timestamp": 1,
    }
    (review_dir / "review.json").write_text(
        json.dumps(review, indent=2) + "\n", encoding="utf-8"
    )
    (review_dir / "review-attempts.json").write_text(
        json.dumps({"schema_version": 1, "reviews": [review]}) + "\n",
        encoding="utf-8",
    )
    git(repository, "add", ".")
    git(repository, "commit", "-m", "review metadata")
    metadata = git(repository, "rev-parse", "HEAD")

    git(repository, "switch", "-c", "forged-review-metadata")
    forged_review = copy.deepcopy(review)
    forged_review["workspace_digest"] = "4" * 64
    (review_dir / "review.json").write_text(
        json.dumps(forged_review, indent=2) + "\n", encoding="utf-8"
    )
    (review_dir / "review-attempts.json").write_text(
        json.dumps({"schema_version": 1, "reviews": [review, forged_review]}) + "\n",
        encoding="utf-8",
    )
    git(repository, "add", ".")
    git(repository, "commit", "-m", "forge review evidence")
    forged_review_commit = git(repository, "rev-parse", "HEAD")
    assert not module.metadata_only_edge(repository, metadata, forged_review_commit)
    git(repository, "switch", "main")

    git(repository, "switch", "-c", "multi-review-append", metadata)
    blocked_review = {**review, "verdict": "block", "timestamp": 2}
    passed_review = {**review, "timestamp": 3}
    (review_dir / "review.json").write_text(
        json.dumps(passed_review, indent=2) + "\n", encoding="utf-8"
    )
    (review_dir / "review-attempts.json").write_text(
        json.dumps(
            {
                "schema_version": 1,
                "reviews": [review, blocked_review, passed_review],
            }
        )
        + "\n",
        encoding="utf-8",
    )
    git(repository, "add", ".")
    git(repository, "commit", "-m", "append multiple review attempts")
    multi_review_commit = git(repository, "rev-parse", "HEAD")
    assert not module.metadata_only_edge(repository, metadata, multi_review_commit)
    git(repository, "switch", "main")

    git(repository, "switch", "-c", "symlink-review-metadata", product)
    (review_dir / "review.json").symlink_to(json.dumps(review, separators=(",", ":")))
    (review_dir / "review-attempts.json").symlink_to(
        json.dumps(
            {"schema_version": 1, "reviews": [review]}, separators=(",", ":")
        )
    )
    git(repository, "add", ".")
    git(repository, "commit", "-m", "symlink review evidence")
    symlink_review_commit = git(repository, "rev-parse", "HEAD")
    assert not module.metadata_only_edge(repository, product, symlink_review_commit)
    git(repository, "switch", "main")

    archive_dir = (
        repository
        / ".specsync/archive/changes/2026-08-02-CHG-0001-test"
    )
    archive_dir.parent.mkdir(parents=True, exist_ok=True)
    git(repository, "mv", str(review_dir), str(archive_dir))
    metadata_tree = git(repository, "rev-parse", f"{metadata}^{{tree}}")
    archived_state = {**state, "state": "archived", "updated_at": 3}
    accepted_state = {**state, "state": "accepted", "updated_at": 2}
    (archive_dir / "state.json").write_text(
        json.dumps(archived_state) + "\n", encoding="utf-8"
    )
    (archive_dir / "accepted-state.json").write_text(
        json.dumps(accepted_state) + "\n", encoding="utf-8"
    )
    archived_change = (archive_dir / "change.md").read_text(encoding="utf-8")
    (archive_dir / "change.md").write_text(
        archived_change.replace("state: verifying\n", "state: archived\n"),
        encoding="utf-8",
    )
    acceptance_manifest = {
        "schema_version": 1,
        "entries": [
            signed_entry(".specsync", "non_file", 0, b""),
            signed_entry(
                ".specsync/archive/legacy-baseline.json",
                "file",
                0o100644,
                b"{}\n",
            ),
            signed_entry(".specsync/sdd.json", "file", 0o100644, b"{}\n"),
            signed_entry("a.txt", "file", 0o100644, b"a\n"),
            signed_entry("commands", "non_file", 0, b""),
            signed_entry("commands/hello.txt", "file", 0o100644, b"hello\n"),
        ],
    }
    omitted_descendant = copy.deepcopy(acceptance_manifest)
    omitted_descendant["entries"].pop()
    assert module.acceptance_manifest_digest(omitted_descendant) is not None
    assert not module.acceptance_manifest_matches_commit(
        repository, metadata, omitted_descendant, state
    )
    forged_missing_directory = copy.deepcopy(acceptance_manifest)
    forged_directory_entry = next(
        entry
        for entry in forged_missing_directory["entries"]
        if entry["path"] == "commands"
    )
    forged_directory_entry["kind"] = "missing"
    forged_directory_entry["entry_digest"] = module.acceptance_entry_digest(
        forged_directory_entry
    )
    assert module.acceptance_manifest_digest(forged_missing_directory) is not None
    assert not module.acceptance_manifest_matches_commit(
        repository, metadata, forged_missing_directory, state
    )
    archived_verification = {
        **verification,
        "commit": metadata,
        "acceptance_input_digest": module.acceptance_manifest_digest(
            acceptance_manifest
        ),
        "acceptance_manifest": acceptance_manifest,
    }
    (archive_dir / "verification.json").write_text(
        json.dumps(archived_verification) + "\n", encoding="utf-8"
    )
    (archive_dir / "verification-attempts.json").write_text(
        json.dumps({"schema_version": 1, "attempts": [archived_verification]})
        + "\n",
        encoding="utf-8",
    )
    finalization = {
        "schema_version": 2,
        "change_id": "CHG-0001-test",
        "implementation_commit": metadata,
        "implementation_tree": metadata_tree,
        "contract_digest": "1" * 64,
        "workspace_digest": "3" * 64,
        "closing_digest": module.closing_digest(
            "CHG-0001-test", archived_verification
        ),
        "review_digest": __import__("hashlib").sha256(
            json.dumps(review, separators=(",", ":")).encode()
        ).hexdigest(),
        "finalization_digest": "",
        "timestamp": 2,
    }
    finalization["finalization_digest"] = module.finalization_digest(finalization)
    (archive_dir / "finalization.json").write_text(
        json.dumps(finalization) + "\n", encoding="utf-8"
    )
    approvals = {
        "approvals": [
            {
                "gate": "definition",
                "actor": "Scope owner",
                "digest": "1" * 64,
            },
            {
                "gate": "finalization",
                "actor": "specsync:finalization",
                "timestamp": 2,
                "digest": finalization["closing_digest"],
                "note": "Same-PR finalization closing digest",
            }
        ],
        "reopenings": [],
    }
    (archive_dir / "approvals.json").write_text(
        json.dumps(approvals) + "\n", encoding="utf-8"
    )
    git(repository, "add", ".")
    git(repository, "commit", "-m", "archive metadata")
    archive = git(repository, "rev-parse", "HEAD")

    chain = module.first_parent_chain(repository, metadata, 10)
    assert chain == [metadata, product, first]
    assert module.first_parent_chain(repository, metadata, 2) == [metadata, product]
    assert module.metadata_only_edge(repository, product, metadata)
    assert module.metadata_only_edge(repository, metadata, archive)
    assert module.metadata_parent(repository, archive) == metadata
    assert not module.metadata_only_edge(repository, first, product)

    git(repository, "switch", "-c", "out-of-range-archive-timestamps", metadata)
    git(repository, "cherry-pick", "-n", archive)
    out_of_range_archive_dir = (
        repository / ".specsync/archive/changes/2026-08-02-CHG-0001-test"
    )
    for state_name in ("state.json", "accepted-state.json"):
        state_record = json.loads(
            (out_of_range_archive_dir / state_name).read_text(encoding="utf-8")
        )
        state_record["updated_at"] = 2**64
        (out_of_range_archive_dir / state_name).write_text(
            json.dumps(state_record) + "\n", encoding="utf-8"
        )
    out_of_range_finalization = json.loads(
        (out_of_range_archive_dir / "finalization.json").read_text(encoding="utf-8")
    )
    out_of_range_finalization["timestamp"] = 2**64
    out_of_range_finalization["finalization_digest"] = module.finalization_digest(
        out_of_range_finalization
    )
    (out_of_range_archive_dir / "finalization.json").write_text(
        json.dumps(out_of_range_finalization) + "\n", encoding="utf-8"
    )
    out_of_range_approvals = json.loads(
        (out_of_range_archive_dir / "approvals.json").read_text(encoding="utf-8")
    )
    out_of_range_approvals["approvals"][-1]["timestamp"] = 2**64
    (out_of_range_archive_dir / "approvals.json").write_text(
        json.dumps(out_of_range_approvals) + "\n", encoding="utf-8"
    )
    git(repository, "add", ".")
    git(repository, "commit", "-m", "out-of-range archive timestamps")
    out_of_range_archive = git(repository, "rev-parse", "HEAD")
    assert not module.metadata_only_edge(
        repository, metadata, out_of_range_archive
    )
    git(repository, "switch", "main")

    git(repository, "switch", "-c", "generated-review-archive", product)
    generated_active_dir = repository / ".specsync/changes/CHG-0001-test"
    generated_archive_dir = (
        repository / ".specsync/archive/changes/2026-08-02-CHG-0001-test"
    )
    generated_archive_dir.parent.mkdir(parents=True, exist_ok=True)
    git(repository, "mv", str(generated_active_dir), str(generated_archive_dir))
    (generated_archive_dir / "state.json").write_text(
        json.dumps(archived_state) + "\n", encoding="utf-8"
    )
    (generated_archive_dir / "accepted-state.json").write_text(
        json.dumps(accepted_state) + "\n", encoding="utf-8"
    )
    generated_change = (generated_archive_dir / "change.md").read_text(
        encoding="utf-8"
    )
    (generated_archive_dir / "change.md").write_text(
        generated_change.replace("state: verifying\n", "state: archived\n"),
        encoding="utf-8",
    )
    (generated_archive_dir / "review.json").write_text(
        json.dumps(review, indent=2) + "\n", encoding="utf-8"
    )
    blocked_review = {
        **review,
        "reviewer": "Earlier independent reviewer",
        "verdict": "block",
        "timestamp": 0,
    }
    (generated_archive_dir / "review-attempts.json").write_text(
        json.dumps({"schema_version": 1, "reviews": [blocked_review, review]})
        + "\n",
        encoding="utf-8",
    )
    generated_verification = {
        **verification,
        "commit": product,
        "acceptance_input_digest": module.acceptance_manifest_digest(
            acceptance_manifest
        ),
        "acceptance_manifest": acceptance_manifest,
    }
    (generated_archive_dir / "verification.json").write_text(
        json.dumps(generated_verification) + "\n", encoding="utf-8"
    )
    (generated_archive_dir / "verification-attempts.json").write_text(
        json.dumps({"schema_version": 1, "attempts": [generated_verification]})
        + "\n",
        encoding="utf-8",
    )
    generated_finalization = {
        "schema_version": 2,
        "change_id": "CHG-0001-test",
        "implementation_commit": product,
        "implementation_tree": git(repository, "rev-parse", f"{product}^{{tree}}"),
        "contract_digest": "1" * 64,
        "workspace_digest": "3" * 64,
        "closing_digest": module.closing_digest(
            "CHG-0001-test", generated_verification
        ),
        "review_digest": __import__("hashlib").sha256(
            module.compact_json(review)
        ).hexdigest(),
        "finalization_digest": "",
        "timestamp": 2,
    }
    generated_finalization["finalization_digest"] = module.finalization_digest(
        generated_finalization
    )
    (generated_archive_dir / "finalization.json").write_text(
        json.dumps(generated_finalization) + "\n", encoding="utf-8"
    )
    generated_approvals = copy.deepcopy(approvals)
    generated_approvals["approvals"][-1]["digest"] = generated_finalization[
        "closing_digest"
    ]
    (generated_archive_dir / "approvals.json").write_text(
        json.dumps(generated_approvals) + "\n", encoding="utf-8"
    )
    git(repository, "add", ".")
    git(repository, "commit", "-m", "archive with generated review evidence")
    generated_review_archive = git(repository, "rev-parse", "HEAD")
    assert module.metadata_only_edge(
        repository, product, generated_review_archive
    )

    git(repository, "switch", "-c", "tampered-archive", metadata)
    git(repository, "cherry-pick", "-n", archive)
    tampered_archive_dir = (
        repository / ".specsync/archive/changes/2026-08-02-CHG-0001-test"
    )
    (tampered_archive_dir / "deltas/github.md").write_text(
        "# Corrupted delta\n", encoding="utf-8"
    )
    git(repository, "add", ".")
    git(repository, "commit", "-m", "tampered archive metadata")
    tampered_archive = git(repository, "rev-parse", "HEAD")
    assert not module.metadata_only_edge(repository, metadata, tampered_archive)

    git(repository, "switch", "-c", "incomplete-archive", metadata)
    git(repository, "cherry-pick", "-n", archive)
    incomplete_archive_dir = (
        repository / ".specsync/archive/changes/2026-08-02-CHG-0001-test"
    )
    (incomplete_archive_dir / "deltas/github.md").unlink()
    git(repository, "add", ".")
    git(repository, "commit", "-m", "incomplete archive metadata")
    incomplete_archive = git(repository, "rev-parse", "HEAD")
    assert not module.metadata_only_edge(repository, metadata, incomplete_archive)

    git(repository, "switch", "-c", "forged-archive", metadata)
    git(repository, "cherry-pick", "-n", archive)
    forged_archive_dir = (
        repository / ".specsync/archive/changes/2026-08-02-CHG-0001-test"
    )
    forged_verification = json.loads(
        (forged_archive_dir / "verification.json").read_text(encoding="utf-8")
    )
    forged_entry = next(
        entry
        for entry in forged_verification["acceptance_manifest"]["entries"]
        if entry["path"] == "a.txt"
    )
    forged_entry["payload_digest"] = __import__("hashlib").sha256(
        b"forged\n"
    ).hexdigest()
    forged_entry["entry_digest"] = module.acceptance_entry_digest(forged_entry)
    forged_verification["acceptance_input_digest"] = module.acceptance_manifest_digest(
        forged_verification["acceptance_manifest"]
    )
    forged_closing = module.closing_digest("CHG-0001-test", forged_verification)
    (forged_archive_dir / "verification.json").write_text(
        json.dumps(forged_verification) + "\n", encoding="utf-8"
    )
    forged_attempts = json.loads(
        (forged_archive_dir / "verification-attempts.json").read_text(
            encoding="utf-8"
        )
    )
    forged_attempts["attempts"][-1] = forged_verification
    (forged_archive_dir / "verification-attempts.json").write_text(
        json.dumps(forged_attempts) + "\n", encoding="utf-8"
    )
    forged_approvals = json.loads(
        (forged_archive_dir / "approvals.json").read_text(encoding="utf-8")
    )
    forged_approvals["approvals"][-1]["digest"] = forged_closing
    (forged_archive_dir / "approvals.json").write_text(
        json.dumps(forged_approvals) + "\n", encoding="utf-8"
    )
    forged_finalization = json.loads(
        (forged_archive_dir / "finalization.json").read_text(encoding="utf-8")
    )
    forged_finalization["closing_digest"] = forged_closing
    forged_finalization["finalization_digest"] = module.finalization_digest(
        forged_finalization
    )
    (forged_archive_dir / "finalization.json").write_text(
        json.dumps(forged_finalization) + "\n", encoding="utf-8"
    )
    git(repository, "add", ".")
    git(repository, "commit", "-m", "forged archive evidence")
    forged_archive = git(repository, "rev-parse", "HEAD")
    assert not module.metadata_only_edge(repository, metadata, forged_archive)

    git(repository, "switch", "-c", "self-reviewed-archive", metadata)
    git(repository, "cherry-pick", "-n", archive)
    self_reviewed_dir = (
        repository / ".specsync/archive/changes/2026-08-02-CHG-0001-test"
    )
    self_review = json.loads(
        (self_reviewed_dir / "review.json").read_text(encoding="utf-8")
    )
    self_review["reviewer"] = "Scope owner"
    self_review["implementation_commit"] = metadata
    (self_reviewed_dir / "review.json").write_text(
        json.dumps(self_review, indent=2) + "\n", encoding="utf-8"
    )
    self_review_attempts = json.loads(
        (self_reviewed_dir / "review-attempts.json").read_text(encoding="utf-8")
    )
    self_review_attempts["reviews"].append(self_review)
    (self_reviewed_dir / "review-attempts.json").write_text(
        json.dumps(self_review_attempts) + "\n", encoding="utf-8"
    )
    self_review_finalization = json.loads(
        (self_reviewed_dir / "finalization.json").read_text(encoding="utf-8")
    )
    self_review_finalization["review_digest"] = __import__("hashlib").sha256(
        module.compact_json(self_review)
    ).hexdigest()
    self_review_finalization["finalization_digest"] = module.finalization_digest(
        self_review_finalization
    )
    (self_reviewed_dir / "finalization.json").write_text(
        json.dumps(self_review_finalization) + "\n", encoding="utf-8"
    )
    git(repository, "add", ".")
    git(repository, "commit", "-m", "self-reviewed archive evidence")
    self_reviewed_archive = git(repository, "rev-parse", "HEAD")
    assert not module.metadata_only_edge(repository, metadata, self_reviewed_archive)
    git(repository, "switch", "main")

    git(repository, "switch", "-c", "bad-archive", metadata)
    bad_archive_dir = (
        repository
        / ".specsync/archive/changes/2026-08-02-CHG-0001-test"
    )
    bad_archive_dir.parent.mkdir(parents=True, exist_ok=True)
    git(repository, "mv", str(review_dir), str(bad_archive_dir))
    (bad_archive_dir / "state.json").write_text(
        '{"workflow_version":2,"id":"CHG-0001-test","state":"archived"}\n',
        encoding="utf-8",
    )
    (bad_archive_dir / "finalization.json").write_text(
        (
            '{"schema_version":2,"change_id":"CHG-0001-test",'
            f'"implementation_commit":"{product}",'
            f'"implementation_tree":"{metadata_tree}"}}\n'
        ),
        encoding="utf-8",
    )
    git(repository, "add", ".")
    git(repository, "commit", "-m", "bad archive metadata")
    bad_archive = git(repository, "rev-parse", "HEAD")
    assert not module.metadata_only_edge(repository, metadata, bad_archive)
    git(repository, "switch", "main")
    with contextlib.redirect_stdout(io.StringIO()):
        module.check_metadata_edge_cli(repository, product, metadata)

    git(repository, "switch", "-c", "side", first)
    (repository / "side.txt").write_text("side\n", encoding="utf-8")
    git(repository, "add", ".")
    git(repository, "commit", "-m", "second parent")
    second_parent = git(repository, "rev-parse", "HEAD")
    git(repository, "switch", "main")
    git(repository, "merge", "--no-ff", "side", "-m", "merge side")
    merge = git(repository, "rev-parse", "HEAD")
    merge_chain = module.first_parent_chain(repository, merge, 10)
    assert merge_chain[:2] == [merge, archive]
    assert second_parent not in merge_chain

    git(repository, "switch", "-c", "evil-review", product)
    evil_review_dir = repository / ".specsync/changes/CHG-0002-test"
    evil_review_dir.mkdir(parents=True)
    (evil_review_dir / "review.json").write_text("{}\n", encoding="utf-8")
    (evil_review_dir / "review-attempts.json").write_text("{}\n", encoding="utf-8")
    git(repository, "add", ".")
    git(repository, "commit", "-m", "review on second parent")
    git(repository, "switch", "-c", "evil-main", product)
    git(repository, "merge", "--no-ff", "evil-review", "-m", "evil metadata merge")
    evil_merge = git(repository, "rev-parse", "HEAD")
    assert not module.metadata_only_edge(repository, product, evil_merge)
    assert module.metadata_parent(repository, evil_merge) is None
    try:
        module.check_metadata_edge_cli(repository, product, evil_merge)
    except SystemExit as error:
        assert "not an exact lifecycle metadata child" in str(error)
    else:
        raise AssertionError("CLI accepted a scoped-review merge commit")
    git(repository, "switch", "main")

    assert module.positive_bounded_limit("1") == 1
    assert module.positive_bounded_limit("32") == 32
    for invalid_limit in ("0", "33", "not-a-number"):
        try:
            module.positive_bounded_limit(invalid_limit)
        except SystemExit:
            pass
        else:
            raise AssertionError(f"accepted invalid limit {invalid_limit}")

    check = {
        "id": 20,
        "name": "trust",
        "head_sha": product,
        "status": "completed",
        "conclusion": "success",
        "app": {"id": 15368, "slug": "github-actions"},
        "details_url": "https://github.com/CorvidLabs/spec-sync/actions/runs/9001/job/10",
    }
    run = {
        "id": 9001,
        "event": "pull_request",
        "status": "completed",
        "conclusion": "success",
        "path": ".github/workflows/trust.yml",
        "head_sha": product,
        "repository": {"full_name": "CorvidLabs/spec-sync"},
        "pull_requests": [{"number": 42}],
    }
    job = {
        "id": 10,
        "run_id": 9001,
        "head_sha": product,
        "name": "trust",
        "status": "completed",
        "conclusion": "success",
        "check_run_url": "https://api.github.com/repos/CorvidLabs/spec-sync/check-runs/20",
    }
    metadata_check = {
        **check,
        "id": 21,
        "head_sha": metadata,
        "details_url": "https://github.com/CorvidLabs/spec-sync/actions/runs/9002/job/11",
    }
    metadata_run = {
        **run,
        "id": 9002,
        "head_sha": metadata,
    }
    checks_endpoint = (
        f"repos/CorvidLabs/spec-sync/commits/{product}/check-runs?per_page=100"
    )
    fixture = {
        "apps/github-actions": {
            "id": 15368,
            "slug": "github-actions",
            "name": "GitHub Actions",
            "owner": {"login": "github"},
        },
        f"repos/CorvidLabs/spec-sync/commits/{metadata}/check-runs?per_page=100": {
            "total_count": 1,
            "check_runs": [metadata_check],
        },
        checks_endpoint: {"total_count": 1, "check_runs": [check]},
        "repos/CorvidLabs/spec-sync/actions/runs/9001": run,
        "repos/CorvidLabs/spec-sync/actions/runs/9002": metadata_run,
        "repos/CorvidLabs/spec-sync/actions/jobs/10": job,
    }
    environment = {
        "REPOSITORY": "CorvidLabs/spec-sync",
        "SERVER_URL": "https://github.com",
        "PR_NUMBER": "42",
        "START_SHA": metadata,
        "CHECK_NAME": "trust",
        "WORKFLOW_PATH": ".github/workflows/trust.yml",
        "REQUIRE_RUN_SUCCESS": "true",
        "MAX_ANCESTORS": "3",
        "GIT_ROOT": str(repository),
        "CHILD_KIND": "review",
    }

    def run_case(
        candidate_fixture: dict,
        *,
        metadata_edge: bool = True,
        max_ancestors: str = "3",
        start_sha: str = metadata,
    ) -> tuple[int, str]:
        prior_environment = os.environ.copy()
        prior_api = module.api
        prior_edge = module.metadata_only_edge
        output = io.StringIO()
        try:
            os.environ.update(environment)
            os.environ["MAX_ANCESTORS"] = max_ancestors
            os.environ["START_SHA"] = start_sha
            def fixture_api(endpoint: str):
                if endpoint in candidate_fixture:
                    return candidate_fixture[endpoint]
                if endpoint.endswith("/check-runs?per_page=100"):
                    return {"total_count": 0, "check_runs": []}
                raise KeyError(endpoint)

            module.api = fixture_api
            module.metadata_only_edge = (
                prior_edge
                if metadata_edge
                else lambda _root, _parent, _child: False
            )
            with contextlib.redirect_stdout(output):
                try:
                    module.main()
                except SystemExit:
                    return 1, output.getvalue()
            return 0, output.getvalue()
        finally:
            os.environ.clear()
            os.environ.update(prior_environment)
            module.api = prior_api
            module.metadata_only_edge = prior_edge

    status, output = run_case(fixture)
    assert status == 0
    assert f"ancestor {product}" in output

    status, output = run_case(fixture, start_sha=archive)
    assert status == 0
    assert f"ancestor {product}" in output

    non_metadata = copy.deepcopy(fixture)
    non_metadata[
        f"repos/CorvidLabs/spec-sync/commits/{metadata}/check-runs?per_page=100"
    ] = {"total_count": 0, "check_runs": []}
    status, _ = run_case(non_metadata, metadata_edge=False)
    assert status == 1

    product_boundary = copy.deepcopy(fixture)
    product_boundary[checks_endpoint] = {"total_count": 0, "check_runs": []}
    product_boundary[
        f"repos/CorvidLabs/spec-sync/commits/{first}/check-runs?per_page=100"
    ] = {
        "total_count": 1,
        "check_runs": [{**check, "head_sha": first}],
    }
    status, _ = run_case(product_boundary)
    assert status == 1

    status, _ = run_case(fixture, max_ancestors="1")
    assert status == 1

    mutations = []
    wrong_app = copy.deepcopy(fixture)
    wrong_app[checks_endpoint]["check_runs"][0]["app"]["id"] = 999
    mutations.append(wrong_app)
    wrong_pr = copy.deepcopy(fixture)
    wrong_pr["repos/CorvidLabs/spec-sync/actions/runs/9001"]["pull_requests"][0][
        "number"
    ] = 99
    mutations.append(wrong_pr)
    wrong_repository = copy.deepcopy(fixture)
    wrong_repository["repos/CorvidLabs/spec-sync/actions/runs/9001"]["repository"][
        "full_name"
    ] = "CorvidLabs/other"
    mutations.append(wrong_repository)
    wrong_workflow = copy.deepcopy(fixture)
    wrong_workflow["repos/CorvidLabs/spec-sync/actions/runs/9001"][
        "path"
    ] = ".github/workflows/ci.yml"
    mutations.append(wrong_workflow)
    wrong_head = copy.deepcopy(fixture)
    wrong_head["repos/CorvidLabs/spec-sync/actions/runs/9001"]["head_sha"] = first
    mutations.append(wrong_head)
    wrong_event = copy.deepcopy(fixture)
    wrong_event["repos/CorvidLabs/spec-sync/actions/runs/9001"][
        "event"
    ] = "pull_request_target"
    mutations.append(wrong_event)
    wrong_details = copy.deepcopy(fixture)
    wrong_details[checks_endpoint]["check_runs"][0][
        "details_url"
    ] = "https://attacker.invalid/actions/runs/9001/job/10"
    mutations.append(wrong_details)
    run_level_details = copy.deepcopy(fixture)
    run_level_details[checks_endpoint]["check_runs"][0][
        "details_url"
    ] = "https://github.com/CorvidLabs/spec-sync/actions/runs/9001"
    mutations.append(run_level_details)
    wrong_job_name = copy.deepcopy(fixture)
    wrong_job_name["repos/CorvidLabs/spec-sync/actions/jobs/10"]["name"] = "other"
    mutations.append(wrong_job_name)
    wrong_job_check = copy.deepcopy(fixture)
    wrong_job_check["repos/CorvidLabs/spec-sync/actions/jobs/10"][
        "check_run_url"
    ] = "https://api.github.com/repos/CorvidLabs/spec-sync/check-runs/999"
    mutations.append(wrong_job_check)
    unsuccessful = copy.deepcopy(fixture)
    unsuccessful[checks_endpoint]["check_runs"][0]["conclusion"] = "cancelled"
    mutations.append(unsuccessful)
    incomplete = copy.deepcopy(fixture)
    incomplete[checks_endpoint]["total_count"] = 2
    mutations.append(incomplete)
    malformed = copy.deepcopy(fixture)
    malformed[checks_endpoint] = {"total_count": 1, "check_runs": "not-a-list"}
    mutations.append(malformed)

    for rejected_fixture in mutations:
        status, _ = run_case(rejected_fixture)
        assert status == 1

with tempfile.TemporaryDirectory() as temporary:
    sequence_repository = Path(temporary) / "sequence-repository"
    sequence_repository.mkdir()
    git(sequence_repository, "init", "-b", "main")
    git(sequence_repository, "config", "user.email", "test@example.com")
    git(sequence_repository, "config", "user.name", "Test")
    # This fixture intentionally creates enough loose objects to trigger Git's
    # background auto-maintenance on CI. Keep the isolated repository stable
    # while the bounded-history assertion is still writing commits.
    git(sequence_repository, "config", "gc.auto", "0")
    git(sequence_repository, "config", "maintenance.auto", "false")
    sequence_path = sequence_repository / ".specsync/change-sequence.json"
    sequence_path.parent.mkdir()
    initial_sequence = {
        "schema_version": 1,
        "sequence": 1,
        "id": "CHG-0001-test",
        "acknowledged_collisions": [],
    }
    initial_bytes = (json.dumps(initial_sequence, indent=2) + "\n").encode()
    sequence_path.write_bytes(initial_bytes)
    git(sequence_repository, "add", ".")
    git(sequence_repository, "commit", "-m", "sequence 1")
    for sequence in range(2, 259):
        sequence_path.write_text(
            json.dumps({**initial_sequence, "sequence": sequence}, indent=2) + "\n",
            encoding="utf-8",
        )
        git(sequence_repository, "add", ".")
        git(sequence_repository, "commit", "-m", f"sequence {sequence}")
    sequence_head = git(sequence_repository, "rev-parse", "HEAD")
    assert module.historical_sequence_payload(
        sequence_repository, sequence_head, {"id": "CHG-0001-test"}
    ) == initial_bytes

    original_limits_path = module.LIMITS_PATH
    limits_path = Path(temporary) / "lifecycle-validation-limits.json"
    module.LIMITS_PATH = limits_path
    try:
        limits_path.write_text(
            json.dumps({"scoped_review_max_descendants": 2}) + "\n",
            encoding="utf-8",
        )
        assert module.historical_sequence_payload(
            sequence_repository, sequence_head, {"id": "CHG-0001-test"}
        ) is None
        for invalid_limit in (None, True, 0, 1001):
            limits_path.write_text(
                json.dumps({"scoped_review_max_descendants": invalid_limit}) + "\n",
                encoding="utf-8",
            )
            assert module.sequence_history_limit() is None
        limits_path.unlink()
        assert module.sequence_history_limit() is None
        limits_path.write_text("{not-json}\n", encoding="utf-8")
        assert module.sequence_history_limit() is None
    finally:
        module.LIMITS_PATH = original_limits_path

print("reuse-check-from-ancestors tests passed")
