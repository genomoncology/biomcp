---
flow: build
priority: 1
---
# Consume BioData clinical-trial eligibility

## Outcome

BioMCP consumes BioData `0.0.9` clinical-trial eligibility directly for ClinicalTrials.gov and NCI. One shared value owns registry text, age bounds, source-coded sex, healthy-subject state, and ordered identified criteria. JSON exposes those discrete facts. Markdown remains readable and bounded. BioMCP removes its duplicate eligibility parsing, formatting storage, and product model.

## Evidence and decisions

BioData record 0102 landed at `685a830aa634545fae1a93d2025717de30cbb348`. Its plan-bound adapters preserve recorded eligibility from both providers through `ClinicalTrialEligibility`. ClinicalTrials.gov carries registry text, age, sex, and healthy-subject state. NCI carries age, sex, healthy-subject state, and every ordered criterion with a deterministic occurrence identity and an extensible classification.

BioMCP currently stores the same area three ways on `Trial`: `age_range`, `eligibility_text`, and a local `TrialEligibility` with one sex string and two local age values. The NCI detail path also converts BioData eligibility back into those local values and formats structured criteria into text. The ClinicalTrials.gov path parses eligibility through both BioData and the legacy wire. `NciCtsV2DetailPlan::new` requests eligibility for every detail call even when the caller did not ask for it.

Replace all three product fields with `eligibility: Option<biodata::ClinicalTrialEligibility>`. Keep `TrialEligibilityProvenance` because document retrieval and follow-up presentation belong to BioMCP. Default detail, explicit `eligibility`, and `all` request the BioData eligibility fields for both providers. Default Markdown derives its existing age line from the aggregate without showing the full eligibility section. Default JSON exposes the richer aggregate. This preserves the machine-enforced default-detail age capability without restoring a root age alias. The cost is a wider default ClinicalTrials.gov field request. Unrelated named section-only requests do not request or store eligibility.

Use the BioData aggregate as the stored field type. Do not add a product eligibility wrapper. A private field-level Serde adapter may use borrowing wire views and constructor-backed decode helpers. Keep those helpers local to the trial entity. Default JSON exposes the complete aggregate whenever eligibility is present. The JSON `eligibility` object always contains the five members `registry_text`, `age_range`, `sexes`, `includes_healthy_subjects`, and `criteria`, including null values. A present age range always contains nullable `minimum` and `maximum`. Every code contains `authority`, `code`, `display`, `vocabulary_version`, and `recognized_meaning`, including null optional values. A limited age bound contains exactly `kind`, `source`, `source_quantity`, `source_unit`, and `bound`. A source-stated no-limit bound contains those members plus `"rule":{"name":"nci-cts-v2-999-years-no-upper-bound","version":"1"}` and rejects every other rule value. Every criterion contains exactly `id`, `description`, and `classification`. Known classification objects contain only `kind`. `other` contains `kind` and the full `source` code. Deserialization reconstructs every value through BioData constructors and rejects invalid or duplicate identities.

JSON preserves complete registry text and every criterion. Apply the existing 12,000-character bound only during Markdown rendering. Markdown renders source-stated sex values as readable labels, source age bounds, a clear healthy-subject line when the source supplies the value, exact registry text, and one linear criterion sequence. Emit a heading whenever the criterion classification changes. Never regroup or sort in BioMCP. Render `Other` under a heading that includes both its source authority and code. Keep the existing eligibility heading and cautious ClinicalTrials.gov posted-document follow-up.

Pin the BioData dependency and boundary contract to exact commit `685a830aa634545fae1a93d2025717de30cbb348`. Do not publish.

## Provider flow

ClinicalTrials.gov builds the existing BioData detail plan. Default detail, `eligibility`, and `all` add the five BioData eligibility fields. Other named section-only requests add none. Read the plan-bound `response.shared.eligibility()` section. `Present` supplies the direct value, `Absent` supplies `None`, and unexpected `NotRequested` or `Unavailable` returns the existing sanitized internal-processing error when eligibility was requested. The legacy `CtGovEligibilityModule` keeps only `eligibility_criteria`, `minimum_age`, and `maximum_age` for mutation verification and age filtering. `NormalizedTimeWire` and the old age parser remain search-only. Remove legacy `sex` and every detail-path consumer. Add a structural test that detail conversion cannot read the legacy eligibility module. Record that the remaining search-only pieces leave during search-summary and filter migration.

NCI passes the computed request state to `NciCtsV2DetailPlan::new`. Default detail, `eligibility`, and `all` request eligibility. Other named section-only requests do not. Read `response.eligibility()` directly with the same section-state rules. Remove `NciCtsV2Eligibility`, `nci_eligibility_text`, `product_eligibility`, and duplicate age conversion from the product path.

ClinicalTrials.gov emits `eligibility_provenance` only for an explicit `eligibility` request when the direct aggregate is present and has registry text. Default detail, `all`, present eligibility without registry text, absent eligibility, and every NCI response omit it. The existing posted-document data still supplies document availability and the follow-up handle. Add exact tests for each state. This preserves the current `all` behavior.

Apart from the eligibility field selection described above, provider request behavior remains unchanged. Response-size limits, cache, retry, timeout, credential, status, source-selection, and safe-error behavior remain unchanged. A malformed provider eligibility value keeps the current sanitized BioData response-validation boundary. No response bytes, criterion text, local identity, URL, or credential enters a public error.

## Observable contract

ClinicalTrials.gov JSON eligibility gains `includes_healthy_subjects` and source-coded sex. Registry text moves from root `eligibility_text` into `eligibility.registry_text`. Age moves from root `age_range` and local `minimum_age` or `maximum_age` into `eligibility.age_range`.

NCI JSON gains `includes_healthy_subjects` and every discrete ordered criterion with its occurrence ID and classification. It no longer exposes a second generated criteria string. Markdown still presents readable criteria and age. This output is semantically richer than the removed shape.

Update executable specifications and public documentation to show the new paths. State that default detail preserves the age summary while the explicit eligibility section exposes the full eligibility presentation and ClinicalTrials.gov provenance. Update typed MCP and raw CLI contracts together. Update section-source provenance detection to use the shared aggregate without growing its file. Keep search filters and their legacy eligibility-text verification unchanged in this ticket.

## Tests

Start with a failing test that requires `Trial.eligibility` to accept `ClinicalTrialEligibility` directly and rejects the removed local fields or type.

Use original recorded bytes for both providers. Assert exact ClinicalTrials.gov registry text, age bounds, sex authority and code, and false healthy-subject state. Assert exact NCI age, sex authority and code, false healthy-subject state, complete criterion count, source order after display sorting, occurrence IDs, and classifications. Mutate registry text, sex, healthy-subject state, NCI criterion display order, and classification in original bytes. Require changed JSON or the current sanitized validation failure. Prove occurrence IDs remain attached to original rows after sorting.

Test request composition exactly. Default detail, `eligibility`, and `all` request provider eligibility once. Every unrelated single section request requests none. NCI must stop requesting eligibility for unrelated section-only calls.

Test the direct JSON adapter for missing eligibility, null eligibility, all-null aggregate members, explicit empty sexes and criteria, false, all known classifications, `Other`, Unicode, deterministic round trips, zero IDs, duplicate IDs, blank values, malformed codes, malformed ages, missing members, duplicate members, unknown members, and wrong types. Pin exact output paths. The no-limit round trip must emit the exact named rule and malformed-age tests must reject every other rule value. Test that complete registry text survives JSON while Markdown truncates it safely.

Test Markdown for both recorded providers. Preserve the default age line, eligibility heading, readable source label, age display, criteria order, and posted-document guidance. Add exact healthy-subject output. Test heading transitions without regrouping and test `Other` with visible authority and code. Test absent and present-empty aggregates without panic or invented claims. Prove exact provenance behavior for explicit eligibility with text, present eligibility without text, absent eligibility, default detail, `all`, and NCI.

Run real CLI and MCP paths against the existing local provider servers. Require raw CLI JSON and typed MCP to expose the same structured eligibility. Preserve not-found, malformed-provider, response-limit, cache, retry, credential, and public error behavior. Update package and documentation contracts without increasing the 1,300-file package ceiling.

Run `make lint`, `make test`, `make spec`, and the existing full-feature check required for a dependency change. Net production Rust growth must stay at or below 278 lines. This keeps the two-area arms-plus-eligibility checkpoint at or below its 500-line allowance. Do not increase large-file, source-line, package-file, MCP catalog, or other quality ceilings. Split a cohesive module or delete obsolete code if a current ceiling is reached.

## Acceptance

1. `Trial` stores one direct `Option<biodata::ClinicalTrialEligibility>`. Root `age_range`, root `eligibility_text`, local `TrialEligibility`, and NCI eligibility formatting conversion are gone.
2. BioData is pinned to exact `0.0.9` commit `685a830aa634545fae1a93d2025717de30cbb348` in Cargo and the package boundary contract.
3. Both provider detail paths request eligibility for default detail, `eligibility`, and `all`. Other named section-only requests do not. Default Markdown preserves the existing age line without a root age field. Exact successful response bytes reach BioData, and request-aware section states map without a second owner.
4. JSON exposes all five aggregate members and reconstructs the direct BioData value through its public constructors. It preserves absence, explicit empties, false, Unicode, occurrence identities, and extensible classifications.
5. Recorded ClinicalTrials.gov output preserves text, age, sex, and healthy-subject state. Recorded NCI output preserves age, sex, healthy-subject state, and every ordered identified criterion.
6. Markdown derives only from the shared value. It remains bounded and readable for both providers, renders unknown classifications safely, and keeps ClinicalTrials.gov document provenance.
7. Raw CLI and typed MCP agree on the new structured JSON. The executable specs and public docs name the new paths and do not promise removed root fields.
8. Search eligibility verification, source selection, transport, cache, retries, credentials, limits, not-found behavior, and sanitized errors remain unchanged.
9. No wrapper, generic migration framework, second provider call, live capture, proprietary extraction, matching, scoring, interpretation, release, or unrelated cleanup enters this ticket.
10. Net production Rust growth is no more than 278 lines. The package remains at or below 1,300 files. No quality ceiling increases.
11. Independent design and code reviews accept the result. `make lint`, `make test`, `make spec`, and the dependency full-feature check pass.

## Dependencies

BioData record 0102 and BioMCP records 1175 through 1177 are complete. Both Factory channels remain paused. The manual subagent SDLC owns this work. The two-area checkpoint starts after this ticket lands.

## Review

- Manual approval: Ian approved both provider sources, direct BioData adoption, output improvement instead of byte parity, and implementation through the subagent SDLC without Factory.
- Design review: rejected once pending correction. The first revision removed the machine-enforced default-detail age line, left provenance and criterion ordering ambiguous, did not name the search-only legacy wire, and referred to BioData's strict shapes without spelling out BioMCP's owned JSON contract. The revised ticket preserves default age through the one shared aggregate, pins exact request and provenance states, retains only named search consumers, requires linear criterion order, and defines every emitted member.
- Implementation review: rejected three times pending correction. The first review found five missing proofs for the NCI typed MCP path, the complete recorded criterion set, the real Markdown process path, the request-state table, and absent-versus-null JSON. The second review found two test timeouts raised from 10 to 30 seconds. The third review found stale packaged schema, example, and jq contracts. Each defect received a focused red-green remediation. The final independent review accepted the complete implementation and confirmed the direct BioData owner, exact wire shape, both provider paths, restored timeouts, packaged contracts, unchanged ceilings, and the 1,300-file package limit.
- Verification: the final pre-integration revision passed `make lint`, `make test`, `make spec`, and `make full-feature-check`. The routine Python contracts passed 911 tests with 3 skipped. Strict documentation passed. Every specification suite passed. All-feature Clippy passed. The six optional AlphaGenome tests passed. The release PNG, SVG, and terminal artifact smoke passed. An independent count measured 236 net nonblank production Rust lines, below the ticket limit.
