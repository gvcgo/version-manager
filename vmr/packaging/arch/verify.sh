#!/usr/bin/env bash
# Install vmr from a generated pacman repository. Runs INSIDE a disposable Arch container:
#
#   docker run --rm -v "$PWD/checkout:/repo:ro" archlinux:base bash /repo/../verify.sh /repo
#
# It trusts the exported key exactly the way a user would (pacman-key --add + --lsign-key), points
# pacman at the repository through file:// and installs the package, so a broken database, missing
# signature or bad payload fails here instead of on a user's machine.
set -euo pipefail

repo="$(cd "${1:?usage: verify.sh <repo-dir>}" && pwd)"

pacman-key --init >/dev/null
pacman-key --add "$repo/vmr.gpg" >/dev/null
key="$(pacman-key --list-keys --with-colons | awk -F: '/^fpr/ {print $10; exit}')"
if [ -z "$key" ]; then
    echo "error: no key found in $repo/vmr.gpg" >&2
    exit 1
fi
pacman-key --lsign-key "$key" >/dev/null

cat >>/etc/pacman.conf <<EOF

[vmr]
SigLevel = Required DatabaseOptional
Server = file://$repo/\$arch
EOF

pacman -Sy --noconfirm vmr

pacman -Qi vmr | sed -n '1,6p'
vmr version
