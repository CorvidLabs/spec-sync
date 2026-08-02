#!/usr/bin/env python3
"""Locate reusable GitHub Actions check provenance on a first-parent chain.

Used by review_only / archive_only tips so agents need not push and wait on
every intermediate metadata tip. Starting at HEAD^, it skips only exact
scoped-review pairs or parent-bound workflow-v2 archive moves, then accepts
evidence only at the first product boundary when the check is:

- named exactly CHECK_NAME
- completed/success
- authored by the official GitHub Actions app
- bound to its exact successful job in a qualifying pull_request workflow run
- head_sha equal to the ancestor under consideration

Environment:
  REPOSITORY, SERVER_URL, PR_NUMBER, START_SHA, CHECK_NAME, WORKFLOW_PATH
  REQUIRE_RUN_SUCCESS (default true)
  MAX_ANCESTORS (default 32)
  GIT_ROOT (default cwd)
  GH_TOKEN (via gh)
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import struct
import subprocess
import sys
import tomllib
import unicodedata
from pathlib import Path


MAX_ANCESTORS = 32
LIMITS_PATH = Path(__file__).with_name("lifecycle-validation-limits.json")
LEGACY_BASELINE_PATH = ".specsync/archive/legacy-baseline.json"
CANONICAL_SPEC_COMPANIONS = {
    "requirements.md",
    "tasks.md",
    "context.md",
    "testing.md",
    "design.md",
}
MAX_ACCEPTANCE_OWNER_CORRECTIONS = 1024
MAX_ACCEPTANCE_PATH_BYTES = 4096
MAX_ACCEPTANCE_OWNER_BYTES = 256
REGULAR_FILE_MODES = {"100644", "100755"}
SOURCE_EXTENSIONS = {
    "R", "bash", "c", "cc", "cjs", "clj", "cljc", "cljs", "cpp", "cr",
    "cs", "cts", "cxx", "d", "dart", "el", "erl", "ex", "exs", "fs",
    "fsi", "fsx", "go", "groovy", "gvy", "h", "hpp", "hs", "java", "js",
    "jsx", "kt", "kts", "lisp", "lsp", "lua", "m", "mjs", "ml", "mli",
    "mm", "mts", "nim", "php", "pl", "pl6", "pm", "pm6", "ps1", "py",
    "r", "rb", "rs", "scala", "scm", "sh", "swift", "ts", "tsx", "vala",
    "yaml", "yml",
}
STATIC_SOURCE_EXTENSIONS = {"html", "htm", "css"}
IGNORED_SOURCE_DIRS = {
    "node_modules", ".git", ".hg", ".svn", "dist", "build", "out",
    "target", "vendor", ".next", ".nuxt", ".output", ".cache",
    ".turbo", "coverage", "__pycache__", ".mypy_cache", ".pytest_cache",
    ".tox", ".venv", "venv", "env", ".env", ".idea", ".vscode",
    ".DS_Store", "specs", "docs", "doc", ".github", ".gitlab",
    "migrations", "Pods", ".dart_tool", ".gradle", "bin", "obj",
}


def portable_project_path_valid(path: object) -> bool:
    return (
        isinstance(path, str)
        and bool(path)
        and len(path.encode()) <= MAX_ACCEPTANCE_PATH_BYTES
        and not path.startswith(("/", "\\"))
        and not path.endswith("/")
        and "\\" not in path
        and not (
            len(path) >= 2 and path[0].isascii() and path[0].isalpha() and path[1] == ":"
        )
        and all(part not in {"", ".", ".."} for part in path.split("/"))
        and not any(unicodedata.category(character) == "Cc" for character in path)
    )


def path_matches_scope(path: str, scope: str) -> bool:
    scope = scope.replace("\\", "/").rstrip("/")
    return scope == "." or path == scope or path.startswith(f"{scope}/")


def module_name_valid(module: object) -> bool:
    if (
        not isinstance(module, str)
        or not module
        or len(module.encode()) > MAX_ACCEPTANCE_OWNER_BYTES
        or len(module.encode()) + len(".spec.md") > 255
        or module != module.strip()
        or module.endswith(".")
        or any(character in module for character in '/\\<>:"|?*')
        or any(unicodedata.category(character) == "Cc" for character in module)
        or module in {".", ".."}
    ):
        return False
    basename = module.split(".", 1)[0].upper()
    return basename not in {"CON", "PRN", "AUX", "NUL"} and not (
        len(basename) == 4
        and basename[:3] in {"COM", "LPT"}
        and basename[3] in "123456789"
    )


def spec_source_paths(root: Path, revision: str, spec_path: str) -> set[str] | None:
    try:
        frontmatter = git_bytes(root, revision, spec_path).decode("utf-8").split(
            "---", 2
        )[1]
    except (IndexError, OSError, subprocess.SubprocessError, UnicodeDecodeError):
        return None
    files: list[str] | None = None
    lines = frontmatter.splitlines()
    index = 0
    while index < len(lines):
        line = lines[index]
        stripped = line.strip()
        index += 1
        if not stripped or stripped.startswith("#") or ":" not in line:
            continue
        key, raw_value = line.split(":", 1)
        if key.strip() != "files":
            continue
        if files is not None:
            return None
        value = strip_yaml_comment(raw_value.strip())
        if value == "[]":
            files = []
        elif value.startswith("["):
            if not value.endswith("]"):
                return None
            files = []
            for raw in value[1:-1].split(","):
                item = raw.strip()
                if not item:
                    continue
                if item[:1] in {"'", '"'}:
                    if len(item) < 2 or item[-1] != item[0]:
                        return None
                    item = item[1:-1]
                files.append(item)
        elif value.startswith("{"):
            return None
        elif value:
            files = [value]
        else:
            files = []
            while index < len(lines):
                candidate = lines[index]
                candidate_stripped = candidate.strip()
                if not candidate_stripped or candidate_stripped.startswith("#"):
                    break
                if not candidate.lstrip().startswith("- "):
                    break
                item = strip_yaml_comment(candidate.lstrip()[2:].strip())
                files.append(item)
                index += 1
    sources: set[str] = set()
    for source in files or []:
        if not portable_project_path_valid(source):
            return None
        sources.add(source)
    return sources


def strip_yaml_comment(value: str) -> str:
    if value.startswith(("'", '"', "[")):
        return value
    marker = value.find(" #")
    if marker >= 0:
        after = value[marker + 2 :]
        if not after or after.startswith(" "):
            return value[:marker].rstrip()
    return value


def balanced_call_blocks(content: str, marker: str) -> list[str] | None:
    blocks: list[str] = []
    search_from = 0
    while True:
        start = content.find(marker, search_from)
        if start < 0:
            return blocks
        index = start + len(marker)
        depth = 1
        quote: str | None = None
        escaped = False
        while index < len(content):
            character = content[index]
            if quote is not None:
                if escaped:
                    escaped = False
                elif character == "\\":
                    escaped = True
                elif character == quote:
                    quote = None
            elif character in {"'", '"'}:
                quote = character
            elif character == "(":
                depth += 1
            elif character == ")":
                depth -= 1
                if depth == 0:
                    blocks.append(content[start + len(marker) : index])
                    search_from = index + 1
                    break
            index += 1
        else:
            return None


def strip_gradle_comments(content: str) -> str | None:
    output: list[str] = []
    index = 0
    while index < len(content):
        character = content[index]
        following = content[index + 1] if index + 1 < len(content) else ""
        if character in {"'", '"'} and content[index : index + 3] == character * 3:
            delimiter = character * 3
            output.extend("   ")
            index += 3
            end = content.find(delimiter, index)
            if end < 0:
                return None
            output.extend("\n" if value == "\n" else " " for value in content[index:end])
            output.extend("   ")
            index = end + 3
        elif character in {"'", '"'}:
            delimiter = character
            escaped = False
            output.append(character)
            index += 1
            while index < len(content):
                quoted = content[index]
                output.append(quoted)
                index += 1
                if escaped:
                    escaped = False
                elif quoted == "\\":
                    escaped = True
                elif quoted == delimiter:
                    break
            else:
                return None
            if escaped:
                return None
        elif character == "/" and following == "/":
            newline = content.find("\n", index + 2)
            if newline < 0:
                break
            output.append("\n")
            index = newline + 1
        elif character == "/" and following == "*":
            output.extend("  ")
            index += 2
            depth = 1
            while index < len(content) and depth:
                pair = content[index : index + 2]
                if pair == "/*":
                    depth += 1
                    output.extend("  ")
                    index += 2
                elif pair == "*/":
                    depth -= 1
                    output.extend("  ")
                    index += 2
                else:
                    output.append("\n" if content[index] == "\n" else " ")
                    index += 1
            if depth:
                return None
        else:
            output.append(character)
            index += 1
    return "".join(output)


def detected_source_dirs_at_revision(
    root: Path,
    revision: str,
    revision_tree: dict[str, tuple[str, str, str]],
) -> set[str] | None:
    """Mirror native manifest-first source-root detection over a committed tree."""

    def regular_text(path: str) -> str | None:
        entry = revision_tree.get(path)
        if entry is None:
            return ""
        if entry[0] not in REGULAR_FILE_MODES or entry[1] != "blob":
            raise ValueError(f"source manifest {path} is not a regular file")
        return git_bytes(root, revision, path).decode("utf-8")

    def directory_exists(path: str) -> bool:
        return revision_tree.get(path, (None, None, None))[1] == "tree"

    discovered: set[str] = set()
    try:
        cargo = regular_text("Cargo.toml")
        if cargo:
            document = tomllib.loads(cargo)
            if isinstance(document.get("package"), dict) and isinstance(
                document["package"].get("name"), str
            ):
                discovered.add("src")
            for target in document.get("bin", []):
                if isinstance(target, dict) and isinstance(target.get("name"), str):
                    target_path = target.get("path", f"src/bin/{target['name']}.rs")
                    if isinstance(target_path, str):
                        parent = Path(target_path).parent.as_posix()
                        discovered.add(parent if parent != "." else "src")
            workspace = document.get("workspace")
            members = workspace.get("members", []) if isinstance(workspace, dict) else []
            if not isinstance(members, list):
                return None
            for member in members:
                if not isinstance(member, str) or not portable_project_path_valid(member):
                    return None
                if member.endswith("/*"):
                    prefix = member[:-2].rstrip("/")
                    candidates = {
                        path[len(prefix) + 1 :].split("/", 1)[0]
                        for path in revision_tree
                        if path.startswith(f"{prefix}/")
                    }
                    discovered.update(f"{prefix}/{name}" for name in candidates if name)
                else:
                    discovered.add(member.rstrip("/"))

        package_swift = regular_text("Package.swift")
        if package_swift:
            targets: list[str] = []
            for marker in (".target(", ".executableTarget(", ".systemLibrary("):
                parsed = balanced_call_blocks(package_swift, marker)
                if parsed is None:
                    return None
                targets.extend(parsed)
            for target in targets:
                name = re.search(r'\bname\s*:\s*"([^"]+)"', target)
                explicit = re.search(r'\bpath\s*:\s*"([^"]+)"', target)
                if name:
                    discovered.add(
                        explicit.group(1)
                        if explicit
                        else f"Sources/{name.group(1)}"
                    )
            if not targets and directory_exists("Sources"):
                discovered.add("Sources")

        gradle_names = (
            "build.gradle.kts", "build.gradle", "settings.gradle.kts", "settings.gradle"
        )
        gradle = {name: regular_text(name) for name in gradle_names}
        if any(gradle.values()):
            build = gradle["build.gradle.kts"] or gradle["build.gradle"] or ""
            roots = (
                ("app/src/main/java", "app/src/main/kotlin", "src/main/java", "src/main/kotlin")
                if "android {" in build or "android{" in build
                else ("src/main/kotlin", "src/main/java", "src/main/scala")
            )
            discovered.update(path for path in roots if directory_exists(path))
            settings_raw = gradle["settings.gradle.kts"] or gradle["settings.gradle"] or ""
            settings = strip_gradle_comments(settings_raw)
            if settings is None:
                return None
            overrides: dict[str, str] = {}
            override_pattern = re.compile(
                r'''project\(\s*["'](?P<module>:?[^"']+)["']\s*\)\s*'''
                r'''(?:\.projectDir\s*=|\.setProjectDir\()\s*'''
                r'''(?:file\(|new\s+File\(\s*rootDir\s*,\s*)'''
                r'''["'](?P<path>[^"']+)["']'''
            )
            for match in override_pattern.finditer(settings):
                module = match.group("module").lstrip(":").replace(":", "/")
                overrides[module] = match.group("path").strip("/")
            included: set[str] = set()
            for statement in re.findall(
                r"(?ms)^\s*include\s*(?:\((.*?)\)|(.*?))\s*$", settings
            ):
                body = statement[0] or statement[1]
                included.update(
                    value.lstrip(":").replace(":", "/")
                    for value in re.findall(r'''["']\s*(:?[^"']+)["']''', body)
                    if value.strip(":")
                )
            for module in included:
                module_path = overrides.get(module, module)
                kotlin = f"{module_path}/src/main/kotlin"
                java = f"{module_path}/src/main/java"
                discovered.add(
                    kotlin if directory_exists(kotlin) else java if directory_exists(java)
                    else f"{module_path}/src/main"
                )

        package_json = regular_text("package.json")
        if package_json:
            package = json.loads(package_json)
            if not isinstance(package, dict):
                return None
            main = package.get("main", "")
            source = "src" if directory_exists("src") else "lib" if directory_exists("lib") else (
                Path(main).parent.as_posix() if isinstance(main, str) and main.startswith("./")
                else "src"
            )
            discovered.add(source)
            workspaces = package.get("workspaces", [])
            if isinstance(workspaces, dict):
                workspaces = workspaces.get("packages", [])
            if not isinstance(workspaces, list):
                return None
            for pattern in workspaces:
                if not isinstance(pattern, str):
                    return None
                base = pattern.removesuffix("/**").removesuffix("/*").rstrip("/")
                if not base or not portable_project_path_valid(base):
                    return None
                children = {
                    path[len(base) + 1 :].split("/", 1)[0]
                    for path in revision_tree
                    if path.startswith(f"{base}/")
                }
                for child in children:
                    workspace = f"{base}/{child}"
                    if f"{workspace}/package.json" in revision_tree:
                        discovered.add(
                            f"{workspace}/src"
                            if directory_exists(f"{workspace}/src") else workspace
                        )

        if regular_text("pubspec.yaml"):
            discovered.add("lib")
        if regular_text("go.mod"):
            go_roots = {
                path
                for path in ("cmd", "internal", "pkg", "api")
                if directory_exists(path)
            }
            discovered.update(go_roots or {"."})
        pyproject = regular_text("pyproject.toml")
        if pyproject:
            document = tomllib.loads(pyproject)
            project = document.get("project")
            poetry = (
                document.get("tool", {}).get("poetry", {})
                if isinstance(document.get("tool"), dict)
                else {}
            )
            name = project.get("name") if isinstance(project, dict) else None
            if not isinstance(name, str):
                name = poetry.get("name") if isinstance(poetry, dict) else None
            name = name if isinstance(name, str) else "app"
            discovered.add(
                "src"
                if directory_exists("src")
                else name if directory_exists(name) else "."
            )
    except (
        json.JSONDecodeError,
        OSError,
        subprocess.SubprocessError,
        tomllib.TOMLDecodeError,
        UnicodeDecodeError,
        ValueError,
    ):
        return None

    if discovered:
        return {path.strip("/") or "." for path in discovered}

    scanned: set[str] = set()
    root_source = False
    detectable = SOURCE_EXTENSIONS | STATIC_SOURCE_EXTENSIONS
    for path, entry in revision_tree.items():
        if entry[1] != "blob" or entry[0] not in REGULAR_FILE_MODES:
            continue
        parts = path.split("/")
        extension = Path(path).suffix.removeprefix(".")
        if extension not in detectable:
            continue
        if len(parts) == 1:
            root_source = True
            continue
        top = parts[0]
        if top.startswith(".") or top in IGNORED_SOURCE_DIRS or len(parts) - 1 > 3:
            continue
        scanned.add(top)
    if scanned:
        return scanned
    return {"."} if root_source else {"src"}

def configured_layout(
    root: Path,
    revision: str,
    revision_tree: dict[str, tuple[str, str, str]] | None = None,
) -> tuple[str, set[str] | None] | None:
    revision_tree = revision_tree or revision_entries(root, revision)
    candidates = (
        (".specsync/config.toml", "toml"),
        (".specsync/config.json", "json"),
        (".specsync.toml", "toml"),
        ("specsync.json", "json"),
    )
    for path, kind in candidates:
        entry = revision_tree.get(path)
        if entry is None:
            continue
        if entry[0] not in REGULAR_FILE_MODES or entry[1] != "blob":
            return None
        try:
            raw = git_bytes(root, revision, path).decode("utf-8")
        except (OSError, subprocess.SubprocessError, UnicodeDecodeError):
            return None
        try:
            config = tomllib.loads(raw) if kind == "toml" else json.loads(raw)
        except (tomllib.TOMLDecodeError, json.JSONDecodeError):
            return None
        if not isinstance(config, dict):
            return None
        specs_dir = config.get("specs_dir" if kind == "toml" else "specsDir", "specs")
        if not portable_project_path_valid(specs_dir):
            return None
        source_dirs = config.get("source_dirs" if kind == "toml" else "sourceDirs")
        if source_dirs is None:
            detected = detected_source_dirs_at_revision(root, revision, revision_tree)
            return (specs_dir, detected) if detected is not None else None
        if not isinstance(source_dirs, list) or any(
            not isinstance(source, str) or not source.strip("/")
            for source in source_dirs
        ):
            return None
        return specs_dir, {source.strip("/") for source in source_dirs}
    detected = detected_source_dirs_at_revision(root, revision, revision_tree)
    return ("specs", detected) if detected is not None else None


def configured_source_dirs(root: Path, revision: str) -> set[str] | None:
    layout = configured_layout(root, revision)
    return layout[1] if layout is not None else None


def registry_specs_at_revision(
    root: Path,
    revision: str,
    revision_tree: dict[str, tuple[str, str, str]],
) -> dict[str, str] | None:
    path = next(
        (
            candidate
            for candidate in (".specsync/registry.toml", "specsync-registry.toml")
            if candidate in revision_tree
        ),
        None,
    )
    if path is None:
        return {}
    mode, object_type, _object_id = revision_tree[path]
    if mode not in REGULAR_FILE_MODES or object_type != "blob":
        return None
    try:
        registry = tomllib.loads(git_bytes(root, revision, path).decode("utf-8"))
    except (
        OSError,
        subprocess.SubprocessError,
        UnicodeDecodeError,
        tomllib.TOMLDecodeError,
    ):
        return None
    if not isinstance(registry, dict):
        return None
    top_name = registry.get("name")
    nested_registry = registry.get("registry")
    if top_name is not None and (not isinstance(top_name, str) or not top_name):
        return None
    nested_name = None
    if nested_registry is not None:
        if not isinstance(nested_registry, dict):
            return None
        nested_name = nested_registry.get("name")
        if nested_name is not None and (
            not isinstance(nested_name, str) or not nested_name
        ):
            return None
    if top_name is not None and nested_name is not None and top_name != nested_name:
        return None
    name = nested_name or top_name or ""
    mappings: dict[str, str] = {}

    def add_mapping(module: object, spec_path: object) -> bool:
        if (
            not isinstance(module, str)
            or not module
            or module in mappings
            or not isinstance(spec_path, str)
            or not spec_path
        ):
            return False
        mappings[module] = spec_path
        return True

    specs = registry.get("specs")
    if specs is not None:
        if not isinstance(specs, dict):
            return None
        for module, spec_path in specs.items():
            if not add_mapping(module, spec_path):
                return None
    modules = registry.get("modules")
    if modules is not None:
        if isinstance(modules, list):
            for item in modules:
                if (
                    not isinstance(item, dict)
                    or "name" not in item
                    or "spec" not in item
                    or not add_mapping(item["name"], item["spec"])
                ):
                    return None
        elif not (isinstance(modules, dict) and not modules and not name and not mappings):
            return None
    if not name:
        return {} if not mappings else None
    return mappings


def path_is_production_source(path: str, source_dirs: set[str]) -> bool:
    extension = Path(path).suffix.removeprefix(".")
    in_source_dir = any(
        source == "." or path == source or path.startswith(f"{source}/")
        for source in source_dirs
    )
    return (
        extension in SOURCE_EXTENSIONS
        and in_source_dir
        and not path_is_governed_test_or_fixture(path)
    )


def path_is_governed_test_or_fixture(path: str) -> bool:
    return (
        path.startswith(("tests/", "test/", "Tests/"))
        or "/tests/" in path
        or "/test/" in path
        or "/fixtures/" in path
        or "/__fixtures__/" in path
    )


def path_is_recognized_delivery_metadata(path: str) -> bool:
    root_delivery_files = {
        ".trust.toml",
        "AGENTS.md",
        "Cargo.lock",
        "Cargo.toml",
        "LICENSE",
        "Package.resolved",
        "Package.swift",
        "README.md",
        "action.yaml",
        "action.yml",
        "bun.lock",
        "bun.lockb",
        "fledge.toml",
        "go.mod",
        "go.sum",
        "package-lock.json",
        "package.json",
        "pnpm-lock.yaml",
        "pyproject.toml",
        "requirements.txt",
        "specsync-registry.toml",
        "uv.lock",
        "yarn.lock",
    }
    return (
        path.startswith((".github/", ".specsync/", "docs/"))
        or "/" not in path
        and path in root_delivery_files
    )


def portable_symlink_target_valid(payload: bytes) -> bool:
    try:
        target = payload.decode("utf-8")
    except UnicodeDecodeError:
        return False
    return (
        bool(target)
        and not target.startswith("/")
        and "\\" not in target
        and not any(unicodedata.category(character) == "Cc" for character in target)
        and not (
            len(target) >= 2
            and target[0].isascii()
            and target[0].isalpha()
            and target[1] == ":"
        )
    )


def required(name: str) -> str:
    value = os.environ.get(name, "").strip()
    if not value:
        raise SystemExit(f"{name} is required")
    return value


def api(endpoint: str) -> dict | list:
    output = subprocess.check_output(
        [
            "gh",
            "api",
            "-H",
            "Accept: application/vnd.github+json",
            "-H",
            "X-GitHub-Api-Version: 2022-11-28",
            endpoint,
        ],
        text=True,
    )
    return json.loads(output)


def git(root: Path, *args: str) -> str:
    return subprocess.check_output(
        ["git", *args],
        cwd=root,
        text=True,
        timeout=30,
    ).strip()


def commit_parents(root: Path, commit: str) -> list[str]:
    fields = git(root, "rev-list", "--parents", "-n", "1", commit).split()
    return fields[1:]


def first_parent_chain(root: Path, start: str, limit: int) -> list[str]:
    chain: list[str] = []
    sha = start
    for _ in range(limit):
        if re.fullmatch(r"[0-9a-f]{40}", sha) is None:
            break
        chain.append(sha)
        parents = commit_parents(root, sha)
        if not parents:
            break
        sha = parents[0]
        if sha in chain:
            break
    return chain


def diff_records(
    root: Path, parent: str, child: str
) -> list[tuple[str, tuple[str, ...]]] | None:
    try:
        name_status = subprocess.check_output(
            ["git", "diff", "--name-status", "-z", "-M", parent, child],
            cwd=root,
            timeout=30,
        )
    except (OSError, subprocess.SubprocessError):
        return False

    try:
        fields = name_status.decode("utf-8").split("\0")
    except UnicodeDecodeError:
        return None
    if not fields or fields[-1] != "":
        return None
    fields.pop()
    records: list[tuple[str, tuple[str, ...]]] = []
    index = 0
    while index < len(fields):
        status = fields[index]
        index += 1
        path_count = 2 if status.startswith(("R", "C")) else 1
        if not status or index + path_count > len(fields):
            return None
        paths = tuple(fields[index : index + path_count])
        if any(not path for path in paths):
            return None
        records.append((status, paths))
        index += path_count
    return records


def review_metadata_only_edge(
    root: Path,
    parent: str,
    child: str,
    records: list[tuple[str, tuple[str, ...]]],
) -> bool:
    if len(records) != 2 or any(
        status not in {"A", "M"} or len(paths) != 1 for status, paths in records
    ):
        return False

    pattern = re.compile(
        r"^\.specsync/changes/(CHG-[0-9]{4,}-[^/]+)/"
        r"(review(?:-attempts)?\.json)$"
    )
    matched = [pattern.fullmatch(paths[0]) for _status, paths in records]
    if any(match is None for match in matched):
        return False
    change_ids = {match.group(1) for match in matched if match is not None}
    names = {match.group(2) for match in matched if match is not None}
    if len(change_ids) != 1 or names != {"review.json", "review-attempts.json"}:
        return False
    change_id = next(iter(change_ids))
    change_root = f".specsync/changes/{change_id}"
    review_path = f"{change_root}/review.json"
    attempts_path = f"{change_root}/review-attempts.json"
    try:
        child_entries = revision_entries(root, child)
        if any(
            child_entries.get(path, (None, None, None))[:2]
            not in {(mode, "blob") for mode in REGULAR_FILE_MODES}
            for path in (review_path, attempts_path)
        ):
            return False
        approvals = json_object_at(root, child, f"{change_root}/approvals.json")
        verification = json_object_at(root, child, f"{change_root}/verification.json")
        review = json_object_at(root, child, review_path)
        attempts = json_object_at(root, child, attempts_path)
        reviews = attempts.get("reviews")
        if (
            attempts.keys() != {"schema_version", "reviews"}
            or attempts.get("schema_version") != 1
            or not isinstance(reviews, list)
            or not reviews
            or reviews[-1] != review
            or not all(
                scoped_review_attempt_valid(attempt, change_id, approvals)
                for attempt in reviews
            )
            or review.get("contract_digest") != verification.get("contract_digest")
            or review.get("execution_digest") != verification.get("execution_digest")
            or review.get("workspace_digest") != verification.get("workspace_digest")
            or any(
                not digest_is_sha256(review.get(name))
                for name in ("contract_digest", "workspace_digest")
            )
            or review.get("execution_digest") is not None
            and not digest_is_sha256(review.get("execution_digest"))
        ):
            return False
        parent_has_review = git_object_exists(root, parent, review_path)
        parent_has_attempts = git_object_exists(root, parent, attempts_path)
        if parent_has_review != parent_has_attempts:
            return False
        if not parent_has_review:
            return review.get("implementation_commit") == parent
        parent_entries = revision_entries(root, parent)
        if any(
            parent_entries.get(path, (None, None, None))[:2]
            not in {(mode, "blob") for mode in REGULAR_FILE_MODES}
            for path in (review_path, attempts_path)
        ):
            return False
        parent_review = json_object_at(root, parent, review_path)
        parent_attempts = json_object_at(root, parent, attempts_path)
        parent_reviews = parent_attempts.get("reviews")
        parent_metadata_parent = metadata_parent(root, parent)
        expected_implementation = (
            parent_review.get("implementation_commit")
            if parent_metadata_parent is not None
            else parent
        )
        return (
            parent_attempts.keys() == {"schema_version", "reviews"}
            and parent_attempts.get("schema_version") == 1
            and isinstance(parent_reviews, list)
            and bool(parent_reviews)
            and parent_reviews[-1] == parent_review
            and reviews == parent_reviews + [review]
            and review.get("implementation_commit") == expected_implementation
        )
    except (
        json.JSONDecodeError,
        OSError,
        subprocess.SubprocessError,
        UnicodeDecodeError,
        ValueError,
    ):
        return False


def git_object_exists(root: Path, revision: str, path: str) -> bool:
    return subprocess.run(
        ["git", "cat-file", "-e", f"{revision}:{path}"],
        cwd=root,
        capture_output=True,
        timeout=30,
        check=False,
    ).returncode == 0


def git_bytes(root: Path, revision: str, path: str) -> bytes:
    return subprocess.check_output(
        ["git", "show", f"{revision}:{path}"],
        cwd=root,
        timeout=30,
    )


def revision_entries(root: Path, revision: str) -> dict[str, tuple[str, str, str]]:
    output = subprocess.check_output(
        ["git", "ls-tree", "-r", "-t", "-z", revision],
        cwd=root,
        timeout=30,
    )
    entries: dict[str, tuple[str, str, str]] = {}
    for record in output.split(b"\0"):
        if not record:
            continue
        metadata, separator, path = record.partition(b"\t")
        if not separator:
            raise ValueError("archive tree contains an invalid entry")
        mode, object_type, object_id = metadata.decode("ascii").split()
        decoded_path = path.decode("utf-8")
        if decoded_path in entries:
            raise ValueError("archive tree contains duplicate entries")
        entries[decoded_path] = (mode, object_type, object_id)
    return entries


def tree_entries(
    root: Path, revision: str, tree: str
) -> dict[str, tuple[str, str, str]]:
    prefix = f"{tree}/"
    return {
        path[len(prefix) :]: entry
        for path, entry in revision_entries(root, revision).items()
        if path.startswith(prefix) and entry[1] != "tree"
    }


def git_blob_payloads(
    root: Path, object_ids: list[str], maximum_bytes: int = 64 * 1024 * 1024
) -> dict[str, bytes]:
    unique = list(dict.fromkeys(object_ids))
    process = subprocess.Popen(
        ["git", "cat-file", "--batch"],
        cwd=root,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
    )
    if process.stdin is None or process.stdout is None:
        process.kill()
        raise ValueError("could not open bounded Git object reader")
    payloads: dict[str, bytes] = {}
    total = 0
    try:
        for object_id in unique:
            process.stdin.write(object_id.encode("ascii") + b"\n")
            process.stdin.flush()
            header = process.stdout.readline(256)
            fields = header.rstrip(b"\n").split()
            if (
                len(fields) != 3
                or fields[0].decode("ascii") != object_id
                or fields[1] != b"blob"
            ):
                raise ValueError("Git object reader returned an invalid blob header")
            size = int(fields[2])
            total += size
            if size < 0 or total > maximum_bytes:
                raise ValueError("archive evidence payload exceeds its bounded size")
            payload = process.stdout.read(size)
            if len(payload) != size or process.stdout.read(1) != b"\n":
                raise ValueError("Git object reader returned a truncated blob")
            payloads[object_id] = payload
        process.stdin.close()
        if process.wait(timeout=30) != 0:
            raise ValueError("Git object reader failed")
    except Exception:
        process.kill()
        process.wait(timeout=5)
        raise
    return payloads


def json_object_at(root: Path, revision: str, path: str) -> dict:
    value = json.loads(git_bytes(root, revision, path))
    if not isinstance(value, dict):
        raise ValueError(f"{path} is not a JSON object")
    return value


def differs_only(left: dict, right: dict, names: set[str]) -> bool:
    if left.keys() != right.keys():
        return False
    return all(left[name] == right[name] for name in left.keys() - names)


def digest_is_sha256(value: object) -> bool:
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) is not None


def u64_json_integer(value: object) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and 0 <= value < 2**64


def archive_timestamp_sequence_valid(
    parent_updated_at: object,
    accepted_updated_at: object,
    archived_updated_at: object,
    finalization_timestamp: object,
    closing_timestamp: object,
) -> bool:
    values = (
        parent_updated_at,
        accepted_updated_at,
        archived_updated_at,
        finalization_timestamp,
        closing_timestamp,
    )
    return all(u64_json_integer(value) for value in values) and (
        parent_updated_at <= accepted_updated_at <= archived_updated_at
        and accepted_updated_at <= finalization_timestamp <= archived_updated_at
        and accepted_updated_at <= closing_timestamp <= archived_updated_at
    )


def framed_digest(domain: str, frames: list[tuple[str, bytes]]) -> str:
    digest = hashlib.sha256()

    def frame(tag: str, value: bytes) -> None:
        tag_bytes = tag.encode()
        digest.update(struct.pack(">Q", len(tag_bytes)))
        digest.update(tag_bytes)
        digest.update(struct.pack(">Q", len(value)))
        digest.update(value)

    frame("domain", domain.encode())
    for tag, value in frames:
        frame(tag, value)
    return digest.hexdigest()


def finalization_digest(finalization: dict) -> str:
    return framed_digest(
        "specsync.finalization-digest.v2",
        [
            (tag, str(finalization[name]).encode())
            for tag, name in (
        ("change-id", "change_id"),
        ("implementation-commit", "implementation_commit"),
        ("implementation-tree", "implementation_tree"),
        ("contract", "contract_digest"),
        ("workspace", "workspace_digest"),
        ("closing", "closing_digest"),
        ("review", "review_digest"),
            )
        ],
    )


def acceptance_entry_digest(entry: dict) -> str:
    kind = str(entry["kind"])
    framed_kind = "non-file" if kind == "non_file" else kind
    return framed_digest(
        "specsync.acceptance-entry.v1",
        [
            ("path", str(entry["path"]).encode()),
            ("kind", framed_kind.encode()),
            ("mode", struct.pack(">I", int(entry["mode"]))),
            ("payload-digest", str(entry["payload_digest"]).encode()),
        ],
    )


def acceptance_manifest_digest(manifest: dict) -> str | None:
    if manifest.keys() != {"schema_version", "entries"} or manifest.get(
        "schema_version"
    ) != 1:
        return None
    entries = manifest.get("entries")
    if not isinstance(entries, list) or len(entries) > 100_000:
        return None
    frames: list[tuple[str, bytes]] = [("schema-version", struct.pack(">I", 1))]
    previous_path: str | None = None
    empty_digest = hashlib.sha256(b"").hexdigest()
    valid_modes = {
        "file": {0o100644, 0o100755},
        "symlink": {0o120000},
        "gitlink": {0o160000},
        "missing": {0},
        "non_file": {0},
    }
    for entry in entries:
        if not isinstance(entry, dict) or entry.keys() != {
            "path",
            "kind",
            "mode",
            "payload_digest",
            "entry_digest",
            "owners",
        }:
            return None
        path = entry.get("path")
        kind = entry.get("kind")
        mode = entry.get("mode")
        owners = entry.get("owners")
        if (
            not isinstance(path, str)
            or not path
            or len(path.encode()) > 4096
            or path.startswith(("/", "\\"))
            or path.endswith("/")
            or "\\" in path
            or any(part in {"", ".", ".."} for part in path.split("/"))
            or any(ord(character) < 32 or 127 <= ord(character) <= 159 for character in path)
            or (previous_path is not None and previous_path >= path)
            or kind not in valid_modes
            or not isinstance(mode, int)
            or isinstance(mode, bool)
            or mode not in valid_modes[kind]
            or not digest_is_sha256(entry.get("payload_digest"))
            or not digest_is_sha256(entry.get("entry_digest"))
            or entry["entry_digest"] != acceptance_entry_digest(entry)
            or kind in {"missing", "non_file"}
            and entry["payload_digest"] != empty_digest
            or not isinstance(owners, list)
            or not 1 <= len(owners) <= 1024
        ):
            return None
        previous_owner: str | None = None
        for owner in owners:
            if (
                not isinstance(owner, str)
                or not owner
                or len(owner.encode()) > 256
                or previous_owner is not None
                and previous_owner >= owner
                or owner.startswith("@exact:")
                and owner not in {"@exact:test", "@exact:delivery"}
            ):
                return None
            previous_owner = owner
        previous_path = path
        frames.extend(
            [
                ("entry", b""),
                ("path", path.encode()),
                ("kind", kind.replace("_", "-").encode()),
                ("mode", struct.pack(">I", mode)),
                ("payload-digest", entry["payload_digest"].encode()),
                ("entry-digest", entry["entry_digest"].encode()),
                *(("owner", owner.encode()) for owner in owners),
            ]
        )
    return framed_digest("specsync.acceptance-manifest.v1", frames)


def acceptance_manifest_matches_commit(
    root: Path, revision: str, manifest: dict, state: dict
) -> bool:
    entries = manifest.get("entries")
    if not isinstance(entries, list):
        return False
    by_path = {entry.get("path"): entry for entry in entries if isinstance(entry, dict)}
    affected_paths = state.get("affected_paths")
    if not isinstance(affected_paths, list) or any(
        not isinstance(path, str) for path in affected_paths
    ):
        return False
    revision_objects = revision_entries(root, revision)
    revision_tree = {
        path: entry
        for path, entry in revision_objects.items()
        if entry[1] != "tree"
    }
    expected_paths: set[str] = set()
    for path in affected_paths:
        if path.endswith("/"):
            expected_paths.update(
                candidate
                for candidate in revision_tree
                if candidate.startswith(path)
            )
        else:
            expected_paths.add(path)
            if revision_objects.get(path, (None, None, None))[1] == "tree":
                prefix = f"{path}/"
                expected_paths.update(
                    candidate
                    for candidate in revision_tree
                    if candidate.startswith(prefix)
                )
    supersedes = state.get("supersedes", [])
    if not isinstance(supersedes, list):
        return False
    for edge in supersedes:
        if not isinstance(edge, dict) or not isinstance(edge.get("obligations", []), list):
            return False
        for obligation in edge.get("obligations", []):
            if not isinstance(obligation, dict) or not isinstance(
                obligation.get("path"), str
            ):
                return False
            obligation_path = obligation["path"]
            expected_paths.add(obligation_path)
            if revision_objects.get(obligation_path, (None, None, None))[1] == "tree":
                prefix = f"{obligation_path}/"
                expected_paths.update(
                    candidate
                    for candidate in revision_tree
                    if candidate.startswith(prefix)
                )

    affected_specs = state.get("affected_specs", [])
    corrections = state.get("acceptance_owner_corrections", [])
    if (
        not isinstance(affected_specs, list)
        or any(not isinstance(module, str) for module in affected_specs)
        or not isinstance(corrections, list)
        or len(corrections) > MAX_ACCEPTANCE_OWNER_CORRECTIONS
    ):
        return False
    owners_by_path: dict[str, set[str]] = {}
    specs: dict[str, str] = {}
    layout = configured_layout(root, revision, revision_objects)
    if layout is None:
        return False
    specs_dir, source_dirs = layout
    if affected_specs or corrections:
        loaded_specs = registry_specs_at_revision(root, revision, revision_objects)
        if loaded_specs is None:
            return False
        specs = loaded_specs
    for module in affected_specs:
        if not isinstance(module, str):
            return False
        spec_path = specs.get(
            module, f"{specs_dir}/{module}/{module}.spec.md"
        )
        if (
            not isinstance(spec_path, str)
            or not portable_project_path_valid(spec_path)
            or spec_path not in revision_tree
            or revision_tree[spec_path][0] not in REGULAR_FILE_MODES
            or revision_tree[spec_path][1] != "blob"
        ):
            return False
        spec_dir = str(Path(spec_path).parent)
        companion_prefix = f"{spec_dir}/"
        companion_entries = {
            path for path in revision_tree if path.startswith(companion_prefix)
        }
        expected_paths.update(companion_entries)
        canonical_owned = {spec_path} | {
            (Path(spec_dir) / name).as_posix()
            for name in CANONICAL_SPEC_COMPANIONS
        }
        # Native ownership is path-defined: a declared missing canonical
        # companion still belongs to its module and must not fall through to
        # the exact-delivery owner merely because no tree entry exists.
        for path in canonical_owned:
            owners_by_path.setdefault(path, set()).add(module)
        sources = spec_source_paths(root, revision, spec_path)
        if sources is None:
            return False
        for source in sources:
            owners_by_path.setdefault(source, set()).add(module)
    correction_keys = {
        "schema_version",
        "sequence",
        "path",
        "module",
        "actor",
        "reason",
        "timestamp",
    }
    exact_pairs: set[tuple[str, str]] = set()
    corrected_module_sources: dict[str, set[str]] = {}
    if corrections and source_dirs is None:
        return False
    for index, correction in enumerate(corrections, start=1):
        path = correction.get("path") if isinstance(correction, dict) else None
        module = correction.get("module") if isinstance(correction, dict) else None
        pair = (path, module)
        if (
            not isinstance(correction, dict)
            or correction.keys() != correction_keys
            or correction.get("schema_version") != 1
            or correction.get("sequence") != index
            or not portable_project_path_valid(path)
            or not module_name_valid(module)
            or not isinstance(correction.get("actor"), str)
            or not correction["actor"].strip()
            or correction["actor"] != correction["actor"].strip()
            or not isinstance(correction.get("reason"), str)
            or not correction["reason"].strip()
            or correction["reason"] != correction["reason"].strip()
            or isinstance(correction.get("timestamp"), bool)
            or not isinstance(correction.get("timestamp"), int)
            or not 0 <= correction["timestamp"] < 2**64
            or not any(path_matches_scope(path, scope) for scope in affected_paths)
            or module in affected_specs
            or pair in exact_pairs
        ):
            return False
        exact_pairs.add(pair)
        if module not in corrected_module_sources:
            corrected_spec = specs.get(
                module, f"{specs_dir}/{module}/{module}.spec.md"
            )
            if (
                not isinstance(corrected_spec, str)
                or not portable_project_path_valid(corrected_spec)
                or corrected_spec not in revision_tree
                or revision_tree[corrected_spec][0] not in REGULAR_FILE_MODES
                or revision_tree[corrected_spec][1] != "blob"
            ):
                return False
            corrected_sources = spec_source_paths(root, revision, corrected_spec)
            if corrected_sources is None:
                return False
            corrected_module_sources[module] = corrected_sources
        tree_entry = revision_tree.get(path)
        if (
            tree_entry is None
            or tree_entry[1] != "blob"
            or tree_entry[0] not in REGULAR_FILE_MODES
            or not path_is_production_source(path, source_dirs or set())
            or path not in corrected_module_sources[module]
        ):
            return False
        owners_by_path.setdefault(path, set()).add(module)
    expected_paths = {
        path
        for path in expected_paths
        if not project_input_is_volatile(path) or path == LEGACY_BASELINE_PATH
    }
    if expected_paths != set(by_path):
        return False

    blob_ids = [
        revision_tree[entry["path"]][2]
        for entry in entries
        if entry["kind"] in {"file", "symlink"}
        and entry["path"] in revision_tree
        and revision_tree[entry["path"]][1] == "blob"
    ]
    blob_payloads = git_blob_payloads(root, blob_ids)
    for entry in entries:
        path = entry["path"]
        expected_owners = sorted(owners_by_path.get(path, set()))
        if path_is_governed_test_or_fixture(path):
            if entry["owners"] != ["@exact:test"]:
                return False
        elif path_is_recognized_delivery_metadata(path):
            if entry["owners"] != ["@exact:delivery"]:
                return False
        elif expected_owners:
            if entry["owners"] != expected_owners:
                return False
        else:
            source_extension = Path(path).suffix.removeprefix(".") in SOURCE_EXTENSIONS
            unowned_production_source = (
                source_extension
                and not path_is_governed_test_or_fixture(path)
                and (
                    source_dirs is None
                    or path_is_production_source(path, source_dirs)
                )
            )
            if unowned_production_source or entry["owners"] != ["@exact:delivery"]:
                return False
        kind = entry["kind"]
        is_non_file = kind == "non_file"
        if kind == "missing":
            if path in revision_objects or entry["payload_digest"] != hashlib.sha256(
                b""
            ).hexdigest():
                return False
            continue
        tree_entry = (
            revision_objects.get(path)
            if is_non_file
            else revision_tree.get(path)
        )
        if tree_entry is None:
            return False
        mode, object_type, object_id = tree_entry
        if not is_non_file and int(mode, 8) != entry["mode"]:
            return False
        if is_non_file:
            if object_type != "tree":
                return False
            payload = b""
        elif kind == "gitlink":
            if object_type != "commit":
                return False
            payload = object_id.encode()
        else:
            if object_type != "blob":
                return False
            payload = blob_payloads.get(object_id)
            if payload is None:
                return False
            if kind == "symlink" and not portable_symlink_target_valid(payload):
                return False
        if path == ".specsync/change-sequence.json":
            payload = historical_sequence_payload(root, revision, state)
            if payload is None:
                return False
        if hashlib.sha256(payload).hexdigest() != entry["payload_digest"]:
            return False
    return True


def project_input_is_volatile(path: str) -> bool:
    prefixes = (
        ".git/",
        "target/",
        "node_modules/",
        "site/node_modules/",
        "site/dist/",
        "site/.astro/",
        ".specsync/changes/",
        ".specsync/archive/",
    )
    if any(
        path == prefix.removesuffix("/") or path.startswith(prefix)
        for prefix in prefixes
    ):
        return True
    return path in {
        ".specsync/hashes.json",
        ".specsync/change.lock",
        ".specsync/change-transaction.json",
    }


def historical_sequence_payload(
    root: Path, revision: str, state: dict
) -> bytes | None:
    match = re.match(r"^CHG-([0-9]+)-", str(state.get("id", "")))
    if match is None:
        return None
    sequence = int(match.group(1))
    path = ".specsync/change-sequence.json"
    current = json.loads(git_bytes(root, revision, path))
    if not isinstance(current, dict) or not isinstance(current.get("sequence"), int):
        return None
    if current["sequence"] <= sequence:
        return git_bytes(root, revision, path)
    history_limit = sequence_history_limit()
    if history_limit is None:
        return None
    commits = git(
        root,
        "rev-list",
        f"--max-count={history_limit + 1}",
        revision,
        "--",
        path,
    ).splitlines()
    if len(commits) > history_limit:
        return None
    for commit in commits:
        try:
            content = git_bytes(root, commit, path)
            candidate = json.loads(content)
        except (json.JSONDecodeError, subprocess.SubprocessError):
            continue
        if (
            isinstance(candidate, dict)
            and candidate.get("sequence") == sequence
            and candidate.get("id") == state.get("id")
        ):
            return content
    collisions = current.get("acknowledged_collisions", [])
    if not isinstance(collisions, list):
        return None
    historical = {
        "schema_version": current.get("schema_version"),
        "sequence": sequence,
        "id": state.get("id"),
        "acknowledged_collisions": [
            collision
            for collision in collisions
            if isinstance(collision, dict)
            and isinstance(collision.get("sequence"), int)
            and collision["sequence"] <= sequence
        ],
    }
    return (json.dumps(historical, indent=2) + "\n").encode()


def sequence_history_limit() -> int | None:
    try:
        limits = json.loads(LIMITS_PATH.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError):
        return None
    value = (
        limits.get("scoped_review_max_descendants")
        if isinstance(limits, dict)
        else None
    )
    if (
        isinstance(value, bool)
        or not isinstance(value, int)
        or not 1 <= value <= 1000
    ):
        return None
    return value


def semantic_succession_digest(evidence: object) -> str | None:
    if not isinstance(evidence, dict) or evidence.keys() != {"schema_version", "tuples"}:
        return None
    tuples = evidence.get("tuples")
    if evidence.get("schema_version") != 1 or not isinstance(tuples, list):
        return None
    frames: list[tuple[str, bytes]] = [("schema-version", struct.pack(">I", 1))]
    previous: tuple[int, str, str, str] | None = None
    for item in tuples:
        if not isinstance(item, dict) or item.keys() != {
            "predecessor_id",
            "path",
            "module",
            "predecessor_entry_digest",
            "successor_entry_digest",
        }:
            return None
        match = re.match(r"^CHG-([0-9]+)-", str(item.get("predecessor_id", "")))
        key = (
            int(match.group(1)) if match else -1,
            str(item.get("predecessor_id", "")),
            str(item.get("path", "")),
            str(item.get("module", "")),
        )
        if (
            match is None
            or previous is not None
            and previous >= key
            or not portable_project_path_valid(item.get("path"))
            or not module_name_valid(item.get("module"))
            or not digest_is_sha256(item.get("predecessor_entry_digest"))
            or not digest_is_sha256(item.get("successor_entry_digest"))
        ):
            return None
        previous = key
        frames.extend(
            [
                ("tuple", b""),
                ("predecessor-id", key[1].encode()),
                ("path", key[2].encode()),
                ("module", key[3].encode()),
                ("predecessor-entry-digest", item["predecessor_entry_digest"].encode()),
                ("successor-entry-digest", item["successor_entry_digest"].encode()),
            ]
        )
    return framed_digest("specsync.semantic-succession.v1", frames)


def change_root_at_revision(
    root: Path,
    revision: str,
    change_id: str,
) -> tuple[str, dict] | None:
    entries = revision_entries(root, revision)
    candidates: dict[str, str] = {}
    active = f".specsync/changes/{change_id}"
    if f"{active}/state.json" in entries:
        candidates[active] = f"{active}/state.json"
    archive_prefix = ".specsync/archive/changes/"
    for path in entries:
        if not path.startswith(archive_prefix):
            continue
        relative = path[len(archive_prefix) :]
        directory, separator, name = relative.partition("/")
        if separator and directory.endswith(change_id) and name in {
            "accepted-state.json", "state.json"
        }:
            archive_root = f"{archive_prefix}{directory}"
            if name == "accepted-state.json" or archive_root not in candidates:
                candidates[archive_root] = path
    if len(candidates) != 1:
        return None
    directory, state_path = candidates.popitem()
    entry = entries.get(state_path)
    if entry is None or entry[0] not in REGULAR_FILE_MODES or entry[1] != "blob":
        return None
    try:
        state = json_object_at(root, revision, state_path)
    except (
        json.JSONDecodeError,
        OSError,
        subprocess.SubprocessError,
        UnicodeDecodeError,
        ValueError,
    ):
        return None
    if state.get("id") != change_id:
        return None
    return directory, state


def predecessor_entry_digest_at_base(
    root: Path,
    base: str,
    predecessor_id: str,
    path: str,
) -> str | None:
    located = change_root_at_revision(root, base, predecessor_id)
    if located is None:
        return None
    predecessor_root, predecessor_state = located
    if predecessor_state.get("state") not in {"accepted", "archived"}:
        return None
    del predecessor_root
    objects = revision_entries(root, base)
    tree_entry = objects.get(path)
    try:
        if tree_entry is None:
            kind = "missing"
            mode = 0
            payload = b""
        elif tree_entry[1] == "tree":
            kind = "non_file"
            mode = 0
            payload = b""
        elif tree_entry[0] in REGULAR_FILE_MODES and tree_entry[1] == "blob":
            kind = "file"
            mode = int(tree_entry[0], 8)
            payload = git_bytes(root, base, path)
        elif tree_entry[0] == "120000" and tree_entry[1] == "blob":
            kind = "symlink"
            mode = 0o120000
            payload = git_bytes(root, base, path)
            if not portable_symlink_target_valid(payload):
                return None
        elif tree_entry[0] == "160000" and tree_entry[1] == "commit":
            kind = "gitlink"
            mode = 0o160000
            payload = tree_entry[2].encode("ascii")
        else:
            return None
    except (OSError, subprocess.SubprocessError, UnicodeDecodeError, ValueError):
        return None
    entry = {
        "path": path,
        "kind": kind,
        "mode": mode,
        "payload_digest": hashlib.sha256(payload).hexdigest(),
    }
    return acceptance_entry_digest(entry)


def semantic_delta_has_nonremoved_item(
    root: Path,
    revision: str,
    change_id: object,
    module: str,
) -> bool:
    if not isinstance(change_id, str):
        return False
    path = f".specsync/changes/{change_id}/deltas/{module}.md"
    entries = revision_entries(root, revision)
    entry = entries.get(path)
    if entry is None or entry[0] not in REGULAR_FILE_MODES or entry[1] != "blob":
        return False
    try:
        content = git_bytes(root, revision, path).decode("utf-8")
    except (OSError, subprocess.SubprocessError, UnicodeDecodeError):
        return False
    operation: str | None = None
    target = False
    body: list[str] = []

    def valid_item() -> bool:
        return operation in {"ADDED", "MODIFIED"} and target and bool("\n".join(body).strip())

    for line in content.splitlines():
        if line.startswith("## "):
            if valid_item():
                return True
            operation = line[3:].strip().upper()
            target = False
            body = []
        elif line.startswith("### "):
            if valid_item():
                return True
            heading = line[4:]
            target = heading.startswith(("REQUIREMENT ", "SPEC SECTION ")) and bool(
                heading.split(" ", 1)[1].strip()
            )
            body = []
        elif target:
            body.append(line)
    return valid_item()


def semantic_succession_matches_state(
    root: Path,
    revision: str,
    evidence: object,
    state: dict,
    manifest: dict,
) -> bool:
    expected: set[tuple[str, str, str, str]] = set()
    supersedes = state.get("supersedes", [])
    if not isinstance(supersedes, list):
        return False
    for edge in supersedes:
        predecessor = edge.get("predecessor_id") if isinstance(edge, dict) else None
        obligations = edge.get("obligations") if isinstance(edge, dict) else None
        if (
            not isinstance(predecessor, str)
            or re.fullmatch(r"CHG-[0-9]+-.+", predecessor) is None
            or not isinstance(obligations, list)
        ):
            return False
        for obligation in obligations:
            if not isinstance(obligation, dict):
                return False
            item = (
                predecessor,
                obligation.get("path"),
                obligation.get("module"),
                obligation.get("predecessor_entry_digest"),
            )
            if (
                not portable_project_path_valid(item[1])
                or not module_name_valid(item[2])
                or not digest_is_sha256(item[3])
                or item in expected
            ):
                return False
            expected.add(item)
    if not expected:
        return evidence is None
    base = state.get("base_commit")
    if (
        not isinstance(base, str)
        or re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", base) is None
        or subprocess.run(
            ["git", "merge-base", "--is-ancestor", base, revision],
            cwd=root,
            capture_output=True,
            timeout=30,
            check=False,
        ).returncode
        != 0
    ):
        return False
    if semantic_succession_digest(evidence) is None or not isinstance(evidence, dict):
        return False
    tuples = evidence.get("tuples")
    entries = manifest.get("entries") if isinstance(manifest, dict) else None
    if not isinstance(tuples, list) or not isinstance(entries, list):
        return False
    entries_by_path = {
        entry.get("path"): entry for entry in entries if isinstance(entry, dict)
    }
    actual: set[tuple[str, str, str, str]] = set()
    for item in tuples:
        if not isinstance(item, dict):
            return False
        obligation = (
            item.get("predecessor_id"),
            item.get("path"),
            item.get("module"),
            item.get("predecessor_entry_digest"),
        )
        successor = entries_by_path.get(item.get("path"))
        if (
            obligation in actual
            or obligation not in expected
            or not isinstance(successor, dict)
            or item.get("module") not in successor.get("owners", [])
            or item.get("successor_entry_digest") != successor.get("entry_digest")
            or item.get("successor_entry_digest")
            == item.get("predecessor_entry_digest")
            or predecessor_entry_digest_at_base(
                root,
                base,
                item.get("predecessor_id"),
                item.get("path"),
            )
            != item.get("predecessor_entry_digest")
            or not semantic_delta_has_nonremoved_item(
                root, revision, state.get("id"), item.get("module")
            )
        ):
            return False
        actual.add(obligation)
    return actual == expected


def compact_json(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=False, separators=(",", ":")).encode()


def closing_digest(change_id: str, verification: dict) -> str | None:
    manifest = verification.get("acceptance_manifest")
    acceptance = verification.get("acceptance_input_digest")
    if acceptance_manifest_digest(manifest) != acceptance:
        return None
    frames: list[tuple[str, bytes]] = [
        ("record-id", change_id.encode()),
        ("contract", str(verification.get("contract_digest", "")).encode()),
    ]
    execution = verification.get("execution_digest")
    if execution is not None:
        frames.append(("execution", str(execution).encode()))
    frames.extend(
        [
            ("workspace", str(verification.get("workspace_digest", "")).encode()),
            ("commit", str(verification.get("commit") or "").encode()),
            ("acceptance", b"\x01" + str(acceptance).encode()),
            ("acceptance-input-manifest-v1", compact_json(manifest)),
        ]
    )
    succession = verification.get("semantic_succession")
    if succession is not None:
        succession_digest = semantic_succession_digest(succession)
        if succession_digest is None:
            return None
        frames.append(("semantic-succession-v1", succession_digest.encode()))
    return framed_digest("specsync.closing-digest.v2", frames)


def definition_approver(approvals: dict, contract_digest: object) -> str | None:
    items = approvals.get("approvals")
    if not isinstance(items, list) or not isinstance(contract_digest, str):
        return None
    for item in reversed(items):
        if not isinstance(item, dict) or item.get("gate") != "definition":
            continue
        definition_pair = item.get("definition_pair")
        pair_matches = isinstance(definition_pair, dict) and contract_digest in {
            definition_pair.get("current_digest"),
            definition_pair.get("legacy_digest"),
        }
        if (
            (item.get("digest") == contract_digest or pair_matches)
            and isinstance(item.get("actor"), str)
        ):
            return item["actor"].strip()
    adoptions = approvals.get("scope_adoptions", [])
    if not isinstance(adoptions, list):
        return None
    for adoption in reversed(adoptions):
        if (
            not isinstance(adoption, dict)
            or adoption.get("adopted_scope_digest") != contract_digest
            or not isinstance(adoption.get("source_approval_index"), int)
        ):
            continue
        index = adoption["source_approval_index"]
        if 0 <= index < len(items):
            source = items[index]
            if (
                isinstance(source, dict)
                and source.get("gate") == "definition"
                and isinstance(source.get("actor"), str)
            ):
                return source["actor"].strip()
    return None


def scoped_review_attempt_valid(
    attempt: object, change_id: str, approvals: dict
) -> bool:
    expected_keys = {
        "schema_version",
        "change_id",
        "reviewer",
        "provenance",
        "verdict",
        "implementation_commit",
        "contract_digest",
        "workspace_digest",
        "timestamp",
    }
    if isinstance(attempt, dict) and "execution_digest" in attempt:
        expected_keys.add("execution_digest")
    reviewer = attempt.get("reviewer") if isinstance(attempt, dict) else None
    canonical_reviewer = reviewer.strip() if isinstance(reviewer, str) else ""
    implementation_commit = (
        attempt.get("implementation_commit") if isinstance(attempt, dict) else None
    )
    timestamp = attempt.get("timestamp") if isinstance(attempt, dict) else None
    approver = (
        definition_approver(approvals, attempt.get("contract_digest"))
        if isinstance(attempt, dict)
        else None
    )
    return (
        isinstance(attempt, dict)
        and attempt.keys() == expected_keys
        and attempt.get("schema_version") == 2
        and attempt.get("change_id") == change_id
        and isinstance(reviewer, str)
        and 1 <= len(canonical_reviewer) <= 128
        and canonical_reviewer.isascii()
        and all(
            character.isalnum() or character in " ._:@/-"
            for character in canonical_reviewer
        )
        and attempt.get("verdict") in {"pass", "block"}
        and attempt.get("provenance")
        == {
            "schema_version": 1,
            "provider": "github_actions_check",
            "required_check": "SpecSync scoped review",
        }
        and isinstance(implementation_commit, str)
        and re.fullmatch(r"[0-9a-f]{40}", implementation_commit) is not None
        and digest_is_sha256(attempt.get("contract_digest"))
        and (
            attempt.get("execution_digest") is None
            or digest_is_sha256(attempt.get("execution_digest"))
        )
        and digest_is_sha256(attempt.get("workspace_digest"))
        and isinstance(timestamp, int)
        and not isinstance(timestamp, bool)
        and 0 <= timestamp < 2**64
        and approver is not None
        and reviewer.casefold() != approver.casefold()
    )


def canonical_archive_transition(
    root: Path,
    parent: str,
    child: str,
    change_id: str,
    active_root: str,
    archive_root: str,
    parent_tree: str,
) -> bool:
    mutable = {
        "approvals.json",
        "change.md",
        "review-attempts.json",
        "review.json",
        "state.json",
        "verification-attempts.json",
        "verification.json",
    }
    generated = {"accepted-state.json", "finalization.json"}
    try:
        active_entries = tree_entries(root, parent, active_root)
        archive_entries = tree_entries(root, child, archive_root)
        parent_review_names = active_entries.keys() & {
            "review.json",
            "review-attempts.json",
        }
        if parent_review_names not in (
            set(),
            {"review.json", "review-attempts.json"},
        ):
            return False
        if not parent_review_names:
            generated |= {"review.json", "review-attempts.json"}
        if archive_entries.keys() != active_entries.keys() | generated:
            return False
        for relative, entry in active_entries.items():
            if relative not in mutable and archive_entries.get(relative) != entry:
                return False

        parent_state = json_object_at(root, parent, f"{active_root}/state.json")
        archived_state = json_object_at(root, child, f"{archive_root}/state.json")
        accepted_state = json_object_at(
            root, child, f"{archive_root}/accepted-state.json"
        )
        finalization = json_object_at(root, child, f"{archive_root}/finalization.json")
        if (
            not differs_only(parent_state, archived_state, {"state", "updated_at"})
            or not differs_only(parent_state, accepted_state, {"state", "updated_at"})
            or parent_state.get("workflow_version") != 2
            or parent_state.get("id") != change_id
            or parent_state.get("state") != "verifying"
            or archived_state.get("state") != "archived"
            or accepted_state.get("state") != "accepted"
        ):
            return False

        parent_change = git_bytes(root, parent, f"{active_root}/change.md")
        archived_change = git_bytes(root, child, f"{archive_root}/change.md")
        expected_change, replacements = re.subn(
            br"(?m)^state: (?:implementing|verifying)$",
            b"state: archived",
            parent_change,
        )
        if replacements != 1 or archived_change != expected_change:
            return False

        parent_approvals = json_object_at(root, parent, f"{active_root}/approvals.json")
        archived_approvals = json_object_at(root, child, f"{archive_root}/approvals.json")
        parent_items = parent_approvals.get("approvals")
        archived_items = archived_approvals.get("approvals")
        if (
            not differs_only(parent_approvals, archived_approvals, {"approvals"})
            or not isinstance(parent_items, list)
            or not isinstance(archived_items, list)
            or archived_items[:-1] != parent_items
            or len(archived_items) != len(parent_items) + 1
        ):
            return False
        closing = archived_items[-1]

        parent_verification = json_object_at(
            root, parent, f"{active_root}/verification.json"
        )
        archived_verification = json_object_at(
            root, child, f"{archive_root}/verification.json"
        )
        inherited = {
            name: value
            for name, value in archived_verification.items()
            if name
            not in {
                "commit",
                "acceptance_input_digest",
                "acceptance_manifest",
                "semantic_succession",
            }
        }
        parent_inherited = {
            name: value
            for name, value in parent_verification.items()
            if name not in {"commit", "semantic_succession"}
        }
        if inherited != parent_inherited or archived_verification.get("commit") != parent:
            return False
        manifest = archived_verification.get("acceptance_manifest")
        if (
            acceptance_manifest_digest(manifest)
            != archived_verification.get("acceptance_input_digest")
            or not isinstance(manifest, dict)
            or not acceptance_manifest_matches_commit(root, parent, manifest, parent_state)
            or not semantic_succession_matches_state(
                root,
                parent,
                archived_verification.get("semantic_succession"),
                parent_state,
                manifest,
            )
            or archived_verification.get("passed") is not True
        ):
            return False

        parent_attempts = json_object_at(
            root, parent, f"{active_root}/verification-attempts.json"
        )
        archived_attempts = json_object_at(
            root, child, f"{archive_root}/verification-attempts.json"
        )
        if (
            parent_attempts.keys() != archived_attempts.keys()
            or parent_attempts.get("schema_version") != 1
            or not isinstance(parent_attempts.get("attempts"), list)
            or archived_attempts.get("attempts")
            != parent_attempts["attempts"] + [archived_verification]
        ):
            return False

        expected_finalization_keys = {
            "schema_version",
            "change_id",
            "implementation_commit",
            "implementation_tree",
            "contract_digest",
            "workspace_digest",
            "closing_digest",
            "review_digest",
            "finalization_digest",
            "timestamp",
        }
        if (
            finalization.keys() != expected_finalization_keys
            or finalization.get("schema_version") != 2
            or finalization.get("change_id") != change_id
            or finalization.get("implementation_commit") != parent
            or finalization.get("implementation_tree") != parent_tree
            or any(
                not digest_is_sha256(finalization.get(name))
                for name in (
                    "contract_digest",
                    "workspace_digest",
                    "closing_digest",
                    "review_digest",
                    "finalization_digest",
                )
            )
            or finalization.get("contract_digest")
            != archived_verification.get("contract_digest")
            or finalization.get("workspace_digest")
            != archived_verification.get("workspace_digest")
            or finalization.get("closing_digest")
            != closing_digest(change_id, archived_verification)
            or finalization_digest(finalization)
            != finalization.get("finalization_digest")
        ):
            return False
        review = json_object_at(root, child, f"{archive_root}/review.json")
        archived_reviews = json_object_at(
            root, child, f"{archive_root}/review-attempts.json"
        )
        if parent_review_names:
            parent_review = json_object_at(
                root, parent, f"{active_root}/review.json"
            )
            parent_reviews = json_object_at(
                root, parent, f"{active_root}/review-attempts.json"
            )
        else:
            parent_review = None
            parent_reviews = {"schema_version": 1, "reviews": []}
        parent_review_items = parent_reviews.get("reviews")
        archived_review_items = archived_reviews.get("reviews")
        review_ledger_valid = (
            archived_reviews.keys() == {"schema_version", "reviews"}
            and archived_reviews.get("schema_version") == 1
            and isinstance(archived_review_items, list)
            and bool(archived_review_items)
            and archived_review_items[-1] == review
            and all(
                scoped_review_attempt_valid(attempt, change_id, parent_approvals)
                for attempt in archived_review_items
            )
        )
        review_unchanged = (
            parent_review is not None
            and review == parent_review
            and archived_reviews == parent_reviews
        )
        review_appended = (
            parent_reviews.keys() == archived_reviews.keys()
            and parent_reviews.get("schema_version") == 1
            and isinstance(parent_review_items, list)
            and isinstance(archived_review_items, list)
            and archived_review_items == parent_review_items + [review]
        )
        review_generated = not parent_review_names and review_ledger_valid
        review_commit = str(review.get("implementation_commit") or "")
        review_digest = hashlib.sha256(compact_json(review)).hexdigest()
        approver = definition_approver(parent_approvals, review.get("contract_digest"))
        if (
            not review_ledger_valid
            or not (review_unchanged or review_appended or review_generated)
            or review.get("schema_version") != 2
            or review.get("change_id") != change_id
            or review.get("verdict") != "pass"
            or not isinstance(review.get("reviewer"), str)
            or not review["reviewer"].strip()
            or not review["reviewer"].isascii()
            or approver is None
            or review["reviewer"].strip().casefold() == approver.casefold()
            or review.get("provenance")
            != {
                "schema_version": 1,
                "provider": "github_actions_check",
                "required_check": "SpecSync scoped review",
            }
            or re.fullmatch(r"[0-9a-f]{40}", review_commit) is None
            or (review_appended or review_generated)
            and review_commit != parent
            or finalization["review_digest"] != review_digest
            or finalization["contract_digest"] != review.get("contract_digest")
            or finalization["workspace_digest"] != review.get("workspace_digest")
            or archived_verification.get("execution_digest")
            != review.get("execution_digest")
            or not isinstance(closing, dict)
            or closing.keys() != {"gate", "actor", "timestamp", "digest", "note"}
            or closing.get("gate") != "finalization"
            or closing.get("actor") != "specsync:finalization"
            or closing.get("digest") != finalization["closing_digest"]
            or closing.get("note") != "Same-PR finalization closing digest"
            or not archive_timestamp_sequence_valid(
                parent_state.get("updated_at"),
                accepted_state.get("updated_at"),
                archived_state.get("updated_at"),
                finalization.get("timestamp"),
                closing.get("timestamp"),
            )
        ):
            return False
    except (
        json.JSONDecodeError,
        KeyError,
        OSError,
        subprocess.SubprocessError,
        UnicodeDecodeError,
        ValueError,
    ):
        return False
    return True


def archive_metadata_only_edge(
    root: Path,
    parent: str,
    child: str,
    records: list[tuple[str, tuple[str, ...]]],
) -> bool:
    active_pattern = re.compile(
        r"^\.specsync/changes/(?P<change>CHG-[0-9]{4,}-[^/]+)/(?P<relative>.+)$"
    )
    archive_pattern = re.compile(
        r"^\.specsync/archive/changes/"
        r"(?P<dated>[0-9]{4}-[0-9]{2}-[0-9]{2}-(?P<change>CHG-[0-9]{4,}-[^/]+))/"
        r"(?P<relative>.+)$"
    )
    change_id: str | None = None
    archive_dir: str | None = None
    active_seen = False
    archive_seen = False

    def bind(match: re.Match[str] | None, *, archive: bool) -> bool:
        nonlocal change_id, archive_dir, active_seen, archive_seen
        if match is None:
            return False
        candidate_id = match.group("change")
        candidate_dir = match.groupdict().get("dated")
        if change_id is not None and candidate_id != change_id:
            return False
        if archive and archive_dir is not None and candidate_dir != archive_dir:
            return False
        change_id = candidate_id
        if archive:
            archive_dir = candidate_dir
            archive_seen = True
        else:
            active_seen = True
        return True

    for status, paths in records:
        kind = status[:1]
        if kind == "R" and len(paths) == 2:
            active = active_pattern.fullmatch(paths[0])
            archive = archive_pattern.fullmatch(paths[1])
            if (
                not bind(active, archive=False)
                or not bind(archive, archive=True)
                or active is None
                or archive is None
                or active.group("relative") != archive.group("relative")
            ):
                return False
        elif kind == "D" and len(paths) == 1:
            if not bind(active_pattern.fullmatch(paths[0]), archive=False):
                return False
        elif kind == "A" and len(paths) == 1:
            if not bind(archive_pattern.fullmatch(paths[0]), archive=True):
                return False
        else:
            return False

    if not active_seen or not archive_seen or change_id is None or archive_dir is None:
        return False
    active_root = f".specsync/changes/{change_id}"
    archive_root = f".specsync/archive/changes/{archive_dir}"
    if (
        not git_object_exists(root, parent, active_root)
        or git_object_exists(root, parent, archive_root)
        or git_object_exists(root, child, active_root)
        or not git_object_exists(root, child, archive_root)
    ):
        return False
    try:
        parent_tree = git(root, "rev-parse", f"{parent}^{{tree}}")
    except subprocess.CalledProcessError:
        return False
    return canonical_archive_transition(
        root,
        parent,
        child,
        change_id,
        active_root,
        archive_root,
        parent_tree,
    )


def metadata_only_edge(root: Path, parent: str, child: str) -> bool:
    """Authenticate one historical review-only or workflow-v2 archive-only edge."""
    records = diff_records(root, parent, child)
    if records is None:
        return False
    return review_metadata_only_edge(
        root, parent, child, records
    ) or archive_metadata_only_edge(root, parent, child, records)


def metadata_parent(root: Path, child: str) -> str | None:
    parents = commit_parents(root, child)
    if not parents or not metadata_only_edge(root, parents[0], child):
        return None
    if len(parents) != 1:
        raise ValueError("lifecycle metadata child is a merge commit")
    return parents[0]


def check_metadata_edge_cli(root: Path, parent: str, child: str) -> None:
    try:
        resolved_parent = metadata_parent(root, child)
    except ValueError as error:
        raise SystemExit(str(error)) from error
    if resolved_parent != parent:
        raise SystemExit(f"{child} is not an exact lifecycle metadata child")
    print(f"Verified lifecycle metadata edge {parent}..{child}.")


def positive_bounded_limit(raw: str) -> int:
    try:
        limit = int(raw)
    except ValueError as error:
        raise SystemExit("MAX_ANCESTORS must be an integer from 1 through 32") from error
    if not 1 <= limit <= MAX_ANCESTORS:
        raise SystemExit("MAX_ANCESTORS must be an integer from 1 through 32")
    return limit


def main() -> None:
    repository = required("REPOSITORY")
    server_url = required("SERVER_URL").rstrip("/")
    pull_request = int(required("PR_NUMBER"))
    if pull_request <= 0:
        raise SystemExit("PR_NUMBER must be positive")
    start_sha = required("START_SHA")
    check_name = required("CHECK_NAME")
    workflow_path = required("WORKFLOW_PATH")
    require_run_success = os.environ.get("REQUIRE_RUN_SUCCESS", "true").strip().lower() in {
        "1",
        "true",
        "yes",
    }
    max_ancestors = positive_bounded_limit(os.environ.get("MAX_ANCESTORS", "32"))
    root = Path(os.environ.get("GIT_ROOT", ".")).resolve()
    child_kind = os.environ.get("CHILD_KIND", "child")

    github_actions_app = api("apps/github-actions")
    github_actions_owner = (
        github_actions_app.get("owner")
        if isinstance(github_actions_app, dict)
        else None
    )
    if (
        not isinstance(github_actions_app, dict)
        or github_actions_app.get("slug") != "github-actions"
        or github_actions_app.get("name") != "GitHub Actions"
        or not isinstance(github_actions_owner, dict)
        or github_actions_owner.get("login") != "github"
    ):
        raise SystemExit("could not resolve the official GitHub Actions app")

    errors: list[str] = []
    try:
        chain = first_parent_chain(root, start_sha, max_ancestors)
    except subprocess.CalledProcessError as error:
        raise SystemExit("START_SHA must name an exact available commit") from error
    if not chain or chain[0] != start_sha:
        raise SystemExit("START_SHA must name an exact available commit")
    for index, ancestor in enumerate(chain):
        try:
            parent = metadata_parent(root, ancestor)
        except ValueError as error:
            errors.append(f"stopped at {ancestor}: {error}")
            break
        if parent is not None:
            errors.append(f"skipped lifecycle metadata child {ancestor}")
            if index + 1 == len(chain) or chain[index + 1] != parent:
                errors.append("ancestor search limit exhausted before the product boundary")
                break
            continue

        # This is the nearest product boundary. It may use only its own exact
        # checks; never cross a product commit to borrow older green evidence.
        payload = api(f"repos/{repository}/commits/{ancestor}/check-runs?per_page=100")
        checks = payload.get("check_runs", []) if isinstance(payload, dict) else None
        total_count = payload.get("total_count") if isinstance(payload, dict) else None
        if (
            not isinstance(checks, list)
            or not isinstance(total_count, int)
            or isinstance(total_count, bool)
            or total_count != len(checks)
            or total_count > 100
        ):
            errors.append(f"{ancestor}: malformed check-run payload")
            break
        matches = sorted(
            (
                check
                for check in checks
                if isinstance(check, dict) and check.get("name") == check_name
            ),
            key=lambda check: int(check.get("id", 0))
            if str(check.get("id", "")).isdigit()
            else -1,
            reverse=True,
        )
        for check in matches:
            try:
                if check.get("head_sha") != ancestor:
                    raise ValueError("wrong head SHA")
                if check.get("status") != "completed" or check.get("conclusion") != "success":
                    raise ValueError(
                        f"check not successful: {check.get('status')}/{check.get('conclusion')}"
                    )
                app = check.get("app")
                if (
                    not isinstance(app, dict)
                    or app.get("id") != github_actions_app.get("id")
                    or app.get("slug") != github_actions_app.get("slug")
                ):
                    raise ValueError("check is not from GitHub Actions")
                check_id = check.get("id")
                if (
                    not isinstance(check_id, int)
                    or isinstance(check_id, bool)
                    or check_id <= 0
                ):
                    raise ValueError("check has no valid GitHub identity")
                details_url = str(check.get("details_url") or "")
                match = re.fullmatch(
                    rf"{re.escape(server_url)}/{re.escape(repository)}"
                    r"/actions/runs/([0-9]+)/job/([0-9]+)",
                    details_url,
                )
                if match is None:
                    raise ValueError("check does not name an exact GitHub Actions job")
                run_id = int(match.group(1))
                job_id = int(match.group(2))
                if run_id <= 0 or job_id <= 0:
                    raise ValueError("check has no valid workflow run or job identity")
                workflow_run = api(f"repos/{repository}/actions/runs/{run_id}")
                if not isinstance(workflow_run, dict):
                    raise ValueError("malformed workflow run")
                if workflow_run.get("id") != run_id:
                    raise ValueError("wrong workflow run ID")
                if workflow_run.get("head_sha") != ancestor:
                    raise ValueError("workflow run has the wrong head SHA")
                if workflow_run.get("event") != "pull_request":
                    raise ValueError("workflow run is not for a pull request")
                if workflow_run.get("status") != "completed":
                    raise ValueError("workflow run is not completed")
                if require_run_success and workflow_run.get("conclusion") != "success":
                    raise ValueError("workflow run is not successful")
                if str(workflow_run.get("path") or "").split("@", 1)[0] != workflow_path:
                    raise ValueError("workflow run has the wrong path")
                run_repository = workflow_run.get("repository")
                if (
                    not isinstance(run_repository, dict)
                    or run_repository.get("full_name") != repository
                ):
                    raise ValueError("workflow run belongs to another repository")
                pull_requests = workflow_run.get("pull_requests")
                if not isinstance(pull_requests, list) or any(
                    not isinstance(item, dict) for item in pull_requests
                ):
                    raise ValueError("workflow run has malformed pull-request bindings")
                if not any(item.get("number") == pull_request for item in pull_requests):
                    raise ValueError("workflow run is not bound to this PR")
                job = api(f"repos/{repository}/actions/jobs/{job_id}")
                if not isinstance(job, dict) or job.get("id") != job_id:
                    raise ValueError("malformed workflow job")
                if job.get("run_id") != run_id or job.get("head_sha") != ancestor:
                    raise ValueError("workflow job has the wrong run or head SHA")
                if job.get("name") != check_name:
                    raise ValueError("workflow job has the wrong name")
                if job.get("status") != "completed" or job.get("conclusion") != "success":
                    raise ValueError("workflow job is not successful")
                api_base = (
                    "https://api.github.com"
                    if server_url == "https://github.com"
                    else f"{server_url}/api/v3"
                )
                if job.get("check_run_url") != (
                    f"{api_base}/repos/{repository}/check-runs/{check_id}"
                ):
                    raise ValueError("workflow job is not bound to the selected check")
                if os.environ.get("OUTPUT_FORMAT", "human") == "env":
                    print(f"ancestor_sha={ancestor}")
                    print(f"workflow_run_id={run_id}")
                else:
                    print(
                        f"Reused {check_name} run {run_id} from ancestor {ancestor} "
                        f"(start {start_sha}) for the {child_kind} child."
                    )
                return
            except (
                json.JSONDecodeError,
                subprocess.CalledProcessError,
                TypeError,
                ValueError,
            ) as error:
                errors.append(f"{ancestor} check {check.get('id')}: {error}")
        break

    detail = "; ".join(errors) if errors else "no matching checks on first-parent chain"
    raise SystemExit(
        f"{check_name} provenance is not reusable from ancestors of {start_sha}: {detail}"
    )


if __name__ == "__main__":
    if len(sys.argv) == 4 and sys.argv[1] == "--check-metadata-edge":
        check_metadata_edge_cli(Path.cwd().resolve(), sys.argv[2], sys.argv[3])
    elif len(sys.argv) == 1:
        main()
    else:
        raise SystemExit(
            "usage: reuse-check-from-ancestors.py "
            "[--check-metadata-edge <parent-sha> <child-sha>]"
        )
