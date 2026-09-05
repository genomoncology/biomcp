# Clinical-trial capability contract

This inventory defines the user capabilities that BioData adoption must preserve or improve. It describes operations and observable behavior. It does not require historical byte parity. A later area may change an answer when the change preserves the capability and provides equal or better information.

## Capability inventory

| ID | User operation | Sources | Observable behavior | Code owner | Executable contract | Public document |
| --- | --- | --- | --- | --- | --- | --- |
| CT-SEARCH | Search with condition, intervention, molecular, study, sponsor, eligibility, geography, status, and phase controls | CTGov and NCI with source limits below | Returns filtered trial summaries and rejects unsupported source-filter combinations | src/entities/trial/search/mod.rs | spec/entity/trial.md#Condition-First Search | docs/how-to/find-trials.md |
| CT-NCI-CONDITION | Search NCI by condition | NCI | Uses an NCI disease concept when grounding succeeds and uses the original term as a keyword otherwise | src/entities/trial/search/nci.rs | spec/entity/trial.md#NCI Condition Search | docs/user-guide/trial.md |
| CT-PAGING | Page search results and request counts | CTGov and NCI | Returns bounded result pages; CTGov also supports cursors and reports exact, approximate, or unknown counts | src/entities/trial/search/mod.rs | spec/entity/trial.md#Terminal Pagination | docs/user-guide/trial.md |
| CT-MOLECULAR | Search trial eligibility and biomarker text | CTGov and NCI with source limits below | Keeps broad discovery while checking simple CTGov mutation inclusion; NCI accepts one combined molecular value | src/entities/trial/search/eligibility.rs | spec/entity/trial.md#Simple mutation search verifies molecular inclusion | docs/user-guide/trial.md |
| CT-INTERVENTION | Search by intervention with controlled alias expansion | CTGov and NCI | CTGov exposes matched aliases and preserves requested-name results when an expanded alias fails | src/entities/trial/search/ctgov.rs | spec/entity/trial-intervention-aliases.md#Alias fanout continues after detail-backed rejection | docs/user-guide/trial.md |
| CT-DETAIL | Get one trial by NCT ID | CTGov and NCI | Returns the source trial overview in Markdown or JSON | src/entities/trial/get.rs | spec/entity/trial.md#Trial Detail & Eligibility | docs/user-guide/trial.md |
| CT-ELIGIBILITY | Request eligibility facts and text | CTGov and NCI | Returns age and sex facts plus available registry criteria; CTGov also reports document provenance | src/entities/trial/get.rs | spec/entity/trial-documents.md#Identify registry eligibility provenance | docs/user-guide/trial.md |
| CT-SITES | Request locations and contacts with location paging | CTGov; NCI currently accepts the names without a distinct unavailable state | Keeps every named site contact attached to its site and exposes continuation metadata | src/cli/trial/dispatch.rs | spec/entity/trial.md#Every Named Site Contact Reaches Its Location | docs/sources/clinicaltrials-gov.md |
| CT-DESIGN | Request outcomes, arms, interventions, and references | CTGov; NCI currently accepts the names without a distinct unavailable state | Returns available structured study-design sections and retains empty references as an explicit empty list | src/entities/trial/get.rs | spec/entity/trial.md#Source-Provided Intervention Aliases in JSON | docs/sources/clinicaltrials-gov.md |
| CT-DOCUMENTS | List and retrieve posted trial documents | CTGov only | Lists a JSON manifest and retrieves one exact advertised file as bounded raw bytes | src/entities/trial/documents.rs | spec/entity/trial-documents.md#Retrieve one posted trial document | docs/sources/clinicaltrials-gov.md |
| CT-BATCH | Get up to ten trial records in one command | CTGov and NCI | Returns independent trial detail results through the shared batch surface | src/cli/system/dispatch.rs | spec/entity/trial.md#Trial Batch Detail | docs/reference/quick-reference.md |
| CT-OUTPUT | Use readable Markdown or structured JSON | CTGov and NCI | Preserves complete structured conditions in JSON and discloses Markdown abbreviation | src/render/markdown/trial.rs | spec/entity/trial.md#Complete JSON Conditions and Disclosed Markdown Abbreviation | docs/user-guide/trial.md |
| CT-MCP | Search and get trials through raw or typed MCP tools | CTGov and NCI | Typed MCP exposes its bounded subset; raw MCP retains the read-only CLI surface | src/mcp/shell.rs | spec/surface/mcp.md#Typed Tool Schemas Are Advertised | docs/reference/mcp-server.md |
| CT-PIVOTS | Start a trial search from a gene, variant, drug, or disease command | CTGov and NCI | Accepts all four anchor-specific trial command forms and keeps the selected source | src/cli/disease/dispatch.rs<br>src/cli/drug/dispatch.rs<br>src/cli/gene/related.rs<br>src/cli/variant/dispatch.rs | spec/entity/trial.md#Cross-entity Trial Pivot Commands | docs/reference/quick-reference.md |
| CT-TRANSPORT | Execute provider requests through shared transport controls | CTGov and NCI | Applies bounded retries, response limits, cache policy, safe errors, and the selected-provider boundary | src/sources/mod.rs | spec/surface/request-plan-ratchets.md#Shared Retry-After Waits Stay Bounded | docs/reference/configuration.md |

## Source support

| Capability group | CTGov | NCI |
| --- | --- | --- |
| Condition, intervention, facility, status, phase | Yes | Yes, with the documented NCI mappings and limits |
| Mutation, criteria, biomarker | Separate controls | One quoted value total across the three controls |
| Age, sex, study type, sponsor, sponsor type, update dates, prior therapy, progression, line of therapy, posted-results filter | Yes | No; rejected before provider work |
| Offset paging | Yes | Yes |
| Cursor paging and intervention alias expansion controls | Yes | No |
| Overview and eligibility detail | Yes | Yes |
| Contacts, locations, outcomes, arms, references | Yes | Accepted by the shared command; unsupported, absent, and failed states are not distinguishable yet |
| Posted-document manifest and bytes | Yes | No |

BioMCP keeps two internal source-specific fallbacks. NCI condition search uses the original condition as a CTS keyword when disease grounding returns no usable concept or fails. CTGov intervention expansion keeps successful requested-name results when a detail-backed alias check fails. Neither fallback changes the selected trial provider. A direct operation never retries against the other provider after its selected provider fails.

The current trial detail result cannot distinguish a section that the caller did not request, a section the provider does not support, a supported section with no value, and a section whose retrieval failed. NCI makes this gap visible because the shared CLI accepts section names that its current detail transform does not populate. The partial-response work must add that distinction instead of treating accepted CLI syntax as provider support.

## Surface declarations

The tests read these declarations directly. Each list applies only to its named surface.

<!-- contract:cli-search-flags -->
```text
--age
--biomarker
--condition
--count-only
--criteria
--date-from
--date-to
--distance
--facility
--has-results
--help
--intervention
--json
--lat
--limit
--line-of-therapy
--lon
--mutation
--next-page
--no-alias-expand
--no-cache
--offset
--phase
--prior-therapies
--progression-on
--results-available
--sex
--source
--sponsor
--sponsor-type
--status
--study-type
```

<!-- contract:typed-mcp-search-fields -->
```text
biomarker
condition
criteria
entity
intervention
json
limit
mutation
offset
phase
source
status
```

<!-- contract:cli-detail-sections -->
```text
all
arms
contacts
document
documents
eligibility
locations
outcomes
references
```

<!-- contract:typed-mcp-detail-sections -->
```text
all
arms
contacts
eligibility
locations
outcomes
references
```

<!-- contract:trial-sources -->
```text
ctgov
nci
```

<!-- contract:typed-mcp-cli-only-exclusions -->
```text
document
documents
```
