from __future__ import annotations

import os
from pathlib import Path
import signal
import subprocess
import time

import pytest

ROOT = Path(__file__).resolve().parents[1]
TOOL = ROOT / "tools/prepare-biodata-release-dependency"
HTTPS = "https://github.com/genomoncology/biodata"
SSH = "git@github.com:genomoncology/biodata.git"
CONFIG = f"url.{SSH}.insteadOf={HTTPS}\n"
KEYS = "GIT_CONFIG_GLOBAL GIT_CONFIG_NOSYSTEM GIT_TERMINAL_PROMPT GIT_SSH_COMMAND CARGO_NET_GIT_FETCH_WITH_CLI GIT_CONFIG_COUNT GIT_CONFIG_PARAMETERS GIT_CONFIG"
GIT_FAKE = "#!/usr/bin/env bash\nif [[ $1 == config && $2 == --file ]]; then printf '%s=%s\\n' \"$4\" \"$5\" > \"$3.new\" && mv \"$3.new\" \"$3\"; fi\n"
CARGO_FAKE = ("#!/usr/bin/env bash\n{ printf 'argv=%s\\n' \"$*\"; for k in " + KEYS + "; do [[ -v $k ]] && printf '%s=%s\\n' \"$k\" \"${!k}\"; done; stat -c 'mode=%a' \"$GIT_CONFIG_GLOBAL\"; } > \"$FAKE_LOG/env\"\n"
    'cp "$GIT_CONFIG_GLOBAL" "$FAKE_LOG/config"; if (("${FAKE_CARGO_SLEEP:-0}")); then echo $$ > "$FAKE_LOG/cargopid"; sleep "$FAKE_CARGO_SLEEP" & echo $! > "$FAKE_LOG/descendant"; wait; fi\n'
    'exit "${FAKE_CARGO_STATUS:-0}"\n')


def _gone(path: Path) -> None:
    pid, deadline = int(path.read_text()), time.monotonic() + 5
    while True:
        try:
            os.kill(pid, 0)
        except ProcessLookupError:
            return
        assert time.monotonic() < deadline, f"{path.name} survived signal"
        time.sleep(0.05)


def _run(script, tmp_path, status="0", sig=0, cargo_sleep="0", extra=None):
    for name in ("bin", "log", "tmp"):
        (tmp_path / name).mkdir()
    for name, body in (("git", GIT_FAKE), ("cargo", CARGO_FAKE)):
        (tmp_path / f"bin/{name}").write_text(body, encoding="utf-8")
        (tmp_path / f"bin/{name}").chmod(0o755)
    env = {"PATH": f"{tmp_path}/bin{os.pathsep}{os.environ['PATH']}",
           "FAKE_LOG": f"{tmp_path}/log", "FAKE_CARGO_STATUS": status,
           "FAKE_CARGO_SLEEP": "30" if sig else cargo_sleep,
           "TMPDIR": f"{tmp_path}/tmp", "HOME": f"{tmp_path}/home"} | (extra or {})
    probe = subprocess.run(
        ["env", "--default-signal=HUP,INT,TERM", "true"], capture_output=True)
    assert probe.returncode == 0, f"GNU env --default-signal failed: {probe.stderr!r}"
    command = ["env", "--default-signal=HUP,INT,TERM", str(script)]
    if sig:
        # Replay xdist's ignored dispositions so removing the reset fails here.
        command = ["env", "--ignore-signal=HUP,INT,TERM", *command]
    process = subprocess.Popen(
        command, cwd=tmp_path, env=env, text=True, stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT, start_new_session=True)
    if sig:
        deadline = time.monotonic() + 10
        while not (tmp_path / "log/descendant").exists():
            assert time.monotonic() < deadline, "fake cargo never spawned"
            time.sleep(0.02)
        os.kill(process.pid, sig)
    output = process.communicate(timeout=30)[0]
    env_file, config_file = tmp_path / "log/env", tmp_path / "log/config"
    text = env_file.read_text() if env_file.exists() else ""
    recorded = dict(line.split("=", 1) for line in text.splitlines() if "=" in line)
    config = config_file.read_text() if config_file.exists() else ""
    return (process.returncode, recorded, config,
            sorted(p.name for p in (tmp_path / "tmp").iterdir()), output)


def _clean(record, argv):
    code, env, config, leftovers, _ = record
    return all((
        code == 0, env.get("argv") == argv, config == CONFIG, leftovers == [],
        env.get("GIT_CONFIG_NOSYSTEM") == "1", env.get("GIT_TERMINAL_PROMPT") == "0",
        env.get("GIT_SSH_COMMAND") == "ssh -oBatchMode=yes", env.get("mode") == "600",
        env.get("CARGO_NET_GIT_FETCH_WITH_CLI") == "true",
        env.get("GIT_CONFIG_COUNT") == "0",
        not Path(env.get("GIT_CONFIG_GLOBAL", str(TOOL))).exists()))


def test_owner_rewrites_exactly_and_leaves_no_residue(tmp_path):
    record = _run(TOOL, tmp_path)
    assert _clean(record, f"fetch --manifest-path {ROOT}/Cargo.toml --locked")
    assert Path(record[1]["GIT_CONFIG_GLOBAL"]).parent == tmp_path / "tmp"
    assert not (tmp_path / "home").exists() and not record[4]


@pytest.mark.parametrize(("status", "sig", "code"), [
    ("7", 0, 7), ("0", signal.SIGHUP, 129), ("0", signal.SIGINT, 130), ("0", signal.SIGTERM, 143)])
def test_owner_preserves_status_and_reaps_cargo_group(tmp_path, status, sig, code):
    record = _run(TOOL, tmp_path, status=status, sig=sig)
    assert record[0] == code and record[3] == [] and not Path(
        record[1]["GIT_CONFIG_GLOBAL"]).exists()
    for name in ("cargopid", "descendant") if sig else ():
        _gone(tmp_path / "log" / name)


def test_owner_forwards_pending_signal_from_startup_window(tmp_path):
    bash_env = tmp_path / "bash-env"
    bash_env.write_text(
        "trap '[[ $BASH_COMMAND == \"cargo_pid=\\$!\" && -z $cargo_pid ]] && "
        'trap -p TERM | grep -q "forward TERM 143" && printf hit > "$FAKE_LOG/window"'
        " && sleep 0.5 && kill -TERM $$; true' DEBUG\n")
    record = _run(TOOL, tmp_path, cargo_sleep="30", extra={"BASH_ENV": str(bash_env)})
    assert record[0] == 143 and record[3] == []
    assert (tmp_path / "log/window").is_file() and not Path(record[1]["GIT_CONFIG_GLOBAL"]).exists()
    for name in ("cargopid", "descendant"):
        _gone(tmp_path / "log" / name)


def test_owner_neutralizes_inherited_git_config_injection(tmp_path):
    hostile = {k: "sentinel" for k in (
        "GIT_CONFIG_PARAMETERS GIT_CONFIG GIT_CONFIG_KEY_0 GIT_CONFIG_VALUE_0 "
        "GIT_CONFIG_GLOBAL GIT_CONFIG_NOSYSTEM GIT_TERMINAL_PROMPT "
        "GIT_SSH_COMMAND").split()} | {"GIT_CONFIG_COUNT": "1"}
    record = _run(TOOL, tmp_path, extra=hostile)
    assert _clean(record, f"fetch --manifest-path {ROOT}/Cargo.toml --locked")
    assert not {"GIT_CONFIG_PARAMETERS", "GIT_CONFIG"} & record[1].keys()


def test_owner_is_credential_free_and_excluded_from_the_package() -> None:
    source = TOOL.read_text(encoding="utf-8")
    manifest = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    assert source.count("insteadOf") == 1
    for required in ("umask 077", f'url."{SSH}".insteadOf', "trap cleanup EXIT",
                     "forward HUP 129", "forward INT 130", "forward TERM 143",
                     "GIT_CONFIG_COUNT=0", "setsid cargo fetch",
                     "unset GIT_CONFIG_PARAMETERS GIT_CONFIG"):
        assert required in source
    forbidden = ("credential", "token", "gh auth", "--global", ".ssh",
                 "GIT_ASKPASS", "SSH_ASKPASS", 'url."https://github.com/"')
    assert all(item not in source for item in forbidden)
    assert '"tools/prepare-biodata-release-dependency"' in manifest
    assert '"tests/test_prepare_biodata_release_dependency.py"' in manifest
    assert "prepare-biodata" not in (ROOT / "Makefile").read_text()


@pytest.mark.parametrize(("old", "new"), [
    (HTTPS, HTTPS[:-1] + "x"), (SSH, SSH[:-1] + "x"), ("--locked", "--Locked"),
    ("BatchMode=yes", "BatchMode=Yes"), ("trap cleanup EXIT", "trap cleanup EXIx")])
def test_one_byte_mutations_break_the_contract(tmp_path, old, new):
    source = TOOL.read_text(encoding="utf-8")
    assert source.count(old) == 1
    mutated = tmp_path / "tools" / "prepare"
    mutated.parent.mkdir()
    mutated.write_text(source.replace(old, new, 1), encoding="utf-8")
    mutated.chmod(0o755)
    assert not _clean(_run(mutated, tmp_path),
                      f"fetch --manifest-path {tmp_path}/Cargo.toml --locked")
