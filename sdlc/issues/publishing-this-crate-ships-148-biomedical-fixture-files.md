# Publishing this crate ships 148 biomedical fixture files

`Cargo.toml` declares no `include` and no `exclude`, so everything tracked
goes into the published package. Verified:

    cargo package --list  ->  148 files under testdata/   (2.5 MB)

Nothing is broken today — the crate is not published. This is a question to
answer **before** the first `cargo publish`, because a crates.io release
cannot be unpublished, only yanked, and a yanked version stays downloadable.

## What is actually in there

Real records captured from PubMed, Europe PMC, PubTator, MyVariant, MyGene,
ClinGen, ClinicalTrials.gov, gnomAD and others. They are excellent test
fixtures and they are the reason this repo's coverage is as good as it is.
The question is not whether to keep them — it is whether shipping them inside
a public package is the same act as keeping them in the repo.

## Why this needs a qualified owner, not an engineering decision

Two things I am not in a position to judge:

1. **Redistribution terms.** Each source has its own. Bundling captured
    responses into a distributed artifact is redistribution, which is not
    always what an API's terms of use permit even when the data is public.
2. **Content.** These are clinical and genomics records. Whether any of them
    carry constraints beyond the ordinary is a domain call.

## The options

1. **Exclude `testdata/` from the package.** One line:
    `exclude = ["testdata/**"]`. Tests keep working from the repo; the
    published crate loses 2.5 MB it does not need at runtime. This is the
    default I would expect unless someone wants the opposite.
2. **Ship it deliberately**, after the terms of each source are checked and
    the answer is written down.
3. **Split the fixtures into a separate, unpublished dev-dependency crate.**
    More machinery than the problem needs today.

Recommend 1 as the safe default, with 2 available if there is a reason to
want the fixtures in the package. Either way the choice should be recorded,
so the next person does not have to re-derive it.

## Ask

This is Ian's call, not the factory's. Flagging it now so it is decided
deliberately rather than discovered after a release.

Raised by the 2026-08-09 adversarial review; confirmed here against the
current `Cargo.toml`.
