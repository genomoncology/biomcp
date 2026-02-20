# České zdravotnické nástroje

CzechMedMCP rozšiřuje BioMCP o 14 českých zdravotnických MCP nástrojů v 5 modulech.

## Moduly

| Modul | Nástrojů | Zdroj dat |
|-------|----------|-----------|
| SUKL | 5 | Státní ústav pro kontrolu léčiv (prehledy.sukl.cz) |
| MKN-10 | 3 | Mezinárodní klasifikace nemocí, 10. revize (mkn10.uzis.cz) |
| NRPZS | 2 | Národní registr poskytovatelů zdravotních služeb (nrpzs.uzis.cz) |
| SZV | 2 | Seznam zdravotních výkonů (nzip.cz) |
| VZP | 2 | Číselníky Všeobecné zdravotní pojišťovny (vzp.cz) |

---

## SUKL - Registr léčiv

### sukl_drug_searcher

Vyhledávání v registru léčiv podle názvu přípravku, účinné látky nebo ATC kódu.

```bash
# Vyhledání podle názvu
biomcp czech sukl search --query "Ibuprofen"

# Vyhledání podle ATC kódu
biomcp czech sukl search --query "M01AE01"

# Vyhledání s diakritikou i bez
biomcp czech sukl search --query "Paralen"
```

**Parametry:**
| Parametr | Typ | Povinný | Výchozí | Popis |
|----------|-----|---------|---------|-------|
| `query` | str | ano | - | Název léku, účinná látka nebo ATC kód |
| `page` | int | ne | 1 | Číslo stránky (od 1) |
| `page_size` | int | ne | 10 | Počet výsledků na stránku (1-100) |

**Odpověď:** JSON s polemi `total`, `page`, `page_size`, `results` (seznam `DrugSummary`).

### sukl_drug_getter

Získání podrobností o léku podle SUKL kódu včetně složení, registračních údajů a odkazů na dokumenty.

```bash
biomcp czech sukl get "0001234"
```

**Parametry:**
| Parametr | Typ | Povinný | Popis |
|----------|-----|---------|-------|
| `sukl_code` | str | ano | 7místný identifikátor SUKL |

**Odpověď:** JSON s kompletním záznamem `Drug` - název, účinné látky, léková forma, ATC kód, registrační číslo, držitel registrace, URL pro SPC a PIL.

### sukl_spc_getter

Získání odkazu na Souhrn údajů o přípravku (SPC/SmPC) pro daný lék.

```bash
biomcp czech sukl spc "0001234"
```

**Parametry:**
| Parametr | Typ | Povinný | Popis |
|----------|-----|---------|-------|
| `sukl_code` | str | ano | Identifikátor SUKL |

**Odpověď:** JSON s polemi `sukl_code`, `name`, `spc_text`, `spc_url`, `source`.

### sukl_pil_getter

Získání odkazu na Příbalovou informaci (PIL) pro daný lék.

```bash
biomcp czech sukl pil "0001234"
```

**Parametry:**
| Parametr | Typ | Povinný | Popis |
|----------|-----|---------|-------|
| `sukl_code` | str | ano | Identifikátor SUKL |

**Odpověď:** JSON s polemi `sukl_code`, `name`, `pil_text`, `pil_url`, `source`.

### sukl_availability_checker

Kontrola aktuální dostupnosti léku na českém trhu. Ověřuje, zda je lék v aktivní distribuci.

```bash
biomcp czech sukl availability "0001234"
```

**Parametry:**
| Parametr | Typ | Povinný | Popis |
|----------|-----|---------|-------|
| `sukl_code` | str | ano | Identifikátor SUKL |

**Odpověď:** JSON s polemi `sukl_code`, `name`, `status` (`available`/`limited`/`unavailable`), `last_checked` (ISO 8601), `note`, `source`.

---

## MKN-10 - Kódy diagnóz

### mkn_diagnosis_searcher

Vyhledávání diagnóz v české klasifikaci MKN-10 podle kódu nebo volného textu v češtině. Podporuje vyhledávání s diakritikou i bez.

```bash
# Vyhledání podle kódu
biomcp czech mkn search --query "J06.9"

# Vyhledání volným textem
biomcp czech mkn search --query "infarkt"
biomcp czech mkn search --query "angina"
```

**Parametry:**
| Parametr | Typ | Povinný | Výchozí | Popis |
|----------|-----|---------|---------|-------|
| `query` | str | ano | - | Kód MKN-10 nebo text v češtině |
| `max_results` | int | ne | 10 | Maximální počet výsledků (1-100) |

**Odpověď:** JSON s polemi `query`, `total`, `results` (seznam s `code`, `name_cs`, `kind`).

### mkn_diagnosis_getter

Získání kompletních podrobností diagnózy včetně hierarchie (kapitola, blok, kategorie).

```bash
biomcp czech mkn get "J06.9"
```

**Parametry:**
| Parametr | Typ | Povinný | Popis |
|----------|-----|---------|-------|
| `code` | str | ano | Kód MKN-10 (např. "J06.9") |

**Odpověď:** JSON s kompletním záznamem `Diagnosis` - kód, český název, hierarchie (kapitola, blok, kategorie), zahrnuté/vyloučené stavy, modifikátory.

### mkn_category_browser

Procházení hierarchie kategorií MKN-10. Bez zadání kódu zobrazí kořenové kapitoly, s kódem zobrazí daný uzel a jeho přímé potomky.

```bash
# Zobrazení kapitol
biomcp czech mkn browse

# Procházení konkrétní kategorie
biomcp czech mkn browse "J00-J06"
```

**Parametry:**
| Parametr | Typ | Povinný | Popis |
|----------|-----|---------|-------|
| `code` | str \| None | ne | Kód kategorie pro procházení (bez = kořenové kapitoly) |

**Odpověď:** JSON s hierarchickým stromem - kód, český název, typ uzlu, potomci.

---

## NRPZS - Poskytovatelé zdravotních služeb

### nrpzs_provider_searcher

Vyhledávání poskytovatelů zdravotních služeb v Národním registru (NRPZS) podle názvu, města nebo odbornosti. Podporuje kombinaci filtrů.

```bash
# Vyhledání kardiologů v Praze
biomcp czech nrpzs search --city "Praha" --specialty "kardiologie"

# Vyhledání podle názvu
biomcp czech nrpzs search --query "Nemocnice"

# Kombinace filtrů
biomcp czech nrpzs search --query "MUDr" --city "Brno"
```

**Parametry:**
| Parametr | Typ | Povinný | Výchozí | Popis |
|----------|-----|---------|---------|-------|
| `query` | str \| None | ne | None | Název poskytovatele nebo klíčové slovo |
| `city` | str \| None | ne | None | Název obce |
| `specialty` | str \| None | ne | None | Lékařská odbornost |
| `page` | int | ne | 1 | Číslo stránky (od 1) |
| `page_size` | int | ne | 10 | Počet výsledků na stránku (1-100) |

**Odpověď:** JSON s polemi `total`, `page`, `page_size`, `results` (seznam `ProviderSummary` - ID, název, město, odbornosti).

### nrpzs_provider_getter

Získání kompletních podrobností poskytovatele včetně pracovišť, kontaktních údajů a typů péče.

```bash
biomcp czech nrpzs get "12345"
```

**Parametry:**
| Parametr | Typ | Povinný | Popis |
|----------|-----|---------|-------|
| `provider_id` | str | ano | Identifikátor poskytovatele v NRPZS |

**Odpověď:** JSON s kompletním záznamem `HealthcareProvider` - název, právní forma, IČO, adresa, odbornosti, typy péče, seznam pracovišť s kontakty.

---

## SZV - Zdravotní výkony

### szv_procedure_searcher

Vyhledávání zdravotních výkonů ze Seznamu zdravotních výkonů (SZV) podle kódu nebo názvu. Data pocházejí z NZIP Open Data API.

```bash
# Vyhledání podle názvu
biomcp czech szv search --query "EKG"

# Vyhledání podle kódu
biomcp czech szv search --query "09513"
```

**Parametry:**
| Parametr | Typ | Povinný | Výchozí | Popis |
|----------|-----|---------|---------|-------|
| `query` | str | ano | - | Kód výkonu nebo název |
| `max_results` | int | ne | 10 | Maximální počet výsledků (1-100) |

**Odpověď:** JSON s polemi `total`, `results` (seznam s `code`, `name`, `point_value`, `category`).

### szv_procedure_getter

Získání kompletních podrobností zdravotního výkonu včetně bodové hodnoty, času, omezení frekvence a požadovaných odborností.

```bash
biomcp czech szv get "09513"
```

**Parametry:**
| Parametr | Typ | Povinný | Popis |
|----------|-----|---------|-------|
| `code` | str | ano | Kód výkonu (např. "09513") |

**Odpověď:** JSON s kompletním záznamem `HealthProcedure` - kód, název, kategorie, bodová hodnota, čas v minutách, omezení frekvence, kódy odborností, materiálové požadavky, poznámky.

---

## VZP - Číselníky pojišťovny

### vzp_codebook_searcher

Vyhledávání v číselnících Všeobecné zdravotní pojišťovny (VZP). Prohledává jeden nebo všechny dostupné typy číselníků.

```bash
# Vyhledání ve všech číselnících
biomcp czech vzp search --query "antibiotika"

# Vyhledání v konkrétním číselníku
biomcp czech vzp search --query "EKG" --type "seznam_vykonu"
```

**Parametry:**
| Parametr | Typ | Povinný | Výchozí | Popis |
|----------|-----|---------|---------|-------|
| `query` | str | ano | - | Hledaný text (kód, název nebo popis) |
| `codebook_type` | str \| None | ne | None | Filtr podle typu číselníku |
| `max_results` | int | ne | 10 | Maximální počet výsledků (1-100) |

**Typy číselníků:** `seznam_vykonu`, `diagnoza`, `lekarsky_predpis`, `atc`

**Odpověď:** JSON s polemi `total`, `results` (seznam s `codebook_type`, `code`, `name`).

### vzp_codebook_getter

Získání podrobností položky číselníku VZP podle typu a kódu.

```bash
biomcp czech vzp get "seznam_vykonu" "09513"
```

**Parametry:**
| Parametr | Typ | Povinný | Popis |
|----------|-----|---------|-------|
| `codebook_type` | str | ano | Identifikátor typu číselníku |
| `code` | str | ano | Kód položky |

**Odpověď:** JSON s kompletním záznamem `CodebookEntry` - typ číselníku, kód, název, popis, platnost od/do, pravidla úhrad, související kódy.

---

## Zdroje dat

| Zdroj | URL | Typ dat | Autentizace |
|-------|-----|---------|-------------|
| SUKL DLP API v1 | prehledy.sukl.cz | Registr léčiv, SPC, PIL, dostupnost | Nevyžadována |
| NRPZS API v1 | nrpzs.uzis.cz/api/v1 | Poskytovatelé zdravotních služeb | Nevyžadována |
| MKN-10 ClaML | mkn10.uzis.cz | Diagnózy MKN-10 (lokální XML) | Nevyžadována |
| NZIP Open Data v3 | nzip.cz/api/v3 | Zdravotní výkony | Nevyžadována |
| VZP | vzp.cz | Číselníky pojišťovny | Nevyžadována |

Všechny zdroje dat jsou veřejná API české státní správy. Autentizace není vyžadována.

## Diakritika

Všechny vyhledávací nástroje podporují transparentní práci s diakritikou. Vyhledávání funguje shodně s diakritikou i bez ní:

- "leky" najde "léky"
- "Usti" najde "Ústí"
- "kardiologie" = "kardiologie" (beze změny)

Normalizace se provádí pomocí Unicode NFD dekompozice a odstranění kombinujících znaků (kategorie "Mn"). Originální text v datech je vždy zachován.

## Cachování

Odpovědi z externích API jsou lokálně cachovány pomocí `diskcache` pro zrychlení opakovaných dotazů a offline přístup:

| Modul | Typ dat | TTL cache |
|-------|---------|-----------|
| SUKL | Seznam léčiv | 24 hodin |
| SUKL | Detail léku, SPC, PIL | 7 dní |
| SUKL | Dostupnost | 1 hodina |
| MKN-10 | Parsovaný ClaML XML | 30 dní |
| NRPZS | Výsledky vyhledávání | 24 hodin |
| NRPZS | Detail poskytovatele | 7 dní |
| SZV | Seznam výkonů | 24 hodin |
| SZV | Detail výkonu | 7 dní |
| VZP | Číselník | 24 hodin |
| VZP | Detail položky | 7 dní |

## Chybové odpovědi

Všechny nástroje vracejí chybové zprávy ve formátu JSON:

```json
{"error": "Drug not found: 9999999"}
{"error": "SUKL API unavailable: Connection refused"}
{"error": "Code not found: Z99.9"}
{"error": "Provider not found: 00000"}
```

Při nedostupnosti externího API se nástroje pokusí použít cachovaná data. Pokud cache neobsahuje relevantní data, vrátí chybovou zprávu s popisem problému.
