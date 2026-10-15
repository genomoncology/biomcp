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

# A ps command line joins arguments with single spaces, so the marker's root
# path can run past its own end; accept it only when it also validates as a
# canonical fixture child.
MARKER_LINE_RE = re.compile(
    r"routine-fixture-owner:([a-z0-9][a-z0-9-]*):([0-9a-f]{32}):(/.+)$", re.DOTALL
)

PROCFS = Path("/proc").is_dir()


def proc_fields(pid: int) -> tuple[int, int, str] | None:
    """Return (parent PID, process group, start identity) for a live PID.

    procfs supplies all three on Linux; without it `ps` does, with a start
    identity coarse to the second. The identity is only compared for equality
    on the host that produced it.
    """
    if PROCFS:
        try:
            stat = Path(f"/proc/{pid}/stat").read_text(encoding="utf-8")
            _, fields = stat.rsplit(") ", 1)
            values = fields.split()
            return int(values[1]), int(values[2]), values[19]
        except (FileNotFoundError, IndexError, OSError, ValueError):
            return None
    try:
        listing = subprocess.run(
            ["ps", "-o", "ppid=,pgid=,lstart=", "-p", str(pid)],
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError:
        return None
    if listing.returncode != 0 or not listing.stdout.strip():
        return None
    values = listing.stdout.split()
    if len(values) < 7:
        return None
    try:
        return int(values[0]), int(values[1]), " ".join(values[2:])
    except ValueError:
        return None


def valid_identity(pid: int, start_id: str) -> bool:
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


def marker_root_in_line(line: str, parent: Path, kind: str, prefix: str) -> Path | None:
    """Find a marker in one ps command line, spaces included.

    The path may continue into later arguments; extend it across each space
    boundary and accept the first extent that validates canonically, so a
    real space in the path still resolves while a truncated prefix of another
    argument's text never does.
    """
    for matched in MARKER_LINE_RE.finditer(line):
        if matched.group(1) != kind:
            continue
        rest = matched.group(3)
        boundaries = [index for index, char in enumerate(rest) if char == " "]
        boundaries.append(len(rest))
        for boundary in boundaries:
            root = validated_root(parent, rest[:boundary], prefix)
            if root is not None:
                return root
    return None


def is_supervisor_command(command: str) -> bool:
    """True for a supervisor launch, whose argv also carries the marker.

    Recovery targets the separately sessioned server child, never the live
    supervisor watching it.
    """
    return "fixture-supervisor.py launch " in command


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
    owner_pid: int, owner_start_id: str, pidfd: int | None, child: subprocess.Popen[bytes]
) -> None:
    poller: select.poll | None = None
    if pidfd is not None:
        poller = select.poll()
        poller.register(pidfd, select.POLLIN | select.POLLHUP | select.POLLERR)
    next_identity_check = 0.0
    while child.poll() is None:
        if poller is not None:
            if poller.poll(100):
                return
        else:
            # Without a pidfd, check the owner cheaply and revalidate its full
            # identity on a slow cadence so a long run does not spawn one `ps`
            # per poll on platforms without procfs.
            try:
                os.kill(owner_pid, 0)
            except ProcessLookupError:
                return
            except PermissionError:
                pass
            now = time.monotonic()
            if now >= next_identity_check:
                if not valid_identity(owner_pid, owner_start_id):
                    return
                next_identity_check = now + 5
            time.sleep(0.2)
    return


def open_owner_pidfd(pid: int, start_id: str) -> int | None:
    try:
        pidfd = os.pidfd_open(pid)
    except (AttributeError, OSError):
        return None
    if not valid_identity(pid, start_id):
        os.close(pidfd)
        raise ValueError("owner identity changed while opening pidfd")
    return pidfd


def ensure_detached_session() -> None:
    """Move into a new session like setsid(1), forking if a group leader.

    Detaching here replaces the setsid(1) binary, which the macOS gate host
    does not ship, and keeps the detached lifecycle defined in one place.
    """
    try:
        os.setsid()
        return
    except OSError:
        if os.getpid() != os.getpgrp():
            raise
    if os.fork() != 0:
        os._exit(0)
    os.setsid()


def launch(args: argparse.Namespace) -> int:
    ensure_detached_session()
    if not valid_identity(args.owner_pid, args.owner_start_id):
        raise ValueError("owner PID and start identity do not match the running process")
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


def iter_processes() -> list[tuple[int, tuple[int, int, str] | None, str]]:
    """One snapshot of every process as (pid, fields, command line)."""
    snapshot: list[tuple[int, tuple[int, int, str] | None, str]] = []
    if PROCFS:
        for entry in Path("/proc").iterdir():
            if not entry.name.isdecimal():
                continue
            pid = int(entry.name)
            try:
                command = Path(f"/proc/{pid}/cmdline").read_bytes().replace(
                    b"\0", b" "
                ).decode("utf-8", "replace").rstrip(" ")
            except (FileNotFoundError, PermissionError, OSError):
                command = ""
            snapshot.append((pid, proc_fields(pid), command))
        return snapshot
    try:
        listing = subprocess.run(
            ["ps", "-axo", "pid=,ppid=,pgid=,command="],
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError:
        return snapshot
    for line in listing.stdout.splitlines():
        values = line.split(maxsplit=3)
        if len(values) < 4:
            continue
        try:
            pid, ppid, pgid = (int(value) for value in values[:3])
        except ValueError:
            continue
        snapshot.append((pid, (ppid, pgid, ""), values[3]))
    return snapshot


def recover(args: argparse.Namespace) -> int:
    parent = Path(args.parent)
    try:
        parent = parent.resolve(strict=True)
    except OSError:
        raise ValueError("fixture cache parent does not exist") from None
    collected = 0
    for pid, fields, command in iter_processes():
        if fields is None or fields[0] != 1 or fields[1] != pid:
            continue
        if is_supervisor_command(command):
            continue
        start_id = fields[2]
        root = marker_root_in_line(command, parent, args.kind, args.prefix)
        if root is None:
            continue
        # Process-table observations are mutable: authenticate the same process
        # again immediately before any signal or filesystem mutation.
        fields = proc_fields(pid)
        if (
            fields is None
            or fields[0] != 1
            or fields[1] != pid
            or fields[2] != start_id
        ):
            continue
        if marker_root_in_line(process_command(pid), parent, args.kind, args.prefix) != root:
            continue
        stop_group(pid)
        remove_root(root, parent, args.prefix)
        collected += 1
    print(
        f"fixture supervisor: collected {collected} {args.kind} orphan(s)",
        file=sys.stderr,
    )
    return 0


def process_command(pid: int) -> str:
    """Re-read one process's command line for revalidation."""
    if PROCFS:
        try:
            return (
                Path(f"/proc/{pid}/cmdline")
                .read_bytes()
                .replace(b"\0", b" ")
                .decode("utf-8", "replace")
                .rstrip(" ")
            )
        except (FileNotFoundError, PermissionError, OSError):
            return ""
    try:
        listing = subprocess.run(
            ["ps", "-ww", "-o", "command=", "-p", str(pid)],
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError:
        return ""
    if listing.returncode != 0:
        return ""
    return listing.stdout.splitlines()[0] if listing.stdout else ""


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser()
    commands = result.add_subparsers(dest="mode", required=True)
    launch_parser = commands.add_parser("launch")
    launch_parser.add_argument("owner_pid", type=int)
    launch_parser.add_argument("owner_start_id")
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
