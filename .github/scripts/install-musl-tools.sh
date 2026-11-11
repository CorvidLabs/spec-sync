#!/usr/bin/env bash
# Install musl-tools on a GitHub-hosted Ubuntu runner, and keep going when the runner's
# Ubuntu mirror stops resolving.
#
# On 2026-11-11 (CI run 43037386037) the runners could not resolve the Ubuntu archive
# names they use: azure.archive.ubuntu.com, archive.ubuntu.com and security.ubuntu.com
# on x86_64, ports.ubuntu.com on arm64. `apt-get update` only warned, and
# `apt-get install -y musl-tools` then failed both musl release-baseline jobs.
#
# 1. Update (failing on any index error) and install from the runner's own sources.
# 2. If either fails, point the runner's Ubuntu archive sources at a mirror list
#    (apt's mirror+file method tries each host in priority order) and try again.
#    Mirrors outside ubuntu.com come first because every ubuntu.com name failed together.
# 3. If that fails too, stop with ::error::.
#
# apt checks every index against the Ubuntu archive key, whichever mirror serves it.
# .github/scripts/test-install-musl-tools.sh runs this against a stubbed apt-get.
set -euo pipefail

apt_dir="${MUSL_APT_DIR:-/etc/apt}"
mirror_list="${apt_dir}/specsync-fallback-mirrors.txt"
apt_options=(
    -o Acquire::Retries=3
    -o Acquire::http::Timeout=30
    -o Acquire::https::Timeout=30
)
# The runner's own mirror list, or any http(s) Ubuntu archive or ports URI on ubuntu.com.
ubuntu_source='(mirror\+file:[^[:space:]]*apt-mirrors\.txt|https?://([A-Za-z0-9-]+\.)*ubuntu\.com/ubuntu(-ports)?/?)'

install_musl_tools() {
    sudo apt-get "${apt_options[@]}" install -y musl-tools
}

if sudo apt-get "${apt_options[@]}" -o APT::Update::Error-Mode=any update && install_musl_tools; then
    exit 0
fi

arch="$(dpkg --print-architecture)"
case "$arch" in
    amd64)
        mirrors=(
            https://mirrors.edge.kernel.org/ubuntu/
            https://mirrors.mit.edu/ubuntu/
            https://mirrors.ocf.berkeley.edu/ubuntu/
            http://archive.ubuntu.com/ubuntu/
        )
        ;;
    arm64)
        mirrors=(
            https://mirrors.ocf.berkeley.edu/ubuntu-ports/
            https://mirrors.mit.edu/ubuntu-ports/
            https://ftp.tu-chemnitz.de/pub/linux/ubuntu-ports/
            http://ports.ubuntu.com/ubuntu-ports/
        )
        ;;
    *)
        echo "::error::musl-tools did not install and there are no fallback Ubuntu mirrors for ${arch}"
        exit 1
        ;;
esac

sources=()
for file in "${apt_dir}/sources.list" "${apt_dir}"/sources.list.d/*.list "${apt_dir}"/sources.list.d/*.sources; do
    if [[ -f "$file" ]] && grep -Eq "$ubuntu_source" "$file"; then
        sources+=("$file")
    fi
done
if [[ "${#sources[@]}" -eq 0 ]]; then
    echo "::error::musl-tools did not install, and no Ubuntu archive source under ${apt_dir} can be pointed at a fallback mirror"
    exit 1
fi

echo "::warning::musl-tools did not install from the runner's Ubuntu mirror; retrying through ${mirrors[*]}"
priority=0
for mirror in "${mirrors[@]}"; do
    priority=$((priority + 1))
    printf '%s\tpriority:%d\n' "$mirror" "$priority"
done | sudo tee "$mirror_list" >/dev/null
for file in "${sources[@]}"; do
    rewritten="$(mktemp)"
    sed -E "s#${ubuntu_source}([[:space:]]|\$)#mirror+file:${mirror_list}\\4#g" "$file" >"$rewritten"
    sudo cp "$rewritten" "$file"
    rm -f "$rewritten"
    echo "Pointed the Ubuntu archive URIs in ${file} at mirror+file:${mirror_list}"
done

# No Error-Mode here: a third-party source failing must not block musl-tools; the install decides.
if sudo apt-get "${apt_options[@]}" update && install_musl_tools; then
    exit 0
fi
echo "::error::musl-tools did not install from the runner's Ubuntu sources or from the fallback mirrors (${mirrors[*]})"
exit 1
