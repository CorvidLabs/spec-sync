#!/usr/bin/env python3
"""Unit-level tests for first-parent chain helper (no live GitHub)."""

import importlib.util
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / ".github/scripts/reuse-check-from-ancestors.py"


def load_module():
    spec = importlib.util.spec_from_file_location("reuse_check", SCRIPT)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def git(cwd: Path, *args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=cwd, text=True).strip()


module = load_module()

with tempfile.TemporaryDirectory() as temporary:
    repo = Path(temporary)
    git(repo, "init", "-b", "main")
    git(repo, "config", "user.email", "test@example.com")
    git(repo, "config", "user.name", "Test")
    (repo / "a.txt").write_text("a\n", encoding="utf-8")
    git(repo, "add", ".")
    git(repo, "commit", "-m", "a")
    a = git(repo, "rev-parse", "HEAD")
    (repo / "b.txt").write_text("b\n", encoding="utf-8")
    git(repo, "add", ".")
    git(repo, "commit", "-m", "b")
    b = git(repo, "rev-parse", "HEAD")
    (repo / "c.txt").write_text("c\n", encoding="utf-8")
    git(repo, "add", ".")
    git(repo, "commit", "-m", "c")
    c = git(repo, "rev-parse", "HEAD")

    chain = module.first_parent_chain(repo, c, 10)
    assert chain[0] == c
    assert chain[1] == b
    assert chain[2] == a
    assert len(chain) == 3

    limited = module.first_parent_chain(repo, c, 2)
    assert limited == [c, b]

print("reuse-check-from-ancestors tests passed")
