---
flow: build
priority: 5
---

# Publish the documentation site on every push, and remove the Cloudflare worker

biomcp.org has served the same documentation since 8 July 2026. Two months of documentation changes, including three landed tickets written to make the site readable by agents, have never reached a reader.

Verified 2026-09-02:

- The site is GitHub Pages, built from the `gh-pages` branch with CNAME `biomcp.org`. That branch was last written on 2026-07-08 22:49 by `github-actions[bot]`, "Deployed a6694289 with MkDocs version: 1.6.1".
- `gh-pages` carries no `llms.txt` and zero `.md` files.
- `.github/workflows/release.yml` no longer deploys documentation. `.github/workflows/docs-edge.yml` runs on every push to main and is now the only publisher.
- Every run of `docs-edge.yml` has failed. All 39 of them, back to the first on 2026-08-29T02:46. Its deploy step runs `wrangler deploy` and needs repository secrets `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID`, and neither exists.
- `https://biomcp.org/llms.txt`, `/llms-full.txt` and the markdown twin of a docs page all return 404, while `https://biomcp.org/` returns 200 and serves the July build.

The built site already contains what the site is missing. `site/llms.txt`, `site/llms-full.txt`, and 108 `.md` files are ordinary static files, and a static host serves them without any request-time logic.

## The decision this ticket carries

Ian ruled on 2026-09-02 to remove Cloudflare from the documentation path and keep the site on GitHub Pages.

The Cloudflare worker exists to do one thing a static host cannot: read the `Accept` header and return the markdown twin when a client asks for `text/markdown` on an HTML URL. **That capability is given up deliberately.** An agent reaches markdown by following `llms.txt` to an explicit `.md` path instead. Do not preserve, reimplement, or work around the content negotiation.

## Required behavior

A push to main publishes the documentation the repository builds, so the live host serves the current documentation rather than an older build.

The agent-readable indexes and the markdown twins are reachable on the live host at the addresses `llms.txt` advertises.

A publish that does not reach the live host fails visibly rather than passing quietly. The condition this ticket exists to prevent is a green check beside an unchanged site.

BioMCP no longer depends on Cloudflare to serve documentation, and it carries no configuration, code, or workflow step for a deployment path it does not use.

## Done, observably

- `curl https://biomcp.org/llms.txt` returns the file rather than a 404, and so do `llms-full.txt` and the markdown twin of a documentation page.
- The live site reflects a documentation change made after 2026-07-08.
- Every link inside the published `llms.txt` resolves on the live host.
- The repository contains no `wrangler` configuration, no edge worker source, and no workflow step that deploys one.
- A publish failure is reported as a failure.

## Boundary

Do not change the content of any documentation page, `llms.txt`, or `llms-full.txt`. Do not change which pages get markdown twins. Do not change the site's theme or navigation. Do not touch DNS: `biomcp.org` resolves through Cloudflare to the GitHub Pages origin today, and that stays as it is. Do not change any other job in `release.yml`. Ticket 1075 covers the installed package carrying its own documentation, which is a separate delivery path and stays as it is.

Publishing credentials are in scope only to the extent that the work must need none beyond what GitHub Actions already grants a workflow in this repository. If the work turns out to require a new secret, stop and say so rather than adding one.

## Addendum, 2026-09-03: the missing Cloudflare secrets are the premise, not a blocker

The first attempt refused in `01-design` on 2026-09-02 at 11:31. It ran the mandatory declaration preflight, observed that `.github/workflows/docs-edge.yml` needs `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID`, observed that `gh secret list --repo genomoncology/biomcp` lists neither, and refused on a missing required declaration.

That observation is correct and it is exactly why this ticket exists. Read it as the premise.

`docs-edge.yml` is not this ticket's settlement path. It is one of the files this ticket deletes. After the change the documentation site publishes through GitHub Pages using the `GITHUB_TOKEN` that Actions already grants a workflow in this repository, plus the `pages: write` and `id-token: write` permissions a workflow declares for itself. No repository secret is involved.

So the declaration preflight is satisfied by observing the settlement path this ticket creates, not the one it removes. Do not refuse on the absence of the two Cloudflare secrets. Their absence is the defect.

If the work turns out to need a secret that does not exist, that instruction at the end of the Boundary section still stands: stop and say so.
