---
flow: quickfix
priority: 6
---
# Read Europe PMC's not-open-access answer as absent, not failed

## Done when

`biomcp get article 30311380 fulltext` emits no warning, and the asset
resolution records the Europe PMC route as absent rather than failed,
so no "retry later" suggestion is offered for a permanent condition.
Every `warn!` in `resolve_archive_package` names its reason, not just
its step. A test feeds the 200/`errorBean` response and asserts absent.

## The finding

Raised as `sdlc/issues/europe-pmc-not-open-access-is-reported-as-a-failure.md`; that file is deleted when this
lands. The text below is the issue as filed.

become a ticket.

Running `biomcp get article 30311380 fulltext` emits:

    WARN biomcp_cli::entities::article::assets:
      Europe PMC supplementary request failed for article assets

Nothing failed. Europe PMC answered, and its answer was "there is
nothing here". It just said so in a shape the code does not read.

## What the wire actually carries

    $ curl -D - https://www.ebi.ac.uk/europepmc/webservices/rest/\
    PMC6329583/supplementaryFiles

    HTTP/2 200
    content-type: application/xml
    content-length: 164

    <?xml version="1.0" encoding="UTF-8" standalone="yes"?>
    <errorBean><errCode>0</errCode>
    <errMsg>Article with id PMC6329583 is not open access one</errMsg>
    </errorBean>

**HTTP 200, `application/xml`, an error bean where a ZIP was
expected.** Not a 404, not a 204. Reproduced the same day against
PMC3014373, so it is the provider's normal way of saying an article
is outside the OA subset, not a blip.

## Why the code mishandles it

`supplementary_status_has_package` (`src/sources/europepmc.rs:385`)
decides purely on the status code:

    if status == NOT_FOUND || status == NO_CONTENT { return Ok(false) }
    if !status.is_success() { return Err(...) }
    Ok(true)

200 is success, so it returns `true` — "there is a package". The
bytes then reach `parse_supplementary_zip`, which correctly refuses
them, and the resulting `invalid supplementary ZIP: …` error travels
back up to `resolve_archive_package`
(`src/entities/article/assets.rs:1163`), where it meets:

    Err(_) => {
        tracing::warn!("Europe PMC supplementary request failed …");
        SourceAttempt::Failed
    }

Two separate defects, and they compound.

**1. Absent is classified as Failed.** "This article is not open
access" is permanent. `SourceAttempt::Failed` is transient, and it
sets the `failed` flag that makes `final_asset_source_error` return
`asset_sources_unavailable()`, whose suggestion is *"Retry later or
inspect the article at its source."* Retrying will never work. The
tool tells the caller to do a thing that cannot succeed, and it
poisons the honest `Absent` verdict the other sources reached.

**2. The error text is thrown away.** `Err(_)` discards a message
that literally contains the diagnosis. Every `warn!` in
`resolve_archive_package` has this shape — five of them — so each
names a step and never a reason. Diagnosing this took a curl; it
should have taken reading the log line.

## Fix shape

- In `supplementary_status_has_package`, treat a success response
  whose content type is not a ZIP as absent rather than present.
  Better still, parse the `errorBean` and match `is not open access`
  explicitly, so a genuinely malformed ZIP stays distinguishable
  from a well-formed "no".
- Return `SourceAttempt::Absent` for that case so the retry
  suggestion is not offered.
- Include the error's message in each `warn!` in
  `resolve_archive_package`. Bind the error instead of `Err(_)`.

## Worth knowing

The article's supplementary file was found anyway, through the
JATS/PMC HTML route — `biomcp --json get article 30311380 assets`
lists `NIHMS987696-supplement-Supp_Tables.xlsx`. So the fan-in did
its job and the warning was pure noise on a successful run. That is
the strongest argument for fixing the classification: a permanent
"not in the OA subset" should be quiet, and a real failure should be
loud and say why.

Found 2026-08-08 while researching PTEN GN003 for varclassify2.
