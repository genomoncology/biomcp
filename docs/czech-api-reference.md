# Referenční příručka API - České zdravotnické nástroje (CzechMedMCP)

Komplexní referenční dokumentace pro 14 MCP nástrojů českého zdravotnického systému rozdělených do 5 modulů.

**Poslední aktualizace:** 2026-02-20
**Verze:** 1.0
**Zdrojový kód:** `/Users/petrsovadina/Desktop/Develope/personal/biomcp/src/biomcp/czech/`

---

## Obsah

1. [Přehled](#přehled)
2. [SUKL - Registr léčiv (5 nástrojů)](#sukl---registr-léčiv-5-nástrojů)
3. [MKN-10 - Diagnózy (3 nástroje)](#mkn-10---diagnózy-3-nástroje)
4. [NRPZS - Poskytovatelé (2 nástroje)](#nrpzs---poskytovatelé-2-nástroje)
5. [SZV - Zdravotnické výkony (2 nástroje)](#szv---zdravotnické-výkony-2-nástroje)
6. [VZP - Číselníky pojišťovny (2 nástroje)](#vzp---číselníky-pojišťovny-2-nástroje)
7. [Datové modely](#datové-modely)
8. [Společné vzory](#společné-vzory)

---

## Přehled

Projekt CzechMedMCP poskytuje 14 MCP (Model Context Protocol) nástrojů pro práci s českými zdravotnickými registy a databázemi. Nástroje jsou organizovány do 5 modulů:

| Modul | Počet nástrojů | Zdroj dat | Popis |
|-------|----------------|-----------|-------|
| SUKL | 5 | SUKL DLP API v1 | Registr léčiv |
| MKN-10 | 3 | UZIS ClaML XML | Diagnózy (ICD-10) |
| NRPZS | 2 | NRPZS API | Zdravotnické poskytovatele |
| SZV | 2 | NZIP Open Data v3 | Zdravotnické výkony |
| VZP | 2 | VZP veřejné API | Pojišťovací číselníky |

Všechny nástroje vrací odpovědi ve formátu **JSON** s popořízeným textem ve čeština.

---

## SUKL - Registr léčiv (5 nástrojů)

Modul pro práci s Státním ústavem pro kontrolu léčiv (SUKL). Poskytuje vyhledávání léčiv, detaily léčiv, přístup k příbalovým letákům a informacím o dostupnosti.

### 1. sukl_drug_searcher

Vyhledávání léčiv v českém registru SUKL.

**Typ nástroje:** `async function`
**MCP identifikátor:** `sukl_drug_searcher`
**Zdroj dat:** SUKL DLP API v1
**Cache TTL:** 24 hodin (seznam léčiv)

#### Parametry

| Parametr | Typ | Povinný | Výchozí | Omezení | Popis |
|----------|-----|---------|---------|---------|-------|
| query | string | Ano | - | - | Název léčiva, aktivní látka nebo ATC kód |
| page | integer | Ne | 1 | >= 1 | Číslo stránky (1-indexed) |
| page_size | integer | Ne | 10 | 1-100 | Počet výsledků na stránku |

#### Formát odpovědi

```json
{
  "total": 42,
  "page": 1,
  "page_size": 10,
  "results": [
    {
      "sukl_code": "0254045",
      "name": "Ibuprofen 400 mg",
      "active_substance": "Ibuprofen",
      "atc_code": "M01AE01",
      "pharmaceutical_form": "tableta"
    }
  ]
}
```

**Pole odpovědi:**
- `total` (int): Celkový počet nalezených léčiv
- `page` (int): Číslo aktuální stránky
- `page_size` (int): Počet výsledků na stránku
- `results` (array): Pole shrnujících informací o léčivech
  - `sukl_code` (string): 7místný identifikátor SUKL
  - `name` (string): Obchodní název léčiva
  - `active_substance` (string | null): Primární aktivní látka
  - `atc_code` (string | null): ATC klasifikační kód
  - `pharmaceutical_form` (string | null): Lékařská forma (tableta, kapsula, injekce, atd.)

#### Příklady

**Vyhledávání podle názvu:**
```json
{
  "query": "Aspirin",
  "page": 1,
  "page_size": 10
}
```

**Vyhledávání podle ATC kódu:**
```json
{
  "query": "M01AE01",
  "page": 1,
  "page_size": 5
}
```

**Vyhledávání podle aktivní látky:**
```json
{
  "query": "Ibuprofen",
  "page": 1,
  "page_size": 20
}
```

#### Chybové odpovědi

```json
{
  "total": 0,
  "page": 1,
  "page_size": 10,
  "results": [],
  "error": "SUKL API unavailable: Connection timeout"
}
```

#### Normalizace

Vyhledávání je necitlivé na diakritiku (é→e, č→c, ř→r, ž→z atd.). Velká/malá písmena jsou automaticky normalizována.

---

### 2. sukl_drug_getter

Získání úplných informací o léčivu podle SUKL kódu.

**Typ nástroje:** `async function`
**MCP identifikátor:** `sukl_drug_getter`
**Zdroj dat:** SUKL DLP API v1
**Cache TTL:** 7 dní

#### Parametry

| Parametr | Typ | Povinný | Výchozí | Popis |
|----------|-----|---------|---------|-------|
| sukl_code | string | Ano | - | 7místný SUKL identifikátor |

#### Formát odpovědi

```json
{
  "sukl_code": "0254045",
  "name": "Ibuprofen Ibalgin 400 mg",
  "active_substances": [
    {
      "name": "Ibuprofen",
      "strength": "400 mg"
    }
  ],
  "pharmaceutical_form": "tableta",
  "atc_code": "M01AE01",
  "registration_number": "Reg. číslo XYZ",
  "mah": "ABC Pharma s.r.o.",
  "registration_valid_to": "2026-12-31T00:00:00Z",
  "spc_url": "https://prehledy.sukl.cz/v1/dokumenty/0254045/spc",
  "pil_url": "https://prehledy.sukl.cz/v1/dokumenty/0254045/pil",
  "source": "SUKL"
}
```

**Pole odpovědi:**
- `sukl_code` (string): Identifikátor SUKL
- `name` (string): Obchodní název
- `active_substances` (array): Pole aktivních látek
  - `name` (string): Název látky
  - `strength` (string | null): Síla (např. "400 mg")
- `pharmaceutical_form` (string | null): Lékařská forma
- `atc_code` (string | null): ATC kód
- `registration_number` (string | null): Registrační číslo
- `mah` (string | null): Držitel povolení na trh (Marketing Authorization Holder)
- `registration_valid_to` (string | null): Platnost registrace (ISO 8601)
- `spc_url` (string | null): URL na Shrnutí vlastností léčiva
- `pil_url` (string | null): URL na Příbalový leták
- `source` (string): Zdroj dat ("SUKL")

#### Příklad dotazu

```json
{
  "sukl_code": "0254045"
}
```

#### Příklad chybové odpovědi

```json
{
  "error": "Drug not found: 9999999"
}
```

---

### 3. sukl_spc_getter

Získání Shrnutí vlastností léčiva (SmPC - Summary of Product Characteristics).

**Typ nástroje:** `async function`
**MCP identifikátor:** `sukl_spc_getter`
**Zdroj dat:** SUKL DLP API v1 (metadata dokumentu)
**Cache TTL:** 7 dní

#### Parametry

| Parametr | Typ | Povinný | Výchozí | Popis |
|----------|-----|---------|---------|-------|
| sukl_code | string | Ano | - | 7místný SUKL identifikátor |

#### Formát odpovědi

```json
{
  "sukl_code": "0254045",
  "name": "Ibuprofen Ibalgin 400 mg",
  "spc_text": "SmPC document available at: https://prehledy.sukl.cz/v1/dokumenty/0254045/spc",
  "spc_url": "https://prehledy.sukl.cz/v1/dokumenty/0254045/spc",
  "source": "SUKL"
}
```

**Pole odpovědi:**
- `sukl_code` (string): Identifikátor SUKL
- `name` (string): Název léčiva
- `spc_text` (string | null): Referenční text s URL
- `spc_url` (string | null): Přímý odkaz na PDF
- `source` (string): Zdroj dat

#### Příklad dotazu

```json
{
  "sukl_code": "0254045"
}
```

#### Příklad chybové odpovědi (bez dostupného SmPC)

```json
{
  "error": "SmPC not available for 0254045",
  "sukl_code": "0254045",
  "name": "Ibuprofen Ibalgin 400 mg",
  "spc_text": null,
  "spc_url": null,
  "source": "SUKL"
}
```

---

### 4. sukl_pil_getter

Získání Příbalového letáku (PIL - Patient Information Leaflet).

**Typ nástroje:** `async function`
**MCP identifikátor:** `sukl_pil_getter`
**Zdroj dat:** SUKL DLP API v1 (metadata dokumentu)
**Cache TTL:** 7 dní

#### Parametry

| Parametr | Typ | Povinný | Výchozí | Popis |
|----------|-----|---------|---------|-------|
| sukl_code | string | Ano | - | 7místný SUKL identifikátor |

#### Formát odpovědi

```json
{
  "sukl_code": "0254045",
  "name": "Ibuprofen Ibalgin 400 mg",
  "pil_text": "PIL document available at: https://prehledy.sukl.cz/v1/dokumenty/0254045/pil",
  "pil_url": "https://prehledy.sukl.cz/v1/dokumenty/0254045/pil",
  "source": "SUKL"
}
```

**Pole odpovědi:**
- `sukl_code` (string): Identifikátor SUKL
- `name` (string): Název léčiva
- `pil_text` (string | null): Referenční text s URL
- `pil_url` (string | null): Přímý odkaz na PDF
- `source` (string): Zdroj dat

#### Příklad dotazu

```json
{
  "sukl_code": "0254045"
}
```

---

### 5. sukl_availability_checker

Kontrola dostupnosti léčiva na trhu.

**Typ nástroje:** `async function`
**MCP identifikátor:** `sukl_availability_checker`
**Zdroj dat:** SUKL DLP API v1 (VPOIS endpoint)
**Cache TTL:** 1 hodina

#### Parametry

| Parametr | Typ | Povinný | Výchozí | Popis |
|----------|-----|---------|---------|-------|
| sukl_code | string | Ano | - | 7místný SUKL identifikátor |

#### Formát odpovědi

```json
{
  "sukl_code": "0254045",
  "name": "Ibuprofen Ibalgin 400 mg",
  "status": "available",
  "last_checked": "2026-02-20T14:35:22.123456Z",
  "note": null,
  "source": "SUKL"
}
```

**Pole odpovědi:**
- `sukl_code` (string): Identifikátor SUKL
- `name` (string): Název léčiva
- `status` (string): Stav dostupnosti: `available`, `limited`, `unavailable`
- `last_checked` (string): Čas poslední kontroly (ISO 8601)
- `note` (string | null): Dodatečná poznámka
- `source` (string): Zdroj dat

#### Příklad dotazu

```json
{
  "sukl_code": "0254045"
}
```

#### Stavy dostupnosti

| Stav | Popis |
|------|-------|
| `available` | Léčivo je na trhu k dispozici |
| `limited` | Léčivo je dostupné v omezené míře |
| `unavailable` | Léčivo není k dispozici |

---

## MKN-10 - Diagnózy (3 nástroje)

Modul pro práci s českou verzí ICD-10 klasifikace (Mezinárodní klasifikace nemocí, 10. revize) spravovanou ÚZIS. Umožňuje vyhledávání diagnóz, načítání podrobností a procházení hierarchie.

### 1. mkn_diagnosis_searcher

Vyhledávání diagnóz v MKN-10 podle kódu či textu.

**Typ nástroje:** `async function`
**MCP identifikátor:** `mkn_diagnosis_searcher`
**Zdroj dat:** UZIS ClaML XML (ClaML formát)
**Cache TTL:** 30 dní (XML parsing)

#### Parametry

| Parametr | Typ | Povinný | Výchozí | Omezení | Popis |
|----------|-----|---------|---------|---------|-------|
| query | string | Ano | - | - | MKN-10 kód nebo volný text v češtině |
| max_results | integer | Ne | 10 | 1-100 | Maximální počet výsledků |

#### Formát odpovědi

```json
{
  "query": "J06",
  "total": 5,
  "results": [
    {
      "code": "J06",
      "name_cs": "Akutní infekce horních cest dýchacích více lokalizací",
      "kind": "category"
    },
    {
      "code": "J06.0",
      "name_cs": "Akutní laryngofaryingitida",
      "kind": "category"
    }
  ]
}
```

**Pole odpovědi:**
- `query` (string): Vyhledávaný dotaz (normalizovaný)
- `total` (int): Počet nalezených výsledků
- `results` (array): Pole výsledků
  - `code` (string): MKN-10 kód (např. "J06.9")
  - `name_cs` (string): Název v češtině
  - `kind` (string): Typ uzlu: `chapter`, `block`, `category`

#### Režimy vyhledávání

**Vyhledávání podle kódu** (když query vypadá jako kód):
- Příklad: `"J06"`, `"J06.9"`, `"A01-B99"`
- Provádí prefix matching na MKN-10 kódech (case-insensitive)

**Vyhledávání podle textu** (volný text):
- Příklad: `"Bronchitis"`, `"akutní infekce"`, `"úpal dýchacích cest"`
- Normalizuje diakritiku a hledá shody v názvech

#### Příklady

**Vyhledávání podle kódu:**
```json
{
  "query": "J06",
  "max_results": 10
}
```

**Vyhledávání podle textu:**
```json
{
  "query": "akutní infekce dýchacích cest",
  "max_results": 10
}
```

---

### 2. mkn_diagnosis_getter

Získání úplných informací o diagnóze včetně hierarchie.

**Typ nástroje:** `async function`
**MCP identifikátor:** `mkn_diagnosis_getter`
**Zdroj dat:** UZIS ClaML XML
**Cache TTL:** 30 dní

#### Parametry

| Parametr | Typ | Povinný | Výchozí | Popis |
|----------|-----|---------|---------|-------|
| code | string | Ano | - | MKN-10 kód (např. "J06.9") |

#### Formát odpovědi

```json
{
  "code": "J06.9",
  "name_cs": "Akutní infekce horních cest dýchacích, neurčená",
  "name_en": null,
  "definition": null,
  "hierarchy": {
    "chapter": "X",
    "chapter_name": "Nemoci dýchací soustavy",
    "block": "J00-J06",
    "block_name": "Akutní infekce horních cest dýchacích",
    "category": "J06"
  },
  "includes": [],
  "excludes": [],
  "modifiers": [],
  "source": "UZIS/MKN-10"
}
```

**Pole odpovědi:**
- `code` (string): MKN-10 kód
- `name_cs` (string): Název v češtině
- `name_en` (string | null): Název v angličtině (obvykle prázdné)
- `definition` (string | null): Textová definice
- `hierarchy` (object): Hierarchická pozice
  - `chapter` (string): Kód kapitoly
  - `chapter_name` (string): Název kapitoly
  - `block` (string): Kód bloku
  - `block_name` (string): Název bloku
  - `category` (string): Kategoriální kód
- `includes` (array): Zahrnuté stavy
- `excludes` (array): Vyloučené stavy
- `modifiers` (array): Diagnostické modifikátory
- `source` (string): Zdroj dat

#### Příklad dotazu

```json
{
  "code": "J06.9"
}
```

#### Příklad chybové odpovědi

```json
{
  "error": "Code not found: ZZZ.99"
}
```

---

### 3. mkn_category_browser

Procházení hierarchie MKN-10 kategorií.

**Typ nástroje:** `async function`
**MCP identifikátor:** `mkn_category_browser`
**Zdroj dat:** UZIS ClaML XML
**Cache TTL:** 30 dní

#### Parametry

| Parametr | Typ | Povinný | Výchozí | Popis |
|----------|-----|---------|---------|-------|
| code | string | Ne | null | Kód kategorie k procházení (null = root kapitoly) |

#### Formát odpovědi (root - bez kódu)

```json
{
  "type": "chapters",
  "items": [
    {
      "code": "I",
      "name_cs": "Určité infekční a parazitární nemoci",
      "kind": "chapter",
      "children": ["A01", "A02", ...]
    },
    {
      "code": "X",
      "name_cs": "Nemoci dýchací soustavy",
      "kind": "chapter",
      "children": ["J00", "J01", ...]
    }
  ]
}
```

#### Formát odpovědi (konkrétní kategorie)

```json
{
  "code": "J06",
  "name_cs": "Akutní infekce horních cest dýchacích více lokalizací",
  "kind": "category",
  "parent_code": "J00-J06",
  "children": [
    {
      "code": "J06.0",
      "name_cs": "Akutní laryngofaryingitida",
      "kind": "category",
      "children": []
    },
    {
      "code": "J06.9",
      "name_cs": "Akutní infekce horních cest dýchacích, neurčená",
      "kind": "category",
      "children": []
    }
  ]
}
```

**Pole odpovědi (kategorie):**
- `code` (string): Kód kategorie
- `name_cs` (string): Název v češtině
- `kind` (string): Typ: `chapter`, `block`, `category`
- `parent_code` (string | null): Kód nadřazené kategorie
- `children` (array): Pole podkategorií
  - `code` (string): Kód podkategorie
  - `name_cs` (string): Název podkategorie
  - `kind` (string): Typ podkategorie
  - `children` (array): Pole podkategorií další úrovně

#### Příklady

**Zobrazit všechny kapitoly (bez parametru):**
```json
{
  "code": null
}
```

**Zobrazit obsah kapitoly:**
```json
{
  "code": "X"
}
```

**Zobrazit obsah bloku:**
```json
{
  "code": "J00-J06"
}
```

---

## NRPZS - Poskytovatelé (2 nástroje)

Modul pro práci s Národním registrem poskytovatelů zdravotních služeb (NRPZS). Umožňuje vyhledávání zdravotnických zařízení a pracovišť s filtrováním podle místa a specializace.

### 1. nrpzs_provider_searcher

Vyhledávání zdravotnických poskytovatelů v registru NRPZS.

**Typ nástroje:** `async function`
**MCP identifikátor:** `nrpzs_provider_searcher`
**Zdroj dat:** NRPZS API (nrpzs.uzis.cz)
**Cache TTL:** 24 hodin

#### Parametry

| Parametr | Typ | Povinný | Výchozí | Omezení | Popis |
|----------|-----|---------|---------|---------|-------|
| query | string | Ne | null | - | Název poskytovatele nebo klíčové slovo |
| city | string | Ne | null | - | Název obce k filtrování |
| specialty | string | Ne | null | - | Zdravotnická odbornost k filtrování |
| page | integer | Ne | 1 | >= 1 | Číslo stránky |
| page_size | integer | Ne | 10 | 1-100 | Počet výsledků na stránku |

#### Formát odpovědi

```json
{
  "total": 127,
  "page": 1,
  "page_size": 10,
  "results": [
    {
      "provider_id": "12345",
      "name": "Zdravotnické zařízení Na Dolanech",
      "city": "Praha",
      "specialties": ["Všeobecné lékařství", "Chirurgie"]
    },
    {
      "provider_id": "12346",
      "name": "Infekční klinika VFN",
      "city": "Praha",
      "specialties": ["Infekční nemoci"]
    }
  ]
}
```

**Pole odpovědi:**
- `total` (int): Celkový počet nalezených poskytovatelů
- `page` (int): Číslo aktuální stránky
- `page_size` (int): Počet výsledků na stránku
- `results` (array): Pole poskytovatelů
  - `provider_id` (string): NRPZS identifikátor
  - `name` (string): Název poskytovatele
  - `city` (string | null): Město
  - `specialties` (array): Pole zdravotnických odborností

#### Příklady

**Vyhledávání všech poskytovatelů v Praze:**
```json
{
  "query": null,
  "city": "Praha",
  "specialty": null,
  "page": 1,
  "page_size": 20
}
```

**Vyhledávání stomatologů:**
```json
{
  "query": null,
  "city": null,
  "specialty": "Stomatologie",
  "page": 1,
  "page_size": 10
}
```

**Vyhledávání podle názvu a místa:**
```json
{
  "query": "infekční klinika",
  "city": "Praha",
  "specialty": null,
  "page": 1,
  "page_size": 10
}
```

#### Normalizace

Všechna vyhledávání (query, city, specialty) jsou necitlivá na diakritiku a velká/malá písmena.

---

### 2. nrpzs_provider_getter

Získání úplných informací o poskytovateli včetně pracovišť.

**Typ nástroje:** `async function`
**MCP identifikátor:** `nrpzs_provider_getter`
**Zdroj dat:** NRPZS API
**Cache TTL:** 7 dní

#### Parametry

| Parametr | Typ | Povinný | Výchozí | Popis |
|----------|-----|---------|---------|-------|
| provider_id | string | Ano | - | NRPZS identifikátor poskytovatele |

#### Formát odpovědi

```json
{
  "provider_id": "12345",
  "name": "Zdravotnické zařízení Na Dolanech",
  "legal_form": "Nemocnice (příspěvková organizace)",
  "ico": "64524580",
  "address": {
    "street": "Na Dolanech 123",
    "city": "Praha 4",
    "postal_code": "14000",
    "region": "Hlavní město Praha"
  },
  "specialties": ["Chirurgie", "Ortopedika", "Onkologie"],
  "care_types": ["lůžková", "ambulantní"],
  "workplaces": [
    {
      "workplace_id": "12345-1",
      "name": "Chirurgická klinika",
      "address": {
        "street": "Na Dolanech 123",
        "city": "Praha 4",
        "postal_code": "14000",
        "region": "Hlavní město Praha"
      },
      "specialties": ["Chirurgie"],
      "contact": {
        "phone": "+420261082111",
        "email": "chirurgie@nemocnice.cz",
        "website": "http://www.nemocnice.cz/chirurgie"
      }
    }
  ],
  "registration_number": "NRPZS123456",
  "source": "NRPZS"
}
```

**Pole odpovědi:**
- `provider_id` (string): NRPZS identifikátor
- `name` (string): Název poskytovatele
- `legal_form` (string | null): Právní forma
- `ico` (string | null): Identifikační číslo organizace
- `address` (object | null): Hlavní adresa
  - `street` (string | null): Ulice a číslo
  - `city` (string | null): Město
  - `postal_code` (string | null): PSČ
  - `region` (string | null): Kraj
- `specialties` (array): Zdravotnické odbornosti
- `care_types` (array): Druhy péče (ambulantní, lůžková, domácí atd.)
- `workplaces` (array): Pracovní místa
  - `workplace_id` (string): Identifikátor pracoviště
  - `name` (string): Název pracoviště
  - `address` (object | null): Adresa pracoviště
  - `specialties` (array): Odbornosti na pracovišti
  - `contact` (object | null): Kontaktní údaje
    - `phone` (string | null): Telefonní číslo
    - `email` (string | null): E-mailová adresa
    - `website` (string | null): Webová stránka
- `registration_number` (string | null): Registrační číslo
- `source` (string): Zdroj dat

#### Příklad dotazu

```json
{
  "provider_id": "12345"
}
```

#### Příklad chybové odpovědi

```json
{
  "error": "Provider not found: 99999"
}
```

---

## SZV - Zdravotnické výkony (2 nástroje)

Modul pro práci se zdravotnickými výkony. Vyhledávání procedur používaných v péči o pacienty s bodovým ohodnocením a dalšími parametry.

### 1. szv_procedure_searcher

Vyhledávání zdravotnických procedur.

**Typ nástroje:** `async function`
**MCP identifikátor:** `szv_procedure_searcher`
**Zdroj dat:** NZIP Open Data API v3 (nzip.cz)
**Cache TTL:** 24 hodin

#### Parametry

| Parametr | Typ | Povinný | Výchozí | Omezení | Popis |
|----------|-----|---------|---------|---------|-------|
| query | string | Ano | - | - | Kód procedury nebo název |
| max_results | integer | Ne | 10 | 1-100 | Maximální počet výsledků |

#### Formát odpovědi

```json
{
  "total": 5,
  "results": [
    {
      "code": "09513",
      "name": "Konzultační prohlídka specialisty - poprvé",
      "point_value": 150,
      "category": "09"
    },
    {
      "code": "09514",
      "name": "Konzultační prohlídka specialisty - opakovaná",
      "point_value": 100,
      "category": "09"
    }
  ]
}
```

**Pole odpovědi:**
- `total` (int): Počet nalezených procedur
- `results` (array): Pole procedur
  - `code` (string): Kód procedury (např. "09513")
  - `name` (string): Název procedury
  - `point_value` (int | null): Bodové ohodnocení
  - `category` (string | null): Kategoriální kód

#### Příklady

**Vyhledávání podle kódu:**
```json
{
  "query": "09513",
  "max_results": 10
}
```

**Vyhledávání podle názvu:**
```json
{
  "query": "konzultace specialista",
  "max_results": 10
}
```

**Vyhledávání podle kategorie:**
```json
{
  "query": "09",
  "max_results": 20
}
```

#### Normalizace

Diakritika je ignorována. Vyhledávání je case-insensitive.

---

### 2. szv_procedure_getter

Získání úplných informací o proceduře.

**Typ nástroje:** `async function`
**MCP identifikátor:** `szv_procedure_getter`
**Zdroj dat:** NZIP Open Data API v3
**Cache TTL:** 7 dní

#### Parametry

| Parametr | Typ | Povinný | Výchozí | Popis |
|----------|-----|---------|---------|-------|
| code | string | Ano | - | Kód procedury (např. "09513") |

#### Formát odpovědi

```json
{
  "code": "09513",
  "name": "Konzultační prohlídka specialisty - poprvé",
  "category": "09",
  "category_name": "Konzultace a prohlídky",
  "point_value": 150,
  "time_minutes": 30,
  "frequency_limit": "1x za klinický případ",
  "specialty_codes": ["chirurgie", "ortopedika"],
  "material_requirements": null,
  "notes": "Zahrnuje běžné vyšetřovací prostředky",
  "source": "MZCR/SZV"
}
```

**Pole odpovědi:**
- `code` (string): Kód procedury
- `name` (string): Název procedury
- `category` (string | null): Kategoriální kód
- `category_name` (string | null): Název kategorie
- `point_value` (int | null): Bodové ohodnocení
- `time_minutes` (int | null): Odhadovaná doba procedury v minutách
- `frequency_limit` (string | null): Omezení frekvence (např. "1x za rok")
- `specialty_codes` (array): Kódy oborů oprávněných provádět proceduru
- `material_requirements` (string | null): Požadavky na materiály/vybavení
- `notes` (string | null): Dodatečné poznámky
- `source` (string): Zdroj dat

#### Příklad dotazu

```json
{
  "code": "09513"
}
```

#### Příklad chybové odpovědi

```json
{
  "error": "Procedure not found: 99999"
}
```

---

## VZP - Číselníky pojišťovny (2 nástroje)

Modul pro práci s veřejnými číselníky pojišťovny VZP. Umožňuje vyhledávání záznamů v různých typech číselníků (výkony, diagnózy, ATC kódy atd.).

### 1. vzp_codebook_searcher

Vyhledávání v číselníkech pojišťovny VZP.

**Typ nástroje:** `async function`
**MCP identifikátor:** `vzp_codebook_searcher`
**Zdroj dat:** VZP veřejné API
**Cache TTL:** 24 hodin

#### Parametry

| Parametr | Typ | Povinný | Výchozí | Omezení | Popis |
|----------|-----|---------|---------|---------|-------|
| query | string | Ano | - | - | Vyhledávací text (kód, název, popis) |
| codebook_type | string | Ne | null | - | Typ číselníku k filtrování |
| max_results | integer | Ne | 10 | 1-100 | Maximální počet výsledků |

#### Podporované typy číselníků

| Typ | Popis |
|-----|-------|
| `seznam_vykonu` | Seznam zdravotnických výkonů |
| `diagnoza` | Diagnózy (MKN-10 mapování) |
| `lekarsky_predpis` | Lékařské předpisy |
| `atc` | ATC klasifikace léčiv |

#### Formát odpovědi

```json
{
  "total": 12,
  "results": [
    {
      "codebook_type": "seznam_vykonu",
      "code": "09513",
      "name": "Konzultační prohlídka specialisty - poprvé"
    },
    {
      "codebook_type": "seznam_vykonu",
      "code": "09514",
      "name": "Konzultační prohlídka specialisty - opakovaná"
    }
  ]
}
```

**Pole odpovědi:**
- `total` (int): Počet nalezených záznamů
- `results` (array): Pole položek
  - `codebook_type` (string): Typ číselníku
  - `code` (string): Kód v číselníku
  - `name` (string): Název položky

#### Příklady

**Vyhledávání konkrétního typu číselníku:**
```json
{
  "query": "09513",
  "codebook_type": "seznam_vykonu",
  "max_results": 10
}
```

**Vyhledávání ve všech číselníkách:**
```json
{
  "query": "diabetes",
  "codebook_type": null,
  "max_results": 20
}
```

**Vyhledávání ATC kódů:**
```json
{
  "query": "ibuprofen",
  "codebook_type": "atc",
  "max_results": 10
}
```

---

### 2. vzp_codebook_getter

Získání úplných informací o položce v číselníku VZP.

**Typ nástroje:** `async function`
**MCP identifikátor:** `vzp_codebook_getter`
**Zdroj dat:** VZP veřejné API
**Cache TTL:** 7 dní

#### Parametry

| Parametr | Typ | Povinný | Výchozí | Popis |
|----------|-----|---------|---------|-------|
| codebook_type | string | Ano | - | Typ číselníku (např. "seznam_vykonu") |
| code | string | Ano | - | Kód v číselníku |

#### Formát odpovědi

```json
{
  "codebook_type": "seznam_vykonu",
  "code": "09513",
  "name": "Konzultační prohlídka specialisty - poprvé",
  "description": "Konzultační prohlídka lékaře specialisty bez předchozí péče na oddělení",
  "valid_from": "2024-01-01",
  "valid_to": null,
  "rules": [
    "Není možná v případě ambulantního léčení",
    "Vyžaduje lékařský předpis"
  ],
  "related_codes": ["09514", "09515"],
  "source": "VZP"
}
```

**Pole odpovědi:**
- `codebook_type` (string): Typ číselníku
- `code` (string): Kód položky
- `name` (string): Název položky
- `description` (string | null): Detailný popis
- `valid_from` (string | null): Počátek platnosti (ISO 8601)
- `valid_to` (string | null): Konec platnosti (ISO 8601)
- `rules` (array): Pravidla pro aplikaci/fakturaci
- `related_codes` (array): Související kódy
- `source` (string): Zdroj dat

#### Příklady

**Získání detailů o výkonu:**
```json
{
  "codebook_type": "seznam_vykonu",
  "code": "09513"
}
```

**Získání informací o diagnóze:**
```json
{
  "codebook_type": "diagnoza",
  "code": "J06.9"
}
```

**Získání ATC kódu:**
```json
{
  "codebook_type": "atc",
  "code": "M01AE01"
}
```

#### Příklad chybové odpovědi

```json
{
  "error": "Codebook entry not found: seznam_vykonu/99999"
}
```

---

## Datové modely

Všechny nástroje vracejí data v Pydantic v2 modelech. Zde jsou hlavní datové struktury:

### SUKL modely

#### Drug (Úplné informace o léčivu)
```python
class Drug(BaseModel):
    sukl_code: str  # 7místný SUKL identifikátor
    name: str  # Obchodní název
    active_substances: list[ActiveSubstance]  # Aktivní látky
    pharmaceutical_form: str | None  # Forma (tableta, kapsula, atd.)
    atc_code: str | None  # ATC klasifikace
    registration_number: str | None  # Registrační číslo
    mah: str | None  # Držitel povolení na trh
    registration_valid_to: str | None  # Platnost registrace (ISO 8601)
    availability: AvailabilityStatus | None  # Stav dostupnosti
    spc_url: str | None  # URL na SmPC
    pil_url: str | None  # URL na PIL
    source: str = "SUKL"
```

#### ActiveSubstance (Aktivní látka v léčivu)
```python
class ActiveSubstance(BaseModel):
    name: str  # Název látky
    strength: str | None  # Síla (např. "400 mg")
```

#### AvailabilityStatus (Stav dostupnosti)
```python
class AvailabilityStatus(BaseModel):
    status: str  # "available", "limited", "unavailable"
    last_checked: str | None  # ISO 8601 čas
    note: str | None  # Dodatečná poznámka
```

### MKN-10 modely

#### Diagnosis (Úplná diagnóza)
```python
class Diagnosis(BaseModel):
    code: str  # MKN-10 kód (např. "J06.9")
    name_cs: str  # Název v češtině
    name_en: str | None  # Název v angličtině
    definition: str | None  # Textová definice
    hierarchy: DiagnosisHierarchy | None  # Pozice v hierarchii
    includes: list[str]  # Zahrnuté stavy
    excludes: list[str]  # Vyloučené stavy
    modifiers: list[Modifier]  # Diagnostické modifikátory
    source: str = "UZIS/MKN-10"
```

#### DiagnosisHierarchy (Hierarchická pozice)
```python
class DiagnosisHierarchy(BaseModel):
    chapter: str  # Kód kapitoly
    chapter_name: str  # Název kapitoly
    block: str  # Kód bloku
    block_name: str  # Název bloku
    category: str  # Kategoriální kód
```

### NRPZS modely

#### HealthcareProvider (Poskytovatel zdravotní péče)
```python
class HealthcareProvider(BaseModel):
    provider_id: str  # NRPZS identifikátor
    name: str  # Název
    legal_form: str | None  # Právní forma
    ico: str | None  # IČO
    address: Address | None  # Adresa
    specialties: list[str]  # Zdravotnické odbornosti
    care_types: list[str]  # Druhy péče
    workplaces: list[Workplace]  # Pracovní místa
    registration_number: str | None  # Registrační číslo
    source: str = "NRPZS"
```

#### Workplace (Pracovní místo poskytovatele)
```python
class Workplace(BaseModel):
    workplace_id: str  # Identifikátor pracoviště
    name: str  # Název
    address: Address | None  # Adresa
    specialties: list[str]  # Odbornosti
    contact: Contact | None  # Kontakt
```

### SZV modely

#### HealthProcedure (Zdravotnická procedura)
```python
class HealthProcedure(BaseModel):
    code: str  # Kód procedury
    name: str  # Název
    category: str | None  # Kategoriální kód
    category_name: str | None  # Název kategorie
    point_value: int | None  # Bodové ohodnocení
    time_minutes: int | None  # Odhadovaná doba v minutách
    frequency_limit: str | None  # Omezení frekvence
    specialty_codes: list[str]  # Oprávněné obory
    material_requirements: str | None  # Požadavky na materiál
    notes: str | None  # Poznámky
    source: str = "MZCR/SZV"
```

### VZP modely

#### CodebookEntry (Položka číselníku)
```python
class CodebookEntry(BaseModel):
    codebook_type: str  # Typ číselníku
    code: str  # Kód
    name: str  # Název
    description: str | None  # Popis
    valid_from: str | None  # Počátek platnosti (ISO 8601)
    valid_to: str | None  # Konec platnosti (ISO 8601)
    rules: list[str]  # Pravidla
    related_codes: list[str]  # Související kódy
    source: str = "VZP"
```

---

## Společné vzory

### Cachování

Všechny HTTP požadavky jsou kešovány pomocí diskcache pro zvýšení výkonu a snížení zátěže na externí API:

| Datový typ | TTL | Důvod |
|-----------|-----|--------|
| SUKL - seznam léčiv | 24 hodin | Relativně stabilní, občasné změny |
| SUKL - detaily léčiva | 7 dní | Záznamy se mění vzácně |
| SUKL - dostupnost | 1 hodina | Nejčastěji se měnící data |
| MKN-10 - XML parsing | 30 dní | Statická data |
| NRPZS - seznam | 24 hodin | Mívají moderní synchronizaci |
| NRPZS - detaily | 7 dní | Mívají vzácné změny |
| SZV - seznam procedur | 24 hodin | Aktuální seznam |
| SZV - detaily procedury | 7 dní | Stabilní podrobnosti |
| VZP - číselníky | 24 hodin | Běžné aktualizace |
| VZP - položka | 7 dní | Stabilní záznamy |

### Normalizace diakritiky

Všechna vyhledávání jsou **necitlivá na diakritiku** a **case-insensitive**:

- `é` → `e`
- `č` → `c`
- `ř` → `r`
- `ž` → `z`
- `ů` → `u`
- `ň` → `n`
- `š` → `s`
- atd.

**Příklady ekvivalentních vyhledávání:**
- `"Ibuprofen"` = `"ibuprofen"` = `"ibuprofén"`
- `"ČESKÁ"` = `"ceska"` = `"česká"`

Normalizace je implementována v modulu `diacritics.py`:

```python
def normalize_query(query: str) -> str:
    """Normalizuje vyhledávací dotaz."""
    # Strip diacritics + lowercase
    nfkd = unicodedata.normalize("NFD", query)
    return "".join(c for c in nfkd
                   if unicodedata.category(c) != "Mn").lower()
```

### Stránkování

Nástroje, které vracejí více výsledků, podporují stránkování:

**Parametry:**
- `page` (int, >= 1): Číslo stránky (1-indexed)
- `page_size` (int, 1-100): Počet výsledků na stránku

**Příklad:**
```json
{
  "query": "ibuprofen",
  "page": 2,
  "page_size": 20
}
```

**Odpověď:**
```json
{
  "total": 150,
  "page": 2,
  "page_size": 20,
  "results": [...]  // Výsledky 21-40
}
```

### Chybové odpovědi

Všechny chyby jsou vráceny jako JSON objekty s polem `error`:

```json
{
  "error": "Popis chyby v angličtině"
}
```

**Běžné chybové scénáře:**

| Chyba | Příčina | Řešení |
|-------|--------|--------|
| "Drug not found: 9999999" | SUKL kód neexistuje | Zkontrolujte kód |
| "SUKL API unavailable" | Porucha API | Zkuste později |
| "Code not found: ZZZ.9" | MKN-10 kód neexistuje | Ověřte kód |
| "No MKN-10 data loaded" | XML není načten | Interní chyba |
| "Provider not found: 99999" | NRPZS ID neexistuje | Zkontrolujte ID |
| "Procedure not found: 99999" | SZV kód neexistuje | Ověřte kód |
| "Codebook entry not found" | VZP položka chybí | Zkontrolujte parametry |

### API URL a zdroje

| Modul | Endpoint | Zdrojová dokumentace |
|-------|----------|---------------------|
| SUKL | `https://prehledy.sukl.cz/api/v1/` | [SUKL DLP API](https://prehledy.sukl.cz) |
| MKN-10 | ClaML XML | [UZIS MKN-10](https://www.uzis.cz) |
| NRPZS | `https://nrpzs.uzis.cz/api/` | [NRPZS API](https://nrpzs.uzis.cz) |
| SZV | `https://nzip.cz/api/v3/` | [NZIP Open Data](https://nzip.cz) |
| VZP | `https://vzp.cz/o-vzp/...` | [VZP API](https://vzp.cz) |

---

## Příklady praktické aplikace

### Příklad 1: Vyhledání léčiva a ověření dostupnosti

```python
# Vyhledejte léčivo
search_result = await sukl_drug_searcher(
    query="ibuprofen",
    page=1,
    page_size=5
)

# Pokud má výsledky, vezměte první
if search_result["total"] > 0:
    sukl_code = search_result["results"][0]["sukl_code"]

    # Získejte úplné detaily
    drug_details = await sukl_drug_getter(sukl_code=sukl_code)

    # Ověřte dostupnost
    availability = await sukl_availability_checker(sukl_code=sukl_code)
```

### Příklad 2: Procházení MKN-10 hierarchie

```python
# Zobrazit všechny kapitoly
chapters = await mkn_category_browser(code=None)

# Vybrat kapitolu X (Nemoci dýchací soustavy)
chapter_x = await mkn_category_browser(code="X")

# Zkoumat bloky v kapitole
blocks = chapter_x["children"]

# Získat detaily konkrétní diagnózy
diagnosis = await mkn_diagnosis_getter(code="J06.9")
```

### Příklad 3: Vyhledání poskytovatele a jeho pracovišť

```python
# Vyhledej stomatology v Praze
providers = await nrpzs_provider_searcher(
    query=None,
    city="Praha",
    specialty="Stomatologie",
    page=1,
    page_size=10
)

# Viz detaily prvního nalezeného poskytovatele
if providers["total"] > 0:
    provider_id = providers["results"][0]["provider_id"]
    details = await nrpzs_provider_getter(provider_id=provider_id)

    # Vypsat všechna pracovní místa
    for workplace in details["workplaces"]:
        print(f"{workplace['name']} - {workplace['address']['city']}")
```

### Příklad 4: Vyhledání procedury a jejích parametrů

```python
# Vyhledej konzultaci specialisty
results = await szv_procedure_searcher(
    query="konzultace specialista",
    max_results=10
)

# Získej detaily
procedure_code = results["results"][0]["code"]
procedure = await szv_procedure_getter(code=procedure_code)

# Přístup k bodům a času
print(f"Bodů: {procedure['point_value']}")
print(f"Čas: {procedure['time_minutes']} minut")
print(f"Limit: {procedure['frequency_limit']}")
```

---

## Poznámky pro vývojáře

### Import modulů

```python
# SUKL nástroje
from biomcp.czech.sukl.search import _sukl_drug_search
from biomcp.czech.sukl.getter import _sukl_drug_details, _sukl_spc_getter, _sukl_pil_getter
from biomcp.czech.sukl.availability import _sukl_availability_check

# MKN-10 nástroje
from biomcp.czech.mkn.search import _mkn_search, _mkn_get, _mkn_browse

# NRPZS nástroje
from biomcp.czech.nrpzs.search import _nrpzs_search, _nrpzs_get

# SZV nástroje
from biomcp.czech.szv.search import _szv_search, _szv_get

# VZP nástroje
from biomcp.czech.vzp.search import _vzp_search, _vzp_get
```

### Asynchronní volání

Všechny veřejné nástroje jsou **asynchronní** a musí být volány s `await`:

```python
async def example():
    result = await sukl_drug_searcher(query="ibuprofen")
    return result
```

### Zpracování odpovědí

Všechny nástroje vracejí **JSON řetězce**, nikoli objekty:

```python
import json

response = await sukl_drug_getter(sukl_code="0254045")
data = json.loads(response)  # Parsování JSON
```

### Metriky výkonu

Všechny nástroje jsou instrumentovány metrikou `@track_performance`:

```python
@mcp_app.tool()
@track_performance("czechmedmcp.sukl_drug_searcher")
async def sukl_drug_searcher(...):
    ...
```

---

## Další zdroje

- **Projekt:** `/Users/petrsovadina/Desktop/Develope/personal/biomcp/`
- **Zdrojový kód:** `src/biomcp/czech/`
- **Testování:** `tests/`
- **Konfigurační soubor:** `CLAUDE.md`

---

**Verze dokumentu:** 1.0
**Poslední aktualizace:** 2026-02-20
**Autoři:** Zdravotnický datový tým
