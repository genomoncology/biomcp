#!/usr/bin/env bash
# Decide whether this release's container image should carry the
# `latest` tag (2026-09-30 second go-request review: the old check
# asked GitHub for "the latest release", which SKIPS drafts, so a
# draft v0.9.1 read v0.9.0 as latest and the step silently skipped).
# The decision compares the publishing tag's version number against
# every already-PUBLISHED release: if no published release sorts
# newer, this tag becomes latest once published.
#
# Usage: should-move-latest.sh <publishing-tag> <published-tag>...
# Exit 0 when latest should move; exit 3 when it legitimately stays;
# exit 1 or 2 on error, so the workflow step can fail on anything but
# an explicit stay (fourth go-request review: a missing script reads
# as 127 and must not count as "stay" either).
set -euo pipefail

publishing="${1:?usage: should-move-latest.sh <publishing-tag> <published-tag>...}"
shift

if ! printf '%s' "$publishing" | grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "not a stable tag: $publishing" >&2
    exit 2
fi

key_of() {
    local tag="$1"
    if ! printf '%s' "$tag" | grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+$'; then
        echo "not a stable tag: $tag" >&2
        return 2
    fi
    local major minor patch
    major="${tag#v}"
    minor="${major#*.}"
    patch="${minor#*.}"
    major="${major%%.*}"
    minor="${minor%%.*}"
    printf '%06d%06d%06d' "$major" "$minor" "$patch"
}

publishing_key="$(key_of "$publishing")"
for published in "$@"; do
    published_key="$(key_of "$published")"
    if [[ "$published_key" > "$publishing_key" ]]; then
        echo "$published sorts newer than $publishing; latest stays" >&2
        exit 3
    fi
done
exit 0
