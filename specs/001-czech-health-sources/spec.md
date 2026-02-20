# Feature Specification: CzechMedMCP - Czech Healthcare Data Sources

**Feature Branch**: `001-czech-health-sources`
**Created**: 2026-02-17
**Status**: Draft
**Input**: Fork BioMCP (MIT) rozšířený o 14 MVP českých zdravotnických MCP nástrojů pro SUKL, MKN-10, NRPZS, SZV a VZP (4 další deferred to v1.1)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Drug Information Lookup (Priority: P1)

As a physician using an AI assistant integrated with Medevio, I want to search for
Czech drugs by name, active substance, or ATC code, retrieve full drug details
including registration data, read the SmPC (Summary of Product Characteristics)
and PIL (Patient Information Leaflet), and check whether a drug is currently
available on the Czech market.

This is the highest-priority story because drug information lookup is the most
frequent clinical query. 4,000+ Medevio physicians need immediate access to
authoritative SUKL drug data through their AI assistant.

**Why this priority**: Drug queries represent the highest-volume use case for
physicians. Availability checks and SmPC access directly support prescribing
decisions and patient safety.

**Independent Test**: Can be fully tested by issuing drug search queries (e.g.,
"Ibuprofen", ATC code "M01AE01") through the MCP interface and verifying that
results match SUKL registry data. Delivers standalone value for any physician
needing drug information.

**Acceptance Scenarios**:

1. **Given** a connected MCP client, **When** a physician searches for a drug
   by trade name (e.g., "Nurofen"), **Then** the system returns a list of
   matching drugs with SUKL codes, active substances, strengths, and
   pharmaceutical forms within 2 seconds
2. **Given** a valid SUKL code, **When** a physician requests drug details,
   **Then** the system returns complete registration data including
   marketing authorization holder, registration number, and ATC classification
3. **Given** a valid SUKL code, **When** a physician requests the SmPC,
   **Then** the system returns the full Summary of Product Characteristics
   text as provided by SUKL
4. **Given** a valid SUKL code, **When** a physician requests the PIL,
   **Then** the system returns the Patient Information Leaflet content
5. **Given** a valid SUKL code, **When** a physician checks drug availability,
   **Then** the system returns the current market availability status from
   the SUKL distribution database
6. **Given** a search with no matching results, **When** the query is submitted,
   **Then** the system returns an empty result set with a clear message
   indicating no drugs were found

---

### User Story 2 - Diagnosis Code Lookup (Priority: P2)

As a physician, I want to search the Czech localization of ICD-10 (MKN-10) by
code or free text, view diagnosis details including the hierarchical category
structure, and browse categories and subcategories for correct coding.

**Why this priority**: Correct MKN-10 coding is mandatory for Czech healthcare
documentation, insurance claims, and reporting. Physicians frequently need to
verify codes or find the right code for a condition.

**Independent Test**: Can be tested by searching for diagnoses (e.g., "J06.9",
"angina") and verifying results match the official Czech MKN-10 classification.
Delivers standalone value for any clinical coding task.

**Acceptance Scenarios**:

1. **Given** a connected MCP client, **When** a physician searches for a
   diagnosis by code (e.g., "J06.9"), **Then** the system returns the
   diagnosis name, description, and hierarchy (chapter > block > category)
2. **Given** a free-text query (e.g., "akutni infarkt myokardu"), **When**
   submitted, **Then** the system returns matching MKN-10 codes ranked
   by relevance
3. **Given** a valid MKN-10 category code (e.g., "J00-J06"), **When** a
   physician browses that category, **Then** the system returns all
   subcategories and their descriptions
4. **Given** an invalid or non-existent MKN-10 code, **When** queried,
   **Then** the system returns a clear error indicating the code is invalid

---

### User Story 3 - Healthcare Provider Search (Priority: P3)

As a physician, I want to search the National Registry of Healthcare Providers
(NRPZS) by name, city, or medical specialty to find facilities for patient
referrals, and retrieve full provider details including workplaces and contact
information.

**Why this priority**: Provider search supports the patient referral workflow.
Physicians need to find specialists, labs, or hospitals that accept patients
and are geographically accessible.

**Independent Test**: Can be tested by searching for providers (e.g.,
"kardiologie Praha") and verifying results match the NRPZS registry.
Delivers standalone referral support value.

**Acceptance Scenarios**:

1. **Given** a connected MCP client, **When** a physician searches for
   providers by specialty and city (e.g., "kardiologie" in "Brno"),
   **Then** the system returns matching providers with names, addresses,
   and specialties
2. **Given** a valid provider identifier, **When** details are requested,
   **Then** the system returns the full provider record including all
   workplaces, operating hours, and contact information
3. **Given** a search with no results, **When** the query is submitted,
   **Then** the system returns an empty set with a clear message

---

### User Story 4 - Health Procedure and Reimbursement Lookup (Priority: P4)

As a physician or clinic administrator, I want to search the Czech List of
Health Procedures (SZV) by code or name, view procedure details including
point values, and look up VZP (General Health Insurance Company) codebook
entries to understand reimbursement rules.

**Why this priority**: Procedure and reimbursement lookup supports billing
accuracy and financial planning. While not as time-critical as drug or
diagnosis queries, it is essential for administrative workflows.

**Independent Test**: Can be tested by searching for procedures (e.g.,
code "09513" or "EKG") and VZP codebook entries, and verifying results match
official published data. Delivers standalone billing support value.

**Acceptance Scenarios**:

1. **Given** a connected MCP client, **When** a user searches for a procedure
   by code or name, **Then** the system returns matching procedures with
   codes, names, and point values
2. **Given** a valid procedure code, **When** details are requested, **Then**
   the system returns the full procedure record including point value,
   time allocation, material requirements, and specialty restrictions
3. **Given** a connected MCP client, **When** a user searches VZP codebooks
   by keyword, **Then** the system returns matching codebook entries
4. **Given** a valid codebook entry identifier, **When** details are requested,
   **Then** the system returns the full entry with description and applicable
   rules

---

### User Story 5 - Cross-Domain Clinical Query (Priority: P5)

As a physician, I want to combine Czech healthcare data with BioMCP's existing
global biomedical tools in a single MCP session, such as looking up a drug in
SUKL and then finding related clinical trials in ClinicalTrials.gov, or mapping
a Czech MKN-10 code to global variant data.

**Why this priority**: The unique value proposition of CzechMedMCP is the
combination of Czech and global biomedical data. This story validates the
dual-use architecture.

**Independent Test**: Can be tested by using Czech tools (e.g., SUKL drug
search) and global tools (e.g., trial search, variant lookup) in the same
MCP session and verifying both work correctly. Delivers the "whole is greater
than the sum of parts" value.

**Acceptance Scenarios**:

1. **Given** a connected MCP client with all tools registered, **When** a
   physician uses a SUKL tool followed by a BioMCP trial search tool in
   the same session, **Then** both tools return valid results without
   interference
2. **Given** all 14 MVP Czech tools and 21 global tools, **When** the MCP server
   starts, **Then** all 35 tools are registered and discoverable by the client
3. **Given** a running server, **When** global BioMCP tools are invoked,
   **Then** they function identically to upstream BioMCP (no regression)

---

### Edge Cases

- What happens when any external API (SUKL, NRPZS, SZV, VZP) is unavailable?
  System MUST return cached results from diskcache if available, or a clear
  error message indicating temporary unavailability. All modules share the
  same uniform fallback pattern: diskcache + graceful error.
- What happens when ClaML XML data for MKN-10 is corrupted or missing? System
  MUST report a startup error and refuse to serve MKN-10 queries rather than
  returning incorrect data
- What happens when a physician searches with diacritics vs. without (e.g.,
  "léky" vs "leky")? Search MUST handle both forms and return equivalent results
- What happens when NRPZS returns providers that no longer operate? Data
  reflects the registry state; system does not filter beyond what NRPZS provides
- What happens when VZP codebook format changes between versions? System MUST
  detect version incompatibility and log a warning rather than silently returning
  incorrect data
- What happens when upstream APIs rate-limit or throttle requests? System MUST
  use BioMCP's existing resilience stack (rate limiter at 10 req/s per-domain,
  circuit breaker, exponential retry with backoff) to protect against upstream
  rate limiting and gracefully degrade under load

## Clarifications

### Session 2026-02-17

- Q: Jak se má systém chovat při selhání externích API (NRPZS, SZV, VZP)? → A: Uniform fallback - všechny moduly používají diskcache + graceful error při nedostupnosti
- Q: Jak se ClaML XML soubor pro MKN-10 dostane do systému? → A: Lazy download - stažení z UZIS při prvním použití, uložení do diskcache
- Q: Jaký je přístup k datům SZV (Seznam zdravotních výkonů)? → A: Primárně NZIP otevřená data (CSV/API v3) pro výkony a statistiky; szv.mzcr.cz jako fallback pro registrační listy a číselníky (HTML); VZP ČR pro aktuální bodové hodnoty
- Q: Jak se má systém chránit před rate limitingem upstream API? → A: Reuse BioMCP resilience stack - rate limiter (10 req/s default) + circuit breaker + exponential retry per-domain
- Q: Jaká observabilita je potřeba pro české moduly? → A: Reuse BioMCP - @track_performance() na každém toolu + Python logging modul, žádné extra health checks pro MVP

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST expose 14 MVP Czech healthcare tools via the MCP protocol
  (5 SUKL + 3 MKN-10 + 2 NRPZS + 2 SZV + 2 VZP)
- **FR-002**: System MUST retain all 21 existing BioMCP tools without modification
  to their behavior or interfaces
- **FR-003**: System MUST support drug search by trade name, active substance name,
  and ATC code through the SUKL module
- **FR-004**: System MUST provide drug detail retrieval by SUKL code including
  registration data, SmPC text, and PIL text
- **FR-005**: System MUST report current drug market availability from SUKL
  distribution data
- **FR-006**: System MUST download the Czech MKN-10 ClaML XML file from UZIS
  on first use (lazy download), cache it locally via diskcache, parse and index
  it in memory, and support both code-based and free-text search
- **FR-007**: System MUST support hierarchical browsing of MKN-10 categories
  (chapter > block > category > subcategory)
- **FR-008**: System MUST search NRPZS providers by name, city, and medical
  specialty
- **FR-009**: System MUST retrieve full provider details including workplaces
  from NRPZS
- **FR-010**: System MUST search SZV procedures by code or name and return
  point values, using NZIP Open Data API v3 (nzip.cz) as primary data source
  with szv.mzcr.cz as supplementary source for registration sheets
- **FR-011**: System MUST search and retrieve VZP codebook entries
- **FR-012**: System MUST handle Czech diacritics in search queries transparently
  (queries with and without diacritics MUST return equivalent results)
- **FR-013**: System MUST expose all Czech tools via CLI commands mirroring the
  MCP tool interface
- **FR-014**: System MUST cache external API responses using diskcache for all
  modules (SUKL, NRPZS, SZV, VZP) to reduce latency on repeated queries and
  provide fallback data when upstream APIs are unavailable
- **FR-015**: System MUST attribute all returned data to its authoritative source
  (SUKL, UZIS, NRPZS, VZP, MZCR)
- **FR-016**: System MUST provide bilingual documentation (Czech and English)

### Key Entities

- **Drug (Lek)**: A medicinal product registered with SUKL. Key attributes:
  SUKL code, trade name, active substance(s), strength, pharmaceutical form,
  ATC code, marketing authorization holder, registration number, market
  availability status
- **Diagnosis (Diagnoza)**: An MKN-10 classification entry. Key attributes:
  MKN-10 code, Czech name, English name, category hierarchy (chapter, block,
  category), inclusion/exclusion notes
- **Healthcare Provider (Poskytovatel)**: An entity registered in NRPZS.
  Key attributes: provider name, legal form, address, medical specialties,
  workplaces, registration number
- **Health Procedure (Zdravotni vykon)**: A billable medical procedure from SZV.
  Key attributes: procedure code, name, point value, time allocation, specialty
  restriction, material requirements
- **Codebook Entry (Polozka ciselniku)**: A VZP insurance codebook item.
  Key attributes: codebook type, entry code, description, validity period,
  applicable rules

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Physicians can search for any registered Czech drug and receive
  results within 2 seconds (first query) or 100 milliseconds (cached)
- **SC-002**: All 14 MVP Czech tools are discoverable and functional from
  Claude Desktop, Cursor, and VS Code MCP clients
- **SC-003**: All 21 existing BioMCP tools continue to function identically
  after the Czech extensions are added (zero regression)
- **SC-004**: MKN-10 diagnosis search returns correct results for at least
  95% of queries when benchmarked against official UZIS code tables
- **SC-005**: Test coverage for Czech modules reaches at least 80%
- **SC-006**: The system can serve requests from at least 10 concurrent MCP
  clients without degradation
- **SC-007**: ClaML XML parsing completes within 5 seconds on cold start
- **SC-008**: Documentation covers all Czech tools in both Czech and English
  with usage examples

## Assumptions

- SUKL provides a publicly accessible REST API or downloadable data (DLP)
  that does not require special authorization for read-only access
- The Czech MKN-10 ClaML XML file is available for download from UZIS
  (lazy download at first use, cached locally) and its format is stable
  across minor revisions
- NRPZS provides a publicly accessible search API with no authentication
  required for basic queries
- VZP codebooks are published as downloadable CSV/XML files that can be
  parsed without proprietary tools
- SZV (List of Health Procedures) data is available from multiple sources:
  (1) NZIP Open Data API v3 (nzip.cz/data/2627, nzip.cz/data/1744) in
  CSV/JSON format for automated access, (2) szv.mzcr.cz/Vykon for
  structured HTML browsing of registrational sheets, (3) szv.mzcr.cz/Ciselnik
  for code lists (specialties, categories, carriers), (4) VZP codebooks at
  vzp.cz/poskytovatele/ciselniky/zdravotni-vykony for current point values
- BioMCP upstream project remains MIT-licensed and structurally compatible
  with the fork approach described

## Scope Boundaries

**In scope for MVP (v1.0)**:
- 14 Czech MCP tools (5 SUKL, 3 MKN-10, 2 NRPZS, 2 SZV, 2 VZP)
- CLI commands mirroring all Czech MCP tools
- Unit and integration tests for Czech modules
- Docker deployment support
- Bilingual documentation

**Deferred to v1.1**:
- sukl_batch_checker (batch availability for up to 50 drugs)
- mkn_code_validator (MKN-10 code validation with Czech specifics)
- nrpzs_nearby_finder (GPS-based proximity search)
- szv_reimbursement_checker (reimbursement conditions and limits)

**Out of scope**:
- FHIR Terminology Server integration
- eRecept / eNalez integration (requires IDRR certification)
- Direct EHR system integration
- Graphical user interface
- MZCR EZ systems (EZCA, KRPZS, eZadanky)
