#!/usr/bin/env bash
# Publish built packages into the pacman repository checkout that is merged into gvcgo/vmr-pacman.
#
#   packaging/arch/build-repo.sh <repo-dir> <package>...
#
# <repo-dir> is a checkout of vmr-pacman; packages are copied into <repo-dir>/x86_64/, signed, and
# the database is regenerated from everything in that directory. Requires repo-add (pacman) and a
# signing key in GNUPGHOME, so it runs inside an Arch container in CI.
#
# Env:
#   PACMAN_GPG_KEY  key id/fingerprint used to sign the database and packages (required)
#
# Pages serves plain files, so the `vmr.db` / `vmr.files` symlinks repo-add creates are replaced
# with regular copies; pacman requests those names directly.
set -euo pipefail

if [ "$#" -lt 2 ]; then
    echo "usage: $0 <repo-dir> <package>..." >&2
    exit 2
fi

repo="$1"
shift

: "${PACMAN_GPG_KEY:?PACMAN_GPG_KEY must be set}"

arch=x86_64
dir="$repo/$arch"
mkdir -p "$dir"

for pkg in "$@"; do
    if [ ! -f "$pkg" ]; then
        echo "error: missing package: $pkg" >&2
        exit 1
    fi
    cp -f "$pkg" "$dir/"
done

packages=()
for pkg in "$dir"/*.pkg.tar.*; do
    case "$pkg" in
        *.sig) continue ;;
    esac
    [ -f "$pkg.sig" ] || gpg --batch --yes --detach-sign -u "$PACMAN_GPG_KEY" "$pkg"
    packages+=("$pkg")
done

# repo-add refuses to create the database over an existing symlink of the same name.
rm -f "$dir/vmr.db" "$dir/vmr.files" "$dir/vmr.db.sig" "$dir/vmr.files.sig"
repo-add --sign --verify "$dir/vmr.db.tar.gz" "${packages[@]}" >/dev/null

# GitHub Pages stores symlinks as their target path, so every database file that pacman requests
# must be materialised as a regular file.
materialize() {
    target="$1"
    [ -e "$target" ] || return 0
    temporary="$target.tmp"
    cp -Lf "$target" "$temporary"
    rm -f "$target"
    mv "$temporary" "$target"
}

[ -e "$dir/vmr.db.sig" ] || cp -f "$dir/vmr.db.tar.gz.sig" "$dir/vmr.db.sig"
for file in vmr.db vmr.db.sig vmr.files; do
    materialize "$dir/$file"
done

gpg --armor --export "$PACMAN_GPG_KEY" >"$repo/vmr.gpg"

find "$repo" -type f -not -path '*/.git/*' | sort
