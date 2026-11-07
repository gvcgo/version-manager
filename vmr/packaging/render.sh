#!/usr/bin/env bash
# Render a packaging template from a release checksum manifest.
#
#   packaging/render.sh <template> <version> <SHA256SUMS> [output]
#
# Placeholders in the template:
#   @@VERSION@@            bare semver, e.g. 0.8.0
#   @@SHA256:<asset>@@     digest of that release asset, looked up in SHA256SUMS
#
# <output> defaults to stdout. Exits non-zero when a referenced asset is absent from the manifest
# or the version is malformed, so a partially published release never reaches a tap/bucket.
# Written for bash 3.2 (the macOS system shell): no associative arrays, no `mapfile`.
set -euo pipefail

if [ "$#" -lt 3 ] || [ "$#" -gt 4 ]; then
    echo "usage: $0 <template> <version> <SHA256SUMS> [output]" >&2
    exit 2
fi

template_path="$1"
version="$2"
sums="$3"
out="${4:-}"

case "$version" in
    v* | *-* | "") echo "error: version must be bare semver, got: $version" >&2; exit 2 ;;
esac

if [ ! -r "$template_path" ]; then
    echo "error: cannot read template: $template_path" >&2
    exit 1
fi

if [ ! -r "$sums" ]; then
    echo "error: cannot read checksum manifest: $sums" >&2
    exit 1
fi

template="$(<"$template_path")"

hash_of() {
    awk -v file="$1" '
        {
            name = $2
            sub(/^\*/, "", name)
            if (name == file) { print $1; exit }
        }
    ' "$sums"
}

tokens="$(printf '%s' "$template" | grep -o '@@SHA256:[^@]*@@' | sort -u || true)"
while IFS= read -r token; do
    [ -n "$token" ] || continue
    artifact="${token#@@SHA256:}"
    artifact="${artifact%@@}"
    digest="$(hash_of "$artifact")"
    if [ -z "$digest" ]; then
        echo "error: missing checksum for $artifact" >&2
        exit 1
    fi
    template="${template//"$token"/$digest}"
done <<<"$tokens"

template="${template//@@VERSION@@/$version}"

if printf '%s' "$template" | grep -q '@@'; then
    echo "error: unresolved placeholders remain in the rendered output" >&2
    exit 1
fi

if [ -n "$out" ]; then
    mkdir -p "$(dirname "$out")"
    printf '%s\n' "$template" >"$out"
else
    printf '%s\n' "$template"
fi
