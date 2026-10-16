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

## Discovery Advertises Stateless And Legacy Revisions

Stdio clients may ask what BioMCP serves before sending any ordinary request.
Discovery advertises the stateless 2026-07-28 revision alongside the two legacy
revisions, together with the same tools, resources, and package identity used by
both eras. Because this answer is public and static, clients may cache it for
five minutes. Modern clients send their version and capabilities on every
request; legacy clients retain their initialization handshake.

```bash
response_file="$(mktemp)"
set +e
"${BIOMCP_BIN:-../../target/release/biomcp}" serve >"$response_file" 2>/dev/null <<'EOF'
{"jsonrpc":"2.0","id":"discover","method":"server/discover","params":{}}
EOF
set -e
head -n 1 "$response_file" | mustmatch like '"supportedVersions":["2025-06-18","2025-11-25","2026-07-28"]
"capabilities":{"resources":{},"tools":{}}
"io.modelcontextprotocol/serverInfo":{"name":"biomcp"
"resultType":"complete"
"ttlMs":300000
"cacheScope":"public"'
rm -f "$response_file"
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
text = "\n".join(
    Path(path).read_text(encoding="utf-8")
    for path in ("src/mcp/shell.rs", "src/mcp/catalog.rs")
)
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
"${BIOMCP_SPEC_MCP_EXAMPLE_BIN:?spec preparation did not export MCP example}" typed-tools "$port" | mustmatch like 'MCP tools: biomcp, search, get, variant_normalize_car, variant_erepo, gene_cspec, variant_articles
ClinGen schemas validate their named properties
all listed MCP tools are read-only annotated
all listed MCP tools have titles and descriptions
search and get schemas publish flat roots without combinators
search and get schemas declare object roots
search schema includes a bounded limit
search and get schemas include author entity
get schema merges per-entity sections and hides CLI-only forms
article schema exposes assets manifest but not asset download
variant_articles schema includes identity verification controls
indexing'
```

## Stateless Metadata Names Only The Modern Revision

Per-request metadata is the 2026-07-28 way to ask for stateless service, and that
mode does not exist on the legacy revisions. A request whose metadata names a
legacy revision is not the legacy handshake path, so BioMCP rejects it with
`UnsupportedProtocolVersionError` instead of serving it statelessly. Legacy
clients keep their `initialize` handshake, and discovery still advertises every
supported revision.

```bash
python3 - <<'PY' | mustmatch like 'legacy metadata cannot request stateless service'
import json, os, subprocess

proc = subprocess.Popen(
    [os.environ["BIOMCP_BIN"], "serve"],
    stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True,
    env=os.environ.copy(),
)

def call(message):
    proc.stdin.write(json.dumps(message) + "\n")
    proc.stdin.flush()
    return json.loads(proc.stdout.readline())

response = call({"jsonrpc":"2.0","id":"legacy","method":"tools/list","params":{"_meta":{
    "io.modelcontextprotocol/protocolVersion":"2025-11-25",
    "io.modelcontextprotocol/clientCapabilities":{}}}})
assert response["error"]["code"] == -32022
assert response["error"]["data"]["requested"] == "2025-11-25"
assert set(response["error"]["data"]["supported"]) == {
    "2025-06-18", "2025-11-25", "2026-07-28"}
proc.terminate()
proc.wait(timeout=5)
print("legacy metadata cannot request stateless service")
PY
```

## Subscriptions Acknowledge Opted-In Types They Honor

`subscriptions/listen` opens a 2026-07-28 subscription with the notification
types the client opts into, and the acknowledgment reports the subset the server
agreed to honor. BioMCP advertises tools and resources, so it acknowledges those
list-change types when the client asks for them. It has no prompts and its
resources never update, so `promptsListChanged` and `resourceSubscriptions` stay
out of the acknowledgment, as does any type the client did not request.

```bash
python3 - <<'PY' | mustmatch like 'subscription acknowledgment honors opted-in types'
import json, os, subprocess

meta = {
    "io.modelcontextprotocol/protocolVersion": "2026-07-28",
    "io.modelcontextprotocol/clientCapabilities": {},
}
proc = subprocess.Popen(
    [os.environ["BIOMCP_BIN"], "serve"],
    stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True,
    env=os.environ.copy(),
)

def listen(identifier, notifications):
    proc.stdin.write(json.dumps({
        "jsonrpc": "2.0", "id": identifier, "method": "subscriptions/listen",
        "params": {"_meta": meta, "notifications": notifications},
    }) + "\n")
    proc.stdin.flush()
    return json.loads(proc.stdout.readline())

ack = listen("listen", {
    "toolsListChanged": True, "promptsListChanged": True,
    "resourcesListChanged": True, "resourceSubscriptions": ["biomcp://help"],
})
assert ack["method"] == "notifications/subscriptions/acknowledged"
assert ack["params"]["notifications"] == {
    "toolsListChanged": True, "resourcesListChanged": True}
assert ack["params"]["_meta"]["io.modelcontextprotocol/subscriptionId"] == "listen"
assert listen("tools-only", {"toolsListChanged": True})["params"]["notifications"] == {
    "toolsListChanged": True}
assert listen("empty", {})["params"]["notifications"] == {}
proc.terminate()
proc.wait(timeout=5)
print("subscription acknowledgment honors opted-in types")
PY
```

## Article Query Validation Converges Across MCP Tools

Raw and typed article calls return the same actionable tool error without
ending the session. The typed keyword keeps its published array shape.

Literal reserved-label prose keeps the same runtime quote byte through raw
Markdown, raw JSON, and typed MCP decoding. Each reaches exactly one selected
Semantic Scholar request.

## Diagnostic Synonym Provenance Reaches Raw MCP

Raw MCP keeps the diagnostic CLI body and its optional disease-match provenance
for both JSON and Markdown responses. WHO-only disease search remains literal
and does not need a typed diagnostic branch.

## WHO Degrade Reason Reaches Raw MCP

A drug search that lost its WHO section carries the degrade reason in the JSON
envelope, so the raw MCP tool returns the same `note` field the CLI prints:
MCP callers must see why a region is empty, not a silent zero-row bucket
(tickets 1304 and 2021). The WHO export URLs point at a closed local port so
the case runs offline.

## Variant filter evaluation is identical through raw and typed tools

Raw and typed MCP search calls use the same CLI execution and rendering path.
Both formats therefore distinguish exact variant identity from per-filter
evaluation without introducing an MCP-only response model.

## ORCID Author Surfaces Match The CLI Through Raw MCP

The ORCID exact-record surfaces are reachable through the raw `biomcp` tool
with byte-identical text, and an ORCID command never requests a Semantic
Scholar route.

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

## MCP Responses Surface Provenance Metadata

Default MCP tool text should carry upstream provenance without requiring the caller to know `--json`, while structured callers can opt in with the tool input `json: true`.

```bash
python3 - <<'PY' | mustmatch like 'MCP provenance metadata contract is wired'
from pathlib import Path

repo = Path('../..')
shell = (repo / 'src/mcp/shell.rs').read_text()
catalog = (repo / 'src/mcp/catalog.rs').read_text()
tests = (repo / 'tests/rmcp_client_contract.rs').read_text()
contract = (repo / 'crates/biomcp-mcp-contract-client/src/lib.rs').read_text()

assert 'json: bool' in shell
assert 'args_with_json' in shell
assert 'append_default_mcp_footer' in shell
assert 'mcp_meta_footer_from_json' in shell
assert '## Sources' in shell
assert '## Next commands' in shell
assert 'pub(super) const TOOLS' in catalog
assert 'name: "biomcp"' in catalog
assert 'biomcp list <entity>' in catalog
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
runtime layer can report a silent green, and both layers run the one
dev-profile build the gate prepares.

```bash
env -u BIOMCP_BIN -u SPEC_BIN -u MAKEFLAGS -u MAKEOVERRIDES \
  make -C ../.. -n test \
  2>&1 | mustmatch like 'cargo nextest run
cargo nextest archive
cargo build --locked --profile dev
/target/debug/biomcp" uv run --no-sync pytest tests/ -v
/target/debug/biomcp" uv run --no-sync mkdocs build --strict'
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
/target/debug/biomcp" uv run --no-sync pytest tests/ -v
/target/debug/biomcp" uv run --no-sync mkdocs build --strict
make spec SPEC_PROFILE=release SPEC_BIN='
```

## Repository Make Check Is Not A Public Target

BioMCP should not keep a compatibility `check` target. Operators use the
standard gates directly.

```bash
awk '/^check:/{print}' ../../Makefile | mustmatch not like "check:"
```

## Root Agent Guide Declares The Contract

A dispatched agent starts at the repository root. The root guide must declare
the executable contract path, the targeted-test command, and the three gates
without requiring the agent to infer them from stale docs.

```bash
cat ../../AGENTS.md 2>/dev/null | mustmatch like "mustmatch executable docs at"
cat ../../AGENTS.md 2>/dev/null | mustmatch like "cargo nextest run --no-default-features -E 'test(...)'"
cat ../../AGENTS.md 2>/dev/null | mustmatch like '`make spec` is the routine mustmatch contract gate'
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
default_biomcp_bin="$ROOT/target/debug/biomcp"
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

## Release Prep Pins The Candidate Version

Before candidate testing, the package metadata should already use the canonical
Rust and Python candidate versions. The local version-sync check is the
operator's quick proof that those identities agree while MCP registry,
citation, and plugin metadata truthfully remain on the latest public release.
Between releases the candidate is a development build whose Cargo version
carries a `-dev` suffix (`0.9.3-dev`, Python `0.9.3.dev0`); the release commit
drops the suffix, so the check reports one line for either shape.

```bash
bash ../../scripts/check-version-sync.sh | mustmatch '/Versions in sync: [0-9]+\.[0-9]+\.[0-9]+(-dev(\.[0-9]+)?)? \(Python [0-9]+\.[0-9]+\.[0-9]+(\.dev[0-9]+)?; (release|development) candidate\)/'
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
