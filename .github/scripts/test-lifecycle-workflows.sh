#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$script_dir/../.." && pwd)"
ci="$root/.github/workflows/ci.yml"
finalize="$root/.github/workflows/finalize-change.yml"
trusted_policy="$root/.github/workflows/lifecycle-policy-guard.yml"
post_merge="$root/.github/workflows/post-merge-archive.yml"
release="$root/.github/workflows/release.yml"
classifier="$root/.github/scripts/classify-ci-paths.sh"
policy_verifier="$root/.github/scripts/verify-trusted-policy-check.py"
archive_verifier="$root/.github/scripts/verify-archive-introduction.py"
limits="$root/.github/scripts/lifecycle-validation-limits.json"

ruby - "$ci" "$finalize" "$trusted_policy" "$post_merge" "$release" <<'RUBY'
require "psych"

ARGV.each do |path|
  document = Psych.safe_load(
    File.read(path, encoding: "UTF-8"),
    permitted_classes: [],
    aliases: false
  )
  raise "#{path} is not a workflow mapping" unless document.is_a?(Hash)
  raise "#{path} has no jobs mapping" unless document["jobs"].is_a?(Hash)
end
RUBY

python3 - "$ci" "$finalize" "$trusted_policy" "$post_merge" "$release" "$classifier" "$policy_verifier" "$archive_verifier" "$limits" <<'PY'
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile

ci = Path(sys.argv[1]).read_text(encoding="utf-8")
finalize = Path(sys.argv[2]).read_text(encoding="utf-8")
trusted_policy = Path(sys.argv[3]).read_text(encoding="utf-8")
post_merge = Path(sys.argv[4]).read_text(encoding="utf-8")
release = Path(sys.argv[5]).read_text(encoding="utf-8")
classifier = Path(sys.argv[6]).read_text(encoding="utf-8")
policy_verifier = Path(sys.argv[7]).read_text(encoding="utf-8")
archive_verifier = Path(sys.argv[8]).read_text(encoding="utf-8")
limits = json.loads(Path(sys.argv[9]).read_text(encoding="utf-8"))
if limits != {
    "git_max_output_bytes": 8 * 1024 * 1024,
    "git_timeout_seconds": 30,
    "scoped_review_max_descendants": 1_000,
    "scoped_review_max_parents": 32,
}:
    raise SystemExit("shared lifecycle validation limits changed unexpectedly")


def require(pattern: str, source: str, message: str) -> None:
    if re.search(pattern, source, flags=re.MULTILINE | re.DOTALL) is None:
        raise SystemExit(message)


require(
    r"pull_request_target:.*?"
    r"permissions:\s+contents: read\s+pull-requests: read.*?"
    r"ref: \$\{\{ github\.workflow_sha \}\}.*?"
    r"persist-credentials: false",
    trusted_policy,
    "trusted lifecycle policy must run from the immutable base revision with read-only permissions",
)
require(
    r'git fetch --no-tags --no-recurse-submodules origin.*?'
    r'"\+refs/pull/\$\{PR_NUMBER\}/head:refs/specsync/candidate".*?'
    r'git diff --name-only --no-renames -z "\$BASE_SHA" "\$HEAD_SHA"',
    trusted_policy,
    "trusted lifecycle policy must inspect the exact candidate as Git objects",
)
require(
    r"action\\\.\(yml\|yaml\).*?"
    r"\\\.github/workflows/\[\^/\]\*\\\.\(yml\|yaml\).*?"
    r"\\\.github/actions\(/\[\^/\]\*\)\*.*?"
    r"classify-ci-paths.*?"
    r"lifecycle-validation-limits.*?"
    r"workflow-v2-baseline",
    trusted_policy,
    "trusted lifecycle policy must protect all workflow/action definitions, classifiers, and limits",
)
protected_pattern_match = re.search(
    r"pattern = re\.compile\(rb'([^']+)'\)",
    trusted_policy,
)
if protected_pattern_match is None:
    raise SystemExit("trusted lifecycle policy has no extractable protected-path pattern")
protected_pattern = protected_pattern_match.group(1)
protected_regex = re.compile(protected_pattern.encode())
with tempfile.TemporaryDirectory() as directory:
    fixture = Path(directory)

    def fixture_git(*arguments: str) -> str:
        return subprocess.run(
            ["git", *arguments],
            cwd=fixture,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()

    fixture_git("init", "-b", "main")
    fixture_git("config", "user.email", "test@example.com")
    fixture_git("config", "user.name", "Test")
    for relative in [
        ".github/actions/modified/action.yml",
        ".github/actions/deleted/action.yml",
        ".github/actions/renamed/action.yml",
    ]:
        path = fixture / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("name: base\n", encoding="utf-8")
    fixture_git("add", ".")
    fixture_git("commit", "-m", "base policy fixtures")
    base = fixture_git("rev-parse", "HEAD")

    (fixture / ".github/actions/modified/action.yml").write_text(
        "name: modified\n",
        encoding="utf-8",
    )
    (fixture / ".github/actions/deleted/action.yml").unlink()
    (fixture / "docs").mkdir()
    fixture_git(
        "mv",
        ".github/actions/renamed/action.yml",
        "docs/renamed-action.yml",
    )
    added = fixture / ".github/actions/added/action.yml"
    added.parent.mkdir(parents=True)
    added.write_text("name: added\n", encoding="utf-8")
    newline_workflow = fixture / ".github/workflows/evil\nworkflow.yml"
    newline_workflow.parent.mkdir(parents=True)
    newline_workflow.write_text("name: protected newline workflow\n", encoding="utf-8")
    fixture_git("add", "--all")
    fixture_git("commit", "-m", "add modify delete and rename policy fixtures")
    head = fixture_git("rev-parse", "HEAD")
    changed = subprocess.run(
        [
            "git",
            "diff",
            "--name-only",
            "--no-renames",
            "-z",
            base,
            head,
        ],
        cwd=fixture,
        check=True,
        capture_output=True,
    ).stdout.split(b"\0")
    protected = {
        path.decode() for path in changed if path and protected_regex.fullmatch(path)
    }
    expected = {
        ".github/actions/added/action.yml",
        ".github/actions/modified/action.yml",
        ".github/actions/deleted/action.yml",
        ".github/actions/renamed/action.yml",
        ".github/workflows/evil\nworkflow.yml",
    }
    if protected != expected:
        raise SystemExit(
            f"trusted policy did not protect exact A/M/D/rename action paths: {protected!r}"
        )

for operation in ("add", "modify", "delete", "rename"):
    with tempfile.TemporaryDirectory() as directory:
        fixture = Path(directory)

        def root_action_git(*arguments: str) -> str:
            return subprocess.run(
                ["git", *arguments],
                cwd=fixture,
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()

        root_action_git("init", "-b", "main")
        root_action_git("config", "user.email", "test@example.com")
        root_action_git("config", "user.name", "Test")
        (fixture / "README.md").write_text("base\n", encoding="utf-8")
        if operation != "add":
            (fixture / "action.yml").write_text("name: base\n", encoding="utf-8")
        root_action_git("add", "--all")
        root_action_git("commit", "-m", "base root action fixture")
        base = root_action_git("rev-parse", "HEAD")

        if operation == "add":
            (fixture / "action.yml").write_text("name: added\n", encoding="utf-8")
        elif operation == "modify":
            (fixture / "action.yml").write_text("name: modified\n", encoding="utf-8")
        elif operation == "delete":
            (fixture / "action.yml").unlink()
        else:
            (fixture / "docs").mkdir()
            root_action_git("mv", "action.yml", "docs/action.yml")
        root_action_git("add", "--all")
        root_action_git("commit", "-m", f"{operation} root action fixture")
        head = root_action_git("rev-parse", "HEAD")
        changed = subprocess.run(
            [
                "git",
                "diff",
                "--name-only",
                "--no-renames",
                "-z",
                base,
                head,
            ],
            cwd=fixture,
            check=True,
            capture_output=True,
        ).stdout.split(b"\0")
        protected = {
            path.decode() for path in changed if path and protected_regex.fullmatch(path)
        }
        if protected != {"action.yml"}:
            raise SystemExit(
                f"trusted policy did not protect root action.yml {operation}: {protected!r}"
            )
require(
    r"publish:\s+name: Publish trusted lifecycle policy result.*?"
    r"permissions:\s+contents: read\s+checks: write.*?"
    r'name "SpecSync trusted policy".*?'
    r"specsync-trusted-policy:\$\{TRUSTED_WORKFLOW_SHA\}:\$\{HEAD_SHA\}",
    trusted_policy,
    "trusted policy publisher must bind its candidate check to the immutable workflow revision",
)
require(
    r'BOOTSTRAP_REPOSITORY = "CorvidLabs/spec-sync".*?'
    r"BOOTSTRAP_PULL_REQUEST = 480.*?"
    r'BOOTSTRAP_BASE_SHA = "fc091c88f72a6d2fb2df168f4baa4370579ff8a2".*?'
    r"pull\.get\(\"number\"\) != pull_request.*?"
    r'\(pull\.get\("base"\) or \{\}\)\.get\("sha"\) != base.*?'
    r'\(pull\.get\("head"\) or \{\}\)\.get\("sha"\) != candidate.*?'
    r"--diff-filter=A.*?"
    r"WORKFLOW_V2_BASELINE_PATH.*?"
    r"cutoff_commit.*?BOOTSTRAP_BASE_SHA.*?"
    r"canonical exact-base adoption anchor",
    policy_verifier,
    "trusted-policy bootstrap must be frozen to one repository, PR, base, branch, and added file set",
)
require(
    r"specsync-trusted-policy:\(\[0-9a-f\]\{\{40\}\}\).*?"
    r'run\.get\("event"\) != "pull_request_target".*?'
    r'run\.get\("head_sha"\) != trusted.*?'
    r'item\.get\("number"\) == pull_request.*?'
    r'\(item\.get\("head"\) or \{\}\)\.get\("sha"\) == candidate',
    policy_verifier,
    "trusted policy verifier must bind the exact workflow revision, event, PR, and candidate",
)
require(
    r"verify-trusted-policy-check\.py",
    ci,
    "review-only CI must verify the implementation's trusted policy result",
)
require(
    r"Require base-trusted lifecycle policy.*?"
    r"verify-trusted-policy-check\.py",
    finalize,
    "archive finalization must verify the parent's trusted policy result",
)
require(
    # After merge, check out the merge commit (already on the base repository) via
    # pull_request closed — not pull_request_target — so CodeQL does not treat this as
    # privileged untrusted-checkout. PR head is still only fetched as an isolated ref.
    r"pull_request:.*?types: \[closed\].*?"
    r"actions/checkout@[0-9a-f]{40}.*?"
    r"ref: \$\{\{ github\.event\.pull_request\.merge_commit_sha \}\}.*?"
    r"persist-credentials: false.*?"
    r"refs/pull/\$\{PR_NUMBER\}/head:refs/specsync/merged-head.*?"
    r"verify-trusted-policy-check\.py",
    post_merge,
    "post-merge publication must check out the merge commit and inspect the finalized head as Git objects",
)
if re.search(
    r"ref:\s*\$\{\{\s*github\.event\.pull_request\.head\.sha",
    post_merge,
):
    raise SystemExit("post-merge publication must never check out a live pull-request head")
if re.search(r"pull_request_target:", post_merge):
    raise SystemExit(
        "post-merge publication must not use pull_request_target after merge "
        "(merged commit checkout is already base-repository code)"
    )
require(
    r"verify-archive-introduction\.py.*?archive_introduction_commit.*?"
    r"finalization_digest.*?"
    r"archive_introduction_commit: \$archive_introduction_commit.*?"
    r"finalization_digest: \$finalization_digest",
    post_merge,
    "post-merge binding must include the unique source introduction and finalization digest",
)
require(
    r"verify-archive-introduction\.py.*?archive_introduction_commit.*?"
    r'"archive_introduction_commit": archive_introduction_commit.*?'
    r'"finalization_digest": finalization\.get',
    release,
    "release must independently reconstruct introduction-bound archive metadata",
)
require(
    r"lifecycle-validation-limits\.json.*?"
    r"git_max_output_bytes.*?"
    r"git_timeout_seconds.*?"
    r"selectors\.DefaultSelector\(\).*?"
    r"total_output > max_git_output_bytes.*?"
    r"process\.kill\(\)",
    release,
    "release Git queries must stream and terminate at the shared resource bounds",
)
require(
    r'lifecycle-validation-limits\.json.*?'
    r'history_limit = int\(limits\["scoped_review_max_descendants"\]\).*?'
    r'parent_limit = int\(limits\["scoped_review_max_parents"\]\).*?'
    r"selectors\.DefaultSelector\(\).*?"
    r"--max-count=\{history_limit \+ 1\}.*?"
    r"len\(parents\) > parent_limit.*?"
    r"head_tree != introduction_tree.*?"
    r"touching_commits = git\(.*?"
    r"commit_parents = fields\[1:\].*?"
    r'git\("rev-parse", f"\{commit\}:\{archive_path\}"\) != introduction_tree.*?'
    r'git\("rev-parse", f"\{parent\}:\{archive_path\}"\) != introduction_tree',
    archive_verifier,
    "archive introduction verification must bound every touching commit/parent and reject rewrites",
)
policy_job = re.search(
    r"^  policy:\n(?P<body>.*?)(?=^  publish:\n)",
    trusted_policy,
    flags=re.MULTILINE | re.DOTALL,
)
if policy_job is None or re.search(r"checks:\s*write", policy_job.group("body")):
    raise SystemExit("trusted policy inspection must not receive check-write permission")
if re.search(
    r"ref:\s*\$\{\{[^}]*pull_request\.head\.sha|"
    r"git\s+(checkout|worktree)[^\n]*(HEAD_SHA|specsync/candidate)",
    trusted_policy,
    flags=re.MULTILINE,
):
    raise SystemExit("trusted lifecycle policy must never check out candidate content")
if re.search(r"\$\{\{\s*secrets\.", trusted_policy):
    raise SystemExit("trusted lifecycle policy must not consume repository secrets")


require(
    r'for field in \("contract_digest", "execution_digest", "workspace_digest"\):',
    finalize,
    "archive validator must validate the workflow-v2 execution digest",
)
require(
    r'"contract_digest": review\["contract_digest"\],\s+'
    r'"execution_digest": review\["execution_digest"\],\s+'
    r'"workspace_digest": review\["workspace_digest"\],',
    finalize,
    "canonical review digest must include execution_digest in Rust field order",
)
require(
    r'"reviewer": review\["reviewer"\],\s+'
    r'"provenance": review\["provenance"\],\s+'
    r'"verdict": review\["verdict"\],',
    finalize,
    "canonical review digest must bind required-check provenance",
)
require(
    r'review\.get\("schema_version"\) != 2.*?'
    r'review_attempts\["reviews"\]\[-1\] != review.*?'
    r'"provider": "github_actions_check".*?'
    r'review\.get\("verdict"\) != "pass".*?'
    r'"verdict": review\["verdict"\],',
    finalize,
    "archive validator must require and hash a workflow-v2 passing review verdict",
)
require(
    r'git_bytes\(\s*"diff-tree".*?'
    r'"--no-renames".*?'
    r'"implementation changed after scoped review:',
    finalize,
    "archive validator must inspect each post-review commit against every parent",
)
require(
    r'lifecycle-validation-limits\.json".*?'
    r'max_git_output_bytes = int\(limits\["git_max_output_bytes"\]\).*?'
    r'max_review_descendants = int\(limits\["scoped_review_max_descendants"\]\).*?'
    r'max_review_parents = int\(limits\["scoped_review_max_parents"\]\).*?'
    r"timeout=git_timeout_seconds.*?"
    r"--max-count=\{max_review_descendants \+ 1\}.*?"
    r"len\(fields\) - 1 > max_review_parents",
    finalize,
    "archive validator must bound Git output, time, descendants, and parents",
)
require(
    r"- name: Validate archive integrity and parent binding.*?"
    r"import selectors.*?"
    r"import time.*?"
    r"selectors\.DefaultSelector\(\).*?"
    r"total_output > max_git_output_bytes.*?"
    r"process\.kill\(\).*?"
    r"def git_status\(\*args: str\).*?"
    r"timeout=git_timeout_seconds",
    finalize,
    "archive validator must stream and terminate Git output at the bound",
)
require(
    r"parent_review_attempts_path.*?"
    r"parent_review_attempts\.get\(\"reviews\"\)\s+"
    r"!= review_attempts\[\"reviews\"\].*?"
    r"removed or rewrote committed scoped-review attempts",
    finalize,
    "archive validator must preserve the exact parent scoped-review ledger",
)
require(
    r"def valid_reviewer_claim\(value: object\).*?"
    r"value\.isascii\(\).*?"
    r're\.fullmatch\(r"\[A-Za-z0-9 \._:@/-\]\+", value\)',
    finalize,
    "archive validator must validate stable reviewer identities while reading",
)
if len(re.findall(r'git_status\(\s*"merge-base",\s*"--is-ancestor"', finalize)) != 2:
    raise SystemExit(
        "all archive ancestry traversals must use the timeout-aware status helper"
    )
require(
    r'for field in \("contract_digest", "execution_digest", "workspace_digest"\):',
    ci,
    "scoped-review child must bind the workflow-v2 execution digest",
)
review_reuse_job = re.search(
    r"^  scoped-review-reuse:\n(?P<body>.*?)(?=^  implementation-gate:\n)",
    ci,
    flags=re.MULTILINE | re.DOTALL,
)
if review_reuse_job is None:
    raise SystemExit("CI workflow has no scoped-review reuse job")
review_reuse = review_reuse_job.group("body")
require(
    r"PARENT_REVIEW_ATTEMPTS_JSON=.*?"
    r"len\(current_reviews\) != len\(parent_reviews\) \+ 1.*?"
    r"current_reviews\[:-1\] != parent_reviews.*?"
    r"strict append-only history extension",
    ci,
    "scoped-review child must append exactly one record to the parent ledger",
)
require(
    r'IMPLEMENTATION_SHA=.*?implementation_commit.*?'
    r'rev-list".*?f"\{implementation\}\.\.\{parent\}".*?'
    r'diff-tree".*?"--no-renames".*?'
    r'commits/\$\{IMPLEMENTATION_SHA\}/check-runs.*?'
    r'proven_check\(\s*"SpecSync implementation ready".*?'
    r'proven_check\("trust".*?'
    r'this job is .*new successful scoped-review check',
    review_reuse,
    "review recovery must reuse implementation/trust from the reviewed ancestor",
)
if re.search(
    r'proven_check\(\s*"SpecSync scoped review"',
    review_reuse,
    flags=re.MULTILINE,
):
    raise SystemExit(
        "a review child must not require a successful scoped-review check on its parent"
    )
require(
    r"def valid_reviewer_claim\(value\).*?"
    r"value\.isascii\(\).*?"
    r'for attempt in current_reviews:.*?'
    r"not valid_reviewer_claim\(attempt\.get\(\"reviewer\"\)\)",
    ci,
    "scoped-review child must validate every persisted reviewer identity",
)
require(
    r'scope_approver.*?'
    r'review\["reviewer"\]\.strip\(\)\.casefold\(\) == scope_approver\.casefold\(\)',
    ci,
    "scoped-review child must reject the scope approver as reviewer",
)
require(
    r'frame\(closing, "contract", verification\["contract_digest"\]\)\s+'
    r'frame\(closing, "execution", verification\["execution_digest"\]\)\s+'
    r'frame\(closing, "workspace", verification\["workspace_digest"\]\)',
    finalize,
    "closing digest must frame execution between contract and workspace",
)
require(
    r'limits\["git_max_output_bytes"\].*?'
    r'limits\["git_timeout_seconds"\].*?'
    r'limits\["scoped_review_max_descendants"\].*?'
    r'limits\["scoped_review_max_parents"\].*?'
    r"selectors\.DefaultSelector\(\).*?"
    r"--max-count=\{descendant_limit \+ 1\}.*?"
    r"len\(fields\) - 1 <= parent_limit",
    classifier,
    "CI classifier must enforce all four shared lifecycle history limits",
)
require(
    r'\( "\$status" != "A" && "\$status" != "M" \).*?'
    r'review-attempts\\\.json',
    classifier,
    "CI classifier must route both initial and appended review children",
)

require(
    r"legacy_archive_only: \$\{\{ steps\.paths\.outputs\.legacy_archive_only \}\}",
    ci,
    "CI classifier outputs must expose legacy workflow-v1 archives",
)
require(
    r"\^\(archive_only\|legacy_archive_only\|review_only\)=true\$",
    ci,
    "single-child classification must retain the legacy archive route",
)
require(
    r'classify-ci-paths\.sh\s*\\?\s*'
    r'"\$GITHUB_WORKSPACE" false name-status "\$parent"',
    ci,
    "CI must authenticate legacy archive versions against the active parent",
)
require(
    r'classify-ci-paths\.sh\s*\\?\s*'
    r'"\$GITHUB_WORKSPACE" false name-status "\$parent_sha"',
    finalize,
    "archive integrity must classify the exact parent-bound child",
)
require(
    r'if \[\[ "\$LEGACY_ARCHIVE_ONLY" == "true" \]\]; then\s+'
    r'echo "Legacy workflow-v1 archive passed the historical full-validation path\."\s+'
    r"exit 0",
    ci,
    "legacy workflow-v1 archives must complete only after the full implementation gate",
)

corvid_job = re.search(
    r"^  corvid-pet:\n(?P<body>.*?)(?=^  attest:\n)",
    ci,
    flags=re.MULTILINE | re.DOTALL,
)
if corvid_job is None:
    raise SystemExit("CI workflow has no scoped-review job")
corvid = corvid_job.group("body")
require(
    r"if: \$\{\{ always\(\) && github\.event_name == 'pull_request' "
    r"&& needs\.classify\.outputs\.review_required == 'true' \}\}",
    corvid,
    "scoped review must run for qualifying fork pull requests",
)
require(
    r"comment-on-pr: \$\{\{ github\.event\.pull_request\.head\.repo\.full_name "
    r"== github\.repository && 'true' \|\| 'false' \}\}",
    corvid,
    "fork scoped review must suppress PR comment writes",
)
require(
    r"review-on-pr: \$\{\{ github\.event\.pull_request\.head\.repo\.full_name "
    r"== github\.repository && 'true' \|\| 'false' \}\}",
    corvid,
    "fork scoped review must suppress PR review writes",
)
if re.search(r"\$\{\{\s*secrets\.", corvid):
    raise SystemExit("fork-safe scoped review must not consume repository secrets")

print("lifecycle workflow assertions passed")
PY

python3 "$root/.github/scripts/test-verify-trusted-policy-check.py"

archive_test="$(mktemp -d)"
trap 'rm -rf "$archive_test"' EXIT
archive_repo="$archive_test/repo"
archive_path=".specsync/archive/changes/2026-07-30-CHG-0001-test"
git -C "$archive_test" init -q repo
git -C "$archive_repo" config user.email test@example.com
git -C "$archive_repo" config user.name "SpecSync Test"
printf 'base\n' > "$archive_repo/README.md"
git -C "$archive_repo" add README.md
git -C "$archive_repo" commit -qm "base"
mkdir -p "$archive_repo/$archive_path"
printf '{"state":"archived"}\n' > "$archive_repo/$archive_path/state.json"
git -C "$archive_repo" add "$archive_path/state.json"
git -C "$archive_repo" commit -qm "introduce archive"
archive_introduction="$(git -C "$archive_repo" rev-parse HEAD)"
introduction_json="$(
  python3 "$archive_verifier" \
    --git-root "$archive_repo" \
    --head "$archive_introduction" \
    --archive-path "$archive_path"
)"
if [[ "$(jq -r '.archive_introduction_commit' <<<"$introduction_json")" \
    != "$archive_introduction" ]]; then
  echo "archive verifier did not return the unique introduction" >&2
  exit 1
fi
printf 'unrelated\n' >> "$archive_repo/README.md"
git -C "$archive_repo" add README.md
git -C "$archive_repo" commit -qm "unrelated successor"
python3 "$archive_verifier" \
  --git-root "$archive_repo" \
  --head "$(git -C "$archive_repo" rev-parse HEAD)" \
  --archive-path "$archive_path" >/dev/null
printf '{"state":"rewritten"}\n' > "$archive_repo/$archive_path/state.json"
git -C "$archive_repo" add "$archive_path/state.json"
git -C "$archive_repo" commit -qm "rewrite archive"
if python3 "$archive_verifier" \
    --git-root "$archive_repo" \
    --head "$(git -C "$archive_repo" rev-parse HEAD)" \
    --archive-path "$archive_path" >/dev/null 2>&1; then
  echo "archive verifier accepted a post-introduction rewrite" >&2
  exit 1
fi
printf '{"state":"archived"}\n' > "$archive_repo/$archive_path/state.json"
git -C "$archive_repo" add "$archive_path/state.json"
git -C "$archive_repo" commit -qm "restore archive bytes"
if python3 "$archive_verifier" \
    --git-root "$archive_repo" \
    --head "$(git -C "$archive_repo" rev-parse HEAD)" \
    --archive-path "$archive_path" >/dev/null 2>&1; then
  echo "archive verifier accepted a rewrite hidden by a later restore" >&2
  exit 1
fi

bounded_verifier_dir="$archive_test/bounded"
mkdir -p "$bounded_verifier_dir"
cp "$archive_verifier" "$bounded_verifier_dir/verify-archive-introduction.py"
printf '%s\n' \
  '{"git_max_output_bytes":8388608,"git_timeout_seconds":30,' \
  '"scoped_review_max_descendants":2,"scoped_review_max_parents":32}' \
  > "$bounded_verifier_dir/lifecycle-validation-limits.json"
if python3 "$bounded_verifier_dir/verify-archive-introduction.py" \
    --git-root "$archive_repo" \
    --head "$(git -C "$archive_repo" rev-parse HEAD)" \
    --archive-path "$archive_path" >/dev/null 2>&1; then
  echo "archive verifier accepted history beyond the shared commit bound" >&2
  exit 1
fi

echo "archive introduction verifier tests passed"
