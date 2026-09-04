---
flow: build
priority: 5
---

# Publish current-main documentation and remove the edge worker

## Outcome

A push to `main` triggers a strict documentation publication through the
repository's existing GitHub Pages branch. Bursts may coalesce, but cannot let
an older revision overwrite a newer one; the channel converges on current main.
The publishing check is green only after its revision is observable at
`https://biomcp.org/`. Agent indexes and explicit Markdown twins are served by
the static site, and the repository no longer contains a Cloudflare Worker
deployment path.

## Current facts

Reverified 2026-09-04:

- The GitHub Pages API reports `status: built`, `build_type: legacy`, custom
  domain `biomcp.org`, and source `gh-pages:/`. The unprotected `gh-pages`
  branch was last written at `5ae10b33` on 2026-07-08 22:49 UTC by
  `github-actions[bot]`: `Deployed a6694289 with MkDocs version: 1.6.1`.
  It contains neither `llms.txt` nor any `.md` file.
- The old `deploy-docs` job used `mkdocs gh-deploy` with `contents: write` and
  successfully fed this branch. It disappeared when the release workflow was
  replaced on 2026-08-10. Documentation publication therefore stopped before
  the edge work began.
- `.github/workflows/docs-edge.yml` is now the only workflow triggered on each
  main push that attempts to publish documentation. All 119 runs from
  2026-08-29T02:46:03Z through 2026-09-04T05:17:06Z failed. The latest failure
  again reached `wrangler deploy` with empty `CLOUDFLARE_API_TOKEN` and
  `CLOUDFLARE_ACCOUNT_ID` values and exited nonzero.
- The live root returns 200 with `Last-Modified: Wed, 08 Jul 2026 22:50:06 GMT`.
  `llms.txt`, `llms-full.txt`, and `user-guide/trial.md` return 404. Sending
  `Accept: text/markdown` to `user-guide/trial/` returns the old HTML, confirming
  that no Worker currently provides negotiation.
- A strict local MkDocs build already emits `CNAME`, both agent indexes, and
  byte-exact twins for all 108 Markdown sources. Static publication is enough.
- The repository allows Actions and its default workflow token has write
  capability; `gh-pages` is not protected. A job-scoped `contents: write`
  permission can therefore restore the already-configured legacy path without
  a repository secret or settings mutation.
- GitHub supports an explicit Pages build request through
  `POST /repos/{owner}/{repo}/pages/builds` and accepts an Actions installation
  token with repository `Pages: write`. GitHub documents that request in terms
  of the latest default-branch commit rather than promising which configured
  legacy-source revision it selects. This workflow still requires the request
  after updating `gh-pages`, but treats the exact live revision witness—not the
  API response—as proof of what reached the host.

The live GitHub Pages configuration matters to the design. GitHub's custom
artifact deployment requires the repository to be configured to use GitHub
Actions as its Pages source; this repository is configured for a branch.
Changing that owner-level setting is not representable by this commit, and
`actions/configure-pages` documents that automatic enablement needs a token
other than `GITHUB_TOKEN`. Do not introduce that external prerequisite. Publish
the built site to the existing `gh-pages` source branch with the locked MkDocs
tooling instead.

## Product decision

Ian ruled on 2026-09-02 to remove Cloudflare from the documentation publication
path and keep the site on GitHub Pages. The Worker offered content negotiation:
an HTML URL could return its Markdown twin for `Accept: text/markdown`. That
capability is deliberately retired. Agents reach Markdown through the explicit
`.md` URLs advertised by `llms.txt`; do not preserve or reimplement negotiation.

This makes one narrow correction to the original content boundary: remove the
`llms.txt` clause that advertises `Accept: text/markdown`, and replace its
Worker-specific test. Leaving a claim for behavior this ticket removes would be
a public contract defect. Do not otherwise change `llms.txt`, `llms-full.txt`,
any documentation page, the set or bytes of Markdown twins, theme, or navigation.

## Test-first design

1. Replace the Worker-specific contracts with a static-publication contract.
   Preserve the existing strict-build proof that every Markdown source has a
   byte-exact twin. Require `llms.txt` to describe explicit `.md` routes and to
   make no content-negotiation claim.
2. Add a focused workflow contract that fails on the current tree and proves
   there is exactly one documentation publisher. It has only the
   `push: branches: [main]` trigger, and the publishing job additionally requires
   `github.event_name == 'push'` and `github.ref == 'refs/heads/main'`; remove
   `workflow_dispatch`. It checks out `github.sha`, uses the locked Python
   environment and pinned actions, grants job-scoped `contents: write` and
   `pages: write`, and has no other write permission.
3. Give the workflow one stable concurrency group with `cancel-in-progress:
   true` so bursts coalesce on the newest run, then immediately before the first
   write fetch `origin/main` and prove it is still `github.sha`. A run already
   stale at that immediate pre-write check must perform no branch write and no
   Pages build request. Main can advance after the check, so do not claim that
   the guard alone makes a now-superseded in-flight run incapable of writing;
   cancellation/coalescing plus the newest run's exact live witness guarantee
   convergence. This guard is required even with concurrency because GitHub
   does not guarantee run ordering.
4. Fetch the existing `gh-pages` ref and its history (`fetch-depth: 0` or an
   explicit ref fetch), then publish with the locked command
   `mkdocs gh-deploy --strict --force`. Do not use `--no-history`: each clean
   generated commit must descend from the fetched publication history so the
   rollback evidence remains usable. Preserve `docs/CNAME`.
5. After the successful branch push, make the supported explicit Pages build
   request relied on here with authenticated
   `POST /repos/$GITHUB_REPOSITORY/pages/builds`, and require a successful
   response. Contract the ordering and the two exact write permissions. The
   ordinary `GITHUB_TOKEN` is the only credential; the later exact live witness
   remains the hosted proof of the deployed revision.
6. Give each deployed revision an inert, generated witness at
   `/__biomcp_revision__/<sha>.txt`, with the exact full lowercase 40-character
   SHA plus newline as its body. The MkDocs hook may emit it only when the
   publication job passes that validated SHA as `BIOMCP_DOCS_REVISION`; invalid
   values fail closed. A clean build removes every stale witness and carries
   only the current SHA-named file. This does not create a documentation page or
   alter a Markdown twin.
7. Replace `docs-edge.yml` with the narrowly scoped Pages-branch publisher. Delete
   `docs-edge-worker.mjs` and `wrangler.toml`; do not add an edge replacement.
8. After the Pages build request, poll the SHA-named witness URL for at most ten
   minutes. Every probe must carry a unique cache-busting query and request
   revalidation/no-cache plus the stable non-browser user agent
   `BioMCP-docs-publication-verifier/1`. The explicit user agent is required on
   every live request because hosted run 33842254673 for revision
   `ab063ef3b8e371b540fa508d775b9d38d0f8387a` received HTTP 403 from the
   Cloudflare-fronted host with Python urllib's
   default identity, while a direct read-only probe of the same witness returned
   200 with that explicit identity. Before any live request, construct the full
   expected inventory from the trusted local build, validate every advertised
   link, and require every resolved comparison file to remain beneath the
   resolved build directory. Require the normalized scheme/host/port origin and
   URL path to remain exact across redirects. Within one global deadline of at
   most ten minutes, retry a complete attempt consisting of the byte-exact SHA
   witness, `llms.txt`, `llms-full.txt`, every explicit `.md` twin, and every
   other advertised non-HTML asset, plus a successful status/origin/path check
   for actual HTML targets advertised by the trusted index. Determine that
   exception from the trusted resolved local target: only `.html` files,
   including the
   `index.html` resolved for root and directory routes, may differ in body. This
   whole-publication
   convergence is required because hosted run 33843221076 for revision
   `69a17318add8e688f80256d816e968816f78b935` observed the current witness while
   `/` was temporarily stale; an explicit-UA cache-busted diagnostic moments
   later showed `/` byte-identical to the local 93,168-byte build. Transient
   network, HTTP status, or exact-asset body mismatches retry from the witness,
   but invalid local inventory, path traversal, cross-origin redirects, and
   unexpected response paths fail closed immediately. Hosted run 33844251288 at
   `9ea3941581245bd41e1115cfdfade89909a30227` established this distinction:
   Cloudflare intermittently injected its Web Analytics beacon immediately
   before the closing body tag on `/`, producing 93,535 live bytes from the
   93,168-byte local HTML while another request was unmodified. DNS and proxy
   configuration remain out of scope.
9. At closure, reconcile the duplicate live planning state: remove the stale
   live 1073 ticket and update its existing record as satisfied by the published
   indexes; remove the stale live 1074 ticket and update its existing record as
   superseded for header/negotiation behavior while preserving its completed
   static-twin outcome. Search remaining live ticket dependencies and prose and
   update any reference whose status changed.

## Acceptance

- Focused workflow, Markdown-twin, agent-index, hook/witness, and live-verifier
  tests pass without contacting the live site during routine local tests.
- The workflow contract proves main-push-only authorization, stable concurrency
  with cancellation, the immediately-pre-write current-main guard, fetched
  `gh-pages` history, strict forced MkDocs deployment without `--no-history`,
  ordered branch push then authenticated Pages build request, and cache-busted
  live verification capped at ten minutes.
- `make lint`, `make test`, and `make spec` pass.
- The first exact-head hosted publication run succeeds. Its live verification
  proves the pushed revision is served, `llms.txt` and `llms-full.txt` return
  successfully, an advertised Markdown twin is byte-exact, and every
  `biomcp.org` link in published `llms.txt` resolves.
- The repository contains no Wrangler configuration, Worker source, Cloudflare
  credential reference in active delivery configuration/code/tests, or workflow
  step that deploys an edge service. Historical ticket records may retain the
  credential names as evidence.
- A failed, stale, or superseded Pages publication is non-green—failed or
  canceled. A successful branch push alone is not accepted as live publication.

## Boundaries and trust

- Do not change DNS or the existing Pages custom domain/source setting.
  Cloudflare may remain the DNS/proxy provider in front of the GitHub Pages
  origin; this ticket removes the Worker application path, not DNS.
- Do not change any job in `release.yml`. Ticket 1075's installed-package docs
  are a separate delivery path.
- Do not add a repository secret, personal token, third-party deploy action, or
  deployment from pull-request code. If branch publication unexpectedly needs
  any new credential or external setting, stop and report it.
- Keep write authority confined to the documentation job on trusted main pushes.
  Check out the triggering SHA, re-prove it is current main immediately before
  writing, and make already-stale and manually invoked runs incapable of
  publishing. A superseded in-flight run cannot report success; the newest run's
  exact live witness guarantees convergence.

## Failure and rollback

The publisher must fail before pushing if the strict build, witness validation,
or current-main check fails, after pushing if the Pages build request fails, and
after that if the exact live witness and static content do not appear within ten
minutes. A later main push republishes from scratch, so rollback is a normal
revert of the offending main change. Because the workflow fetches `gh-pages`
and does not use `--no-history`, its generated commits retain the prior
publication chain for diagnosis or an explicit operator rollback. Do not hide a
failed build request or live check with `continue-on-error` or a fallback host.

## Dependencies

None. The configured Pages branch, custom domain, unprotected publication
branch, locked MkDocs toolchain, and ordinary Actions token are already present.

## Review

- Design review: REJECT 2026-09-04 — require an explicit Pages build
  request and permission, main-only/stale-run controls, retained branch history,
  cache-safe bounded live proof, exact witness semantics, scoped residue checks,
  and reconciliation of stale tickets 1073/1074. Amendments applied; re-review
  rejected the overbroad stale-run and legacy-build claims. Those claims are now
  bounded to the immediate pre-write observation and exact live witness. A third
  review found the old absolute claim repeated once in the trust boundary; it is
  now aligned with the canceled/non-green in-flight-run contract. Design review:
  ACCEPT 2026-09-04 — the corrected ticket is internally consistent, keeps the
  write boundary on trusted current-main pushes, and makes the exact hosted
  witness rather than branch publication or the Pages API response the proof.
- Code review: REJECT 2026-09-04 — the live verifier accepted a same-path
  redirect to a different origin, and parsed live `llms.txt` before proving it
  matched the trusted local build. Require normalized scheme/host/port origin
  equality on every response, compare the live index byte-for-byte before
  parsing only the local copy, and confine every resolved expected file beneath
  the resolved build directory. Remediation and adversarial regressions applied;
  independent re-review then ACCEPTED the implementation on 2026-09-04.
- Hosted verification: FAIL 2026-09-04 — run 33842254673 at
  `ab063ef3b8e371b540fa508d775b9d38d0f8387a` remained stuck retrying HTTP 403
  responses to urllib's default user agent and could not pass; the same live
  witness was 200 with `BioMCP-docs-publication-verifier/1`. The stable explicit
  user-agent contract and all-request regression were added; remediation is
  applied and independent re-review accepted it.
- Hosted verification: FAIL 2026-09-04 — run 33843221076 at
  `69a17318add8e688f80256d816e968816f78b935` reached the exact witness, then
  found stale `/` bytes at 06:10:34 UTC. A cache-busted explicit-UA diagnostic
  around 06:11 UTC found live `/` byte-identical to the local build (SHA-256 and
  93,168-byte length), proving non-atomic CDN propagation across paths. Require
  bounded retries of the complete trusted inventory. Deterministic convergence,
  non-convergence, and pre-network inventory regressions were added; remediation
  is applied and independent re-review accepted it.
- Hosted verification: FAIL 2026-09-04 — run 33844251288 at
  `9ea3941581245bd41e1115cfdfade89909a30227` could not converge because the
  Cloudflare proxy intermittently injected the
  `static.cloudflareinsights.com/beacon.min.js` analytics script into `/` before
  the closing body tag. The live response was 93,535 bytes versus the
  byte-identical local/unmodified response's 93,168 bytes. Keep byte equality
  for immutable witnesses, indexes, and explicit Markdown twins; require only
  status/origin/path for actual HTML targets. Deterministic injected-HTML,
  changed-Markdown, and changed-index regressions were added; remediation was
  applied.
- Code review: REJECT 2026-09-04 — classifying every non-`.md` advertised URL as
  availability-only let an adversarially changed `/install.sh` pass. Classify
  from the trusted resolved local target instead: only actual `.html` files may
  tolerate body transformation; the installer and every other non-HTML asset
  remain byte-exact. Installer and generic non-HTML classification regressions
  were added; remediation is applied and independent re-review accepted it.

## Completed 2026-09-04

Replaced the failed Cloudflare Worker delivery path with a main-push-only,
serialized GitHub Pages branch publisher. The workflow strictly builds the
triggering SHA, rejects an already-stale writer, retains `gh-pages` history,
requests a Pages build with job-scoped `contents: write` and `pages: write`, and
cannot report success until the exact hosted revision is verified. Worker
source, Wrangler configuration, credential references in active delivery code,
and the obsolete content-negotiation promise were removed.

The live verifier now reflects the constraints learned from three failed hosted
runs: it identifies itself to the proxy, retries a complete publication through
non-atomic CDN convergence, rejects cross-origin redirects and local path
escape, trusts only the locally built index, requires exact bytes for the
witness, indexes, Markdown twins, installer, and all other non-HTML assets, and
allows proxy body transformation only for trusted HTML targets.

Hosted run 33845621279 succeeded for exact main revision
`fce136f3df2ff0fbc720ef8d168b9f41d7681b4a` in 1 minute 9 seconds. Its mandatory
live step verified the revision witness, both agent indexes, all Markdown twins,
111 byte-exact immutable assets including `install.sh`, and all four advertised
HTML routes at the exact configured origin and path.

Independent design review accepted the design after three correction rounds.
Fresh code review rejected and then accepted remediations for cross-origin and
path-traversal trust boundaries, proxy user-agent handling, non-atomic
propagation, proxy-mutated HTML, and executable installer integrity. Final local
gates passed on the reviewed implementation: `make lint`; `make test`, including
the complete Rust lane, 883 Python tests passed (3 skipped), and strict docs;
and `make spec`, including its static lane.
