#!/usr/bin/env python3
"""Verify a finalized archive's unique, immutable Git introduction."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import selectors
import subprocess
import sys
import time


def fail(message: str) -> None:
    raise SystemExit(message)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--git-root", required=True)
    parser.add_argument("--head", required=True)
    parser.add_argument("--archive-path", required=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    root = Path(args.git_root).resolve()
    head = args.head
    archive_path = args.archive_path
    if re.fullmatch(r"[0-9a-f]{40}", head) is None:
        fail("archive verification requires an exact 40-character head SHA")
    if (
        re.fullmatch(
            r"\.specsync/archive/changes/"
            r"[0-9]{4}-[0-9]{2}-[0-9]{2}-CHG-[0-9]{4,}-[^/\x00]+",
            archive_path,
        )
        is None
    ):
        fail("archive verification received an invalid archive path")

    limits_path = Path(__file__).with_name("lifecycle-validation-limits.json")
    with limits_path.open(encoding="utf-8") as handle:
        limits = json.load(handle)
    max_output_bytes = int(limits["git_max_output_bytes"])
    timeout_seconds = int(limits["git_timeout_seconds"])
    history_limit = int(limits["scoped_review_max_descendants"])
    parent_limit = int(limits["scoped_review_max_parents"])
    if min(max_output_bytes, timeout_seconds, history_limit, parent_limit) <= 0:
        fail("lifecycle validation limits must be positive")

    def git_bytes(*git_args: str) -> bytes:
        process = subprocess.Popen(
            ["git", "-C", str(root), *git_args],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        if process.stdout is None or process.stderr is None:
            process.kill()
            fail("failed to capture bounded Git query")
        selector = selectors.DefaultSelector()
        selector.register(process.stdout, selectors.EVENT_READ, "stdout")
        selector.register(process.stderr, selectors.EVENT_READ, "stderr")
        stdout = bytearray()
        stderr = bytearray()
        total_output = 0
        deadline = time.monotonic() + timeout_seconds
        try:
            while selector.get_map():
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    process.kill()
                    process.wait()
                    fail(f"bounded Git query timed out: {' '.join(git_args)}")
                events = selector.select(timeout=remaining)
                if not events:
                    continue
                for key, _ in events:
                    chunk = os.read(key.fileobj.fileno(), 64 * 1024)
                    if not chunk:
                        selector.unregister(key.fileobj)
                        continue
                    total_output += len(chunk)
                    if total_output > max_output_bytes:
                        process.kill()
                        process.wait()
                        fail(
                            "bounded Git query exceeded "
                            f"{max_output_bytes} bytes: {' '.join(git_args)}"
                        )
                    if key.data == "stdout":
                        stdout.extend(chunk)
                    elif len(stderr) < 4096:
                        stderr.extend(chunk[: 4096 - len(stderr)])
            returncode = process.wait(timeout=5)
        finally:
            selector.close()
            if process.poll() is None:
                process.kill()
                process.wait()
        if returncode != 0:
            detail = bytes(stderr).decode(errors="replace").strip()
            fail(f"git {' '.join(git_args)} failed ({returncode}): {detail}")
        return bytes(stdout)

    def git(*git_args: str) -> str:
        return git_bytes(*git_args).decode().strip()

    def git_status(*git_args: str) -> int:
        try:
            return subprocess.run(
                ["git", "-C", str(root), *git_args],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=False,
                timeout=timeout_seconds,
            ).returncode
        except subprocess.TimeoutExpired as error:
            fail(f"bounded Git status query timed out: {' '.join(git_args)}")
            raise AssertionError from error

    commits = git(
        "rev-list",
        f"--max-count={history_limit + 1}",
        head,
    ).splitlines()
    if not commits:
        fail("archive head has no reachable commit history")
    if len(commits) > history_limit:
        fail(
            "archive introduction history exceeds the shared "
            f"{history_limit}-commit validation limit"
        )

    state_path = f"{archive_path}/state.json"
    introductions = git(
        "log",
        "--format=%H",
        "--diff-filter=A",
        head,
        "--",
        state_path,
    ).splitlines()
    if len(introductions) != 1:
        fail("finalized head must have exactly one archive introduction")
    introduction = introductions[0]
    fields = git("rev-list", "--parents", "-n", "1", introduction).split()
    if len(fields) < 2 or fields[0] != introduction:
        fail("archive introduction has invalid parent history")
    parents = fields[1:]
    if len(parents) > parent_limit:
        fail(
            "archive introduction exceeds the shared "
            f"{parent_limit}-parent validation limit"
        )
    for parent in parents:
        if git_status("cat-file", "-e", f"{parent}^{{commit}}") != 0:
            fail("archive introduction has an unreadable parent")
        if git_status("cat-file", "-e", f"{parent}:{state_path}") == 0:
            fail("archive path already exists in an introduction parent")

    head_tree = git("rev-parse", f"{head}:{archive_path}")
    introduction_tree = git("rev-parse", f"{introduction}:{archive_path}")
    if head_tree != introduction_tree:
        fail("archive subtree changed after its unique introduction")
    touching_commits = git(
        "rev-list",
        "--reverse",
        "--full-history",
        f"--max-count={history_limit + 1}",
        head,
        "--",
        archive_path,
    ).splitlines()
    if len(touching_commits) > history_limit:
        fail(
            "archive path history exceeds the shared "
            f"{history_limit}-commit validation limit"
        )
    for commit in touching_commits:
        fields = git("rev-list", "--parents", "-n", "1", commit).split()
        if not fields or fields[0] != commit:
            fail(f"archive path commit {commit} has invalid parent history")
        commit_parents = fields[1:]
        if len(commit_parents) > parent_limit:
            fail(
                "archive path commit exceeds the shared "
                f"{parent_limit}-parent validation limit"
            )
        if git_status("cat-file", "-e", f"{commit}:{archive_path}") != 0:
            fail("archive subtree was deleted after its unique introduction")
        if git("rev-parse", f"{commit}:{archive_path}") != introduction_tree:
            fail("archive subtree was rewritten after its unique introduction")
        for parent in commit_parents:
            if git_status("cat-file", "-e", f"{parent}:{archive_path}") != 0:
                continue
            if git("rev-parse", f"{parent}:{archive_path}") != introduction_tree:
                fail("archive subtree has a rewritten parent in reachable history")

    json.dump(
        {
            "archive_introduction_commit": introduction,
            "archive_tree": head_tree,
            "history_commit_count": len(commits),
            "parent_count": len(parents),
        },
        sys.stdout,
        sort_keys=True,
    )
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
