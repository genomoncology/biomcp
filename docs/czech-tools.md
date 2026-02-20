# Czech Healthcare Tools / České zdravotnické nástroje

CzechMedMCP extends BioMCP with 14 Czech healthcare MCP tools across 5 modules.

CzechMedMCP rozšiřuje BioMCP o 14 českých zdravotnických MCP nástrojů v 5 modulech.

## Modules / Moduly

| Module | Tools | Source / Zdroj |
|--------|-------|----------------|
| SUKL | 5 | State Institute for Drug Control / Státní ústav pro kontrolu léčiv |
| MKN-10 | 3 | Czech ICD-10 / Mezinárodní klasifikace nemocí |
| NRPZS | 2 | National Registry of Healthcare Providers / Národní registr poskytovatelů |
| SZV | 2 | Health Procedure List / Seznam zdravotních výkonů |
| VZP | 2 | General Health Insurance codebooks / Číselníky VZP |

---

## SUKL - Drug Registry / Registr léčiv

### sukl_drug_searcher

Search Czech drug registry by name, active substance, or ATC code.

Vyhledávání v registru léčiv podle názvu, účinné látky nebo ATC kódu.

```bash
# CLI
czechmedmcp czech sukl search --query "Ibuprofen"
czechmedmcp czech sukl search --query "M01AE01"
```

**Parameters / Parametry:**
- `query` (required): Drug name, active substance, or ATC code
- `page` (optional, default=1): Page number
- `page_size` (optional, default=10): Results per page (1-100)

### sukl_drug_getter

Get full drug details by SUKL code including composition and registration info.

Získání podrobností o léku podle SUKL kódu včetně složení a registračních údajů.

```bash
czechmedmcp czech sukl get "0001234"
```

**Parameters / Parametry:**
- `sukl_code` (required): 7-digit SUKL identifier

### sukl_spc_getter

Get Summary of Product Characteristics (SmPC) document URL for a drug.

Získání URL dokumentu Souhrn údajů o přípravku (SPC) pro lék.

```bash
czechmedmcp czech sukl spc "0001234"
```

### sukl_pil_getter

Get Patient Information Leaflet (PIL) document URL for a drug.

Získání URL Příbalové informace (PIL) pro lék.

```bash
czechmedmcp czech sukl pil "0001234"
```

### sukl_availability_checker

Check current market availability of a Czech drug.

Kontrola aktuální dostupnosti léku na trhu.

```bash
czechmedmcp czech sukl availability "0001234"
```

**Returns / Vrací:** Status `available`, `limited`, or `unavailable` with timestamp.

---

## MKN-10 - Diagnosis Codes / Kódy diagnóz

### mkn_diagnosis_searcher

Search Czech ICD-10 (MKN-10) diagnoses by code or free text.

Vyhledávání diagnóz MKN-10 podle kódu nebo volného textu.

```bash
czechmedmcp czech mkn search --query "J06.9"
czechmedmcp czech mkn search --query "infarkt"
```

**Parameters / Parametry:**
- `query` (required): MKN-10 code or Czech text
- `max_results` (optional, default=10): Maximum results (1-100)

### mkn_diagnosis_getter

Get full diagnosis details including hierarchy (chapter, block, category).

Získání podrobností diagnózy včetně hierarchie (kapitola, blok, kategorie).

```bash
czechmedmcp czech mkn get "J06.9"
```

### mkn_category_browser

Browse MKN-10 category hierarchy. Omit code to see chapters.

Procházení hierarchie kategorií MKN-10. Bez kódu zobrazí kapitoly.

```bash
czechmedmcp czech mkn browse
czechmedmcp czech mkn browse "J00-J06"
```

---

## NRPZS - Healthcare Providers / Poskytovatelé zdravotních služeb

### nrpzs_provider_searcher

Search Czech healthcare providers by name, city, or specialty.

Vyhledávání poskytovatelů zdravotních služeb podle názvu, města nebo odbornosti.

```bash
czechmedmcp czech nrpzs search --city "Praha" --specialty "kardiologie"
czechmedmcp czech nrpzs search --query "Nemocnice"
```

**Parameters / Parametry:**
- `query` (optional): Provider name
- `city` (optional): City name
- `specialty` (optional): Medical specialty
- `page`, `page_size`: Pagination

### nrpzs_provider_getter

Get full provider details including workplaces and contact info.

Získání podrobností poskytovatele včetně pracovišť a kontaktních údajů.

```bash
czechmedmcp czech nrpzs get "12345"
```

---

## SZV - Health Procedures / Zdravotní výkony

### szv_procedure_searcher

Search Czech health procedures by code or name.

Vyhledávání zdravotních výkonů podle kódu nebo názvu.

```bash
czechmedmcp czech szv search --query "EKG"
czechmedmcp czech szv search --query "09513"
```

### szv_procedure_getter

Get full procedure details including point value, time, and specialty codes.

Získání podrobností výkonu včetně bodové hodnoty, času a kódů odborností.

```bash
czechmedmcp czech szv get "09513"
```

---

## VZP - Insurance Codebooks / Číselníky pojišťovny

### vzp_codebook_searcher

Search VZP insurance codebook entries.

Vyhledávání v číselnících VZP.

```bash
czechmedmcp czech vzp search --query "antibiotika"
czechmedmcp czech vzp search --query "EKG" --type "seznam_vykonu"
```

**Codebook types / Typy číselníků:** `seznam_vykonu`, `diagnoza`, `lekarsky_predpis`, `atc`

### vzp_codebook_getter

Get codebook entry details by type and code.

Získání podrobností položky číselníku podle typu a kódu.

```bash
czechmedmcp czech vzp get "seznam_vykonu" "09513"
```

---

## Data Sources / Zdroje dat

| Source | URL | Data |
|--------|-----|------|
| SUKL DLP API v1 | prehledy.sukl.cz | Drug registry, SmPC, PIL, availability |
| NRPZS API | nrpzs.uzis.cz | Healthcare providers |
| MKN-10 ClaML | mkn10.uzis.cz | ICD-10 diagnosis codes (local XML) |
| NZIP Open Data v3 | nzip.cz | Health procedures |
| VZP | vzp.cz | Insurance codebooks |

All data sources are public Czech government APIs. No authentication required.

Všechny zdroje dat jsou veřejná API české státní správy. Autentizace není vyžadována.

## Diacritics / Diakritika

All search tools support transparent diacritics handling. Searching for "leky" finds "léky", "Usti" finds "Ústí".

Všechny vyhledávací nástroje podporují transparentní práci s diakritikou. Vyhledání "leky" najde "léky", "Usti" najde "Ústí".
