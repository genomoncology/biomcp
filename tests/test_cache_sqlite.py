"""The CLI shares an indexed cache without removing saved article files."""
from __future__ import annotations

import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys

import pytest

sys.path.insert(0, str(Path(__file__).parent / "golden"))
from replay_env import retained_cache_env  # noqa: E402

import test_cache_persistent_size  # noqa: E402

gene_provider = test_cache_persistent_size.gene_provider

pytestmark = [pytest.mark.needs_binary]
ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("BIOMCP_BIN", ROOT / "target/debug/biomcp"))


def invoke(env, *args):
    result = subprocess.run([str(BINARY), "--json", *args], env=env, cwd=ROOT,
                            capture_output=True, text=True, timeout=20)
    assert result.returncode == 0, (result.stdout, result.stderr)
    return json.loads(result.stdout)


def test_cache_path_names_database_without_creating_it(tmp_path):
    env = retained_cache_env(tmp_path, {}, "http://127.0.0.1:9")
    result = invoke(env, "cache", "path")
    assert result["path"] == str(tmp_path / "cache/biomcp.db")
    assert not (tmp_path / "cache").exists()


def test_sqlite_coexists_with_legacy_cache_and_preserves_saved_downloads(tmp_path, gene_provider):
    cache = tmp_path / "cache"
    for name in ("http", "captures", "citation-evidence", "downloads"):
        directory = cache / name
        directory.mkdir(parents=True)
        (directory / "keep.txt").write_text("saved bytes")
    env = retained_cache_env(tmp_path, {"BIOMCP_MYGENE_BASE": gene_provider}, gene_provider, cache_dir=cache)
    first = invoke(env, "search", "gene", "BRAF", "--limit", "1")
    assert invoke(env, "search", "gene", "BRAF", "--limit", "1") == first
    assert (cache / "biomcp.db").is_file()
    with sqlite3.connect(cache / "biomcp.db") as connection:
        assert connection.execute("pragma journal_mode").fetchone()[0] == "wal"
    invoke(env, "cache", "clean", "--max-size", "1B")
    assert invoke(env,"search","gene","BRAF","--limit","1")["results"]
    invoke(env, "cache", "clear", "--yes")
    with sqlite3.connect(cache / "biomcp.db") as connection:
        assert connection.execute("select count(*) from entries").fetchone()[0]==0
    for name in ("http", "captures", "citation-evidence", "downloads"):
        assert (cache / name / "keep.txt").read_text() == "saved bytes"


def test_killed_uncommitted_writer_preserves_cached_answer(tmp_path, gene_provider):
    env = retained_cache_env(tmp_path, {"BIOMCP_MYGENE_BASE": gene_provider}, gene_provider)
    first = invoke(env, "search", "gene", "BRAF", "--limit", "1")
    database = tmp_path / "cache/biomcp.db"
    script = "import sqlite3,sys; c=sqlite3.connect(sys.argv[1]); c.execute('begin immediate'); c.execute('delete from entries'); print('ready',flush=True); sys.stdin.read()"
    writer = subprocess.Popen([sys.executable, "-c", script, str(database)], stdin=subprocess.PIPE,
                              stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        assert writer.stdout.readline().strip() == "ready"
        writer.kill()
        writer.wait(timeout=5)
        assert invoke(env, "search", "gene", "BRAF", "--limit", "1") == first
        with sqlite3.connect(database) as connection:
            assert connection.execute("pragma integrity_check").fetchone() == ("ok",)
            assert connection.execute("select bytes from cache_meta").fetchone()[0] > 0
    finally:
        if writer.poll() is None:
            writer.kill()
            writer.wait(timeout=5)


def test_small_reply_lookup_uses_index_among_one_hundred_thousand_entries(tmp_path, gene_provider):
    env = retained_cache_env(tmp_path, {"BIOMCP_MYGENE_BASE": gene_provider}, gene_provider,
                             max_size=1000000000)
    first = invoke(env, "search", "gene", "BRAF", "--limit", "1")
    cache = tmp_path / "cache"
    with sqlite3.connect(cache / "biomcp.db") as connection:
        metadata, created = connection.execute(
            "select metadata,created from entries where namespace='http' limit 1").fetchone()
        connection.executemany(
            "insert into entries(namespace,key,base_key,metadata,body,bytes,created,last_access) values('http',?,?,?,?,?,?,?)",
            ((f"other-{index}", f"other-{index}", metadata, b"{}", len(metadata)+2, created, created) for index in range(99999)),
        )
        plans = connection.execute("explain query plan select key,metadata,expires from entries where namespace=? and base_key=?",
                                   ("http", "irrelevant")).fetchall()
        assert any("entries_lookup" in row[-1] for row in plans), plans
    assert invoke(env, "search", "gene", "BRAF", "--limit", "1") == first
    assert not (cache / "http").exists()
    assert not (cache / "blobs").exists(), "small cached replies remain inside the database"


def test_explicit_legacy_cleanup_preserves_saved_files_and_current_cache(tmp_path, gene_provider):
    env = retained_cache_env(tmp_path, {"BIOMCP_MYGENE_BASE": gene_provider}, gene_provider)
    invoke(env, "search", "gene", "BRAF", "--limit", "1")
    cache = tmp_path / "cache"
    for name in ("http", "http-cacache", "captures", "citation-evidence", "downloads", "sessions"):
        (cache / name).mkdir()
        (cache / name / "keep.txt").write_text("saved bytes")
    stats = invoke(env, "cache", "stats")
    assert stats["legacy_cache_bytes"]["http"] == len("saved bytes")
    invoke(env, "cache", "clean-legacy", "--yes")
    for name in ("http", "http-cacache", "captures", "citation-evidence"):
        assert not (cache / name).exists()
    for name in ("downloads", "sessions"):
        assert (cache / name / "keep.txt").read_text() == "saved bytes"
    assert (cache / "biomcp.db").is_file()


def test_initialized_mcp_and_cli_share_cache_during_concurrent_requests(tmp_path, gene_provider, monkeypatch):
    from concurrent.futures import ThreadPoolExecutor
    from test_mcp_2026_protocol import RawStdioMcp

    env = retained_cache_env(tmp_path, {"BIOMCP_MYGENE_BASE": gene_provider}, gene_provider)
    for name in set(os.environ) - set(env):
        monkeypatch.delenv(name)
    for name, value in env.items():
        monkeypatch.setenv(name, value)
    client = RawStdioMcp()
    try:
        initialized = client.call("initialize", "initialize", {
            "protocolVersion": "2025-06-18", "capabilities": {},
            "clientInfo": {"name": "cache-sharing", "version": "1"},
        })
        assert "result" in initialized, initialized
        client.send({"jsonrpc": "2.0", "method": "notifications/initialized"})
        with ThreadPoolExecutor(max_workers=1) as workers:
            cli = workers.submit(invoke, env, "search", "gene", "BRAF-cli", "--limit", "1")
            mcp = client.call("gene", "tools/call", {
                "name": "biomcp", "arguments": {"command": "--json search gene BRAF-mcp --limit 1"},
            })["result"]
            assert not mcp.get("isError"), mcp
            assert cli.result()["results"]
        assert invoke(env, "search", "gene", "BRAF-mcp", "--limit", "1")["results"]
        with sqlite3.connect(tmp_path / "cache/biomcp.db") as connection:
            assert connection.execute("pragma integrity_check").fetchone() == ("ok",)
            assert connection.execute("select count(*) from entries").fetchone()[0] == 2
    finally:
        client.close()
