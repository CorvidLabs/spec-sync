#!/usr/bin/env python3
"""Validate SpecSync's repository-local Trust self-hosting exception."""

from pathlib import Path
import sys
import tomllib


ROOT = Path(__file__).resolve().parents[2]
TRUST_PATH = ROOT / ".trust.toml"
FLEDGE_PATH = ROOT / "fledge.toml"
EXPECTED_REASON = (
    "SpecSync self-hosts strict 100% contract validation with the source-built "
    "binary in the blocking lifecycle lane"
)
EXPECTED_COMMAND = "cargo run -- check --strict --require-coverage 100 --force"


def fail(message: str) -> None:
    print(f"trust self-host policy error: {message}", file=sys.stderr)
    raise SystemExit(1)


def main() -> None:
    trust = tomllib.loads(TRUST_PATH.read_text(encoding="utf-8"))
    fledge = tomllib.loads(FLEDGE_PATH.read_text(encoding="utf-8"))
    contract = trust.get("contract", {})
    if contract.get("enabled") is not False:
        fail("the duplicate released-binary contract component must be disabled")
    if contract.get("require_coverage") != 100:
        fail("the committed contract threshold must remain 100")
    if contract.get("skip_reason") != EXPECTED_REASON:
        fail("the repository-local exception reason changed")

    tasks = fledge.get("tasks", {})
    spec_check = tasks.get("spec-check", {})
    if spec_check.get("cmd") != EXPECTED_COMMAND:
        fail("the source-built strict 100% contract command changed")

    verify_steps = fledge.get("lanes", {}).get("verify", {}).get("steps", [])
    if "trust-self-host-policy" not in verify_steps or "spec-check" not in verify_steps:
        fail("the blocking verify lane must run both policy and contract checks")
    if verify_steps.index("trust-self-host-policy") > verify_steps.index("spec-check"):
        fail("the self-host policy must be validated before the contract check")

    print("Trust self-host policy valid: source-built strict contract remains blocking")


if __name__ == "__main__":
    main()
