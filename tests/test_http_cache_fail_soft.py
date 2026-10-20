"""Local cache failures preserve recorded provider answers through the CLI."""
from __future__ import annotations

from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
import threading
from urllib.error import HTTPError
from urllib.parse import parse_qsl, urlsplit
from urllib.request import urlopen

import pytest

sys.path.insert(0, str(Path(__file__).parent / "golden"))
from replay_env import REPO_ROOT, replay_environment, retained_cache_env  # noqa: E402
from cache_reuse import admitted_provider_keys, assert_admitted_reused  # noqa: E402

pytestmark = [pytest.mark.needs_binary]
BINARY = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target/debug/biomcp"))
ARTICLE = ("search", "article", "--gene", "BRAF", "--disease", "melanoma", "--source", "europepmc", "--limit", "1")
GENE = ("search", "gene", "BRAF", "--limit", "1")


def recorded_path(path):
    return path.replace("query=BRAF+AND+melanoma", "query=GENE_PROTEIN%3ABRAF+AND+melanoma")


@pytest.fixture
def provider(tmp_path: Path):
    with replay_environment(record=False) as replay:
        class Handler(BaseHTTPRequestHandler):
            def do_GET(self):
                # Reuse the recorded article answer: its search-all route anchors
                # the same gene, while this CLI case only exercises cache behavior.
                path = recorded_path(self.path)
                try:
                    reply = urlopen(replay.origin + path, timeout=10)
                except HTTPError as error:
                    reply = error
                with reply:
                    body = reply.read()
                    status = reply.status
                if status == 200 and path.startswith("/europepmc/"):
                    payload = json.loads(body)
                    payload["hitCount"] = len(payload["resultList"]["result"])
                    payload.pop("nextCursorMark", None)
                    payload.pop("nextPageUrl", None)
                    body = json.dumps(payload).encode()
                before_reply[0]()
                self.send_response(status)
                self.send_header("Content-Type", "application/json")
                self.send_header("Cache-Control", "public, max-age=3600" if status == 200 else "no-store")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)

            def log_message(self, *_args):
                pass

        before_reply = [lambda: None]
        server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        run_dir = Path(shutil.move(replay._next_run_dir(), tmp_path / "run"))
        env = retained_cache_env(run_dir, replay.provider_env, replay.origin,
                                 cache_dir=run_dir / "cache")
        origin = f"http://127.0.0.1:{server.server_port}"
        for name, value in replay.provider_env.items():
            env[name] = value.replace(replay.origin, origin)
        env.update(BIOMCP_TEST_UNPACED_ORIGIN=origin, RUST_LOG="warn",
                   BIOMCP_TEST_ARTICLE_SEARCH_DEADLINE_MS="3000")
        cache = Path(env["BIOMCP_CACHE_DIR"])

        def run(argv, *, log_level="warn"):
            command_env = dict(env, RUST_LOG=log_level)
            return subprocess.run([str(BINARY), "--json", *argv], cwd=REPO_ROOT, env=command_env,
                                  capture_output=True, text=True, timeout=20)

        try:
            yield run, cache, before_reply, replay
        finally:
            server.shutdown()
            server.server_close()
            thread.join()
            assert not replay.misses(), replay.misses()


def assert_answer(result, cause):
    assert result.returncode == 0, (result.stdout, result.stderr)
    assert json.loads(result.stdout)["results"]
    warnings = [line for line in result.stderr.splitlines() if "local HTTP cache" in line]
    assert len(warnings) == 1, result.stderr
    assert cause in warnings[0], warnings
    assert "WARN" in warnings[0], warnings
    assert "http://" not in warnings[0]
    assert "https://" not in warnings[0]
    assert "io_kind" in warnings[0] and "os_code" in warnings[0]


@pytest.mark.parametrize("command", [ARTICLE, GENE])
def test_invalid_cache_tree_uses_provider_without_trusting_tree(provider, tmp_path, command):
    run, cache, _, _ = provider
    outside = tmp_path / "outside"
    outside.mkdir()
    sentinel = outside / "keep"
    sentinel.write_bytes(b"unchanged")
    cache.rmdir()
    cache.symlink_to(outside, target_is_directory=True)
    assert_answer(run(command), "initialization")
    assert sentinel.read_bytes() == b"unchanged"
    assert list(outside.iterdir()) == [sentinel]



@pytest.mark.parametrize("command", [ARTICLE, ("search", "all", "--gene", "BRAF", "--disease", "melanoma")])
def test_parallel_article_sources_fill_cold_cache_then_reuse_answers(provider, command):
    run, cache, _, replay = provider
    first = run(command)
    assert first.returncode == 0, first.stderr
    first_payload = json.loads(first.stdout)
    assert first_payload.get("results") or any(section["results"] for section in first_payload["sections"])
    assert (cache / "biomcp.db").is_file(), "cold search must fill SQLite cache"
    # This call can still fan out internally and skip best-effort writes.
    admitted = run(command)
    assert admitted.returncode == 0, admitted.stderr
    admitted_payload = json.loads(admitted.stdout)
    assert admitted_payload.get("results", admitted_payload.get("sections")) == first_payload.get("results", first_payload.get("sections"))
    if command[1] == "all":
        # The standalone gene request has the same size=50 provider identity as
        # search-all, with no competing provider writes in this private cache.
        seeded = run(GENE)
        assert seeded.returncode == 0, seeded.stderr
        gene_section = next(section for section in first_payload["sections"] if section["entity"] == "gene")
        assert json.loads(seeded.stdout)["results"] == gene_section["results"]
        assert "local HTTP cache" not in seeded.stderr, seeded.stderr
    cached_keys = admitted_provider_keys(
        cache, normalize_path=recorded_path, prefixes=("/europepmc/", "/mygene/"),
    )
    before = replay.request_log_line_count()
    second = run(command)
    assert second.returncode == 0, second.stderr
    second_payload = json.loads(second.stdout)
    assert second_payload.get("results", second_payload.get("sections")) == first_payload.get("results", first_payload.get("sections"))
    assert "local HTTP cache" not in first.stderr + admitted.stderr + second.stderr, (first.stderr, admitted.stderr, second.stderr)
    contacted_keys = {
        entry["key"] for entry in replay.request_log_entries()[:before]
        if entry["status"] == 200 and entry["key"].startswith(("GET /europepmc/", "GET /mygene/"))
    }
    # EuropeOnly awaits its primary write before enrichment. Search-all uses
    # the uncontended gene seed instead; its exact original request must exist.
    seed_keys = {key for key in contacted_keys if key.startswith("GET /mygene/")} if command[1] == "all" else {
        key for key in contacted_keys if ("pageSize", "25") in parse_qsl(urlsplit(key.removeprefix("GET ")).query)
    }
    assert len(seed_keys) == 1 and seed_keys <= cached_keys, "single-source admission must preserve the exact provider identity"
    if command[1] == "all":
        assert ("size", "50") in parse_qsl(urlsplit(next(iter(seed_keys)).removeprefix("GET ")).query)
    repeated = {entry["key"] for entry in replay.request_log_entries(since=before)}
    assert_admitted_reused(cached_keys, contacted_keys, repeated)


def test_admitted_reuse_allows_contended_refetch_and_rejects_cache_bypass(provider):
    run, cache, _, replay = provider
    seeded = run(GENE)
    assert seeded.returncode == 0, seeded.stderr
    gene_results = json.loads(seeded.stdout)["results"]
    with sqlite3.connect(cache / "biomcp.db") as held:
        held.execute("begin immediate")
        skipped = run(ARTICLE)
        assert skipped.returncode == 0, skipped.stderr
        article_results = json.loads(skipped.stdout)["results"]
        assert article_results
    admitted = admitted_provider_keys(cache, normalize_path=recorded_path)
    assert admitted and all(key.startswith("GET /mygene/") for key in admitted), admitted
    contacted = {entry["key"] for entry in replay.request_log_entries() if entry["status"] == 200}
    assert any(key.startswith("GET /europepmc/") for key in contacted), contacted
    before = replay.request_log_line_count()
    hit = run(GENE)
    refetch = run(ARTICLE)
    assert hit.returncode == refetch.returncode == 0, (hit.stderr, refetch.stderr)
    assert json.loads(hit.stdout)["results"] == gene_results
    assert json.loads(refetch.stdout)["results"] == article_results
    repeated = {entry["key"] for entry in replay.request_log_entries(since=before)}
    assert_admitted_reused(admitted, contacted, repeated)
    # The former all-contacted predicate rejects the permitted skipped write.
    with pytest.raises(AssertionError, match="every contacted answer"):
        assert not contacted & repeated, "every contacted answer must be reused"
    before = replay.request_log_line_count()
    bypass = run(("--no-cache", *GENE))
    assert bypass.returncode == 0, bypass.stderr
    assert json.loads(bypass.stdout)["results"] == gene_results
    with pytest.raises(AssertionError, match="identical search must reuse"):
        assert_admitted_reused(
            admitted, contacted, (entry["key"] for entry in replay.request_log_entries(since=before)),
        )


def test_undecodable_cache_entry_refetches_then_keeps_cache_hits(provider):
    run, cache, _, replay = provider
    first = run(GENE)
    assert first.returncode == 0, first.stderr
    with sqlite3.connect(cache / "biomcp.db") as connection:
        assert connection.execute("select count(*) from entries where namespace='http'").fetchone()[0] == 1
        connection.execute("update entries set metadata=? where namespace='http'", (b"invalid",))
    before = replay.request_log_line_count()
    assert_answer(run(GENE), "read")
    assert replay.request_log_line_count() == before + 1
    hit = run(GENE)
    assert hit.returncode == 0, hit.stderr
    assert json.loads(hit.stdout)["results"] == json.loads(first.stdout)["results"]
    assert replay.request_log_line_count() == before + 1


def test_failed_cache_write_keeps_the_provider_answer(provider):
    run, cache, before_reply, _ = provider

    def obstruct_write():
        with sqlite3.connect(cache / "biomcp.db") as connection:
            connection.execute("create trigger obstruct_cache before insert on entries begin select raise(abort,'fixture write failure'); end")

    before_reply[0] = obstruct_write
    result = run(GENE)
    assert_answer(result, "write")
    assert "WouldBlock" not in result.stderr, result.stderr


@pytest.mark.skipif(os.name != "posix", reason="uses a POSIX lock holder")
@pytest.mark.parametrize("log_level", ["warn", "debug"])
def test_contended_cache_write_keeps_answer_and_logs_only_at_debug(provider, log_level):
    run, cache, before_reply, replay = provider
    held = None

    def hold_byte_total_lock():
        nonlocal held
        held = sqlite3.connect(cache / "biomcp.db", check_same_thread=False)
        held.execute("begin immediate")

    before_reply[0] = hold_byte_total_lock
    try:
        result = run(GENE, log_level=log_level)
        assert result.returncode == 0, (result.stdout, result.stderr)
        expected = json.loads(result.stdout)["results"]
        assert expected
        messages = [line for line in result.stderr.splitlines() if "local HTTP cache" in line]
        assert not any("WARN" in line for line in messages), result.stderr
        if log_level == "debug":
            assert len(messages) == 1, result.stderr
            assert "DEBUG" in messages[0] and "WouldBlock" in messages[0], messages
            assert 'operation="write"' in messages[0], messages
            assert str(cache) not in messages[0] and "http://" not in messages[0], messages
        else:
            assert not messages, result.stderr
        with sqlite3.connect(cache / "biomcp.db") as connection:
            assert connection.execute("select count(*) from entries").fetchone()[0] == 0

    finally:
        before_reply[0] = lambda: None
        if held is not None:
            held.close()
    before = replay.request_log_line_count()
    refill = run(GENE)
    assert refill.returncode == 0, refill.stderr
    assert json.loads(refill.stdout)["results"] == expected
    assert replay.request_log_line_count() == before + 1, "skipped write must refetch"
    hit = run(GENE)
    assert hit.returncode == 0, hit.stderr
    assert json.loads(hit.stdout)["results"] == expected
    assert replay.request_log_line_count() == before + 1, "settled refill must be reused"



def test_no_cache_ignores_an_invalid_cache_tree_without_warning(provider):
    run, cache, _, _ = provider
    (cache / "http").write_bytes(b"not a directory")
    result = run(("--no-cache", *GENE))
    assert result.returncode == 0, result.stderr
    assert json.loads(result.stdout)["results"]
    assert "local HTTP cache" not in result.stderr
    assert (cache / "http").read_bytes() == b"not a directory"


@pytest.mark.parametrize("command", [GENE, ARTICLE])
def test_cache_database_contention_leaves_time_for_provider(provider, command):
    run, cache, _, replay = provider
    seeded = run(command)
    assert seeded.returncode == 0, seeded.stderr
    with sqlite3.connect(cache / "biomcp.db") as held:
        held.execute("begin immediate")
        before = replay.request_log_line_count()
        result = run(command)
        assert result.returncode == 0, result.stderr
        assert json.loads(result.stdout)["results"]
        assert "WARN" not in result.stderr, result.stderr
        assert replay.request_log_line_count() > before


def test_unreadable_cache_configuration_uses_provider(provider):
    run, cache, _, _ = provider
    config_home = "Library/Application Support" if sys.platform == "darwin" else "xdg-config"
    config = cache.parent / config_home / "biomcp/cache.toml"
    config.mkdir(parents=True)
    assert_answer(run(GENE), "initialization")


@pytest.mark.skipif(os.name != "posix", reason="uses a POSIX lock holder")
@pytest.mark.parametrize("command", [GENE, ARTICLE])
def test_parallel_current_cache_starts_use_cache_while_epoch_lock_is_held(provider, command):
    import fcntl
    from concurrent.futures import ThreadPoolExecutor

    run, cache, _, replay = provider
    seeded = run(command)
    assert seeded.returncode == 0, seeded.stderr
    assert "local HTTP cache" not in seeded.stderr, seeded.stderr
    expected = json.loads(seeded.stdout)["results"]
    cached_keys = admitted_provider_keys(
        cache, normalize_path=recorded_path, prefixes=("/europepmc/", "/mygene/"),
    )
    contacted = {entry["key"] for entry in replay.request_log_entries() if entry["status"] == 200}
    primary = {key for key in contacted if key.startswith("GET /mygene/")} if command == GENE else {
        key for key in contacted if ("pageSize", "25") in parse_qsl(urlsplit(key.removeprefix("GET ")).query)
    }
    assert len(primary) == 1 and primary <= cached_keys, "the primary response must be admitted before parallel reads"
    before = replay.request_log_line_count()
    # A process holding the epoch lock must not prevent readers of a current
    # epoch. Holding it across every start makes the old race deterministic.
    with (cache / ".body-limit-cache-v1.lock").open("a+b") as held:
        fcntl.flock(held, fcntl.LOCK_EX)
        with ThreadPoolExecutor(max_workers=12) as workers:
            results = list(workers.map(lambda _: run(command), range(12)))
    for result in results:
        assert result.returncode == 0, result.stderr
        assert json.loads(result.stdout)["results"] == expected
        assert "local HTTP cache" not in result.stderr, result.stderr
    assert_admitted_reused(
        cached_keys, contacted, (entry["key"] for entry in replay.request_log_entries(since=before)),
    )


@pytest.mark.parametrize("fault", ["integrity", "missing"])
def test_unreadable_cached_blob_refetches_with_a_safe_warning(provider, fault):
    run, cache, _, replay = provider
    first = run(GENE)
    assert first.returncode == 0, first.stderr
    digest = "0" * 64
    with sqlite3.connect(cache / "biomcp.db") as connection:
        connection.execute("update entries set body=null,blob=? where namespace='http'", (digest,))
    if fault == "integrity":
        (cache / "blobs").mkdir(exist_ok=True)
        (cache / "blobs" / digest).write_bytes(b"corrupt cache content")
    before = replay.request_log_line_count()
    result = run(GENE)
    assert_answer(result, "read")
    assert str(cache) not in result.stderr
    assert replay.request_log_line_count() == before + 1
