#!/usr/bin/env python3
"""Verify that an exact documentation revision is observable on the live host."""

from __future__ import annotations

import argparse
from collections.abc import Callable
from pathlib import Path
import re
import secrets
import sys
import time
from urllib.error import URLError
from urllib.parse import parse_qsl, urlencode, urljoin, urlsplit, urlunsplit
from urllib.request import Request, urlopen


REVISION = re.compile(r"^[0-9a-f]{40}$")
BIOMCP_URL = re.compile(r"https://biomcp\.org(?:/[^\s)>`\]]*)?")
REVISION_DIRECTORY = "__biomcp_revision__"
MAX_TIMEOUT_SECONDS = 600.0
REQUEST_TIMEOUT_SECONDS = 20.0


class VerificationError(RuntimeError):
    """The live host did not serve the expected publication."""


def _origin(url: str) -> tuple[str, str, int | None]:
    parsed = urlsplit(url)
    scheme = parsed.scheme.lower()
    hostname = (parsed.hostname or "").lower()
    try:
        port = parsed.port
    except ValueError as error:
        raise VerificationError(f"invalid URL origin: {url}") from error
    if port is None:
        port = {"http": 80, "https": 443}.get(scheme)
    return scheme, hostname, port


def _cache_busted(url: str, attempt: int) -> str:
    parsed = urlsplit(url)
    query = parse_qsl(parsed.query, keep_blank_values=True)
    query.append(
        (
            "biomcp_verify",
            f"{time.time_ns()}-{attempt}-{secrets.token_hex(8)}",
        )
    )
    return urlunsplit(parsed._replace(query=urlencode(query)))


def _fetch(
    *,
    base_url: str,
    path: str,
    attempt: int,
    timeout_seconds: float,
    opener: Callable = urlopen,
) -> bytes:
    target = urljoin(base_url.rstrip("/") + "/", path.lstrip("/"))
    request = Request(
        _cache_busted(target, attempt),
        headers={"Cache-Control": "no-cache", "Pragma": "no-cache"},
    )
    with opener(
        request, timeout=min(REQUEST_TIMEOUT_SECONDS, timeout_seconds)
    ) as response:
        status = getattr(response, "status", None)
        if status != 200:
            raise VerificationError(f"{path} returned HTTP {status}")
        final_url = response.geturl()
        if _origin(final_url) != _origin(base_url):
            raise VerificationError(
                f"{path} resolved to unexpected origin {_origin(final_url)}"
            )
        final_path = urlsplit(final_url).path
        if final_path != path:
            raise VerificationError(f"{path} resolved to unexpected path {final_path}")
        return response.read()


def _site_path(site_dir: Path, path: str) -> Path:
    relative = path.lstrip("/")
    if not relative or path.endswith("/"):
        return site_dir / relative / "index.html"
    return site_dir / relative


def _resolved_site_file(site_dir: Path, path: str) -> Path:
    resolved_site = site_dir.resolve(strict=True)
    local = _site_path(resolved_site, path).resolve(strict=False)
    try:
        local.relative_to(resolved_site)
    except ValueError as error:
        raise VerificationError(
            f"published llms.txt path escapes the built site: {path}"
        ) from error
    return local


def _expected_publication(site_dir: Path, trusted_llms: bytes) -> dict[str, bytes]:
    expected_paths = {"/llms.txt", "/llms-full.txt"}
    expected_paths.update(
        "/" + path.relative_to(site_dir).as_posix() for path in site_dir.rglob("*.md")
    )
    for url in BIOMCP_URL.findall(trusted_llms.decode("utf-8")):
        expected_paths.add(urlsplit(url).path or "/")

    expected = {}
    for path in sorted(expected_paths):
        local = _resolved_site_file(site_dir, path)
        if not local.is_file():
            raise VerificationError(
                f"published llms.txt advertises an unbuilt path: {path}"
            )
        expected[path] = local.read_bytes()
    return expected


def verify_publication(
    *,
    revision: str,
    site_dir: Path,
    base_url: str,
    timeout_seconds: float,
    opener: Callable = urlopen,
    sleep: Callable[[float], None] = time.sleep,
    monotonic: Callable[[], float] = time.monotonic,
) -> None:
    if REVISION.fullmatch(revision) is None:
        raise VerificationError(
            "revision must be a 40-character lowercase hexadecimal SHA"
        )
    if not 0 < timeout_seconds <= MAX_TIMEOUT_SECONDS:
        raise VerificationError(
            "timeout must be greater than zero and at most 600 seconds"
        )
    if not site_dir.is_dir():
        raise VerificationError(f"site directory does not exist: {site_dir}")

    witness_path = f"/{REVISION_DIRECTORY}/{revision}.txt"
    expected_witness = f"{revision}\n".encode()
    deadline = monotonic() + timeout_seconds
    attempt = 0
    last_error = "no response"

    def fetch(path: str, fetch_attempt: int) -> bytes:
        remaining = deadline - monotonic()
        if remaining <= 0:
            raise VerificationError("live verification exceeded its deadline")
        return _fetch(
            base_url=base_url,
            path=path,
            attempt=fetch_attempt,
            timeout_seconds=remaining,
            opener=opener,
        )

    while True:
        if deadline - monotonic() <= 0:
            raise VerificationError(
                f"exact revision witness unavailable after {timeout_seconds:g}s: "
                f"{last_error}"
            )
        attempt += 1
        try:
            observed = fetch(witness_path, attempt)
            if observed == expected_witness:
                break
            last_error = f"stale or malformed witness body: {observed!r}"
        except (OSError, URLError, VerificationError) as error:
            last_error = str(error)

        remaining = deadline - monotonic()
        if remaining <= 0:
            raise VerificationError(
                f"exact revision witness unavailable after {timeout_seconds:g}s: "
                f"{last_error}"
            )
        sleep(min(5.0, remaining))

    live_llms = fetch("/llms.txt", attempt + 1)
    trusted_llms = _resolved_site_file(site_dir, "/llms.txt").read_bytes()
    if live_llms != trusted_llms:
        raise VerificationError("live bytes do not match the built site: /llms.txt")
    expected = _expected_publication(site_dir, trusted_llms)
    for offset, (path, body) in enumerate(expected.items(), start=attempt + 2):
        observed = fetch(path, offset)
        if observed != body:
            raise VerificationError(f"live bytes do not match the built site: {path}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--revision", required=True)
    parser.add_argument("--site-dir", type=Path, required=True)
    parser.add_argument("--base-url", default="https://biomcp.org")
    parser.add_argument("--timeout-seconds", type=float, default=600.0)
    args = parser.parse_args()
    try:
        verify_publication(
            revision=args.revision,
            site_dir=args.site_dir,
            base_url=args.base_url,
            timeout_seconds=args.timeout_seconds,
        )
    except VerificationError as error:
        print(
            f"documentation publication verification failed: {error}", file=sys.stderr
        )
        return 1
    print(f"verified documentation publication for {args.revision}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
