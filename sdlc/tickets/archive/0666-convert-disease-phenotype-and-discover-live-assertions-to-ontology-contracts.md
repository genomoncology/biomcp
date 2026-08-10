---
flow: build
priority: 8
---
# Convert disease phenotype and discover live assertions to ontology contracts

Carried over from March ticket 666 when BioMCP moved to the sdlc
factory. The body below is March's, unchanged; it was already written to
stand alone. Work products from any earlier attempt:

    /home/ian/workspace/planning/biomcp/artifacts/666-convert-disease-phenotype-and-discover-live-assertions-to-ontology-contracts
## A note on `.march/` paths below

March gave each run a `.march/` directory inside its worktree for design
notes, review records and proof files. The sdlc factory has no equivalent:
those files were never committed and are not in this repo.

Read every `.march/...` reference below as **intent, not a path**. Where the
text says to write, amend or delete something in `.march/design-final.md` or
`.march/contract-red-check.json`, do the equivalent in the artefact this
flow's own design stage produces, and record the reasoning in the ticket's
record when it lands. Do not create a `.march/` directory.

## Read this first — four prior attempts, four distinct causes

This ticket has aborted four times. Each abort had a *different* correct root cause, so this
is a converging ticket, not a thrashing one — but you are the fifth attempt and the four
rulings appended at the bottom of this file are long. Here is the whole of it:

1. **Design must author the replacement assertions red.** Do not defer assertion authoring
   to the code step. "It cannot pass yet" is not a reason to defer it. Never write a
   "required code-step proof additions" section.
2. **The no-match seed must be a real query that returns nothing** — not a nonsense string.
   A seed that reads like gibberish is not the same as a seed that returns zero rows. Use
   `SCENAR therapy`; the repo already had it.
3. **Use the repo's established split shape.** The converted file keeps its name and moves to
   `SPEC_ROUTINE_PATHS`; the unconverted remainder splits into `<name>-live.md` under
   `SPEC_LIVE_PATHS`. See `article-assets-live.md`, `article-graph-live.md`,
   `variant-myvariant-live.md`. And actually retire the live blocks — a conversion that
   leaves every live block in place has converted nothing.
4. **Captures must be produced by the production request path.** Point the source base URL at
   a recording proxy and run the real command. Never hand-build a URL with `curl` and file
   the bytes. The tell that this went wrong is editing the runtime to fit the fixture: attempt
   four recorded a bare `q=…&size=5` MyDisease request while production issues a *scoped*
   query at `size=15`, then doubled a production cap from 10 to 20 to match. When a fixture
   and the runtime disagree, **the fixture is wrong until proven otherwise.**

Lessons 1–4 are now also in the flow files (`01-design.md`, `03-code.md`) and apply to every
ticket. What is specific to *this* ticket is the `SCENAR therapy` seed and the MyDisease
scoped-query shape. The full rulings below carry the evidence if you need it.

**On size:** this ticket covers three surfaces (`disease.md`, `phenotype.md`, `discover.md`)
and six providers — roughly three times the scope of a conversion that ships first time. It
has been ruled not to split, because each abort found a real defect rather than repeating
one. If you reach a fifth abort for a *cause already listed above*, that ruling should be
revisited: say so in the abort reason.

## Why
Ticket 645 found disease, phenotype, and discover assertions to be deterministic ontology/fallback behavior, with missing dated Monarch/HPO and OLS4 provider-shape evidence.

## What
Add consumed MyDisease/Monarch/NIH/SEER and OLS4/DiscoverRequest plans, receipted phrase, identifier, no-match, relational, clinical, funding, and survival captures, production decoding/orchestration tests, and local CLI envelope proof. Convert the live blocks in `disease.md`, `phenotype.md`, and `discover.md` without touching their routine local blocks.

## Intermediate State
Ontology normalization, typed fallback, clinical/funding/survival cards, and next commands are deterministic; mixed documents retain local coverage.

## Scope

What is IN scope:
- `src/entities/disease/`, `src/entities/discover.rs`, `src/sources/monarch.rs`, OLS4 tests/captures, and the named spec blocks.

What is OUT of scope:
- Altering parallel-isolation policy, entity grammar, or unrelated enrichments.

## Dependencies
- 651-enforce-receipt-backed-real-captures-for-tier-3-source-tests

## Success Checklist
- [ ] Each converted assertion has a consumed Tier 2 plan, receipt-backed real Tier 3 production decoder/orchestration proof, and fixture-backed CLI proof where it claims presentation.
- [ ] Only replaced live blocks leave `SPEC_LIVE_PATHS`; all retained local proof remains routine.
- [ ] A live test may be replaced by unit tests **only if** those unit tests fully cover the CLI-to-API-call transition and the parsing of a locally captured response. Everything on our side of the socket stays covered; only "does the provider serve this today" is dropped.
- [ ] Tier 2 and Tier 3 coverage lands **before** the live assertion is removed. Retiring a canary for a source with no unit tests reduces coverage rather than relocating it.
- [ ] Captures are real recorded responses with their capture date. A synthesised or edited fixture proves only that a parser agrees with itself — ticket 614's lesson, and the mechanism behind 650.
- [ ] Nothing here closes an issue. The five live-canary reports that used
  to be listed at this point were folded into ticket 0885 and their files
  deleted; 0885 owns them now. Do not go looking for them, and do not treat
  a green live run as evidence — a green run proves the provider is up
  today, which is exactly the property those canaries should stop
  asserting.
- [ ] The Semantic Scholar credential dependency is proven by capturing the outgoing header. It cannot be proven by an anonymous 429 — measured intermittent at 2 of 4 attempts.
- [ ] `make lint`, `make test`, and `make spec` pass.

## Notes
- See `architecture/technical/live-spec-conversion-target.md` for the target boundary and per-path prerequisites.
- No FAQ entry is currently `watching`; FAQ #17's standard gates remain the proof lane.

## Operator addendum — 2026-08-05: observe the provider's shape before asserting against it

Two tickets landed assertions against provider data shapes that do not exist:

- **678** asserted a package at `.../PMC<id>.<v>/metadata/<v>.json`. That path 404s. The
  real layout is flat: `.../PMC7382263.1/PMC7382263.1.json`.
- **663** asserted that the first citing paper for PMID 22663011 has no PubMed ID, no DOI
  and no arXiv ID. That record has 100 citation rows and **zero** such papers.

Neither was a bad capture. In both cases a design imagined what the provider returns, a
local fixture was built to match the imagination, and the assertion confirmed the fixture.
The loop closed cleanly and proved nothing.

Note this is **not** covered by "captures are real recorded responses". Both failures were
in hand-built fixtures, which that constraint does not reach. A real capture sitting beside
an invented fixture still yields a green run that means nothing.

### Requirements

- [ ] Every assertion about a provider's data shape names the request that was issued and
      the value that came back, recorded in the design. No assertion is authored from an
      assumed schema, an API doc, or a remembered shape.
- [ ] Any locally-built fixture is derived from an observed real response. If the fixture
      serves a shape no real response produced, say why in the design and expect challenge.
- [ ] If the design's chosen record cannot exhibit the behavior being pinned, **change the
      record, not the assertion**. 663's provider-only citation row was real; the seed was
      wrong. A bounded search across a handful of candidate records found it.
- [ ] Where an assertion depends on a row's position in a response, do not rely on it.
      663's target rows sat at indexes 74, 88 and 96 — never first.
- [ ] No production behavior is changed to make a capture assert cleanly. Ticket 662's
      review caught exactly that: `verify_ldh_annotation` was altered so a capture could
      assert completeness, and the review reverted it.
- [ ] No fixture is synthesized, hand-edited, or reshaped to match our code. Tickets 614,
      650 and 652 each did this; 652's fixture carried invented PMIDs.

## Operator ruling — 2026-08-06: design must author the replacement assertions red, not defer them to code

**The abort is upheld. The code step had no legal move and was right to stop.**

`.march/design-final.md` ends with a section titled **"Required code-step proof additions"**
listing nine routine assertions — disease clinical features, funding/survival, the phenotype
phrase route, the direct HPO-ID route, the JSON follow-up envelope, discover diabetes
identity, no-match article guidance, and the relational redirect — and says *"The code step
must add and record them, all as `lane: check` routine assertions."*

The code step cannot. `planning/flows/build/03-code.md:27` states plainly:

> You do **not** author new shipped-spec assertions. That authority belongs to design.

And `planning/flows/build/01-design.md:489` puts that authority where it belongs:

> Authored literal assertion bytes into their permanent `spec/*.md` paths

So design-final instructed the code step to do the one thing the code step is forbidden to
do. There was no path forward from that document. This is a design fault and it goes back to
design.

### The mistake, precisely

Design conflated **"this assertion cannot pass yet"** with **"this assertion cannot be
authored yet."** They are not the same thing, and the whole build flow depends on the
difference.

A conversion ticket's replacement assertions obviously cannot pass before the fixture and
captures exist — that work is the code step's. But they can and must be **authored** first,
where they sit red until code makes them green. That is exactly what
`.march/contract-red-check.json`'s `observed_status: "red, behavioral"` records, and 666's
registry has only two entries where it should have eleven.

The design review is not at fault here. It correctly rejected the draft for presenting five
unbuilt assertions as landed evidence. The right response to that correction was to author
the bytes and record them red. Deferring them to code was the wrong response.

### The precedent

Sibling ticket **664**, same family, did this correctly. Commit `8f9c67a9`
*"design: land spec tests for 664-convert-myvariant-and-cancerhotspots-live-assertions-to-captured-response-contracts"*
added a new **"Captured MyVariant Filters and Consequences"** section to
`spec/entity/variant.md` with two literal fixture-backed assertions — authored before the
fixture existed, red until the code step built it, green at merge. 664 shipped 108/108.

666 should look like that.

### Ruling

1. **Rewound to `01-design`.** Author the literal bytes of all nine replacement assertions
   into `spec/entity/disease.md`, `spec/entity/phenotype.md`, and `spec/surface/discover.md`.
2. **Record every one in `.march/contract-red-check.json`** with `observed_status` starting
   `"red, behavioral"`, from a real run that observed them red. Eleven entries, not two.
3. **Delete the "Required code-step proof additions" section from design-final.** Nothing in
   that document may instruct the code step to author a shipped-spec assertion.
4. **Do not retire any live block in the design step.** Authoring the replacements and
   removing the originals are separate acts; the removal and the `SPEC_LIVE_PATHS` move stay
   in the code step, once the fixture makes the replacements green.
5. The assertion design already landed in `spec/entity/disease-survival-fixture.md` is fine
   and stays. That file is not in `SPEC_LIVE_PATHS`; it was a legitimate routine ratchet.

Everything else in design-final — the capture list, the fixture design, the Tier 2/Tier 3
boundaries, the acceptance criteria — is sound and survives the rewind unchanged.

## Operator ruling, second pass — 2026-08-06: the no-match seed is wrong; re-seed to `SCENAR therapy`

**The abort is upheld.** Design authored the assertions this time, which is the correction
from the first ruling working as intended. But one of the nine seeds asserts an outcome the
provider does not produce, and the code step was right to refuse to force it.

### What I observed

I queried OLS4 directly with the exact request the client builds
(`/api/search`, `rows=10`, `groupField=iri`, `ontology=hgnc,mesh,mondo,doid,hp,go,chebi,dron,ncit,ordo,wikipathways,so`
— `src/sources/ols4.rs:57-72`):

| Query | `numFound` | docs |
|---|---|---|
| `not-a-biomedical-concept` | **38** | **10** |
| `SCENAR therapy` | 0 | 0 |
| `genes regulated by MEF2 in the heart` | 0 | 0 |
| `type 2 diabetes mellitus` | 224 | 10 (incl. `MONDO:0005148`) |

`not-a-biomedical-concept` is not a no-match query. The hyphens tokenize and "biomedical"
and "concept" both match real NCIT terms. The seed was chosen because it *reads* like
nonsense, not because it was observed to return nothing — design landmine 3 again, and the
empty-collection rule from 663's fifth ruling on top of it.

The other two discover seeds are sound and stay: the MEF2 relational query genuinely returns
zero concepts, and diabetes genuinely returns `MONDO:0005148`.

### The fix is already in this repo

`src/entities/discover.rs:3083` — the existing unit test
`empty_results_add_review_article_fallback_note_and_command` — already uses **`SCENAR
therapy`** as its no-match seed. OLS4 returns 0/0 for it. The repo picked the right seed
years ago in the unit lane and the design step picked a worse one for the spec lane.

### Ruling

I am authoring the corrected assertion here so the code step transcribes it rather than
authors it. Replace the block in `spec/surface/discover.md` verbatim with:

```
## Captured No-Match Article Guidance

When the recorded OLS4 no-match response contains no concepts, discover must still give a
usable article-search next step instead of ending at an empty result.

```bash
../../tools/biomcp-ci discover "SCENAR therapy" | mustmatch like 'No biomedical entities resolved
biomcp search article -k "SCENAR therapy" --type review --limit 5'
```
```

1. **Re-capture OLS4 for `SCENAR therapy`.** The committed capture is for the wrong query
   and must not be relabelled. Receipt the new body as `real_and_receipted` and drop the
   `not-a-biomedical-concept` capture from the corpus.
2. **Commit this as `code: fix mechanical bug in landed assertion`** with a body naming this
   ruling. The bytes above are operator-authored; the code step is transcribing, not
   exercising assertion authority. It remains forbidden from authoring anything else.
3. **Do not** satisfy the original assertion by filtering NCIT hits in the runtime, by
   trimming the captured response, or by any fixture that serves fewer concepts than the
   provider returned. All three were correctly refused.
4. `.march/contract-red-check.json` keeps eleven entries; update the no-match row's
   `expected_observation` to the new seed.

The remaining eight assertions, the capture inventory, the fixture design, and the work
already staged at `/tmp/biomcp-666-captures` all stand. Resuming at `03-code` rather than
rewinding preserves them.

### Note

This is the second ticket today where a seed was chosen for how it reads rather than for what
it returns, and the second where the shipped codebase already contained the observed answer.
The general lesson is now in `01-design.md`: if the repo already exercises a case, take its
seed before inventing one.

## Operator ruling, third pass — 2026-08-06: use the repo's established split shape, and actually retire the live blocks

**The abort is upheld.** Code review was right that it cannot relocate shipped assertions,
and right to send this back rather than bless the layout.

### What the code step did

It created a new `spec/entity/ontology-contracts.md`, moved the nine design-authored
assertions into it, and added that one file to `SPEC_ROUTINE_PATHS`.
`spec/entity/disease.md`, `spec/entity/phenotype.md`, and `spec/surface/discover.md` show
**no diff at all** and remain in `SPEC_LIVE_PATHS` with all nineteen of their live blocks
intact.

That is a defensible instinct — the constraint is real, since a lane is assigned per file and
these three files hold both converted and unconverted blocks. But the result is that this
ticket **added nine assertions and converted nothing**. The live surface is exactly the size
it was. A conversion ticket that leaves `SPEC_LIVE_PATHS` unchanged has not done its job.

### The repo already solved this, three times

`SPEC_LIVE_PATHS` currently contains `article-assets-live.md`, `article-graph-live.md`,
`variant-myvariant-live.md`, and `variant-articles-live.md`. Those are not incidental names.
They are the residue of previous conversions, and the shape is consistent:

- **The original file keeps its name and moves to `SPEC_ROUTINE_PATHS`**, holding the
  converted, fixture-backed assertions. `spec/entity/article.md` and
  `spec/entity/variant.md` both sit in the routine lane today for exactly this reason.
- **The still-live remainder is split out into `<name>-live.md`**, which stays in
  `SPEC_LIVE_PATHS`.

`spec/entity/article-assets-live.md` was created by a **design** step
(*"design: land spec tests for linked article supplements"*), which also settles the
authority question: the split belongs to design, not to code and not to code review.

### Ruling

**Rewound to `01-design`.** Produce the split in the established shape:

1. `spec/entity/disease.md`, `spec/entity/phenotype.md`, and `spec/surface/discover.md`
   move to `SPEC_ROUTINE_PATHS`, holding the nine fixture-backed assertions.
2. The live blocks in those files that this ticket does **not** convert move verbatim into
   `spec/entity/disease-live.md`, `spec/entity/phenotype-live.md`, and
   `spec/surface/discover-live.md`, which are added to `SPEC_LIVE_PATHS`. Moving a block
   between lanes is relocation, not deletion — every assertion still runs, and none may be
   weakened or dropped in transit.
3. **Do not create `spec/entity/ontology-contracts.md`.** Its nine assertion blocks are
   correct and already reviewed; reuse their bytes at the permanent paths above.
4. `SPEC_LIVE_PATHS` must be measurably smaller in content when this ticket merges. State in
   the design how many live blocks existed before and how many remain, and name each one
   that is retired.
5. Everything already settled stands unchanged: the `SCENAR therapy` re-seed from the second
   ruling, the eight other assertions, the capture inventory, and the ontology fixture.
   Preserve the captures at `/tmp/biomcp-666-captures` and the fixture work already staged.

### Note

This is the third time today a step invented a shape the repository had already established
— after the imagined CAR/LDH route list and the invented no-match seed. The design flow now
says to look in the repo first; that instruction clearly needs to cover file layout and lane
partitioning too, not just query seeds. I have extended it.

## Operator ruling, fourth pass — 2026-08-06: captures must be produced by the production request path

**The abort is upheld on both counts. This is the most important defect this ticket has
surfaced and the rule it establishes applies to every remaining conversion.**

### What is wrong

The receipted MyDisease capture records this request:

```
https://mydisease.info/v1/query?q=chronic%20myeloid%20leukemia&size=5&from=0&fields=…
```

A bare `q`, `size=5`. Production does not issue that. `MyDiseaseClient` builds a **scoped**
query — wrapping the term and appending `AND` clauses for source, inheritance, phenotype and
clinical-course filters (`src/sources/mydisease.rs:215-256`) — and the entity layer requests
`size=15`. The capture and the runtime describe two different requests, so strict fixture
routing correctly refuses to serve it.

The capture was therefore **hand-constructed**, not produced by driving the production
request path. The bytes are real MyDisease bytes; the *request* is not one this program
makes.

### The over-edit

The only `src/` change in this entire ticket is:

```rust
-    const MAX_RESOLVED_TERMS: usize = 10;
+    const MAX_RESOLVED_TERMS: usize = 20;
```

in `resolve_phenotype_query_terms` (`src/entities/disease/search.rs`). A production cap was
doubled so the runtime would fit a capture. That is backwards, and code review was right to
refuse it. **Revert it.** If widening that cap is genuinely correct behavior, it is a
separate ticket with its own justification and its own proof — not a side effect of a
fixture that does not match.

### Ruling

1. **Re-capture every affected body by driving the production request path.** Point the
   source base URL at a recording proxy and run the real CLI command, or capture through the
   client itself. Do not hand-build a URL and do not adjust a recorded request to match.
2. **The recorded request in the receipt must be byte-identical to what production sends** —
   same scoping, same `size`, same `from`, same `fields`, same order. If a receipt's request
   string cannot be reproduced by running the program, the capture is invalid regardless of
   how real its bytes are.
3. **Revert `MAX_RESOLVED_TERMS` to 10.** No `src/` change belongs in this ticket; it is a
   spec-and-testdata conversion. If the recapture then exceeds a cap, that is a finding about
   the capture's seed, not a licence to widen the runtime.
4. **Audit the other five providers the same way.** Monarch, HPO, OLS4, NIH Reporter and SEER
   captures must each be checked for the same defect before review resumes. Report how many
   were hand-built.
5. Repairs 7, 8 and 9 from the design review stand and must still be made.

### On this being the fourth abort

I said I would speak plainly if 666 aborted a fourth time, so: **it is not thrashing, and I
am not splitting it.**

665 was split because it produced the *same* abort four times with no forward motion. 666's
four aborts have four distinct root causes, each fixed once and never seen again, and the
ticket has advanced a step each time — assertions authored, seed corrected, the `-live.md`
split landed, the fixture built, and a ten-finding review completed. Splitting now would
throw away a nearly finished code step to solve a problem the ticket does not have.

What is true is that **666 is roughly three times the size of the conversions that ship on
the first attempt.** 663 covered one surface and two providers; 664 covered two providers.
666 covers three surfaces and six providers and one new fixture. That is the real lesson,
and it is a lesson about how the remaining tickets were scoped, not about this one's
execution. I will check the queue for the same over-scoping rather than act on it here.
