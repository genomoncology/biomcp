#!/usr/bin/env python3
"""Decide whether the live documentation revision is the tag commit or a descendant."""

from __future__ import annotations

import argparse
import subprocess
import sys


PASSING_STATUSES = ("identical", "ahead")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--tag-sha", required=True)
    parser.add_argument("--live-revision", required=True)
    return parser.parse_args()


def compare_status(*, repository: str, tag_sha: str, live_revision: str) -> str:
    """Ask GitHub how the live revision relates to the tag commit."""
    completed = subprocess.run(
        [
            "gh",
            "api",
            f"repos/{repository}/compare/{tag_sha}...{live_revision}",
            "--jq",
            ".status",
        ],
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        raise RuntimeError(completed.stderr.strip() or "gh api compare failed")
    return completed.stdout.strip()


def main() -> int:
    args = parse_args()
    try:
        status = compare_status(
            repository=args.repository,
            tag_sha=args.tag_sha,
            live_revision=args.live_revision,
        )
    except RuntimeError as error:
        print(f"live documentation revision check failed: {error}", file=sys.stderr)
        return 1
    if status in PASSING_STATUSES:
        print(
            f"live documentation revision {args.live_revision} satisfies the tag "
            f"commit {args.tag_sha}: GitHub compare reports {status!r}"
        )
        return 0
    print(
        f"live documentation revision {args.live_revision} does not contain the "
        f"tag commit {args.tag_sha}: GitHub compare reports {status!r}",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
