#!/usr/bin/env bash
# Commit and push paths inside an existing tap/bucket/repository checkout.
#
#   packaging/commit-repo.sh <checkout-dir> <commit-message> <path>...
#
# Paths are relative to <checkout-dir>. Exits 0 without committing when nothing changed.
set -euo pipefail

if [ "$#" -lt 3 ]; then
    echo "usage: $0 <checkout-dir> <commit-message> <path>..." >&2
    exit 2
fi

checkout="$1"
message="$2"
shift 2

cd "$checkout"

git add -- "$@"
if git diff --cached --quiet; then
    echo "nothing to publish in $checkout"
    exit 0
fi

git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
git commit -m "$message"
git push
