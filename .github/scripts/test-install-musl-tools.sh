#!/usr/bin/env bash
# install-musl-tools.sh against a stubbed sudo, dpkg and apt-get: the runner's own Ubuntu
# mirror fails to resolve, as on 2026-11-11 (run 43037386037), and the fallback mirrors work.
# The real DNS failure cannot be reproduced on demand; the musl jobs exercise the real apt.
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
script="$root/.github/scripts/install-musl-tools.sh"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$tmp/bin"
cat >"$tmp/bin/sudo" <<'STUB'
#!/usr/bin/env bash
exec "$@"
STUB
cat >"$tmp/bin/dpkg" <<'STUB'
#!/usr/bin/env bash
[[ "$*" == "--print-architecture" ]] || exit 2
echo "$STUB_ARCH"
STUB
# apt-get: a source is reachable unless its host matches STUB_DEAD; a mirror+file source is
# reachable when one host in its list is. Third-party sources (packages.microsoft.com) always are.
cat >"$tmp/bin/apt-get" <<'STUB'
#!/usr/bin/env bash
printf '%s\n' "$*" >>"$STUB_LOG"
reachable() {
    local uri list host
    for uri in $(cat "$MUSL_APT_DIR"/sources.list "$MUSL_APT_DIR"/sources.list.d/* 2>/dev/null |
        grep -v '^#' | grep -oE '(mirror\+file:|https?://)[^[:space:]]+' | grep -v packages.microsoft.com); do
        if [[ "$uri" == mirror+file:* ]]; then
            list="${uri#mirror+file:}"
            [[ -f "$list" ]] || return 1
            while IFS=$'\t' read -r host _; do
                [[ -n "$host" && ! "$host" =~ $STUB_DEAD ]] && continue 2
            done <"$list"
            return 1
        elif [[ "$uri" =~ $STUB_DEAD ]]; then
            return 1
        fi
    done
}
case " $* " in
    *" update "*)
        reachable && exit 0
        echo "W: Failed to fetch InRelease  Temporary failure resolving the Ubuntu mirror"
        [[ " $* " == *" APT::Update::Error-Mode=any "* ]] && { echo "E: Some index files failed to download"; exit 100; }
        exit 0
        ;;
    *" install -y musl-tools "*)
        reachable && exit 0
        echo "E: Unable to fetch some archives"
        exit 100
        ;;
esac
echo "unexpected apt-get call: $*" >&2
exit 2
STUB
chmod +x "$tmp/bin/sudo" "$tmp/bin/dpkg" "$tmp/bin/apt-get"

fail() {
    echo "test-install-musl-tools: $*" >&2
    echo "--- output" >&2
    cat "$tmp/out" >&2
    echo "--- apt-get calls" >&2
    cat "$tmp/log" >&2 || true
    exit 1
}

# run <case> <arch> <dead-host regex>: the apt dir is $tmp/<case>/apt.
run() {
    local apt="$tmp/$1/apt"
    : >"$tmp/log"
    set +e
    PATH="$tmp/bin:$PATH" MUSL_APT_DIR="$apt" STUB_ARCH="$2" STUB_DEAD="$3" STUB_LOG="$tmp/log" \
        bash "$script" >"$tmp/out" 2>&1
    status=$?
    set -e
}

new_apt() {
    mkdir -p "$tmp/$1/apt/sources.list.d"
    printf 'deb [arch=amd64] https://packages.microsoft.com/ubuntu/24.04/prod noble main\n' \
        >"$tmp/$1/apt/sources.list.d/microsoft-prod.list"
}

expect_apt_options() {
    local calls
    calls="$(wc -l <"$tmp/log" | tr -d ' ')"
    [[ "$calls" -gt 0 ]] || fail "apt-get never ran"
    [[ "$(grep -c -- '-o Acquire::Retries=3 -o Acquire::http::Timeout=30 -o Acquire::https::Timeout=30' "$tmp/log")" == "$calls" ]] ||
        fail "an apt-get call ran without the retry and timeout options"
}

expect_fallback() {
    local case="$1" first="$2" last="$3"
    local apt="$tmp/$case/apt"
    [[ "$status" -eq 0 ]] || fail "$case: exited $status, expected the fallback to install musl-tools"
    grep -q "::warning::musl-tools did not install from the runner's Ubuntu mirror" "$tmp/out" ||
        fail "$case: no ::warning:: says the fallback was used"
    [[ "$(head -1 "$apt/specsync-fallback-mirrors.txt")" == "$first"$'\tpriority:1' ]] ||
        fail "$case: the first fallback mirror is not $first"
    [[ "$(tail -1 "$apt/specsync-fallback-mirrors.txt")" == "$last"$'\tpriority:4' ]] ||
        fail "$case: the last fallback mirror is not $last"
    # The strict update fails, so the first install never runs; then update and install again.
    [[ "$(sed -E 's/^(-o [^ ]+ )*//' "$tmp/log" | tr '\n' '|')" == "update|update|install -y musl-tools|" ]] ||
        fail "$case: expected the failing update, then update and install through the fallback"
    grep -q -- '-o APT::Update::Error-Mode=any update$' <(head -1 "$tmp/log") ||
        fail "$case: the first update does not fail on index errors"
    ! grep -q 'Error-Mode' <(tail -n +2 "$tmp/log") ||
        fail "$case: the fallback update must not fail on a third-party index"
    grep -q 'packages.microsoft.com' "$apt/sources.list.d/microsoft-prod.list" ||
        fail "$case: the third-party source was changed"
    if grep -rE 'ubuntu\.com|apt-mirrors\.txt' "$apt/sources.list.d" "$apt/sources.list" 2>/dev/null |
        grep -v '^[^:]*:#'; then
        fail "$case: an Ubuntu source still names the failing mirror"
    fi
    expect_apt_options
}

# 1. x86_64 runner (ubuntu-24.04): deb822 sources through the runner's own mirror list, whose
#    three ubuntu.com hosts all failed to resolve.
new_apt x86
printf 'http://azure.archive.ubuntu.com/ubuntu/\tpriority:1\nhttps://archive.ubuntu.com/ubuntu/\tpriority:2\nhttps://security.ubuntu.com/ubuntu/\tpriority:3\n' \
    >"$tmp/x86/apt/apt-mirrors.txt"
cat >"$tmp/x86/apt/sources.list.d/ubuntu.sources" <<EOF
Types: deb
URIs: mirror+file:$tmp/x86/apt/apt-mirrors.txt
Suites: noble noble-updates noble-backports
Components: main universe restricted multiverse
Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg

Types: deb
URIs: mirror+file:$tmp/x86/apt/apt-mirrors.txt
Suites: noble-security
Components: main universe restricted multiverse
Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg
EOF
run x86 amd64 'ubuntu\.com'
expect_fallback x86 https://mirrors.edge.kernel.org/ubuntu/ http://archive.ubuntu.com/ubuntu/
[[ "$(grep -c "^URIs: mirror+file:$tmp/x86/apt/specsync-fallback-mirrors.txt$" "$tmp/x86/apt/sources.list.d/ubuntu.sources")" == 2 ]] ||
    fail "x86: both deb822 stanzas should use the fallback mirror list"
grep -q '^Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg$' "$tmp/x86/apt/sources.list.d/ubuntu.sources" ||
    fail "x86: the archive key requirement was dropped"

# 2. arm64 runner (ubuntu-24.04-arm): deb822 sources on ports.ubuntu.com, which failed too.
new_apt arm
cat >"$tmp/arm/apt/sources.list.d/ubuntu.sources" <<'EOF'
Types: deb
URIs: http://ports.ubuntu.com/ubuntu-ports/
Suites: noble noble-updates noble-backports noble-security
Components: main universe restricted multiverse
Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg
EOF
run arm arm64 'ubuntu\.com'
expect_fallback arm https://mirrors.ocf.berkeley.edu/ubuntu-ports/ http://ports.ubuntu.com/ubuntu-ports/
[[ "$(grep -c "^URIs: mirror+file:$tmp/arm/apt/specsync-fallback-mirrors.txt$" "$tmp/arm/apt/sources.list.d/ubuntu.sources")" == 1 ]] ||
    fail "arm: the ports source should use the fallback mirror list"

# 3. One-line sources.list (ubuntu-22.04 style), with two URIs on the archive mirror and security.
new_apt jammy
cat >"$tmp/jammy/apt/sources.list" <<'EOF'
# Ubuntu sources
deb http://azure.archive.ubuntu.com/ubuntu/ jammy main universe
deb http://azure.archive.ubuntu.com/ubuntu/ jammy-updates main universe
deb http://security.ubuntu.com/ubuntu jammy-security main universe
EOF
run jammy amd64 'azure|security'
expect_fallback jammy https://mirrors.edge.kernel.org/ubuntu/ http://archive.ubuntu.com/ubuntu/
[[ "$(grep -c "^deb mirror+file:$tmp/jammy/apt/specsync-fallback-mirrors.txt jammy" "$tmp/jammy/apt/sources.list")" == 3 ]] ||
    fail "jammy: every one-line Ubuntu source should use the fallback mirror list"

# 4. The runner's mirror works: one update, one install, nothing rewritten.
new_apt healthy
printf 'Types: deb\nURIs: http://azure.archive.ubuntu.com/ubuntu/\nSuites: noble\n' \
    >"$tmp/healthy/apt/sources.list.d/ubuntu.sources"
cp "$tmp/healthy/apt/sources.list.d/ubuntu.sources" "$tmp/healthy.before"
run healthy amd64 'no-such-host'
[[ "$status" -eq 0 ]] || fail "healthy: exited $status"
[[ "$(wc -l <"$tmp/log" | tr -d ' ')" == 2 ]] || fail "healthy: expected exactly update and install"
[[ ! -e "$tmp/healthy/apt/specsync-fallback-mirrors.txt" ]] || fail "healthy: wrote a fallback mirror list"
cmp -s "$tmp/healthy.before" "$tmp/healthy/apt/sources.list.d/ubuntu.sources" || fail "healthy: rewrote the sources"
! grep -q '::warning::' "$tmp/out" || fail "healthy: warned about a fallback it did not use"
expect_apt_options

# 5. Every mirror fails, fallback included: a clear ::error:: and a failing exit.
new_apt dead
printf 'Types: deb\nURIs: http://ports.ubuntu.com/ubuntu-ports/\nSuites: noble\n' \
    >"$tmp/dead/apt/sources.list.d/ubuntu.sources"
run dead arm64 '.'
[[ "$status" -ne 0 ]] || fail "dead: exited 0 with every mirror failing"
grep -q '^::error::musl-tools did not install from the runner.s Ubuntu sources or from the fallback mirrors' "$tmp/out" ||
    fail "dead: no ::error:: names the failed fallback"

# 6. No Ubuntu archive source to repoint: a clear ::error::, and nothing is rewritten.
new_apt unknown
printf 'Types: deb\nURIs: http://deb.debian.org/debian/\nSuites: bookworm\n' \
    >"$tmp/unknown/apt/sources.list.d/debian.sources"
run unknown amd64 'debian'
[[ "$status" -ne 0 ]] || fail "unknown: exited 0"
grep -q '^::error::musl-tools did not install, and no Ubuntu archive source' "$tmp/out" ||
    fail "unknown: no ::error:: says there was nothing to repoint"
[[ ! -e "$tmp/unknown/apt/specsync-fallback-mirrors.txt" ]] || fail "unknown: wrote a fallback mirror list"

# 7. CI runs this script for the musl targets instead of a bare apt-get.
ci="$root/.github/workflows/ci.yml"
grep -q '^          \.github/scripts/install-musl-tools\.sh$' "$ci" ||
    fail "ci.yml does not run .github/scripts/install-musl-tools.sh"
! grep -q 'apt-get install -y musl-tools' "$ci" || fail "ci.yml still installs musl-tools with a bare apt-get"

echo "install-musl-tools ok"
