#!/usr/bin/env bash
# Build the signed APT repository that is published to gvcgo/vmr-apt (GitHub Pages).
#
#   packaging/apt/build-repo.sh <version> <tarballs-dir> <repo-dir>
#
# <tarballs-dir> holds the unix release archives (vmr-x86_64-unknown-linux-gnu.tar.xz,
# vmr-aarch64-unknown-linux-gnu.tar.xz); <repo-dir> is the checkout of vmr-apt, which gets
# pool/, dists/ and vmr.gpg written into it.
#
# Env:
#   APT_GPG_KEY         key id/fingerprint used to sign Release/InRelease; unset builds unsigned
#   APT_GPG_PASSPHRASE  passphrase for the signing key, if it has one
#   APT_MAINTAINER      Maintainer field, default gvcgo <gvcgo@users.noreply.github.com>
#
# Requires a Debian-ish host (dpkg-deb, gpg, gzip, coreutils); runs on the CI runner.
set -euo pipefail

if [ "$#" -ne 3 ]; then
    echo "usage: $0 <version> <tarballs-dir> <repo-dir>" >&2
    exit 2
fi

version="$1"
tarballs="$2"
repo="$3"

case "$version" in
    v* | *-* | "") echo "error: version must be bare semver, got: $version" >&2; exit 2 ;;
esac

maintainer="${APT_MAINTAINER:-gvcgo <gvcgo@users.noreply.github.com>}"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

mkdir -p "$repo/pool/main/v/vmr" "$repo/dists/stable/main/binary-amd64" "$repo/dists/stable/main/binary-arm64"

# arch:<release-asset-stem>
for entry in amd64:vmr-x86_64-unknown-linux-gnu arm64:vmr-aarch64-unknown-linux-gnu; do
    arch="${entry%%:*}"
    target="${entry#*:}"
    archive="$tarballs/$target.tar.xz"

    if [ ! -r "$archive" ]; then
        echo "error: missing release archive: $archive" >&2
        exit 1
    fi

    mkdir -p "$work/$arch"
    tar -xJf "$archive" -C "$work/$arch"
    binary="$work/$arch/$target/vmr"
    if [ ! -f "$binary" ]; then
        echo "error: $target.tar.xz does not contain $target/vmr" >&2
        exit 1
    fi

    stage="$work/stage-$arch"
    mkdir -p "$stage/usr/bin" "$stage/DEBIAN"
    install -m 755 "$binary" "$stage/usr/bin/vmr"
    cat >"$stage/DEBIAN/control" <<EOF
Package: vmr
Version: $version
Architecture: $arch
Maintainer: $maintainer
Installed-Size: $(du -sk "$stage/usr" | cut -f1)
Section: utils
Priority: optional
Homepage: https://github.com/gvcgo/version-manager
Description: Cross-platform SDK version manager
 vmr installs multiple versions of Go, Node.js, Python, JDK, Rust and other
 SDKs side by side, and switches between them per shell, project or session.
EOF

    deb="$repo/pool/main/v/vmr/vmr_${version}_${arch}.deb"
    dpkg-deb --root-owner-group -Zxz --build "$stage" "$deb" >/dev/null
    echo "built $deb"
done

# Rebuild the indexes from every package in the pool so older versions stay installable.
dist="$repo/dists/stable"
for arch in amd64 arm64; do
    packages="$dist/main/binary-$arch/Packages"
    : >"$packages"
    found=0
    for deb in "$repo"/pool/main/v/vmr/*_"$arch".deb; do
        [ -f "$deb" ] || continue
        found=1
        relative="pool/main/v/vmr/$(basename "$deb")"
        dpkg-deb -f "$deb" >>"$packages"
        printf 'Filename: %s\nSize: %s\nSHA256: %s\n\n' \
            "$relative" \
            "$(stat -c%s "$deb")" \
            "$(sha256sum "$deb" | cut -d' ' -f1)" >>"$packages"
    done
    if [ "$found" -eq 0 ]; then
        echo "warning: no .deb packages found for $arch" >&2
    fi
    gzip -9nc "$packages" >"$packages.gz"
done

release="$dist/Release"
{
    echo "Origin: vmr"
    echo "Label: vmr"
    echo "Suite: stable"
    echo "Codename: stable"
    echo "Architectures: amd64 arm64"
    echo "Components: main"
    echo "Description: Signed APT repository for vmr"
    echo "Date: $(date -u '+%a, %d %b %Y %H:%M:%S UTC')"
    echo "MD5Sum:"
    (cd "$dist" && find main -type f \( -name 'Packages' -o -name 'Packages.gz' \) | sort |
        while IFS= read -r file; do
            printf ' %s %s %s\n' "$(md5sum "$file" | cut -d' ' -f1)" "$(stat -c%s "$file")" "$file"
        done)
    echo "SHA256:"
    (cd "$dist" && find main -type f \( -name 'Packages' -o -name 'Packages.gz' \) | sort |
        while IFS= read -r file; do
            printf ' %s %s %s\n' "$(sha256sum "$file" | cut -d' ' -f1)" "$(stat -c%s "$file")" "$file"
        done)
} >"$release"

if [ -n "${APT_GPG_KEY:-}" ]; then
    gpg_args=(--batch --yes --digest-algo SHA256)
    if [ -n "${APT_GPG_PASSPHRASE:-}" ]; then
        gpg_args+=(--pinentry-mode loopback --passphrase "$APT_GPG_PASSPHRASE")
    fi
    gpg "${gpg_args[@]}" --clearsign -o "$dist/InRelease" "$release"
    gpg "${gpg_args[@]}" --armor --detach-sign -o "$dist/Release.gpg" "$release"
    gpg --armor --export "$APT_GPG_KEY" >"$repo/vmr.gpg"
    echo "signed Release with $APT_GPG_KEY"
else
    echo "warning: APT_GPG_KEY is unset, repository left unsigned (apt will reject it)" >&2
fi
