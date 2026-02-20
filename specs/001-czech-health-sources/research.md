# Research: Czech Healthcare Data Sources

**Feature**: 001-czech-health-sources
**Date**: 2026-02-17

## 1. SUKL - Drug Registry API

### Decision
Use SUKL Public Data API (prehledy.sukl.cz) with OpenAPI 3.0.3 endpoints
for drug search, detail retrieval, SmPC, PIL, and availability data.

### Rationale
- REST API with JSON responses and Swagger documentation
- No authentication required for public read-only access
- OpenAPI spec available at `/dlp.api.json`
- Monthly data updates with well-defined endpoints

### Key Endpoints

| Endpoint | Purpose |
|----------|---------|
| `https://prehledy.sukl.cz/` | Base URL for public data API |
| `/dlp.api.json` | OpenAPI spec for drug database (DLP) |
| `https://prehledy.sukl.cz/docs/` | Swagger UI documentation |
| `https://opendata.sukl.cz/` | Open data portal |
| `https://api2.sukl.cz/` | Extended API portal |

### Authentication
- Public APIs: No authentication required
- Private API key available for removing rate limits (contact opendata@sukl.gov.cz)
- Certificate-based mTLS available for restricted endpoints

### Data Format
- Primary: JSON (REST API)
- Secondary: CSV (downloadable datasets)
- Swagger/OpenAPI 3.0.3 specification

### Alternatives Considered
- Direct scraping of sukl.cz website: Rejected (fragile, no structured API)
- SUKL Open Data CSV downloads only: Rejected (no real-time availability data)
- Existing SUKL MCP server (claude.ai integration): Noted as reference
  implementation, but we need our own module for full integration with
  BioMCP infrastructure (caching, circuit breaker, rate limiter)

---

## 2. MKN-10 - Czech ICD-10 Classification

### Decision
Download ClaML XML file from UZIS and parse it at startup with lxml.
Build an in-memory index for code and text search. Cache the parsed
index with diskcache.

### Rationale
- ClaML (Classification Markup Language) is the standard format
  (CEN/TS 14463, ISO 13120:2013)
- Static dataset updated annually - no real-time API needed
- In-memory indexing provides sub-millisecond search
- File is available without authentication from UZIS

### Key Resources

| Resource | URL |
|----------|-----|
| MKN-10 Browser | https://mkn10.uzis.cz/ |
| UZIS Registry | https://www.uzis.cz/index.php?pg=registry-sber-dat--klasifikace |
| About MKN-10 | https://mkn10.uzis.cz/o-mkn |

### Data Format
- ClaML XML with elements: `<ClaML>`, `<Class>`, `<SuperClass>`,
  `<SubClass>`, `<ModifierClass>`, `<Modifier>`
- Hierarchy: Chapter > Block > Category > Subcategory
- Last major update: 2018 (with supplementary updates)

### Alternatives Considered
- FHIR Terminology Server for ICD-10: Rejected for MVP (planned for v1.5)
- WHO REST API for ICD-10: Rejected (English-only, no Czech localization)
- Database storage (SQLite): Rejected (ClaML fits in memory, simpler)

---

## 3. NRPZS - Healthcare Provider Registry

### Decision
Use NRPZS REST API (nrpzs.uzis.cz/api/v1/) for provider search and
detail retrieval. Supplement with CSV bulk download for offline fallback.

### Rationale
- REST API with JSON responses and OpenAPI 2.0 documentation
- No authentication required for public queries
- Monthly updates at the beginning of each month
- API endpoint: `/api/v1/mista-poskytovani`

### Key Endpoints

| Endpoint | Purpose |
|----------|---------|
| `https://nrpzs.uzis.cz/api/v1/mista-poskytovani` | Provider search |
| `https://nrpzs.uzis.cz/api/doc` | API documentation |
| CSV download | Bulk data for offline use |

### Authentication
- No authentication required for public API
- OAS 2.0 (OpenAPI 2.0) standard documentation

### Data Format
- Primary: JSON (REST API)
- Download: CSV (bulk exports)
- Fields: facility type, service scope, contact info, care types,
  parent organization, headquarters address

### Alternatives Considered
- Scraping nrpzs.uzis.cz web interface: Rejected (REST API available)
- CSV-only approach: Rejected (API provides real-time data)

---

## 4. VZP - Insurance Codebooks

### Decision
Parse VZP codebook CSV/XML files downloaded from VZP open data.
B2B API access deferred to v1.1+ due to registration requirements.

### Rationale
- B2B API requires registration and certificate-based authentication
- Public codebook files are available in CSV/XML format
- Codebooks are updated periodically and can be cached
- For MVP, file-based parsing is sufficient

### Key Resources

| Resource | URL |
|----------|-----|
| B2B Services | https://www.vzp.cz/e-vzp/b2b-komunikace |
| Service Documentation | https://www.vzp.cz/e-vzp/informace-pro-sw-firmy |
| Codebooks for providers | Available via VZP portal |

### Data Format
- XML (SZP-VZP standard as of 2026-01-01)
- CSV (for codebook downloads)
- Message types: KDAVKA (service files), FDAVKA (period invoices)

### Alternatives Considered
- VZP B2B API: Deferred (requires enterprise registration)
- Scraping VZP website: Rejected (fragile, codebook files available)

---

## 5. SZV - Health Procedures List

### Decision
Use NZIP Open Data API v3 (nzip.cz) as primary data source for procedure
statistics and automated access. Supplement with szv.mzcr.cz for
registration sheets and code lists (HTML parsing via lxml). VZP codebooks
at vzp.cz for current point values.

### Rationale
- NZIP provides structured CSV/JSON data via Open Data API v3
- szv.mzcr.cz provides searchable HTML database for registration sheets
- szv.mzcr.cz/Ciselnik provides code lists (specialties, categories)
- VZP codebooks provide current point values and reimbursement rules
- Legal basis: Decree No. 134/1998 Sb.
- Clarified during /speckit.clarify session 2026-02-17

### Key Resources

| Resource | URL |
|----------|-----|
| NZIP Open Data (výkony dle diagnózy) | https://nzip.cz/data/2627 |
| NZIP Open Data (výkony v pojištění) | https://nzip.cz/data/1744 |
| SZV Database | https://szv.mzcr.cz/ |
| Procedure Search | https://szv.mzcr.cz/Vykon |
| Codebooks | https://szv.mzcr.cz/Ciselnik |
| Categories | https://szv.mzcr.cz/Ciselnik/Kategorie |
| VZP Zdravotní výkony | https://vzp.cz/poskytovatele/ciselniky/zdravotni-vykony |
| Ministry of Health | https://mzd.gov.cz/ |

### Data Format
- Primary: CSV/JSON via NZIP Open Data API v3
- Supplementary: HTML (structured tables at szv.mzcr.cz)
- VZP: CSV/XML codebook downloads
- Decree-based updates

### Alternatives Considered
- PDF parsing: Rejected (unreliable for structured data extraction)
- Manual data entry: Rejected (not maintainable)
- HTML scraping only: Rejected as primary (NZIP API v3 provides better structured access)

---

## 6. Integration Pattern - BioMCP Architecture

### Decision
Follow existing BioMCP module pattern exactly. Place all Czech modules
under `src/biomcp/czech/` as a sub-package. Register tools via
`@mcp_app.tool()` decorator in a new `czech_tools.py` module.

### Rationale
- BioMCP uses `@mcp_app.tool()` decorator for tool registration
  (see individual_tools.py)
- Domain modules follow pattern: `__init__.py` + `search.py` + `getter.py`
- `Annotated[type, Field(...)]` pattern for tool parameters
- `@track_performance()` decorator for metrics
- httpx async client via centralized `http_client.py`

### Integration Points (minimal changes to existing code)

| File | Change | Lines |
|------|--------|-------|
| `core.py` | Rename to "CzechMedMCP" in FastMCP name | 1 |
| `__init__.py` | Add `from . import czech` | 1 |
| `constants.py` | Add 5 Czech domains to VALID_DOMAINS | 5 |
| `domain_handlers.py` | Add 5 Czech handler classes | ~50 |
| `pyproject.toml` | Rebrand + add lxml dependency | 3-5 |

### Diacritics Handling
Use `unicodedata.normalize('NFD', text)` to strip diacritics for search
comparison while preserving original text in results. This handles
"léky" = "leky" requirement transparently.

---

## 7. Third-Party Endpoints (for THIRD_PARTY_ENDPOINTS.md)

New endpoints to document:

| Service | Base URL | Auth |
|---------|----------|------|
| SUKL DLP API | https://prehledy.sukl.cz/ | None (public) |
| SUKL Open Data | https://opendata.sukl.cz/ | None |
| NRPZS API | https://nrpzs.uzis.cz/api/v1/ | None |
| MKN-10 Data | https://mkn10.uzis.cz/ | None (download) |
| SZV Database | https://szv.mzcr.cz/ | None |
| VZP Codebooks | https://www.vzp.cz/ | CSV download |
