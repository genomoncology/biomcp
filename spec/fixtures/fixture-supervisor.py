#!/usr/bin/env python3
"""Supervise routine fixture servers without relying on catchable shell exits."""

from __future__ import annotations

import argparse
import os
import re
import select
import shutil
import signal
import subprocess
import sys
import time
from pathlib import Path

MARKER_RE = re.compile(
    r"^routine-fixture-owner:([a-z0-9][a-z0-9-]*):([0-9a-f]{32}):(/.+)$"
)


def proc_fields(pid: int) -> tuple[int, int, int] | None:
    """Return (parent PID, process group, start identity) from procfs."""
    try:
        stat = Path(f"/proc/{pid}/stat").read_text(encoding="utf-8")
        _, fields = stat.rsplit(") ", 1)
        values = fields.split()
        return int(values[1]), int(values[2]), int(values[19])
    except (FileNotFoundError, IndexError, OSError, ValueError):
        return None


def valid_identity(pid: int, start_id: int) -> bool:
    fields = proc_fields(pid)
    return fields is not None and fields[2] == start_id


def validated_root(parent: Path, root: str, prefix: str) -> Path | None:
    """Accept only a canonical, non-symlink direct fixture child."""
    try:
        canonical_parent = parent.resolve(strict=True)
        candidate = Path(root)
        if candidate.is_symlink() or not candidate.is_dir():
            return None
        canonical_root = candidate.resolve(strict=True)
    except OSError:
        return None
    if candidate != canonical_root:
        return None
    if canonical_root.parent != canonical_parent:
        return None
    if not canonical_root.name.startswith(prefix):
        return None
    return canonical_root


def marker_root(
    argv: list[bytes], parent: Path, kind: str, prefix: str
) -> Path | None:
    for argument in argv:
        try:
            marker = argument.decode("utf-8")
        except UnicodeDecodeError:
            continue
        matched = MARKER_RE.fullmatch(marker)
        if matched is None:
            continue
        if matched.group(1) != kind:
            continue
        root = validated_root(parent, matched.group(3), prefix)
        if root is not None:
            return root
    return None


def process_marker_root(
    pid: int, parent: Path, kind: str, prefix: str
) -> Path | None:
    try:
        argv = Path(f"/proc/{pid}/cmdline").read_bytes().split(b"\0")
    except (FileNotFoundError, PermissionError, OSError):
        return None
    # The detached supervisor also carries the server's marker in its argv and
    # normally has PPID 1. Recovery targets its separately sessioned child.
    if len(argv) > 2 and argv[2] == b"launch":
        try:
            if Path(os.fsdecode(argv[1])).resolve() == Path(__file__).resolve():
                return None
        except OSError:
            return None
    return marker_root(argv, parent, kind, prefix)


def remove_root(root: Path, parent: Path, prefix: str) -> None:
    """Revalidate immediately before deletion so a replaced path is never removed."""
    if validated_root(parent, str(root), prefix) == root:
        shutil.rmtree(root)


def group_alive(pgid: int) -> bool:
    try:
        os.killpg(pgid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def stop_group(pgid: int, child: subprocess.Popen[bytes] | None = None) -> None:
    """Boundedly stop an isolated process group and reap its direct child."""
    try:
        os.killpg(pgid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        if child is not None:
            child.poll()
        if not group_alive(pgid):
            break
        time.sleep(0.05)
    if group_alive(pgid):
        try:
            os.killpg(pgid, signal.SIGKILL)
        except ProcessLookupError:
            pass
    if child is not None:
        try:
            child.wait(timeout=5)
        except subprocess.TimeoutExpired:
            pass


def wait_for_owner_or_child(
    owner_pid: int, owner_start_id: int, pidfd: int | None, child: subprocess.Popen[bytes]
) -> None:
    poller: select.poll | None = None
    if pidfd is not None:
        poller = select.poll()
        poller.register(pidfd, select.POLLIN | select.POLLHUP | select.POLLERR)
    while child.poll() is None:
        if poller is not None:
            if poller.poll(100):
                return
        else:
            if not valid_identity(owner_pid, owner_start_id):
                return
    return


def open_owner_pidfd(pid: int, start_id: int) -> int | None:
    try:
        pidfd = os.pidfd_open(pid)
    except (AttributeError, OSError):
        return None
    if not valid_identity(pid, start_id):
        os.close(pidfd)
        raise ValueError("owner identity changed while opening pidfd")
    return pidfd


def launch(args: argparse.Namespace) -> int:
    if not valid_identity(args.owner_pid, args.owner_start_id):
        raise ValueError("owner PID and start identity do not match procfs")
    parent = Path(args.parent)
    root = validated_root(parent, args.root, args.prefix)
    if root is None:
        raise ValueError("fixture root is not an owned direct child of its parent")
    if marker_root(
        [os.fsencode(value) for value in args.command], parent, args.kind, args.prefix
    ) != root:
        raise ValueError("server command does not carry its authenticated owner marker")
    pidfd = open_owner_pidfd(args.owner_pid, args.owner_start_id)
    child: subprocess.Popen[bytes] | None = None
    try:
        child = subprocess.Popen(args.command, start_new_session=True, close_fds=True)
        pid_file = Path(args.pid_file)
        if pid_file.parent != root:
            raise ValueError("server PID file must be directly inside fixture root")
        pid_file.write_text(f"{child.pid}\n", encoding="utf-8")
        wait_for_owner_or_child(args.owner_pid, args.owner_start_id, pidfd, child)
        return 0
    finally:
        if pidfd is not None:
            os.close(pidfd)
        if child is not None:
            stop_group(child.pid, child)
            remove_root(root, parent, args.prefix)


def recover(args: argparse.Namespace) -> int:
    parent = Path(args.parent)
    try:
        parent = parent.resolve(strict=True)
    except OSError:
        raise ValueError("fixture cache parent does not exist") from None
    collected = 0
    for entry in Path("/proc").iterdir():
        if not entry.name.isdecimal():
            continue
        pid = int(entry.name)
        fields = proc_fields(pid)
        if fields is None or fields[0] != 1 or fields[1] != pid:
            continue
        start_id = fields[2]
        root = process_marker_root(pid, parent, args.kind, args.prefix)
        if root is None:
            continue
        # Procfs observations are mutable: authenticate the same process again
        # immediately before any signal or filesystem mutation.
        fields = proc_fields(pid)
        if (
            fields is None
            or fields[0] != 1
            or fields[1] != pid
            or fields[2] != start_id
        ):
            continue
        if process_marker_root(pid, parent, args.kind, args.prefix) != root:
            continue
        stop_group(pid)
        remove_root(root, parent, args.prefix)
        collected += 1
    print(
        f"fixture supervisor: collected {collected} {args.kind} orphan(s)",
        file=sys.stderr,
    )
    return 0


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser()
    commands = result.add_subparsers(dest="mode", required=True)
    launch_parser = commands.add_parser("launch")
    launch_parser.add_argument("owner_pid", type=int)
    launch_parser.add_argument("owner_start_id", type=int)
    launch_parser.add_argument("kind")
    launch_parser.add_argument("parent")
    launch_parser.add_argument("root")
    launch_parser.add_argument("prefix")
    launch_parser.add_argument("pid_file")
    launch_parser.add_argument("command", nargs=argparse.REMAINDER)
    recover_parser = commands.add_parser("recover")
    recover_parser.add_argument("parent")
    recover_parser.add_argument("kind")
    recover_parser.add_argument("prefix")
    return result


def main() -> int:
    args = parser().parse_args()
    if args.mode == "launch":
        if not args.command:
            raise ValueError("launch requires a server command")
        return launch(args)
    return recover(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ValueError as error:
        print(f"fixture supervisor: {error}", file=sys.stderr)
        raise SystemExit(2)
