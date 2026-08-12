# MCP Surface

BioMCP exposes the same biomedical command surface through stdio MCP and
Streamable HTTP. These canaries keep the transport entrypoints, probe routes,
and remote tool execution honest without re-encoding the whole MCP test suite.

## Stdio Entry Points Stay Guided

`mcp` and `serve` are both documented stdio entrypoints. The user-visible
contract here is that one remains the canonical stdio command and the other
stays the Claude Desktop-friendly alias.

```bash
../../tools/biomcp-ci mcp --help | mustmatch like 'Run MCP server over stdio
Usage: biomcp mcp'
../../tools/biomcp-ci serve --help | mustmatch like 'Alias for `mcp`
Usage: biomcp serve'
```

## Manual Stdio Startup Points Operators to HTTP

When an operator launches a stdio entrypoint without an MCP client, BioMCP
should fail closed but still explain the recovery path. Both spellings should
print the same stderr guidance and keep stdout free for MCP protocol traffic.

```bash
biomcp_bin="${BIOMCP_BIN:-../../target/release/biomcp}"
for cmd in mcp serve; do
  stdout_file="$(mktemp)"
  stderr_file="$(mktemp)"
  set +e
  "$biomcp_bin" "$cmd" </dev/null >"$stdout_file" 2>"$stderr_file"
  status=$?
  set -e
  test "$status" -ne 0
  test ! -s "$stdout_file"
  cat "$stderr_file" | mustmatch like 'expects an MCP client on stdin
biomcp serve-http'
  cat "$stderr_file" | mustmatch not like 'connection closed
initialized request'
done
```

## MCP Client Config Generator Prints Local Stdio Snippets

Client setup should not depend on hand-written JSON blocks drifting away from the
installed BioMCP binary. The generator prints local stdio snippets that point MCP
clients at `biomcp serve`; remote HTTP deployment remains a separate server mode.

```bash
biomcp mcp-config --client claude-desktop | mustmatch like '{"mcpServers":{"biomcp":{"command":"biomcp","args":["serve"]}}}'
```

When a user has not picked a client yet, the command should be a discovery page
rather than a dead end. It names the supported clients and shows the copyable
form for a concrete client.

```bash
biomcp mcp-config | mustmatch like "Supported MCP clients:
codex
claude-desktop
biomcp mcp-config --client claude-desktop"
```

## MCP Guidance Uses the Skill Catalog Instead of Retired Suggest

MCP clients see BioMCP instructions and a raw command escape hatch before they
read the CLI docs. That surface should point agents at the living skill catalog
and should not continue to allow or recommend the retired offline `suggest`
router.

```bash
cd ../.. && uv run --no-sync python3 -c '
from pathlib import Path
text = Path("src/mcp/shell.rs").read_text(encoding="utf-8")
assert "biomcp skill list" in text
assert "biomcp suggest" not in text
assert "discover/suggest/skill" not in text
assert "| \"suggest\" => true" not in text
print("MCP guidance points to skill catalog")
' | mustmatch like "MCP guidance points to skill catalog"
```

## Streamable HTTP Help Names the Canonical Route

The remote/server deployment mode should keep pointing operators at `/mcp` and
the lightweight probe routes rather than drifting back toward legacy SSE copy.

```bash
../../tools/biomcp-ci serve-http --help | mustmatch like 'Streamable HTTP server at /mcp
GET /health, GET /readyz, GET /.
--host <HOST>
--allowed-hosts <ALLOWED_HOSTS>'
```

## Typed Tool Schemas Are Advertised

Agents should be able to choose typed MCP tools instead of composing one large
shell command string. The tool surface keeps `biomcp` as an escape hatch, but
also advertises typed `search` and `get` tools whose schemas expose entity,
section, and limit constraints.

```bash
port="$(../../spec/fixtures/reserve-local-port)"
../../tools/biomcp-ci serve-http --host 127.0.0.1 --port "$port" >/tmp/biomcp-mcp-typed-tools.log 2>&1 &
pid=$!
trap 'kill "$pid" 2>/dev/null || true' EXIT
for _ in $(seq 1 40); do
  if curl -fsS "http://127.0.0.1:$port/readyz" >/dev/null || curl -fsS "http://127.0.0.1:$port/health" >/dev/null; then
    break
  fi
  sleep 0.25
done
curl -fsS "http://127.0.0.1:$port/readyz" >/dev/null || curl -fsS "http://127.0.0.1:$port/health" >/dev/null
"${BIOMCP_SPEC_MCP_EXAMPLE_BIN:?spec preparation did not export MCP example}" typed-tools "$port" | mustmatch like 'MCP typed tools: biomcp, search, get, variant_articles
ClinGen typed tools: variant_normalize_car, variant_erepo, gene_cspec, variant_articles
ClinGen schemas validate their named root properties
all listed MCP tools are read-only annotated
all listed MCP tools have titles and descriptions
search schema includes entity enum and bounded limit
get schema includes entity and sections enum
search and get schemas include author entity
variant_articles schema includes identity verification controls
indexing'
```

## Probe Routes Stay Lightweight

The HTTP surface is intentionally tiny: two readiness probes and one root
descriptor that advertises the streamable transport and canonical MCP path.

```bash
port="$(../../spec/fixtures/reserve-local-port)"
../../tools/biomcp-ci serve-http --host 127.0.0.1 --port "$port" >/tmp/biomcp-mcp-routes.log 2>&1 &
pid=$!
trap 'kill "$pid" 2>/dev/null || true' EXIT
for _ in $(seq 1 40); do
  if curl -fsS "http://127.0.0.1:$port/readyz" >/dev/null || curl -fsS "http://127.0.0.1:$port/health" >/dev/null; then
    break
  fi
  sleep 0.25
done
curl -fsS "http://127.0.0.1:$port/readyz" >/dev/null || curl -fsS "http://127.0.0.1:$port/health" >/dev/null
curl -fsS "http://127.0.0.1:$port/health" | mustmatch like '"status":"ok"'
curl -fsS "http://127.0.0.1:$port/readyz" | mustmatch like '"status":"ok"'
curl -fsS "http://127.0.0.1:$port/" | mustmatch like '"transport":"streamable-http"
"mcp":"/mcp"'
```

## Streamable HTTP Host Headers Default To A Safe Boundary

Loopback `serve-http` accepts local Host values and rejects unrelated values.
Non-loopback binds require a precise allowlist or the explicit unsafe escape
hatch.

```bash
port="$(../../spec/fixtures/reserve-local-port)"
body=/tmp/biomcp-mcp-host-default.body
../../tools/biomcp-ci serve-http --host 127.0.0.1 --port "$port" >/tmp/biomcp-mcp-host-default.log 2>&1 &
pid=$!
trap 'kill "$pid" 2>/dev/null || true' EXIT
for _ in $(seq 1 40); do
  if curl -fsS "http://127.0.0.1:$port/readyz" >/dev/null || curl -fsS "http://127.0.0.1:$port/health" >/dev/null; then
    break
  fi
  sleep 0.25
done
curl -fsS "http://127.0.0.1:$port/readyz" >/dev/null || curl -fsS "http://127.0.0.1:$port/health" >/dev/null
for host in localhost "localhost:$port" 127.0.0.1 "127.0.0.1:$port" '[::1]' "[::1]:$port"; do
  status=$(curl -sS -o "$body" -w '%{http_code}' -X POST -H "Host: $host" "http://127.0.0.1:$port/mcp")
  test "$status" != 403
done
status=$(curl -sS -o "$body" -w '%{http_code}' -X POST -H 'Host: attacker.example' "http://127.0.0.1:$port/mcp")
test "$status" = 403
cat "$body" | mustmatch like 'Host header is not allowed'
kill "$pid" 2>/dev/null || true
wait "$pid" 2>/dev/null || true
trap - EXIT

port="$(../../spec/fixtures/reserve-local-port)"
body=/tmp/biomcp-mcp-host-restricted.body
../../tools/biomcp-ci serve-http --host 127.0.0.1 --port "$port" --allowed-hosts example.com >/tmp/biomcp-mcp-host-restricted.log 2>&1 &
pid=$!
trap 'kill "$pid" 2>/dev/null || true' EXIT
for _ in $(seq 1 40); do
  if curl -fsS "http://127.0.0.1:$port/readyz" >/dev/null || curl -fsS "http://127.0.0.1:$port/health" >/dev/null; then
    break
  fi
  sleep 0.25
done
curl -fsS "http://127.0.0.1:$port/readyz" >/dev/null || curl -fsS "http://127.0.0.1:$port/health" >/dev/null
status=$(curl -sS -o "$body" -w '%{http_code}' -X POST -H 'Host: evil.com' "http://127.0.0.1:$port/mcp")
test "$status" = 403
cat "$body" | mustmatch like 'Host header is not allowed'
status=$(curl -sS -o "$body" -w '%{http_code}' -X POST -H 'Host: example.com' "http://127.0.0.1:$port/mcp")
test "$status" != 403
cat "$body" | mustmatch not like 'Host header is not allowed'

set +e
non_loopback_error=$(../../tools/biomcp-ci serve-http --host 0.0.0.0 --port 0 2>&1)
non_loopback_status=$?
set -e
test "$non_loopback_status" -ne 0
printf '%s' "$non_loopback_error" | mustmatch like '--allowed-hosts
--unsafe-allow-any-host'

../../tools/biomcp-ci serve-http --host 127.0.0.1 --port 0 --unsafe-allow-any-host >/tmp/biomcp-mcp-host-unsafe.log 2>&1 &
unsafe_pid=$!
sleep 0.25
kill "$unsafe_pid" 2>/dev/null || true
wait "$unsafe_pid" 2>/dev/null || true
cat /tmp/biomcp-mcp-host-unsafe.log | mustmatch like 'Host header checks are disabled
does not provide authentication or encryption'
```

## MCP Responses Surface Provenance Metadata

Default MCP tool text should carry upstream provenance without requiring the caller to know `--json`, while structured callers can opt in with the tool input `json: true`.

```bash
python3 - <<'PY' | mustmatch like 'MCP provenance metadata contract is wired'
from pathlib import Path

repo = Path('../..')
shell = (repo / 'src/mcp/shell.rs').read_text()
build = (repo / 'build.rs').read_text()
tests = (repo / 'tests/rmcp_client_contract.rs').read_text()
contract = (repo / 'crates/biomcp-mcp-contract-client/src/lib.rs').read_text()

assert 'json: bool' in shell
assert 'args_with_json' in shell
assert 'append_default_mcp_footer' in shell
assert 'mcp_meta_footer_from_json' in shell
assert '## Sources' in shell
assert '## Next commands' in shell
assert 'MCP RESPONSE METADATA:' in build
assert 'json: true' in build
assert '_meta.section_sources' in build
assert 'assert_mcp_provenance_calls' in tests
assert 'assert_mcp_provenance_calls' in contract
assert 'biomcp discover BRCA1 --json' in contract
assert 'call_biomcp_json' in contract
assert 'Structured Concepts' in contract
assert 'OLS4' in contract
print('MCP provenance metadata contract is wired')
PY
```

## Remote Workflow Calls Keep BioMCP Text

The remote tool should execute normal BioMCP workflows, not collapse them into
an MCP-specific summary. This routine proof owns a fixture-backed local command
so the public streamable-HTTP demo can remain a live operator walkthrough.

```bash
port="$(../../spec/fixtures/reserve-local-port)"
../../tools/biomcp-ci serve-http --host 127.0.0.1 --port "$port" >/tmp/biomcp-mcp-demo.log 2>&1 &
pid=$!
trap 'kill "$pid" 2>/dev/null || true' EXIT
for _ in $(seq 1 40); do
  if curl -fsS "http://127.0.0.1:$port/readyz" >/dev/null || curl -fsS "http://127.0.0.1:$port/health" >/dev/null; then
    break
  fi
  sleep 0.25
done
curl -fsS "http://127.0.0.1:$port/readyz" >/dev/null || curl -fsS "http://127.0.0.1:$port/health" >/dev/null
"${BIOMCP_SPEC_MCP_EXAMPLE_BIN:?spec preparation did not export MCP example}" remote-workflow "$port" | mustmatch like 'Command: biomcp study query --study msk_impact_2017 --gene TP53 --type mutations
# Study Mutation Frequency: TP53 (msk_impact_2017)'
```

## Read-Only Boundaries and Charted Calls Stay Visible

The transport should still reject CLI-only filesystem commands while returning
ordinary study text plus inline SVG for chart-safe read-only calls.

```bash
port="$(../../spec/fixtures/reserve-local-port)"
../../tools/biomcp-ci serve-http --host 127.0.0.1 --port "$port" >/tmp/biomcp-mcp-boundary.log 2>&1 &
pid=$!; trap 'kill "$pid" 2>/dev/null || true' EXIT
for _ in $(seq 1 40); do
  if curl -fsS "http://127.0.0.1:$port/readyz" >/dev/null || curl -fsS "http://127.0.0.1:$port/health" >/dev/null; then
    break
  fi
  sleep 0.25
done
curl -fsS "http://127.0.0.1:$port/readyz" >/dev/null || curl -fsS "http://127.0.0.1:$port/health" >/dev/null
"${BIOMCP_SPEC_MCP_EXAMPLE_BIN:?spec preparation did not export MCP example}" boundaries "$port" | mustmatch like 'CLI-only over MCP
workstation-local filesystem paths
BioMCP allows read-only commands only
# Study Mutation Frequency: TP53 (msk_impact_2017)
IMAGE: image/svg+xml'
```

## Repository Test Gate Runs Both Runtime Layers

`make test` is the gate March uses for focused and baseline validation. It must
run the Rust unit suite and the Python CLI/MCP/docs contract lane so neither
runtime layer can report a silent green.

```bash
env -u BIOMCP_BIN -u SPEC_BIN -u MAKEFLAGS -u MAKEOVERRIDES \
  make -C ../.. -n test SPEC_PROFILE=spec \
  2>&1 | mustmatch like 'cargo nextest run
cargo build --locked --profile spec
/target/spec/biomcp" uv run --no-sync pytest tests/ -v
/target/spec/biomcp" uv run --no-sync mkdocs build --strict'
```

## Routine Markdown Helpers Reuse The Selected Cargo Profile

The local MCP client example is part of the executable-contract surface. It
should use the gate's selected Cargo profile rather than silently compiling the
BioMCP library again under Cargo's default debug profile.

```bash
rg -nP '[c]argo run(?![^\n]*--profile)[^\n]*--example' ../../spec/entity ../../spec/surface | mustmatch ""
```

## Repository Lint Keeps The Quality Ratchet

Dropping `make check` must not orphan the quality-ratchet policy that used to run
through that target. The standard `make lint` gate should continue to run the
repo lint script and the ratchet script.

```bash
make -C ../.. -n lint 2>&1 | mustmatch like "./bin/lint
tools/check-quality-ratchet.sh"
```

## Repository Release Gate Adds One Full-Feature Proof

The release gate should run the small routine lint/test graph, add the named
all-feature check, and then run specs against the release binary. Keeping the
recipe visible prevents an obsolete shim or narrow spec subset from replacing
either the fast routine proof or the shipped-feature proof.

```bash
env -u BIOMCP_BIN -u SPEC_BIN -u SPEC_PROFILE -u MAKEFLAGS -u MAKEOVERRIDES \
  make -C ../.. -n release-gate \
  2>&1 | mustmatch like 'cargo nextest run
make full-feature-check
cargo clippy --locked --all-targets --all-features
cargo test --locked --all-features --lib sources::alphagenome::tests
cargo build --release --locked --all-features --bin biomcp
/target/spec/biomcp" uv run --no-sync pytest tests/ -v
/target/spec/biomcp" uv run --no-sync mkdocs build --strict
make spec SPEC_PROFILE=release SPEC_BIN='
```

## Repository Make Check Is Not A Public Target

BioMCP should not keep a compatibility `check` target now that March validates by
make-target convention. Operators should use the standard gates directly.

```bash
awk '/^check:/{print}' ../../Makefile | mustmatch not like "check:"
```

## Root Agent Guide Declares The Contract

A dispatched agent starts at the repository root. The root guide must declare
the executable contract path, the three gates, and the hybrid Rust/Python skill
rail without requiring the agent to infer them from stale docs.

```bash
cat ../../AGENTS.md 2>/dev/null | mustmatch like "spec/*.md
make lint
make test
make spec
rust-standards
python-standards
cli-design
mustmatch
testing-mindset"
```

## Runtime Artifacts Stay Ignored

March runtime state belongs outside git. The ignore rules should keep the local
`.march-runtime/` tree from appearing as a trackable repository path.

```bash
cat ../../.gitignore | mustmatch like ".march-runtime/"
```

## Public Streamable HTTP Demo Keeps The BRAF Workflow

The shipped Streamable HTTP demo is the public live walkthrough. It should keep
the documented discovery, variant evidence, and melanoma trial commands rather
than shrinking to the offline study fixture used by routine specs.

```bash
uv run --no-sync python3 - <<'PY' | mustmatch like 'biomcp search all --gene BRAF --disease melanoma --counts-only
biomcp get variant "BRAF V600E" clinvar
biomcp search trial -c melanoma --mutation "BRAF V600E" --limit 5'
import ast
from pathlib import Path

module = ast.parse(Path("../../examples/streamable-http/streamable_http_client.py").read_text())
for node in module.body:
    if isinstance(node, ast.Assign) and any(getattr(target, "id", None) == "WORKFLOW" for target in node.targets):
        print("\n".join(ast.literal_eval(node.value)))
        break
PY
```

## MCP Surface Spec Owns Its Offline Workflow

Routine MCP proof should not execute the public demo script. The spec owns its
fixture-backed local command so the demo can remain a live operator walkthrough.

```bash
sed '/Read-Only Boundaries and Charted Calls Stay Visible/q' ../../spec/surface/mcp.md | mustmatch not like 'examples/streamable-http/streamable_http_client.py'
```

## Spec Gates Use The Mustmatch Binary Runner

The executable spec gates should enter through the shared runner script and that
script should use the standalone `mustmatch test` binary. This keeps the routine
and live lane split visible while preventing the deleted pytest plugin from
remaining the real runner.

```bash
make -C ../.. -n spec 2>&1 | mustmatch like "scripts/run-specs.sh"
make -C ../.. -n spec-pr 2>&1 | mustmatch like "scripts/run-specs.sh"
make -C ../.. -n spec-contracts 2>&1 | mustmatch like "scripts/run-specs.sh"
make -C ../.. -n verify 2>&1 | mustmatch like "scripts/run-specs.sh"
find ../../scripts -maxdepth 1 -name run-specs.sh -type f -exec cat {} \; | mustmatch like 'mustmatch test
--lang bash
--timeout 180
SPEC_ROUTINE_PATHS
SPEC_LIVE_PATHS
default_biomcp_bin="$ROOT/target/spec/biomcp"
BIOMCP_BIN="${BIOMCP_BIN:-$default_biomcp_bin}"'
```

## Release Specs Reuse The Already-Built Feature-On Binary

Routine specs always prepare their feature-off CLI because an arbitrary caller
binary cannot prove its feature set. When the release gate already built the
feature-on CLI, it passes that artifact explicitly and preparation copies it
instead of rebuilding it. The dry-run recipe keeps this distinction visible.

```bash run id=caller-provided-feature-on-binary
make -C ../.. -n spec SPEC_PROFILE=release SPEC_BIN=/bin/true 2>&1
```

```text expect=caller-provided-feature-on-binary contains
BIOMCP_FEATURE_ON_BIN="/bin/true" bash scripts/run-specs.sh spec
```

```text expect=caller-provided-feature-on-binary not-contains
cargo build --locked --profile
```

The routine recipe deliberately passes no feature-on artifact and delegates its
single feature-off build to the runner's preparation phase.

```bash
env -u SPEC_PROFILE -u SPEC_BIN -u MAKEFLAGS -u MAKEOVERRIDES \
  make -C ../.. -n spec 2>&1 \
  | mustmatch like 'BIOMCP_FEATURE_ON_BIN="" bash scripts/run-specs.sh spec'
```

## Routine Spec Runner Keeps One Python Canary

Routine specs are mostly executable Markdown contracts. The one static Python
exception is `tests/surface/test_parallel_isolation_contract.py`, which stays in
`make spec` to guard the disease/discover isolation split without reintroducing
Python MCP setup or broad pytest collection.

```bash
awk '/SPEC_ROUTINE_PATHS=\(/,/^\)/' ../../scripts/run-specs.sh | mustmatch like "spec/surface/mcp.md
tests/surface/test_parallel_isolation_contract.py"
```

```bash
python3 - <<'PY'
import re
from pathlib import Path
runner = Path('../../scripts/run-specs.sh').read_text()
assert re.findall(r'tests/surface/\S+\.py', runner) == ['tests/surface/test_parallel_isolation_contract.py']
assert 'uv sync --extra dev --no-install-project' not in runner
assert 'uv run --no-sync pytest' not in runner
print('routine pytest canary is bounded')
PY
```

## Routine Markdown Specs Do Not Relaunch Unit Tests

Request plans, renderer envelopes, and parser edge cases are unit/static proof.
The routine Markdown corpus should drive BioMCP commands instead of relaunching
Cargo tests from spec headings.

```bash
rg -n 'cargo test' ../../spec/entity/article.md ../../spec/entity/study.md ../../spec/entity/variant.md ../../spec/surface/request-plan-ratchets.md | mustmatch ""
```

## Routine Spec Targets Avoid Broad Python Contract Setup

Once Python static contracts move to `make test`, routine spec modes should not
enable a broad Python contract leg before running mustmatch. MCP markdown
contracts use the Rust rmcp helper, so the runner does not prepare Python MCP
client dependencies.

```bash
rg -n 'sync_python_dev|run_python=1|uv run --no-sync pytest' ../../scripts/run-specs.sh | mustmatch ""
rg -n 'prepare_mcp_markdown_deps|uv sync --extra dev --no-install-project' ../../scripts/run-specs.sh | mustmatch ""
```

## Mustmatch Is No Longer A Python Dev Dependency

The binary cutover makes mustmatch a tool on `PATH`, not a Python package in the
repo development environment. The gate and dependency files should not retain
pytest-plugin flags or the temporary `0.0.4` pin.

```bash
sed -n '/mustmatch/p;/--mustmatch/p' ../../Makefile ../../pyproject.toml ../../tests/test_version_sync_script.py ../../uv.lock | mustmatch not like "mustmatch==0.0.4"
sed -n '/mustmatch/p;/--mustmatch/p' ../../Makefile ../../pyproject.toml ../../tests/test_version_sync_script.py ../../uv.lock | mustmatch not like 'specifier = "==0.0.4"'
sed -n '/mustmatch/p;/--mustmatch/p' ../../Makefile ../../pyproject.toml ../../tests/test_version_sync_script.py ../../uv.lock | mustmatch not like "mustmatch-lang"
sed -n '/mustmatch/p;/--mustmatch/p' ../../Makefile ../../pyproject.toml ../../tests/test_version_sync_script.py ../../uv.lock | mustmatch not like "mustmatch-timeout"
```

## Official MCP Registry Metadata

BioMCP publishes local registry metadata for the official MCP Registry. The
routine check validates the root `server.json`, package identity, ownership
marker, and publish docs before a release is cut.

```bash
bash ../../scripts/check-mcp-registry-server.sh | mustmatch like "MCP registry metadata ok"
```

## Release Prep Pins The Next Release Version

Before publishing, the repo metadata should already be synchronized to the next
release version. The local version-sync check is the operator's quick proof that
Cargo, Python, MCP registry, citation, and plugin metadata all agree on the
release being prepared.

```bash
bash ../../scripts/check-version-sync.sh | mustmatch like "Versions in sync: 0.8.25"
```

## Spec Corpus Uses Robust Mustmatch Blocks

BioMCP's executable specs should read like durable documentation rather than a
shell script that captures one command and checks fragments of it later. The
corpus should use named blocks when one run needs separate expectations, use
line-oriented ellipsis for volatile gaps, and avoid pinning local paths, build
dates, and exact volatile counts.

```bash
rg -n 'echo "[[:punct:]][[:alnum:]_]*" [|] mustmatch' ../../spec --glob '*.md' | mustmatch ""
```

```bash
rg -n '^```bash[[:space:]][^`]*run[[:space:]]+id=' ../../spec --glob '*.md' | mustmatch '/```bash[[:space:]].*run[[:space:]]+id=/'
```

```bash
rg -n '^```[[:alnum:]_-]+[[:space:]][^`]*expect=' ../../spec --glob '*.md' | mustmatch '/expect=[[:alnum:]_-]+/'
```

```bash
rg -l -U '```(bash|sh)[^\n]*\n(?s:[^`]*[|][[:space:]]*mustmatch[^`]*[.][.][.][^`]*)```|```[[:alnum:]_-]+[^\n]*expect=[^\n]*\n(?s:[^`]*[.][.][.][^`]*)```' ../../spec --glob '*.md' | mustmatch '/spec\/.+[.]md/'
```

```bash
rg -n 'Saved[[:space:]]to:|date=\[-0-9|Total: \[0-9' ../../spec --glob '*.md' | mustmatch ""
```
