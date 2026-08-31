from __future__ import annotations

import hashlib
import json
import os
import re
import shlex
import shutil
import subprocess
from pathlib import Path

import pytest

# The lifecycle scripts always act on the checkout in which they run. Keeping
# this as cwd lets baseline execute this same test module against origin/main.
REPO_ROOT = Path.cwd()
PROJECT = REPO_ROOT / "sdlc" / "project"


def _git(repo: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=repo,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def _run(repo: Path, script: str, environment: dict[str, str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["bash", str(PROJECT / script)],
        cwd=repo,
        env=os.environ | environment,
        capture_output=True,
        text=True,
        check=False,
    )


def _tasks(repo: Path, environment: dict[str, str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(PROJECT / "tasks")],
        cwd=repo,
        env=os.environ | environment,
        capture_output=True,
        text=True,
        check=False,
    )


def _path_with_blocked_worktree_removal(tmp_path: Path, tree: Path) -> str:
    bin_dir = tmp_path / "blocked-worktree-removal-bin"
    bin_dir.mkdir()
    git_command = shutil.which("git")
    rm_command = shutil.which("rm")
    assert git_command is not None
    assert rm_command is not None
    (bin_dir / "git").write_text(
        "#!/bin/sh\n"
        'if [ "$1" = -C ] && [ "$3" = worktree ] && [ "$4" = remove ]; then exit 1; fi\n'
        f'exec {shlex.quote(git_command)} "$@"\n',
        encoding="utf-8",
    )
    (bin_dir / "rm").write_text(
        "#!/bin/sh\n"
        f'if [ "$1" = -rf ] && [ "$2" = {shlex.quote(str(tree))} ]; then exit 1; fi\n'
        f'exec {shlex.quote(rm_command)} "$@"\n',
        encoding="utf-8",
    )
    (bin_dir / "git").chmod(0o755)
    (bin_dir / "rm").chmod(0o755)
    return f"{bin_dir}:{os.environ['PATH']}"


def _path_with_fetch_failures(
    tmp_path: Path,
    failures: list[tuple[int, str]],
    calls: Path,
) -> str:
    bin_dir = tmp_path / "fetch-failure-bin"
    bin_dir.mkdir()
    timeout_command = shutil.which("timeout")
    assert timeout_command is not None
    cases = []
    for attempt, (status, stderr) in enumerate(failures, start=1):
        output = tmp_path / f"fetch-failure-{attempt}.stderr"
        output.write_text(stderr, encoding="utf-8")
        cases.append(
            f"  {attempt}) cat {shlex.quote(str(output))} >&2; exit {status} ;;"
        )
    wrapper = bin_dir / "timeout"
    wrapper.write_text(
        "#!/bin/sh\n"
        "count=0\n"
        f"[ ! -f {shlex.quote(str(calls))} ] || count=$(cat {shlex.quote(str(calls))})\n"
        "count=$((count + 1))\n"
        f"printf '%s\\n' \"$count\" > {shlex.quote(str(calls))}\n"
        'case "$count" in\n'
        + "\n".join(cases)
        + "\nesac\n"
        + f'exec {shlex.quote(timeout_command)} "$@"\n',
        encoding="utf-8",
    )
    wrapper.chmod(0o755)
    return f"{bin_dir}:{os.environ['PATH']}"


def _fixture(tmp_path: Path) -> tuple[Path, Path]:
    origin = tmp_path / "origin.git"
    repo = tmp_path / "registered"
    subprocess.run(
        ["git", "init", "--bare", "--initial-branch=main", str(origin)],
        check=True,
        capture_output=True,
        text=True,
    )
    subprocess.run(
        ["git", "clone", "--quiet", str(origin), str(repo)],
        check=True,
        capture_output=True,
        text=True,
    )
    _git(repo, "config", "user.name", "Lifecycle contract")
    _git(repo, "config", "user.email", "lifecycle@example.invalid")
    (repo / "sdlc" / "project").mkdir(parents=True)
    (repo / "sdlc" / "scripts").mkdir()
    for folder in ("tickets", "records", "issues"):
        (repo / "sdlc" / folder).mkdir()
    for script in ("before", "success", "failure", "health", "tasks"):
        shutil.copy2(PROJECT / script, repo / "sdlc" / "project" / script)
    for script in ("lint", "test"):
        target = repo / "sdlc" / "scripts" / script
        target.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
        target.chmod(0o755)
    (repo / "Makefile").write_text(".PHONY: lint test\nlint:\n\t@:\ntest:\n\t@:\n", encoding="utf-8")
    _git(repo, "add", ".")
    _git(repo, "commit", "--quiet", "-m", "fixture: add lifecycle contracts")
    _git(repo, "push", "--quiet", "-u", "origin", "main")
    return repo, origin


def _bot(tmp_path: Path) -> Path:
    command = tmp_path / "bot"
    command.write_text("#!/bin/sh\n[ \"$1\" = busy ] && exit 1\nexit 2\n", encoding="utf-8")
    command.chmod(0o755)
    return command


def _witness_environment(tmp_path: Path, tip: str) -> dict[str, str]:
    run_id = "witnessed-run"
    bot_home = tmp_path / "bot-home"
    capture = bot_home / "runs" / run_id / "captures" / "witness"
    capture.parent.mkdir(parents=True)
    capture.write_text(f"verified {tip}\n", encoding="utf-8")
    event = json.dumps(
        {
            "event": "check",
            "check": "gate",
            "file": "flows/build/05-verify/gate/06-witness",
            "exit": 0,
            "capture": "captures/witness",
        }
    )
    bot = tmp_path / "witness-bot"
    bot.write_text(
        "#!/bin/sh\n"
        f"[ \"$1 $2 $3\" = \"show {run_id} --json\" ] || exit 2\n"
        f"printf '%s\\n' {shlex.quote(event)}\n",
        encoding="utf-8",
    )
    bot.chmod(0o755)
    return {
        "TICKET_FLOW": "build",
        "RUN_ID": run_id,
        "BOT_CMD": str(bot),
        "BOT_HOME": str(bot_home),
    }


def _before(repo: Path, tmp_path: Path, ticket_id: str) -> tuple[subprocess.CompletedProcess[str], Path | None]:
    bot = _bot(tmp_path)
    result = _run(
        repo,
        "before",
        {
            "TICKET_ID": ticket_id,
            "TICKET_FLOW": "quickfix",
            "WORKTREE_ROOT": str(tmp_path / "worktrees"),
            "PATH": f"{bot.parent}:{os.environ['PATH']}",
        },
    )
    match = re.search(r"^dir: (.+)$", result.stdout, re.MULTILINE)
    return result, Path(match.group(1)) if match is not None else None


def _commit(repo: Path, path: str, contents: str, message: str) -> str:
    target = repo / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(contents, encoding="utf-8")
    _git(repo, "add", path)
    _git(repo, "commit", "--quiet", "-m", message)
    return _git(repo, "rev-parse", "HEAD")


def _clone(origin: Path, destination: Path) -> Path:
    subprocess.run(
        ["git", "clone", "--quiet", str(origin), str(destination)],
        check=True,
        capture_output=True,
        text=True,
    )
    _git(destination, "config", "user.name", "Lifecycle contract")
    _git(destination, "config", "user.email", "lifecycle@example.invalid")
    return destination


@pytest.mark.parametrize(
    ("name", "expected_hash", "expected_executable"),
    [
        (
            "before",
            "d7070670a2922ecae40e7f445dd575d509efb5b834a9fb8d8cdc93032f10e1fa",
            True,
        ),
        (
            "failure",
            "b513850fc6e07560a7f204a6d8b3ec50aeed95e345f9686dd79fa0592753246d",
            True,
        ),
        (
            "health",
            "d937ce107219e2bddc879f1a04ff4e2f7a37e392b08373c24c1b2ab92194ec59",
            True,
        ),
        (
            "success",
            "a2ad568ae5e7e0f83328fd88aa2ae303a8ffe4d039f6ee4e2985196eaaee34fd",
            True,
        ),
        (
            "tasks",
            "e8e862034c0c3b46e3a6e937a026d82d83d3e2809bbfe45d1dce1ce5d26e5545",
            True,
        ),
        (
            "provenance.json",
            "dd5d2d7019661d53d8229245d2bca1ab89688c72c3f31567020393053103d355",
            None,
        ),
    ],
)
def test_project_files_match_canonical_adoption(
    name: str, expected_hash: str, expected_executable: bool | None
) -> None:
    adopted = PROJECT / name

    assert hashlib.sha256(adopted.read_bytes()).hexdigest() == expected_hash
    if expected_executable is not None:
        assert bool(adopted.stat().st_mode & 0o111) is expected_executable


def test_tasks_retries_an_ordinary_fetch_failure_before_scanning(
    tmp_path: Path,
) -> None:
    repo, _origin = _fixture(tmp_path)
    ticket = "sdlc/tickets/2054-transient-fetch.md"
    _commit(
        repo,
        ticket,
        "---\nflow: build\npriority: 5\n---\n# Retry a transient fetch\n",
        "ticket: add transient fetch fixture",
    )
    _git(repo, "push", "--quiet", "origin", "main")
    calls = tmp_path / "fetch-calls"

    result = _tasks(
        repo,
        {
            "PATH": _path_with_fetch_failures(
                tmp_path, [(1, "connection refused\n")], calls
            )
        },
    )

    assert result.returncode == 0, result.stderr
    assert result.stderr == ""
    row = json.loads(result.stdout)
    assert (row["id"], row["ref"], row["status"]) == ("2054", ticket, "ready")
    assert calls.read_text(encoding="utf-8").strip() == "2"


def test_tasks_double_fetch_failure_keeps_a_bounded_final_diagnostic(
    tmp_path: Path,
) -> None:
    repo, _origin = _fixture(tmp_path)
    calls = tmp_path / "fetch-calls"
    final_line = f"final refusal\x07 {'é' * 600}"

    result = _tasks(
        repo,
        {
            "PATH": _path_with_fetch_failures(
                tmp_path,
                [
                    (1, "first attempt failed\n"),
                    (1, f"discard this line\n{final_line}\n\n"),
                ],
                calls,
            )
        },
    )
    diagnostic = result.stderr.removesuffix("\n")

    assert result.returncode == 2
    assert result.stdout == ""
    assert "final refusal " in diagnostic
    assert "first attempt failed" not in diagnostic
    assert "discard this line" not in diagnostic
    assert not re.search(r"[\x00-\x1f\x7f]", diagnostic)
    assert len(result.stderr.splitlines()) == 1
    assert len(diagnostic.encode("utf-8")) <= 1024
    assert calls.read_text(encoding="utf-8").strip() == "2"


def test_before_returns_raw_gate_output_and_ends_with_a_bounded_verdict(
    tmp_path: Path,
) -> None:
    repo, _origin = _fixture(tmp_path)
    assertion = f"expected fixture bytes {'x' * 3_000} assertion-tail"
    tap = "\n".join(
        [
            "TAP version 13",
            "# Subtest: passing contract",
            "ok 1 - passing contract",
            "# Subtest: fixture drift",
            "not ok 2 - fixture drift",
            "  ---",
            "  name: 'AssertionError'",
            f"  error: '{assertion}'",
            "  ...",
            "1..2",
            "# tests 2",
            "# pass 1",
            "# fail 1",
        ]
    )
    test_script = repo / "sdlc" / "scripts" / "test"
    test_script.write_text(
        f"#!/bin/sh\ncat <<'TAP'\n{tap}\nTAP\nexit 1\n",
        encoding="utf-8",
    )
    _git(repo, "add", "sdlc/scripts/test")
    _git(repo, "commit", "--quiet", "-m", "fixture: add failing gate")
    _git(repo, "push", "--quiet", "origin", "main")
    main = _git(repo, "rev-parse", "origin/main")
    bot = _bot(tmp_path)

    failed = _run(
        repo,
        "before",
        {
            "TICKET_ID": "1077",
            "TICKET_FLOW": "build",
            "WORKTREE_ROOT": str(tmp_path / "worktrees"),
            "PATH": f"{bot.parent}:{os.environ['PATH']}",
        },
    )
    verdict = failed.stderr.rstrip().splitlines()[-1]

    assert failed.returncode == 3
    assert f"{tap}\n" in failed.stderr
    assert re.search(
        r"test: 1/2 passed; failed: fixture drift: .*expected fixture bytes",
        verdict,
    )
    assert main in verdict
    assert len(verdict.encode("utf-8")) <= 1_024


def test_before_bounds_the_complete_unparsed_gate_verdict(tmp_path: Path) -> None:
    repo, _origin = _fixture(tmp_path)
    gate_output = "\n".join(f"non-TAP failure {line}: {'x' * 100}" for line in range(20))
    test_script = repo / "sdlc" / "scripts" / "test"
    test_script.write_text(
        f"#!/bin/sh\ncat <<'OUTPUT'\n{gate_output}\nOUTPUT\nexit 1\n",
        encoding="utf-8",
    )
    _git(repo, "add", "sdlc/scripts/test")
    _git(repo, "commit", "--quiet", "-m", "fixture: add unparsed gate failure")
    _git(repo, "push", "--quiet", "origin", "main")
    main = _git(repo, "rev-parse", "origin/main")
    bot = _bot(tmp_path)

    failed = _run(
        repo,
        "before",
        {
            "TICKET_ID": "1078",
            "TICKET_FLOW": "build",
            "WORKTREE_ROOT": str(tmp_path / "worktrees"),
            "PATH": f"{bot.parent}:{os.environ['PATH']}",
        },
    )
    header = f"test: failed; unparsed; origin/main {main}\n"
    verdict_start = failed.stderr.rfind(header)

    assert failed.returncode == 3
    assert f"{gate_output}\n" in failed.stderr
    assert verdict_start >= 0
    assert len(failed.stderr[verdict_start:].encode("utf-8")) <= 1_025


def test_before_allows_canonical_adoption_to_repair_red_main(
    tmp_path: Path,
) -> None:
    repo, _origin = _fixture(tmp_path)
    ticket_id = "1083"
    ticket = f"sdlc/tickets/{ticket_id}-adopt-canonical-lifecycle.md"
    _commit(
        repo,
        ticket,
        "---\n"
        "flow: build\n"
        "priority: 10\n"
        "---\n"
        "# Adopt canonical lifecycle scripts\n\n"
        "Adoption identity: "
        "sdlc/tickets/0220-consumer-adoption-reaches-a-red-main-consumer.md@"
        "7d0c4bad86cd64d8b556a2182d0a60a8aec342c1\n\n"
        "Adopt the canonical lifecycle changes landed by "
        "`sdlc/tickets/0220-consumer-adoption-reaches-a-red-main-consumer.md` "
        "at exact commit `7d0c4bad86cd64d8b556a2182d0a60a8aec342c1`.\n"
        "Apply exactly these bytes and executable states:\n\n"
        "- path: sdlc/project/before\n"
        f"  sha256: {'a' * 64}\n"
        "  executable: true\n\n"
        "Adopt the matching provenance manifest:\n\n"
        "- path: sdlc/project/provenance.json\n"
        f"  sha256: {'b' * 64}\n",
        "ticket: add canonical adoption fixture",
    )
    test_script = repo / "sdlc" / "scripts" / "test"
    test_script.write_text(
        "#!/bin/sh\n"
        "printf '%s\\n' 'TAP version 13' "
        "'# Subtest: canonical lifecycle drift' "
        "'not ok 1 - canonical lifecycle drift' "
        "'1..1' '# tests 1' '# pass 0' '# fail 1'\n"
        "exit 1\n",
        encoding="utf-8",
    )
    _git(repo, "add", "sdlc/scripts/test")
    _git(repo, "commit", "--quiet", "-m", "fixture: make lifecycle drift red")
    _git(repo, "push", "--quiet", "origin", "main")
    bot = _bot(tmp_path)

    prepared = _run(
        repo,
        "before",
        {
            "TICKET_ID": ticket_id,
            "TICKET_REF": ticket,
            "TICKET_FLOW": "build",
            "WORKTREE_ROOT": str(tmp_path / "worktrees"),
            "PATH": f"{bot.parent}:{os.environ['PATH']}",
        },
    )
    tree_match = re.search(r"^dir: (.+)$", prepared.stdout, re.MULTILINE)

    assert prepared.returncode == 0, prepared.stderr
    assert tree_match is not None
    assert Path(tree_match.group(1)).is_dir()
    assert "canonical lifecycle adoption" in prepared.stderr
    assert "sdlc/scripts/test" in prepared.stderr


def test_before_reclaims_an_owned_orphaned_worktree(tmp_path: Path) -> None:
    repo, _origin = _fixture(tmp_path)
    ticket_id = "1052"
    prepared, tree = _before(repo, tmp_path, ticket_id)
    assert prepared.returncode == 0, prepared.stderr
    assert tree is not None
    marker = tree / "orphan-marker"
    marker.write_text("discard this abandoned worktree\n", encoding="utf-8")

    tree.chmod(0o555)
    try:
        removal = subprocess.run(
            ["git", "-C", str(repo), "worktree", "remove", "--force", str(tree)],
            capture_output=True,
            text=True,
            check=False,
        )
    finally:
        tree.chmod(0o755)
    assert removal.returncode != 0
    assert marker.exists()

    retried, replacement = _before(repo, tmp_path, ticket_id)

    assert retried.returncode == 0, retried.stderr
    assert replacement == tree
    assert not marker.exists()


def test_success_refuses_after_unrelated_ticket_only_main_movement(tmp_path: Path) -> None:
    repo, origin = _fixture(tmp_path)
    ticket_id = "1052"
    prepared, tree = _before(repo, tmp_path, ticket_id)
    assert prepared.returncode == 0, prepared.stderr
    assert tree is not None
    candidate = _commit(
        tree,
        "CANDIDATE",
        "keep this verified change\n",
        "candidate: retain verified change",
    )

    publisher = _clone(origin, tmp_path / "publisher")
    _commit(
        publisher,
        "sdlc/tickets/2001-unrelated.md",
        "---\nflow: build\npriority: 1\n---\n# Unrelated ticket\n",
        "ticket: add unrelated ticket",
    )
    _git(publisher, "push", "--quiet", "origin", "main")

    settled = _run(
        repo,
        "success",
        {
            "TICKET_ID": ticket_id,
            "ATTEMPT_DIR": str(tree),
            "SDLC_REPO": str(repo),
            "SDLC_BRANCH": f"ticket/{ticket_id}",
            "TICKET_REF": "sdlc/tickets/1052-adopt-lifecycle.md",
        }
        | _witness_environment(tmp_path, candidate),
    )

    assert settled.returncode == 3
    _git(repo, "fetch", "--quiet", "origin")
    assert _git(repo, "ls-tree", "--name-only", "origin/main", "CANDIDATE") == ""
    assert _git(repo, "show", "origin/main:sdlc/tickets/2001-unrelated.md") == (
        "---\nflow: build\npriority: 1\n---\n# Unrelated ticket"
    )
    assert tree.exists()


def test_failure_withdrawal_receipt_preserves_evidence_and_cleans_up(tmp_path: Path) -> None:
    repo, origin = _fixture(tmp_path)
    ticket_id = "1052"
    prepared, tree = _before(repo, tmp_path, ticket_id)
    assert prepared.returncode == 0, prepared.stderr
    assert tree is not None
    baseline = Path(f"{tree}.baseline")
    _git(repo, "worktree", "add", "--quiet", "--detach", str(baseline), "origin/main")

    _commit(tree, "LOCAL_FIRST", "first local evidence\n", "candidate: save first evidence")
    _git(tree, "push", "--quiet", "origin", f"ticket/{ticket_id}")
    publisher = _clone(origin, tmp_path / "withdrawal-publisher")
    _git(publisher, "checkout", "--quiet", "-b", "remote-evidence", f"origin/ticket/{ticket_id}")
    remote = _commit(publisher, "REMOTE_EVIDENCE", "remote evidence\n", "candidate: save remote evidence")
    _git(publisher, "push", "--quiet", "origin", f"HEAD:ticket/{ticket_id}")
    local = _commit(tree, "LOCAL_EVIDENCE", "local evidence\n", "candidate: save local evidence")

    settled = _run(
        repo,
        "failure",
        {
            "TICKET_ID": ticket_id,
            "SETTLEMENT": "withdrawn",
            "BOT_CMD": str(_bot(tmp_path)),
        },
    )

    assert settled.returncode == 0, settled.stderr
    assert settled.stdout == f"withdrawal-cleaned {ticket_id}\n"
    assert any(tag.endswith("-local") for tag in _git(repo, "tag", "--points-at", local).splitlines())
    assert any(not tag.endswith("-local") for tag in _git(repo, "tag", "--points-at", remote).splitlines())
    assert not tree.exists()
    assert not baseline.exists()
    assert _git(repo, "branch", "--list", f"ticket/{ticket_id}") == ""
    assert _git(repo, "ls-remote", "--heads", "origin", f"ticket/{ticket_id}") == ""


def test_health_distinguishes_clean_and_dirty_working_trees(
    tmp_path: Path,
) -> None:
    repo, _origin = _fixture(tmp_path)

    clean = _run(repo, "health", {})
    (repo / "UNTRACKED").write_text("local work\n", encoding="utf-8")
    dirty = _run(repo, "health", {})

    assert clean.returncode == dirty.returncode == 0
    clean_lines = clean.stdout.splitlines()
    dirty_lines = dirty.stdout.splitlines()
    differences = [
        (clean_line, dirty_line)
        for clean_line, dirty_line in zip(clean_lines, dirty_lines, strict=True)
        if clean_line != dirty_line
    ]
    assert len(differences) == 1
    clean_line, dirty_line = differences[0]
    assert "working tree" in clean_line
    assert "clean" in clean_line
    assert "working tree" in dirty_line
    assert "uncommitted" in dirty_line

    _git(repo, "remote", "set-url", "origin", str(tmp_path / "missing-origin.git"))
    fetch_failed = _run(repo, "health", {})
    assert fetch_failed.returncode == 1
    assert any(
        "working tree" in line and "uncommitted" in line
        for line in fetch_failed.stdout.splitlines()
    )


def test_health_surfaces_a_failing_project_extension(tmp_path: Path) -> None:
    repo, _origin = _fixture(tmp_path)
    extension = repo / "sdlc" / "scripts" / "health"
    extension.write_text(
        "#!/bin/sh\n"
        "[ \"$(pwd -P)\" = \"$(git rev-parse --show-toplevel)\" ] || exit 2\n"
        "printf '%s\\n' 'upstream checkout is behind its origin'\n"
        "exit 1\n",
        encoding="utf-8",
    )
    extension.chmod(0o755)

    health = _run(repo, "health", {})

    assert health.returncode == 1
    assert "upstream checkout is behind its origin" in health.stdout


def test_health_reports_malformed_opens_from_origin_main(tmp_path: Path) -> None:
    repo, _origin = _fixture(tmp_path)
    policy = repo / "assembly" / "flows" / "build" / "05-verify" / "gate" / "05-path-policy"
    policy.parent.mkdir(parents=True)
    policy.write_text(
        "#!/bin/sh\n"
        "git show \"origin/main:$TICKET_REF\" | grep -Fq 'opens: sdlc/scripts/*' || exit 0\n"
        "echo \"path policy: opens must name exact file paths: sdlc/scripts/*\" >&2\n"
        "exit 1\n",
        encoding="utf-8",
    )
    ticket = "sdlc/tickets/2002-malformed-opening.md"
    _commit(
        repo,
        ticket,
        "---\nflow: build\npriority: 1\nopens: sdlc/scripts/*\n---\n# Invalid opening\n",
        "ticket: add malformed opening",
    )
    _git(repo, "add", str(policy.relative_to(repo)))
    _git(repo, "commit", "--quiet", "-m", "fixture: add path policy")
    _git(repo, "push", "--quiet", "origin", "main")

    health = _run(repo, "health", {})

    assert health.returncode == 1
    assert ticket in health.stdout
    assert "sdlc/scripts/*" in health.stdout


def test_success_without_deploy_hook_is_quiet(tmp_path: Path) -> None:
    repo, _origin = _fixture(tmp_path)
    ticket_id = "1052"
    prepared, tree = _before(repo, tmp_path, ticket_id)
    assert prepared.returncode == 0, prepared.stderr
    assert tree is not None
    tip = _commit(
        tree,
        "LANDED",
        "no deployment extension is needed\n",
        "candidate: settle without deploy",
    )

    settled = _run(
        repo,
        "success",
        {
            "TICKET_ID": ticket_id,
            "ATTEMPT_DIR": str(tree),
            "SDLC_REPO": str(repo),
            "SDLC_BRANCH": f"ticket/{ticket_id}",
        }
        | _witness_environment(tmp_path, tip),
    )

    assert not (repo / "sdlc" / "scripts" / "deploy").exists()
    assert settled.returncode == 0, settled.stderr
    assert settled.stderr == ""


def test_success_reports_pending_activation_for_a_dirty_checkout(
    tmp_path: Path,
) -> None:
    repo, _origin = _fixture(tmp_path)
    ticket_id = "1053"
    prepared, tree = _before(repo, tmp_path, ticket_id)
    assert prepared.returncode == 0, prepared.stderr
    assert tree is not None
    base = _git(repo, "rev-parse", "HEAD")
    tip = _commit(
        tree,
        "LANDED",
        "the ticket must activate after landing\n",
        "candidate: await activation",
    )
    local_work = repo / "LOCAL"
    local_work.write_text("do not overwrite this checkout work\n", encoding="utf-8")

    settled = _run(
        repo,
        "success",
        {
            "TICKET_ID": ticket_id,
            "ATTEMPT_DIR": str(tree),
            "SDLC_REPO": str(repo),
            "SDLC_BRANCH": f"ticket/{ticket_id}",
        }
        | _witness_environment(tmp_path, tip),
    )

    assert settled.returncode == 4, settled.stderr
    assert settled.stdout == f"activation-pending {base}..{tip}\n"
    assert "checkout left alone: checkout is dirty" in settled.stderr
    assert _git(repo, "rev-parse", "HEAD") == base
    assert (
        local_work.read_text(encoding="utf-8")
        == "do not overwrite this checkout work\n"
    )
    _git(repo, "fetch", "--quiet", "origin")
    assert _git(repo, "rev-parse", "origin/main") == tip
    assert not tree.exists()
    assert _git(repo, "branch", "--list", f"ticket/{ticket_id}") == ""


def test_success_activates_a_durable_landing_without_republishing(
    tmp_path: Path,
) -> None:
    repo, origin = _fixture(tmp_path)
    deployed = tmp_path / "activated"
    hook = repo / "sdlc" / "scripts" / "deploy"
    hook.write_text(
        "#!/bin/sh\n"
        f'printf \'%s %s %s\\n\' "$LANDED_BASE" "$LANDED_TIP" "$(git rev-parse HEAD)" > {shlex.quote(str(deployed))}\n',
        encoding="utf-8",
    )
    hook.chmod(0o755)
    base = _commit(
        repo,
        "sdlc/scripts/deploy",
        hook.read_text(encoding="utf-8"),
        "fixture: add deploy hook",
    )
    _git(repo, "push", "--quiet", "origin", "main")
    publisher = _clone(origin, tmp_path / "publisher")
    tip = _commit(
        publisher,
        "LANDED",
        "this commit is already durable\n",
        "fixture: land activation",
    )
    _git(publisher, "push", "--quiet", "origin", "main")

    activated = _run(
        repo,
        "success",
        {
            "SDLC_REPO": str(repo),
            "ACTIVATION_BASE": base,
            "ACTIVATION_TIP": tip,
        },
    )

    assert activated.returncode == 0, activated.stderr
    assert activated.stdout == f"activated {base}..{tip}\n"
    assert _git(repo, "rev-parse", "HEAD") == tip
    assert deployed.read_text(encoding="utf-8") == f"{base} {tip} {tip}\n"


def test_success_activates_the_current_main_descendant_with_the_stored_receipt(
    tmp_path: Path,
) -> None:
    repo, origin = _fixture(tmp_path)
    deployed = tmp_path / "activated-descendant"
    hook = repo / "sdlc" / "scripts" / "deploy"
    hook.write_text(
        "#!/bin/sh\n"
        f'printf \'%s %s %s\\n\' "$(git rev-parse HEAD)" "$LANDED_BASE" "$LANDED_TIP" > {shlex.quote(str(deployed))}\n',
        encoding="utf-8",
    )
    hook.chmod(0o755)
    base = _commit(
        repo,
        "sdlc/scripts/deploy",
        hook.read_text(encoding="utf-8"),
        "fixture: add deploy hook",
    )
    _git(repo, "push", "--quiet", "origin", "main")
    publisher = _clone(origin, tmp_path / "descendant-publisher")
    stored_tip = _commit(
        publisher,
        "LANDED",
        "this is the stored durable landing\n",
        "fixture: land stored activation tip",
    )
    _git(publisher, "push", "--quiet", "origin", "main")
    current_tip = _commit(
        publisher,
        "LATER_MAIN",
        "main advanced after the durable landing\n",
        "fixture: advance main after stored tip",
    )
    _git(publisher, "push", "--quiet", "origin", "main")
    _git(repo, "pull", "--quiet", "--ff-only")

    activated = _run(
        repo,
        "success",
        {
            "SDLC_REPO": str(repo),
            "ACTIVATION_BASE": base,
            "ACTIVATION_TIP": stored_tip,
        },
    )

    assert activated.returncode == 0, activated.stderr
    assert activated.stdout == f"activated {base}..{stored_tip}\n"
    assert _git(repo, "rev-parse", "HEAD") == current_tip
    assert _git(repo, "rev-parse", "origin/main") == current_tip
    assert _git(repo, "status", "--porcelain") == ""
    assert (
        deployed.read_text(encoding="utf-8")
        == f"{current_tip} {base} {stored_tip}\n"
    )


@pytest.mark.parametrize("identity", ["ACTIVATION_BASE", "ACTIVATION_TIP"])
def test_success_rejects_incomplete_activation_before_settlement(
    tmp_path: Path, identity: str
) -> None:
    repo, _origin = _fixture(tmp_path)
    ticket_id = "1053"
    deployed = tmp_path / "deployed"
    hook = repo / "sdlc" / "scripts" / "deploy"
    hook.write_text(
        f"#!/bin/sh\ntouch {shlex.quote(str(deployed))}\n", encoding="utf-8"
    )
    hook.chmod(0o755)
    _commit(
        repo,
        "sdlc/scripts/deploy",
        hook.read_text(encoding="utf-8"),
        "fixture: add deploy hook",
    )
    _git(repo, "push", "--quiet", "origin", "main")
    prepared, tree = _before(repo, tmp_path, ticket_id)
    assert prepared.returncode == 0, prepared.stderr
    assert tree is not None
    candidate = _commit(
        tree,
        "CANDIDATE",
        "an incomplete request must not settle\n",
        "candidate: await activation",
    )
    main = _git(repo, "rev-parse", "origin/main")
    checkout = _git(repo, "rev-parse", "HEAD")
    fetched = tmp_path / "fetched"
    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    git_command = shutil.which("git")
    assert git_command is not None
    (bin_dir / "git").write_text(
        "#!/bin/sh\n"
        f'if [ "$1" = -C ] && [ "$3" = fetch ]; then touch {shlex.quote(str(fetched))}; fi\n'
        f'exec {shlex.quote(git_command)} "$@"\n',
        encoding="utf-8",
    )
    (bin_dir / "git").chmod(0o755)

    refused = _run(
        repo,
        "success",
        {
            "TICKET_ID": ticket_id,
            "ATTEMPT_DIR": str(tree),
            "SDLC_REPO": str(repo),
            "SDLC_BRANCH": f"ticket/{ticket_id}",
            identity: candidate,
            "PATH": f"{bin_dir}:{os.environ['PATH']}",
        },
    )

    assert refused.returncode != 0
    assert (
        "ACTIVATION_BASE and ACTIVATION_TIP must be supplied together" in refused.stderr
    )
    assert not re.search(
        r"^(?:landed|activated|activation-pending|nothing to land)\b",
        refused.stdout,
        re.MULTILINE,
    )
    assert not fetched.exists()
    assert _git(repo, "rev-parse", "origin/main") == main
    assert _git(repo, "rev-parse", "HEAD") == checkout
    assert not deployed.exists()
    assert tree.exists()
    assert _git(tree, "rev-parse", "HEAD") == candidate
    assert _git(repo, "rev-parse", f"ticket/{ticket_id}") == candidate


def test_failure_resolves_the_withdrawal_bot_command_through_path(
    tmp_path: Path,
) -> None:
    repo, _origin = _fixture(tmp_path)
    ticket_id = "1059"
    prepared, tree = _before(repo, tmp_path, ticket_id)
    assert prepared.returncode == 0, prepared.stderr
    assert tree is not None
    bot = _bot(tmp_path)

    settled = _run(
        repo,
        "failure",
        {
            "TICKET_ID": ticket_id,
            "SETTLEMENT": "withdrawn",
            "BOT_CMD": "bot",
            "PATH": f"{bot.parent}:{os.environ['PATH']}",
        },
    )

    assert settled.returncode == 0, settled.stderr
    assert settled.stdout == f"withdrawal-cleaned {ticket_id}\n"
    assert not tree.exists()


def test_success_keeps_pending_activation_authoritative_after_teardown_fails(
    tmp_path: Path,
) -> None:
    repo, _origin = _fixture(tmp_path)
    ticket_id = "1059"
    prepared, tree = _before(repo, tmp_path, ticket_id)
    assert prepared.returncode == 0, prepared.stderr
    assert tree is not None
    base = _git(repo, "rev-parse", "HEAD")
    tip = _commit(
        tree,
        "LANDED",
        "this durable landing still needs checkout activation\n",
        "candidate: await activation after teardown",
    )
    (repo / "LOCAL").write_text("keep this checkout dirty\n", encoding="utf-8")

    settled = _run(
        repo,
        "success",
        {
            "TICKET_ID": ticket_id,
            "ATTEMPT_DIR": str(tree),
            "SDLC_REPO": str(repo),
            "SDLC_BRANCH": f"ticket/{ticket_id}",
            "PATH": _path_with_blocked_worktree_removal(tmp_path, tree),
        }
        | _witness_environment(tmp_path, tip),
    )

    assert settled.returncode == 4
    assert settled.stdout == f"activation-pending {base}..{tip}\n"
    assert f"settlement failed: worktree removal {tree}" in settled.stderr
    assert _git(repo, "rev-parse", "origin/main") == tip
    assert tree.exists()


def test_success_deploys_an_interrupted_landing_after_main_advances(
    tmp_path: Path,
) -> None:
    repo, origin = _fixture(tmp_path)
    deployed = tmp_path / "interrupted-landing-deployed"
    hook = repo / "sdlc" / "scripts" / "deploy"
    hook.write_text(
        "#!/bin/sh\n"
        f"git rev-parse HEAD > {shlex.quote(str(deployed))}\n",
        encoding="utf-8",
    )
    hook.chmod(0o755)
    _git(repo, "add", "sdlc/scripts/deploy")
    _git(repo, "commit", "--quiet", "-m", "fixture: add deploy hook")
    _git(repo, "push", "--quiet", "origin", "main")
    ticket_id = "1059"
    prepared, tree = _before(repo, tmp_path, ticket_id)
    assert prepared.returncode == 0, prepared.stderr
    assert tree is not None
    landed_tip = _commit(
        tree,
        "LANDED",
        "the push completed before settlement stopped\n",
        "candidate: survive interrupted settlement",
    )
    _git(tree, "push", "--quiet", "origin", f"{landed_tip}:main")
    publisher = _clone(origin, tmp_path / "post-interruption-publisher")
    current_tip = _commit(
        publisher,
        "LATER_MAIN",
        "main advanced before settlement retried\n",
        "fixture: advance main after interruption",
    )
    _git(publisher, "push", "--quiet", "origin", "main")

    settled = _run(
        repo,
        "success",
        {
            "TICKET_ID": ticket_id,
            "ATTEMPT_DIR": str(tree),
            "SDLC_REPO": str(repo),
            "SDLC_BRANCH": f"ticket/{ticket_id}",
        },
    )

    assert settled.returncode == 0, settled.stderr
    assert deployed.read_text(encoding="utf-8").strip() == current_tip
    assert _git(repo, "rev-parse", "HEAD") == current_tip
    assert _git(repo, "merge-base", "--is-ancestor", landed_tip, current_tip) == ""
    assert not tree.exists()


def test_failure_keeps_a_ready_fault_candidate_for_the_retry(
    tmp_path: Path,
) -> None:
    repo, _origin = _fixture(tmp_path)
    ticket_id = "1059"
    prepared, tree = _before(repo, tmp_path, ticket_id)
    assert prepared.returncode == 0, prepared.stderr
    assert tree is not None
    sealed_tip = _commit(
        tree,
        "SEALED_CODE",
        "the retry must resume this candidate\n",
        "code: seal candidate output",
    )

    settled = _run(
        repo,
        "failure",
        {
            "TICKET_ID": ticket_id,
            "ATTEMPT_DIR": str(tree),
            "RUN_CAUSE": "fault",
            "SDLC_BRANCH": f"ticket/{ticket_id}",
            "SDLC_REPO": str(repo),
            "SETTLEMENT": "ready",
        },
    )
    retried, replacement = _before(repo, tmp_path, ticket_id)

    assert settled.returncode == 0, settled.stderr
    assert retried.returncode == 0, retried.stderr
    assert replacement == tree
    assert tree.exists()
    assert _git(tree, "rev-parse", "HEAD") == sealed_tip
    assert _git(repo, "rev-parse", f"ticket/{ticket_id}") == sealed_tip
