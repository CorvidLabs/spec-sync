#!/usr/bin/env python3
"""Run the published SpecSync 6 lifecycle examples against one built binary."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
EXAMPLES = (
    ("sdd-lifecycle", "Lifecycle example passed", 180),
    ("sdd-concurrent-changes", "Concurrent-change example passed", 240),
    ("sdd-five-epics", "Five-epic SpecSync 6.0 proof passed", 600),
)


class PublishedExampleTests(unittest.TestCase):
    """The README-linked examples must remain runnable under slug identities."""

    binary: Path

    def test_examples_do_not_construct_retired_numeric_change_ids(self) -> None:
        for name, _, _ in EXAMPLES:
            source = (ROOT / "examples" / name / "run.sh").read_text(encoding="utf-8")
            self.assertIsNone(
                re.search(r"CHG-\\d{4}", source),
                f"{name} must derive the change ID returned by `change new --json`",
            )

    def test_five_epics_executes_product_tests_before_scoped_check(self) -> None:
        source = (ROOT / "examples/sdd-five-epics/run.sh").read_text(encoding="utf-8")
        self.assertLess(source.index("cargo test --quiet"), source.index('"$bin" change check "$id"'))
        self.assertNotIn("Passing product tests: 6", source)

    def test_published_examples_complete(self) -> None:
        for name, marker, timeout in EXAMPLES:
            with self.subTest(example=name):
                with tempfile.TemporaryDirectory(prefix=f"specsync-{name}-") as temporary:
                    environment = os.environ.copy()
                    environment["SPECSYNC_BIN"] = str(self.binary)
                    environment["CARGO_TARGET_DIR"] = str(Path(temporary) / "cargo-target")
                    if name == "sdd-five-epics":
                        environment["DEMO_ROOT"] = str(Path(temporary) / "project")
                    result = subprocess.run(
                        ["bash", str(ROOT / "examples" / name / "run.sh")],
                        cwd=ROOT,
                        env=environment,
                        text=True,
                        capture_output=True,
                        timeout=timeout,
                        check=False,
                    )
                    self.assertEqual(
                        result.returncode,
                        0,
                        result.stdout + result.stderr,
                    )
                    self.assertIn(marker, result.stdout)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--bin", type=Path, required=True)
    arguments, unittest_arguments = parser.parse_known_args()
    PublishedExampleTests.binary = arguments.bin.resolve()
    if not PublishedExampleTests.binary.is_file():
        parser.error(f"missing built SpecSync binary: {PublishedExampleTests.binary}")
    unittest.main(argv=[sys.argv[0], *unittest_arguments])
