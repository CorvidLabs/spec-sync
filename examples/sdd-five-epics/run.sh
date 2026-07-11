#!/usr/bin/env bash
# shellcheck disable=SC2016 # Markdown backticks are intentional literals.
set -euo pipefail

bin="${SPECSYNC_BIN:-specsync}"
root="${DEMO_ROOT:-$(mktemp -d)}"

if [ -e "$root" ] && [ -n "$(ls -A "$root" 2>/dev/null)" ]; then
    echo "DEMO_ROOT must be absent or empty: $root" >&2
    exit 1
fi
mkdir -p "$root"
cd "$root"

git init -b main >/dev/null
git config user.email five-epics@specsync.dev
git config user.name "SpecSync Five-Epic Demo"

mkdir -p src tests specs/product
printf '%s\n' \
    '[package]' \
    'name = "epic-demo"' \
    'version = "0.1.0"' \
    'edition = "2024"' \
    > Cargo.toml
printf '%s\n' \
    '/// Returns the stable product name.' \
    'pub fn product_name() -> &'\''static str { "Corvid" }' \
    > src/lib.rs
printf '%s\n' \
    'use epic_demo::product_name;' \
    '' \
    '#[test]' \
    'fn product_name_is_stable() {' \
    '    assert_eq!(product_name(), "Corvid");' \
    '}' \
    > tests/product.rs
printf '%s\n' \
    '---' \
    'module: product' \
    'version: 1' \
    'status: stable' \
    'files:' \
    '  - src/lib.rs' \
    '---' \
    '' \
    '# Product' \
    '' \
    '## Purpose' \
    '' \
    'Provides the stable public product experience used by the five-epic demonstration.' \
    '' \
    '## Public API' \
    '' \
    '| Name | Description |' \
    '|------|-------------|' \
    '| `product_name` | Return the stable product name |' \
    '' \
    '## Invariants' \
    '' \
    '1. Every public behavior is backed by a permanent requirement ID.' \
    '2. Every accepted epic passes the configured Cargo test command.' \
    '3. Canonical truth changes only through closing approval.' \
    '' \
    '## Behavioral Examples' \
    '' \
    '- Product clients can read a stable product name.' \
    '- New behaviors appear only after their epic is accepted.' \
    '' \
    '## Error Cases' \
    '' \
    '| Condition | Behavior |' \
    '|-----------|----------|' \
    '| Unsupported input | Return a deterministic fallback |' \
    '' \
    '## Dependencies' \
    '' \
    '| Dependency | Purpose |' \
    '|------------|---------|' \
    '| Rust standard library | String and formatting support |' \
    '' \
    '## Change Log' \
    '' \
    '| Date | Change |' \
    '|------|--------|' \
    '| 2026-07-10 | Initial product contract |' \
    > specs/product/product.spec.md
printf '%s\n' \
    '---' \
    'spec: product.spec.md' \
    '---' \
    '' \
    '# Requirements' \
    '' \
    '### REQ-product-000' \
    '' \
    'The system SHALL expose a stable product name.' \
    '' \
    'Acceptance Criteria' \
    '- Product name tests pass.' \
    > specs/product/requirements.md
printf '%s\n' '---' 'spec: product.spec.md' '---' '' '# Context' '' 'Five-epic demonstration baseline.' > specs/product/context.md
printf '%s\n' '---' 'spec: product.spec.md' '---' '' '# Testing' '' '- Cargo tests validate every requirement.' > specs/product/testing.md
printf '%s\n' '---' 'spec: product.spec.md' '---' '' '# Tasks' '' '- [x] Establish the baseline.' > specs/product/tasks.md

"$bin" init >/dev/null
"$bin" agents install >/dev/null
printf '\n# Generated demonstration evidence\nreview-report.md\n' >> .gitignore
git add .
git commit -m "Initialize SpecSync 5.0 product" >/dev/null
git update-ref refs/remotes/origin/main HEAD

titles=(
    "Add welcome message"
    "Localize welcome message"
    "Personalize welcome message"
    "Record welcome audit event"
    "Expose welcome health"
)
slugs=(
    "add-welcome-message"
    "localize-welcome-message"
    "personalize-welcome-message"
    "record-welcome-audit-event"
    "expose-welcome-health"
)
functions=(
    "welcome_message"
    "localized_welcome"
    "personalized_welcome"
    "record_welcome_event"
    "welcome_health"
)
outcomes=(
    "Clients receive a deterministic welcome message"
    "Spanish clients receive a localized welcome message"
    "Clients receive a welcome message containing their name"
    "A deterministic audit event is emitted for a welcomed user"
    "Operators can confirm the welcome subsystem is healthy"
)
requirements=(
    "return a deterministic welcome message to clients"
    "return a localized welcome message for supported locales"
    "include the client name in a personalized welcome message"
    "emit a deterministic audit event for each welcomed user"
    "report whether the welcome subsystem is healthy"
)

api_rows='| `product_name` | Return the stable product name |'
previous_id=""

implement_epic() {
    case "$1" in
        1)
            printf '%s\n' '' '/// Returns the default welcome message.' 'pub fn welcome_message() -> &'\''static str { "Welcome" }' >> src/lib.rs
            printf '%s\n' '' 'use epic_demo::welcome_message;' '' '#[test]' 'fn welcome_message_is_available() {' '    assert_eq!(welcome_message(), "Welcome");' '}' >> tests/product.rs
            ;;
        2)
            printf '%s\n' '' '/// Returns a localized welcome message.' 'pub fn localized_welcome(locale: &str) -> &'\''static str {' '    if locale == "es" { "Bienvenido" } else { "Welcome" }' '}' >> src/lib.rs
            printf '%s\n' '' 'use epic_demo::localized_welcome;' '' '#[test]' 'fn spanish_welcome_is_localized() {' '    assert_eq!(localized_welcome("es"), "Bienvenido");' '}' >> tests/product.rs
            ;;
        3)
            printf '%s\n' '' '/// Returns a personalized welcome message.' 'pub fn personalized_welcome(name: &str) -> String {' '    format!("Welcome, {name}")' '}' >> src/lib.rs
            printf '%s\n' '' 'use epic_demo::personalized_welcome;' '' '#[test]' 'fn welcome_contains_the_name() {' '    assert_eq!(personalized_welcome("Raven"), "Welcome, Raven");' '}' >> tests/product.rs
            ;;
        4)
            printf '%s\n' '' '/// Returns a deterministic welcome audit event.' 'pub fn record_welcome_event(user: &str) -> String {' '    format!("welcome:{user}")' '}' >> src/lib.rs
            printf '%s\n' '' 'use epic_demo::record_welcome_event;' '' '#[test]' 'fn welcome_event_is_deterministic() {' '    assert_eq!(record_welcome_event("raven"), "welcome:raven");' '}' >> tests/product.rs
            ;;
        5)
            printf '%s\n' '' '/// Reports whether the welcome subsystem is healthy.' 'pub fn welcome_health() -> bool { true }' >> src/lib.rs
            printf '%s\n' '' 'use epic_demo::welcome_health;' '' '#[test]' 'fn welcome_subsystem_is_healthy() {' '    assert!(welcome_health());' '}' >> tests/product.rs
            ;;
    esac
}

for index in 0 1 2 3 4; do
    number=$((index + 1))
    requirement=$(printf 'REQ-product-%03d' "$number")
    id=$(printf 'CHG-%04d-%s' "$number" "${slugs[$index]}")
    title="${titles[$index]}"
    function="${functions[$index]}"
    outcome="${outcomes[$index]}"
    requirement_text="${requirements[$index]}"

    "$bin" change new "$title" --kind feature --spec product --path src/lib.rs >/dev/null
    "$bin" change answer "$id" acceptance_criteria "$outcome" >/dev/null
    "$bin" change answer "$id" public_contract yes >/dev/null
    "$bin" change answer "$id" architecture_risk yes >/dev/null
    if [ -n "$previous_id" ]; then
        "$bin" change depend "$id" "$previous_id" >/dev/null
    fi

    change_dir=".specsync/changes/$id"
    for artifact in context requirements research design plan docs; do
        if [ -f "$change_dir/$artifact.md" ]; then
            printf '# %s\n\nEpic %s: %s. Reviewed before implementation.\n' "$artifact" "$number" "$outcome" > "$change_dir/$artifact.md"
        fi
    done
    if [ -f "$change_dir/tasks.md" ]; then
        printf '# Tasks\n\n- [x] Implement epic %s.\n- [x] Add requirement evidence.\n- [x] Review the closing diff.\n' "$number" > "$change_dir/tasks.md"
    fi
    if [ -f "$change_dir/testing.md" ]; then
        printf '# Testing\n\n- `%s` is covered by the Cargo integration test for `%s`.\n' "$requirement" "$function" > "$change_dir/testing.md"
    fi

    api_rows="${api_rows}"$'\n'"| \`${function}\` | ${outcome} |"
    printf '## ADDED\n\n### REQUIREMENT %s\n\nThe system SHALL %s.\n\nAcceptance Criteria\n- %s.\n\n## MODIFIED\n\n### SPEC SECTION Public API\n\n| Name | Description |\n|------|-------------|\n%s\n' \
        "$requirement" "$requirement_text" "$outcome" "$api_rows" \
        > "$change_dir/deltas/product.md"

    "$bin" change approve "$id" --actor "Epic Product Reviewer" >/dev/null
    "$bin" change start "$id" >/dev/null
    implement_epic "$number"
    git add .
    git commit -m "Implement epic $number: $title" >/dev/null
    "$bin" change verify "$id"
    "$bin" change accept "$id" --actor "Epic Closing Reviewer" >/dev/null
    git add .
    git commit -m "Accept epic $number contract" >/dev/null

    # Updating origin/main models the reviewed feature being merged. Archival is
    # intentionally post-merge because its active scope covers the delivery diff.
    git update-ref refs/remotes/origin/main HEAD
    "$bin" change archive "$id" >/dev/null
    git add .
    git commit -m "Archive epic $number evidence" >/dev/null
    git update-ref refs/remotes/origin/main HEAD
    previous_id="$id"
done

{
    printf '# SpecSync 5.0 five-epic review\n\n'
    printf '## Release identity\n\n'
    "$bin" --version
    printf '\n## Strict validation\n\n```text\n'
    "$bin" check --strict --require-coverage 100 --force
    printf '```\n\n## Quality score\n\n```text\n'
    "$bin" score --all --explain
    printf '```\n\n## Lifecycle inventory\n\n'
    printf -- '- Accepted and archived epics: %s/5\n' "$(find .specsync/archive/changes -name state.json | wc -l | tr -d ' ')"
    printf -- '- Active changes: %s\n' "$(find .specsync/changes -name state.json | wc -l | tr -d ' ')"
    printf -- '- Approval gates recorded: %s/10\n' "$(grep -h '\"gate\"' .specsync/archive/changes/*/approvals.json | wc -l | tr -d ' ')"
    printf -- '- Verification records: %s/5\n' "$(find .specsync/archive/changes -name verification.json | wc -l | tr -d ' ')"
    printf -- '- Canonical product spec version: %s\n' "$(awk '/^version:/ { print $2; exit }' specs/product/product.spec.md)"
    printf -- '- Permanent product requirements: %s\n' "$(grep -c '^### REQ-product-' specs/product/requirements.md)"
    printf -- '- Passing product tests: 6\n'
    printf '\n## Epic dependency chain\n\n'
    printf '1. CHG-0001 is the root epic.\n'
    printf '2. CHG-0002 depends on CHG-0001.\n'
    printf '3. CHG-0003 depends on CHG-0002.\n'
    printf '4. CHG-0004 depends on CHG-0003.\n'
    printf '5. CHG-0005 depends on CHG-0004.\n'
    printf '\n## Installed native agent surfaces\n\n```text\n'
    find .claude .cursor .codex .gemini -type f 2>/dev/null | sort
    printf '```\n\n## Git timeline\n\n```text\n'
    git log --oneline --reverse
    printf '```\n'
} > review-report.md

"$bin" check --strict --require-coverage 100 --force
printf '\nFive-epic SpecSync 5.0 proof passed.\nProject: %s\nReport: %s/review-report.md\n' "$root" "$root"
