# Uživatelská příručka CzechMedMCP

Kompletní příručka pro české zdravotníky a vývojáře pracující s rozšířením
CzechMedMCP platformy BioMCP.

---

**Co se naučíte:** Jak nainstalovat, nakonfigurovat a používat všech 14 českých
zdravotnických MCP nástrojů pro vyhledávání léků, kódování diagnóz, hledání
poskytovatelů, vyhledávání výkonů a dotazy na číselníky pojišťoven.

**Předpoklady:** Python 3.10 nebo novější, základní znalost práce s příkazovou
řádkou.

**Časový odhad:** 30--45 minut na celou příručku, nebo 5 minut na jednotlivý
modul.

**Výsledek:** Běžící server CzechMedMCP připojený ke Claude Desktop (nebo
jinému MCP klientovi) s přístupem k datovým zdrojům SUKL, MKN-10, NRPZS, SZV
a VZP.

---

## Obsah

1. [Úvod](#1-úvod)
2. [Instalace](#2-instalace)
3. [Rychlý start](#3-rychlý-start)
4. [Návody k modulům](#4-návody-k-modulům)
    - [4.1 Vyhledávání léků -- SUKL](#41-vyhledávání-léků--sukl)
    - [4.2 Kódy diagnóz -- MKN-10](#42-kódy-diagnóz--mkn-10)
    - [4.3 Poskytovatelé zdravotních služeb -- NRPZS](#43-poskytovatelé-zdravotních-služeb--nrpzs)
    - [4.4 Zdravotní výkony -- SZV](#44-zdravotní-výkony--szv)
    - [4.5 Číselníky pojišťoven -- VZP](#45-číselníky-pojišťoven--vzp)
5. [Společné vzory použití](#5-společné-vzory-použití)
6. [Přehled CLI příkazů](#6-přehled-cli-příkazů)
7. [Řešení problémů](#7-řešení-problémů)

---

## 1. Úvod

### Co je CzechMedMCP

CzechMedMCP je fork projektu [BioMCP](https://github.com/genomoncology/biomcp)
(licence MIT), který přidává **14 českých zdravotnických MCP nástrojů**
k existující sadě 21+ globálních biomedicínských výzkumných nástrojů.
Zpřístupňuje české státní zdravotnické datové zdroje prostřednictvím
[Model Context Protocol (MCP)](https://modelcontextprotocol.io/), takže
AI asistenti jako Claude Desktop, Cursor a VS Code mohou odpovídat na
zdravotnické otázky s využitím autoritativních českých dat.

### Pro koho je určen

- **Čeští lékaři** -- rychlé vyhledávání léků, kódování diagnóz, doporučení
  poskytovatelů a bodové hodnoty výkonů přímo ve vašem AI asistentovi.
- **Vývojáři ve zdravotnictví** -- integrace českých zdravotnických dat do
  aplikací kompatibilních s MCP.
- **Lékařští výzkumníci** -- propojení českých klinických dat (SUKL, MKN-10)
  s globálními databázemi (PubMed, ClinicalTrials.gov, MyVariant.info)
  v jedné relaci.
- **Správci klinik** -- vyhledávání výkonů a úhrad pro přesné vyúčtování.

### Datové zdroje

| Modul | Zdroj (český název) | Popis | URL |
|-------|---------------------|-------|-----|
| **SUKL** | Státní ústav pro kontrolu léčiv | Registr léčiv, SmPC, PIL, dostupnost | prehledy.sukl.cz |
| **MKN-10** | Mezinárodní klasifikace nemocí, 10. revize | České kódy diagnóz ICD-10 | mkn10.uzis.cz |
| **NRPZS** | Národní registr poskytovatelů zdravotních služeb | Registr poskytovatelů zdravotních služeb | nrpzs.uzis.cz |
| **SZV** | Seznam zdravotních výkonů | Seznam výkonů s bodovými hodnotami | szv.mzcr.cz, nzip.cz |
| **VZP** | Všeobecná zdravotní pojišťovna | Číselníky pojišťovny | vzp.cz |

Všechny datové zdroje jsou **veřejná API české státní správy**. Není
vyžadována žádná autentizace ani API klíče.

---

## 2. Instalace

### 2.1 Předpoklady

- **Python 3.10 nebo novější** (ověřte příkazem `python3 --version`)
- **uv** (doporučeno) nebo **pip** jako správce balíčků
- **Git** pro klonování repozitáře

### 2.2 Instalace ze zdrojového kódu

```bash
# Klonování repozitáře
git clone https://github.com/digimedic/czechmedmcp.git
cd czechmedmcp

# Instalace pomocí uv (doporučeno)
uv pip install -e ".[dev]"

# Nebo pomocí pip
pip install -e ".[dev]"
```

Tímto nainstalujete nástroj příkazové řádky `czechmedmcp` spolu se všemi
závislostmi včetně `httpx`, `pydantic`, `lxml` a `diskcache`.

### 2.3 Nastavení dat MKN-10

Modul MKN-10 používá soubor ClaML XML z UZIS. Při prvním použití se systém
pokusí soubor stáhnout a uložit do mezipaměti automaticky. Pokud automatické
stažení selže, můžete soubor umístit ručně:

```bash
mkdir -p data/mkn10
# Stáhněte soubor ClaML XML z https://mkn10.uzis.cz/o-mkn
# Umístěte jej jako: data/mkn10/mkn10.xml
```

### 2.4 Ověření instalace

```bash
# Ověření dostupnosti CLI
czechmedmcp --help

# Rychlý ověřovací test
czechmedmcp czech sukl search --query "Paralen"
```

Pokud uvidíte JSON výstup s výsledky vyhledávání léků, instalace je funkční.

---

## 3. Rychlý start

### 3.1 Spuštění MCP serveru

CzechMedMCP podporuje dva režimy přenosu:

```bash
# Režim STDIO (výchozí) -- pro Claude Desktop, Cursor, VS Code
czechmedmcp run

# Režim HTTP -- pro produkci, Docker, vzdálené klienty
czechmedmcp run --mode streamable_http --port 8080
```

Po spuštění server zaregistruje **35 MVP nástrojů**: 21 globálních BioMCP
nástrojů + 14 českých zdravotnických nástrojů + 3 základní nástroje +
1 kontrola stavu.

### 3.2 Konfigurace Claude Desktop

Přidejte následující konfiguraci do souboru nastavení MCP v Claude Desktop
(`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "czechmedmcp": {
      "command": "uv",
      "args": ["run", "--with", "czechmedmcp", "czechmedmcp", "run"]
    }
  }
}
```

Po uložení restartujte Claude Desktop. V nabídce nástrojů byste měli vidět
nástroje CzechMedMCP. Zkuste se Clauda zeptat:

> "Najdi mi informace o léku Ibuprofen v registru SUKL."

nebo

> "Jaký je kód MKN-10 pro akutní infarkt myokardu?"

### 3.3 Testování s MCP Inspector

Pro vývoj a ladění použijte webové rozhraní MCP Inspector:

```bash
npx @modelcontextprotocol/inspector uv run --with czechmedmcp czechmedmcp run
```

Otevře se prohlížečové rozhraní, kde můžete prozkoumat všechny zaregistrované
nástroje, odesílat testovací požadavky a prohlížet odpovědi.

### 3.4 Nasazení v Dockeru

```bash
docker build -t czechmedmcp:latest .
docker run -p 8080:8080 czechmedmcp:latest czechmedmcp run --mode streamable_http
```

---

## 4. Návody k modulům

Každá sekce níže provede jedním modulem s reálnými českými příklady.
Sekce můžete číst v libovolném pořadí.

---

### 4.1 Vyhledávání léků -- SUKL

**Zdroj:** Státní ústav pro kontrolu léčiv (SUKL)
**API:** prehledy.sukl.cz (DLP API v1)
**Nástroje:** 5 (search, get, spc, pil, availability)

Modul SUKL se připojuje k českému registru léčiv a poskytuje přístup k údajům
o všech registrovaných léčivých přípravcích včetně obchodních názvů, účinných
látek, ATC kódů, dokumentů SmPC, příbalových letáků a dostupnosti na trhu.

#### 4.1.1 Vyhledání léku podle názvu

Vyhledávání podle obchodního názvu, účinné látky nebo ATC kódu:

```bash
# Hledání podle obchodního názvu
czechmedmcp czech sukl search --query "Paralen"

# Hledání podle účinné látky
czechmedmcp czech sukl search --query "Ibuprofen"

# Hledání podle ATC kódu
czechmedmcp czech sukl search --query "M01AE01"

# Hledání konkrétní značky
czechmedmcp czech sukl search --query "Nurofen"
```

**Příklad odpovědi:**

```json
{
  "total": 12,
  "page": 1,
  "page_size": 10,
  "results": [
    {
      "sukl_code": "0052520",
      "name": "PARALEN 500",
      "active_substance": "Paracetamolum",
      "atc_code": "N02BE01",
      "pharmaceutical_form": "tablety"
    },
    {
      "sukl_code": "0159986",
      "name": "PARALEN GRIP",
      "active_substance": "Paracetamolum",
      "atc_code": "N02BE51",
      "pharmaceutical_form": "potahované tablety"
    }
  ]
}
```

**Stránkování:** Pokud je výsledků více než `page_size`, zvyšte parametr
`page` pro načtení dalších stránek:

```bash
# Stránka 2, 20 výsledků na stránku
czechmedmcp czech sukl search --query "Ibuprofen" --page 2 --page-size 20
```

**Ekvivalent MCP nástroje** (jak jej volá AI asistent):

```
Tool: sukl_drug_searcher
Parameters: {"query": "Paralen", "page": 1, "page_size": 10}
```

#### 4.1.2 Detail léku podle kódu SUKL

Jakmile máte kód SUKL z výsledku vyhledávání, můžete získat kompletní záznam
o léku:

```bash
czechmedmcp czech sukl get "0052520"
```

**Příklad odpovědi:**

```json
{
  "sukl_code": "0052520",
  "name": "PARALEN 500",
  "active_substances": [
    {"name": "Paracetamolum", "strength": "500 mg"}
  ],
  "pharmaceutical_form": "tablety",
  "atc_code": "N02BE01",
  "registration_number": "07/124/69-C",
  "mah": "Zentiva, k.s.",
  "registration_valid_to": "2029-12-31",
  "spc_url": "https://prehledy.sukl.cz/v1/dokumenty/0052520/spc",
  "pil_url": "https://prehledy.sukl.cz/v1/dokumenty/0052520/pil",
  "source": "SUKL"
}
```

Odpověď obsahuje:

- **active_substances** -- seznam účinných látek se sílou
- **atc_code** -- klasifikace WHO ATC
- **registration_number** -- číslo registrace (rozhodnutí o registraci)
- **mah** -- Držitel rozhodnutí o registraci (Marketing Authorization Holder)
- **spc_url / pil_url** -- odkazy na oficiální dokumenty

#### 4.1.3 Přečtení SPC

SmPC (Souhrn údajů o přípravku / Summary of Product Characteristics) je
autoritativní referenční dokument pro předepisování léku:

```bash
czechmedmcp czech sukl spc "0052520"
```

**Příklad odpovědi:**

```json
{
  "sukl_code": "0052520",
  "name": "PARALEN 500",
  "spc_text": "SmPC document available at: https://prehledy.sukl.cz/v1/dokumenty/0052520/spc",
  "spc_url": "https://prehledy.sukl.cz/v1/dokumenty/0052520/spc",
  "source": "SUKL"
}
```

`spc_url` odkazuje na oficiální dokument SUKL. AI asistent může tento odkaz
následovat a získat a shrnout celý obsah SmPC.

#### 4.1.4 Přečtení příbalového letáku

PIL (Příbalová informace / Patient Information Leaflet) je dokument
poskytovaný pacientům:

```bash
czechmedmcp czech sukl pil "0052520"
```

**Příklad odpovědi:**

```json
{
  "sukl_code": "0052520",
  "name": "PARALEN 500",
  "pil_text": "PIL document available at: https://prehledy.sukl.cz/v1/dokumenty/0052520/pil",
  "pil_url": "https://prehledy.sukl.cz/v1/dokumenty/0052520/pil",
  "source": "SUKL"
}
```

#### 4.1.5 Kontrola dostupnosti

Ověření, zda je lék aktuálně dostupný na českém trhu:

```bash
czechmedmcp czech sukl availability "0052520"
```

**Příklad odpovědi:**

```json
{
  "sukl_code": "0052520",
  "name": "PARALEN 500",
  "status": "available",
  "last_checked": "2026-02-20T10:30:00+00:00",
  "note": null,
  "source": "SUKL"
}
```

Pole `status` může nabývat tří hodnot:

| Stav | Český název | Popis |
|------|-------------|-------|
| `available` | Dostupný | Lék je v aktivní distribuci |
| `limited` | Omezená dostupnost | Hlášena omezená dostupnost |
| `unavailable` | Nedostupný | Lék aktuálně není distribuován |

Data o dostupnosti jsou ověřována proti koncovému bodu SUKL VPOIS (distribuce)
a ukládána do mezipaměti na 1 hodinu.

#### Typický postup práce se SUKL

Lékař vyhledávající informace o léku by obvykle postupoval takto:

1. **Vyhledání** podle názvu léku nebo účinné látky pro zjištění kódu SUKL
2. **Získání podrobností** pro zobrazení registračních údajů, ATC kódu a
   držitele registrace
3. **Přečtení SmPC** pro informace o předepisování
4. **Kontrola dostupnosti** pro ověření, že je lék na trhu

V AI asistentovi probíhá tento postup konverzačně:

> **Lékař:** "Potřebuji najít informace o Nurofen 400mg -- je dostupný
> a jaké je SPC?"
>
> AI asistent zavolá `sukl_drug_searcher` s dotazem "Nurofen 400",
> poté `sukl_drug_getter` s vráceným kódem SUKL, dále
> `sukl_availability_checker` a `sukl_spc_getter`.

---

### 4.2 Kódy diagnóz -- MKN-10

**Zdroj:** UZIS (Ústav zdravotnických informací a statistiky ČR)
**Data:** ClaML XML, zpracováno lokálně
**Nástroje:** 3 (search, get, browse)

Modul MKN-10 poskytuje českou lokalizaci Mezinárodní klasifikace nemocí,
10. revize (ICD-10). Data jsou zpracována ze souboru ClaML XML a indexována
v paměti pro vyhledávání v řádu mikrosekund.

#### 4.2.1 Hledání podle kódu MKN-10

Pokud znáte kód (nebo jeho prefix), systém provede prefixové vyhledávání:

```bash
# Přesný kód
czechmedmcp czech mkn search --query "J06.9"

# Prefix kódu
czechmedmcp czech mkn search --query "J06"

# Rozsah bloku
czechmedmcp czech mkn search --query "I20-I25"
```

**Příklad odpovědi pro "J06.9":**

```json
{
  "query": "J06.9",
  "total": 1,
  "results": [
    {
      "code": "J06.9",
      "name_cs": "Akutní infekce horních cest dýchacích NS",
      "kind": "category"
    }
  ]
}
```

#### 4.2.2 Hledání podle českého textu

Fulltextové vyhledávání pracuje s českou lékařskou terminologií. Diakritika
je zpracována transparentně (viz sekce 5.1):

```bash
# Vyhledávání českých termínů
czechmedmcp czech mkn search --query "angina"
czechmedmcp czech mkn search --query "akutní infarkt myokardu"
czechmedmcp czech mkn search --query "diabetes mellitus"
czechmedmcp czech mkn search --query "zlomenina femuru"

# Bez ohledu na diakritiku
czechmedmcp czech mkn search --query "zanet plic"
# je ekvivalentní
czechmedmcp czech mkn search --query "zánět plic"
```

**Příklad odpovědi pro "infarkt":**

```json
{
  "query": "infarkt",
  "total": 5,
  "results": [
    {
      "code": "I21",
      "name_cs": "Akutní infarkt myokardu",
      "kind": "category"
    },
    {
      "code": "I22",
      "name_cs": "Následný infarkt myokardu",
      "kind": "category"
    },
    {
      "code": "I23",
      "name_cs": "Některé akutní komplikace po akutním infarktu myokardu",
      "kind": "category"
    },
    {
      "code": "I63",
      "name_cs": "Mozkový infarkt",
      "kind": "category"
    }
  ]
}
```

Textové vyhledávání používá invertovaný slovní index: každé slovo v českém
názvu je normalizováno (odstraněna diakritika, převedeno na malá písmena)
a indexováno ke svému kódu MKN-10. Pro vrácení výsledku musí odpovídat
všechna slova dotazu.

#### 4.2.3 Podrobnosti diagnózy

Získání kompletního záznamu pro konkrétní kód včetně jeho pozice v hierarchii
MKN-10:

```bash
czechmedmcp czech mkn get "I21.0"
```

**Příklad odpovědi:**

```json
{
  "code": "I21.0",
  "name_cs": "Akutní transmurální infarkt myokardu přední stěny",
  "name_en": null,
  "definition": null,
  "hierarchy": {
    "chapter": "IX",
    "chapter_name": "Nemoci oběhové soustavy",
    "block": "I20-I25",
    "block_name": "Ischemické nemoci srdeční",
    "category": "I21"
  },
  "includes": [],
  "excludes": [],
  "modifiers": [],
  "source": "UZIS/MKN-10"
}
```

Objekt `hierarchy` ukazuje přesné zařazení diagnózy:

- **Kapitola IX** -- Nemoci oběhové soustavy
- **Blok I20-I25** -- Ischemické nemoci srdeční
- **Kategorie I21** -- Akutní infarkt myokardu

#### 4.2.4 Procházení hierarchie kategorií

Procházení stromové struktury MKN-10. Bez zadání kódu získáte seznam všech
kapitol:

```bash
# Všechny kapitoly
czechmedmcp czech mkn browse

# Procházení konkrétního bloku
czechmedmcp czech mkn browse "J00-J06"

# Procházení kapitoly
czechmedmcp czech mkn browse "X"
```

**Příklad: procházení kapitol (kořenová úroveň):**

```json
{
  "type": "chapters",
  "items": [
    {"code": "I", "name_cs": "Některé infekční a parazitární nemoci", "kind": "chapter", "children": ["A00-A09", "A15-A19", "..."]},
    {"code": "II", "name_cs": "Novotvary", "kind": "chapter", "children": ["C00-C14", "..."]},
    {"code": "IX", "name_cs": "Nemoci oběhové soustavy", "kind": "chapter", "children": ["I00-I02", "I05-I09", "..."]},
    {"code": "X", "name_cs": "Nemoci dýchací soustavy", "kind": "chapter", "children": ["J00-J06", "J09-J18", "..."]}
  ]
}
```

**Příklad: procházení bloku J00-J06:**

```json
{
  "code": "J00-J06",
  "name_cs": "Akutní infekce horních cest dýchacích",
  "kind": "block",
  "parent_code": "X",
  "children": [
    {"code": "J00", "name_cs": "Akutní nazofaryngitida [běžné nachlazení]", "kind": "category"},
    {"code": "J01", "name_cs": "Akutní sinusitida", "kind": "category"},
    {"code": "J02", "name_cs": "Akutní faryngitida", "kind": "category"},
    {"code": "J03", "name_cs": "Akutní tonzilitida", "kind": "category"},
    {"code": "J04", "name_cs": "Akutní laryngitida a tracheitida", "kind": "category"},
    {"code": "J05", "name_cs": "Akutní obstrukční laryngitida a epiglotitida", "kind": "category"},
    {"code": "J06", "name_cs": "Akutní infekce horních cest dýchacích na více místech a NS", "kind": "category"}
  ]
}
```

#### Porozumění hierarchii

Klasifikace MKN-10 má čtyři úrovně:

```
Kapitola (Chapter)         např. X -- Nemoci dýchací soustavy
  |
  +-- Blok (Block)         např. J00-J06 -- Akutní infekce horních cest dýchacích
       |
       +-- Kategorie (Category)       např. J06 -- Akutní infekce horních ...
            |
            +-- Podkategorie (Subcategory)  např. J06.9 -- ... NS
```

Při kódování diagnózy lékaři obvykle postupují od obecné kapitoly ke
konkrétní podkategorii.

---

### 4.3 Poskytovatelé zdravotních služeb -- NRPZS

**Zdroj:** Národní registr poskytovatelů zdravotních služeb (NRPZS)
**API:** nrpzs.uzis.cz/api/v1
**Nástroje:** 2 (search, get)

Modul NRPZS prohledává Národní registr poskytovatelů zdravotních služeb,
který pokrývá všechna registrovaná zdravotnická zařízení v České republice.
Používejte jej pro doporučení pacientů, hledání specialistů nebo vyhledávání
zařízení podle města.

#### 4.3.1 Hledání podle názvu, města nebo odbornosti

Vyhledávat můžete podle libovolné kombinace názvu, města a odbornosti.
Musí být zadán alespoň jeden filtr:

```bash
# Hledání podle města a odbornosti
czechmedmcp czech nrpzs search --city "Praha" --specialty "kardiologie"

# Hledání podle názvu poskytovatele
czechmedmcp czech nrpzs search --query "Fakultní nemocnice"

# Odbornost ve městě
czechmedmcp czech nrpzs search --city "Brno" --specialty "neurologie"

# Široké hledání ve městě
czechmedmcp czech nrpzs search --city "Ostrava"

# Hledání specifického typu poskytovatele
czechmedmcp czech nrpzs search --query "lékárna" --city "Plzeň"
```

**Příklad odpovědi:**

```json
{
  "total": 47,
  "page": 1,
  "page_size": 10,
  "results": [
    {
      "provider_id": "12345",
      "name": "MUDr. Jan Novák - Kardiologická ambulance",
      "city": "Praha",
      "specialties": ["kardiologie"]
    },
    {
      "provider_id": "12346",
      "name": "Kardiologické centrum IKEM",
      "city": "Praha",
      "specialties": ["kardiologie", "vnitřní lékařství"]
    }
  ]
}
```

**Parametry vyhledávání:**

| Parametr | Povinný | Popis |
|----------|---------|-------|
| `query` | Ne | Název poskytovatele nebo klíčové slovo |
| `city` | Ne | Název města (např. "Praha", "Brno", "Olomouc") |
| `specialty` | Ne | Lékařská odbornost (např. "kardiologie", "ortopedie") |
| `page` | Ne | Číslo stránky (výchozí 1) |
| `page_size` | Ne | Výsledků na stránku (výchozí 10, maximum 100) |

Musí být zadán alespoň jeden z parametrů `query`, `city` nebo `specialty`.

#### 4.3.2 Podrobnosti poskytovatele

Získání kompletního záznamu včetně pracovišť, adres a kontaktních údajů:

```bash
czechmedmcp czech nrpzs get "12345"
```

**Příklad odpovědi:**

```json
{
  "provider_id": "12345",
  "name": "MUDr. Jan Novák - Kardiologická ambulance",
  "legal_form": "fyzická osoba",
  "ico": "12345678",
  "address": {
    "street": "Vinohradská 123",
    "city": "Praha",
    "postal_code": "12000",
    "region": "Hlavní město Praha"
  },
  "specialties": ["kardiologie"],
  "care_types": ["ambulantní"],
  "workplaces": [
    {
      "workplace_id": "67890",
      "name": "Kardiologická ambulance",
      "address": {
        "street": "Vinohradská 123",
        "city": "Praha",
        "postal_code": "12000",
        "region": "Hlavní město Praha"
      },
      "specialties": ["kardiologie"],
      "contact": {
        "phone": "+420 222 333 444",
        "email": "ambulance@example.cz",
        "website": "https://www.example.cz"
      }
    }
  ],
  "registration_number": "A-123-456",
  "source": "NRPZS"
}
```

Seznam `workplaces` obsahuje všechna registrovaná místa péče daného
poskytovatele, každé s vlastní adresou, odbornostmi a kontaktními údaji.

#### 4.3.3 Příklady kombinovaných filtrů

```bash
# Ortopedie v Olomouckém kraji
czechmedmcp czech nrpzs search --city "Olomouc" --specialty "ortopedie"

# Nemocnice v Brně
czechmedmcp czech nrpzs search --query "Nemocnice" --city "Brno"

# Dermatologové v Liberci
czechmedmcp czech nrpzs search --city "Liberec" --specialty "dermatovenerologie"

# Lékárny v Hradci Králové
czechmedmcp czech nrpzs search --query "lékárna" --city "Hradec Králové"

# Pediatři -- široké vyhledávání
czechmedmcp czech nrpzs search --specialty "pediatrie"
```

---

### 4.4 Zdravotní výkony -- SZV

**Zdroj:** Seznam zdravotních výkonů (MZCR), NZIP Open Data API v3
**API:** nzip.cz (primární), szv.mzcr.cz (doplňkový)
**Nástroje:** 2 (search, get)

Modul SZV poskytuje přístup k Seznamu zdravotních výkonů včetně kódů výkonů,
názvů, bodových hodnot, časových dotací a omezení odbornosti. Tato data jsou
nezbytná pro vyúčtování a úhrady.

#### 4.4.1 Hledání podle kódu nebo názvu

```bash
# Hledání podle kódu výkonu
czechmedmcp czech szv search --query "09513"

# Hledání podle názvu
czechmedmcp czech szv search --query "EKG"

# Hledání česky
czechmedmcp czech szv search --query "elektrokardiografie"

# Hledání radiologických výkonů
czechmedmcp czech szv search --query "rentgen"

# Hledání s více výsledky
czechmedmcp czech szv search --query "vyšetření" --max-results 20
```

**Příklad odpovědi:**

```json
{
  "total": 3,
  "results": [
    {
      "code": "09513",
      "name": "EKG natočení a popis 12ti a vícesvodového záznamu",
      "point_value": 101,
      "category": "09"
    },
    {
      "code": "09515",
      "name": "Zátěžové vyšetření EKG",
      "point_value": 386,
      "category": "09"
    }
  ]
}
```

#### 4.4.2 Podrobnosti výkonu

Získání kompletního záznamu pro kód výkonu:

```bash
czechmedmcp czech szv get "09513"
```

**Příklad odpovědi:**

```json
{
  "code": "09513",
  "name": "EKG natočení a popis 12ti a vícesvodového záznamu",
  "category": "09",
  "category_name": "Kardiologie",
  "point_value": 101,
  "time_minutes": 15,
  "frequency_limit": null,
  "specialty_codes": ["101", "107"],
  "material_requirements": null,
  "notes": null,
  "source": "MZCR/SZV"
}
```

Klíčová pole pro vyúčtování:

- **point_value** -- bodová hodnota (základ pro výpočet úhrady)
- **time_minutes** -- odhadovaný čas provedení výkonu
- **frequency_limit** -- případná omezení frekvence vykazování
- **specialty_codes** -- které odbornosti mohou výkon provádět a vykazovat

---

### 4.5 Číselníky pojišťoven -- VZP

**Zdroj:** Všeobecná zdravotní pojišťovna (VZP)
**API:** vzp.cz
**Nástroje:** 2 (search, get)

Modul VZP prohledává číselníky publikované Všeobecnou zdravotní pojišťovnou
České republiky. Tyto číselníky mapují kódy výkonů, kódy diagnóz a ATC kódy
léků na pravidla úhrad.

#### 4.5.1 Prohledávání číselníků

Vyhledávání napříč všemi typy číselníků nebo filtrování podle konkrétního
typu:

```bash
# Prohledání všech číselníků
czechmedmcp czech vzp search --query "antibiotika"

# Filtrování podle typu
czechmedmcp czech vzp search --query "EKG" --type "seznam_vykonu"

# Hledání lékových položek
czechmedmcp czech vzp search --query "ibuprofen" --type "atc"

# Hledání kódů diagnóz
czechmedmcp czech vzp search --query "hypertenze" --type "diagnoza"
```

**Dostupné typy číselníků:**

| Typ | Popis |
|-----|-------|
| `seznam_vykonu` | Seznam zdravotních výkonů (kódy výkonů) |
| `diagnoza` | Diagnózy (kódy diagnóz) |
| `lekarsky_predpis` | Lékařské předpisy |
| `atc` | ATC klasifikace (klasifikace léčiv) |

**Příklad odpovědi:**

```json
{
  "total": 5,
  "results": [
    {
      "codebook_type": "atc",
      "code": "J01CA04",
      "name": "Amoxicilin"
    },
    {
      "codebook_type": "atc",
      "code": "J01CR02",
      "name": "Amoxicilin a inhibitor enzymů"
    }
  ]
}
```

#### 4.5.2 Podrobnosti položky číselníku

Získání kompletního záznamu pro konkrétní položku číselníku podle typu a kódu:

```bash
czechmedmcp czech vzp get "seznam_vykonu" "09513"
czechmedmcp czech vzp get "atc" "J01CA04"
czechmedmcp czech vzp get "diagnoza" "I21"
```

**Příklad odpovědi:**

```json
{
  "codebook_type": "seznam_vykonu",
  "code": "09513",
  "name": "EKG natočení a popis",
  "description": "EKG natočení a popis 12ti a vícesvodového záznamu",
  "valid_from": "2025-01-01",
  "valid_to": null,
  "rules": ["Odbornost 101, 107"],
  "related_codes": ["09515", "09517"],
  "source": "VZP"
}
```

Klíčová pole:

- **valid_from / valid_to** -- období platnosti dané položky číselníku
- **rules** -- pravidla úhrad a omezení
- **related_codes** -- křížové odkazy na související položky

---

## 5. Společné vzory použití

### 5.1 Práce s diakritikou

Všechny vyhledávací nástroje CzechMedMCP zpracovávají českou diakritiku
transparentně. To znamená, že pro správné fungování vyhledávání nemusíte
psát diakritiku -- obě formy vrací ekvivalentní výsledky:

| S diakritikou | Bez diakritiky | Výsledek |
|---------------|----------------|----------|
| "léky" | "leky" | Stejné výsledky |
| "Ústí nad Labem" | "Usti nad Labem" | Stejné výsledky |
| "kardiologie" | "kardiologie" | Stejné výsledky |
| "zánět plic" | "zanet plic" | Stejné výsledky |
| "těhotenství" | "tehotenstvi" | Stejné výsledky |
| "léčivý přípravek" | "lecivy pripravek" | Stejné výsledky |

**Jak to funguje:** Systém používá Unicode NFD normalizaci k rozkladu znaků
a následně odstraní kombinační značky. Tím se převedou znaky jako
`č -> c`, `ř -> r`, `ž -> z`, `é -> e`, `ů -> u`. Dotaz i indexovaná data
jsou před porovnáním normalizovány. Původní český text (s diakritikou) je
v odpovědi vždy zachován.

Implementace v `src/biomcp/czech/diacritics.py`:

```python
import unicodedata

def strip_diacritics(text: str) -> str:
    nfkd = unicodedata.normalize("NFD", text)
    return "".join(c for c in nfkd if unicodedata.category(c) != "Mn").lower()
```

### 5.2 Stránkování

Nástroje, které mohou vracet mnoho výsledků, podporují stránkování:

**SUKL a NRPZS** používají parametry `page` a `page_size`:

```bash
# Prvních 10 výsledků (výchozí)
czechmedmcp czech sukl search --query "Ibuprofen"

# Výsledky 11-20
czechmedmcp czech sukl search --query "Ibuprofen" --page 2

# 50 výsledků na stránku
czechmedmcp czech sukl search --query "Ibuprofen" --page 1 --page-size 50
```

**MKN-10, SZV a VZP** používají parametr `max_results` pro omezení výstupu:

```bash
# Výchozí: 10 výsledků
czechmedmcp czech mkn search --query "infarkt"

# Až 50 výsledků
czechmedmcp czech mkn search --query "infarkt" --max-results 50
```

Odpověď vždy obsahuje pole `total` udávající celkový počet odpovídajících
záznamů bez ohledu na velikost stránky.

### 5.3 Zpracování chyb

Pokud dojde k chybě, nástroje vrátí JSON objekt s klíčem `error` namísto
vyhození výjimky. To umožňuje AI asistentům zpracovat chyby elegantně.

**Lék nenalezen:**

```json
{"error": "Drug not found: 9999999"}
```

**API nedostupné (s návratem z mezipaměti):**

```json
{
  "total": 0,
  "page": 1,
  "page_size": 10,
  "results": [],
  "error": "SUKL API unavailable: Connection timeout"
}
```

**Neplatný kód MKN-10:**

```json
{"error": "Code not found: XYZ"}
```

**Data MKN-10 nenačtena:**

```json
{"error": "No MKN-10 data loaded.", "results": []}
```

**Chybějící povinný parametr (NRPZS):**

Pro vyhledávací nástroj NRPZS musí být zadán alespoň jeden z parametrů
`query`, `city` nebo `specialty`. Pokud není zadán žádný, odpověď bude prázdná.

### 5.4 Mezipaměť a offline režim

Všechny odpovědi z externích API jsou ukládány do mezipaměti pomocí
**diskcache** (perzistentní disková mezipaměť). To přináší dva výhody:

1. **Rychlost** -- opakované dotazy vrací výsledky z mezipaměti v řádu
   milisekund namísto čekání na síťovou odezvu
2. **Offline záloha** -- pokud je nadřazené API dočasně nedostupné, systém
   vrátí poslední uloženou odpověď z mezipaměti

**Doby platnosti mezipaměti:**

| Typ dat | Doba platnosti (TTL) | Odůvodnění |
|---------|---------------------|------------|
| Seznam léků (SUKL) | 24 hodin | SUKL aktualizuje měsíčně |
| Podrobnosti léku | 7 dní | Statická registrační data |
| Dostupnost léku | 1 hodina | Mění se častěji |
| Zpracovaný index MKN-10 | 30 dní | Aktualizováno ročně |
| Výsledky vyhledávání NRPZS | 24 hodin | Měsíční aktualizace registru |
| Podrobnosti poskytovatele NRPZS | 7 dní | Relativně statické |
| Seznam výkonů SZV | 24 hodin | Aktualizace na základě vyhlášky |
| Podrobnosti výkonu SZV | 7 dní | Statické mezi vyhláškami |
| Položky číselníku VZP | 24 hodin | Periodické aktualizace |
| Podrobnosti položky VZP | 7 dní | Statické mezi verzemi |

Mezipaměť je uložena ve výchozím adresáři `diskcache`. Přetrvává i po
restartování serveru. Pro vynucení nového stažení dat smažte adresář
mezipaměti.

### 5.5 Kombinace českých a globálních nástrojů

Síla CzechMedMCP spočívá v kombinaci českých a globálních dat v jedné relaci.
Zde je několik ukázkových konverzačních scénářů:

**Výzkum léku -- z českých do globálních dat:**

> "Najdi mi lék Ibuprofen v SUKL registru a pak mi najdi klinické
> studie pro Ibuprofen na ClinicalTrials.gov."
>
> AI zavolá `sukl_drug_searcher`, poté `trial_searcher` se stejnou
> účinnou látkou.

**Kódování diagnózy -- MKN-10 a literatura:**

> "Jaký je kód MKN-10 pro melanom? Najdi mi nejnovější články o
> BRAF mutacích u melanomu."
>
> AI zavolá `mkn_diagnosis_searcher`, poté `article_search` s parametry
> gene=BRAF a disease=melanoma.

**Doporučení poskytovatele s důkazy:**

> "Najdi kardiology v Brně a ukaž mi nejnovější klinické studie
> pro fibrilaci síní."
>
> AI zavolá `nrpzs_provider_searcher` s parametry city="Brno" a
> specialty="kardiologie", poté `trial_searcher` s parametrem
> condition="atrial fibrillation".

---

## 6. Přehled CLI příkazů

Všechny české nástroje jsou dostupné pod skupinou příkazů `czechmedmcp czech`.
Každý podpříkaz ve výchozím nastavení vrací JSON.

### Příkazy SUKL

```bash
# Vyhledávání léků
czechmedmcp czech sukl search --query "QUERY" [--page N] [--page-size N]

# Podrobnosti léku
czechmedmcp czech sukl get SUKL_CODE

# SmPC (Souhrn údajů o přípravku)
czechmedmcp czech sukl spc SUKL_CODE

# PIL (Příbalový leták)
czechmedmcp czech sukl pil SUKL_CODE

# Kontrola dostupnosti
czechmedmcp czech sukl availability SUKL_CODE
```

### Příkazy MKN-10

```bash
# Vyhledávání diagnóz (podle kódu nebo textu)
czechmedmcp czech mkn search --query "QUERY" [--max-results N]

# Podrobnosti diagnózy
czechmedmcp czech mkn get CODE

# Procházení hierarchie (bez kódu pro kapitoly)
czechmedmcp czech mkn browse [CODE]
```

### Příkazy NRPZS

```bash
# Vyhledávání poskytovatelů
czechmedmcp czech nrpzs search [--query "NAME"] [--city "CITY"] [--specialty "SPEC"] [--page N] [--page-size N]

# Podrobnosti poskytovatele
czechmedmcp czech nrpzs get PROVIDER_ID
```

### Příkazy SZV

```bash
# Vyhledávání výkonů
czechmedmcp czech szv search --query "QUERY" [--max-results N]

# Podrobnosti výkonu
czechmedmcp czech szv get CODE
```

### Příkazy VZP

```bash
# Prohledávání číselníků
czechmedmcp czech vzp search --query "QUERY" [--type "CODEBOOK_TYPE"] [--max-results N]

# Podrobnosti položky číselníku
czechmedmcp czech vzp get CODEBOOK_TYPE CODE
```

### Přehled názvů MCP nástrojů

Pro integraci s AI asistentem jsou zde přesné názvy MCP nástrojů:

| Modul | Název MCP nástroje | Ekvivalent CLI |
|-------|--------------------|----------------|
| SUKL | `sukl_drug_searcher` | `czech sukl search` |
| SUKL | `sukl_drug_getter` | `czech sukl get` |
| SUKL | `sukl_spc_getter` | `czech sukl spc` |
| SUKL | `sukl_pil_getter` | `czech sukl pil` |
| SUKL | `sukl_availability_checker` | `czech sukl availability` |
| MKN-10 | `mkn_diagnosis_searcher` | `czech mkn search` |
| MKN-10 | `mkn_diagnosis_getter` | `czech mkn get` |
| MKN-10 | `mkn_category_browser` | `czech mkn browse` |
| NRPZS | `nrpzs_provider_searcher` | `czech nrpzs search` |
| NRPZS | `nrpzs_provider_getter` | `czech nrpzs get` |
| SZV | `szv_procedure_searcher` | `czech szv search` |
| SZV | `szv_procedure_getter` | `czech szv get` |
| VZP | `vzp_codebook_searcher` | `czech vzp search` |
| VZP | `vzp_codebook_getter` | `czech vzp get` |

---

## 7. Řešení problémů

### "No MKN-10 data loaded"

**Příznak:** Příkazy vyhledávání a procházení MKN-10 vracejí
`{"error": "No MKN-10 data loaded."}`.

**Příčina:** Soubor ClaML XML nebyl stažen nebo se nenachází na očekávaném
místě.

**Řešení:**

1. Stáhněte soubor ClaML XML z
   [mkn10.uzis.cz/o-mkn](https://mkn10.uzis.cz/o-mkn)
2. Umístěte jej jako `data/mkn10/mkn10.xml` relativně ke kořenu projektu
3. Restartujte server

### "SUKL API unavailable"

**Příznak:** Vyhledávání léků vrací výsledky s polem `error` obsahujícím
"SUKL API unavailable" nebo "Connection timeout".

**Příčina:** Veřejné API SUKL na prehledy.sukl.cz je dočasně nedostupné
nebo omezuje vaše požadavky.

**Řešení:**

1. Počkejte několik minut a zkuste to znovu -- API může být v údržbě
2. Zkontrolujte své internetové připojení
3. Pokud máte dříve uložené výsledky v mezipaměti, budou automaticky použity
   jako záloha
4. Při přetrvávajících problémech ověřte dostupnost
   [prehledy.sukl.cz](https://prehledy.sukl.cz/) v prohlížeči

### "Drug not found: XXXXXXX"

**Příznak:** `sukl_drug_getter`, `sukl_spc_getter` nebo
`sukl_availability_checker` vrací chybu "Drug not found".

**Příčina:** Kód SUKL v registru neexistuje, nebo mohl být lék stažen
z registrace.

**Řešení:**

1. Ověřte, že kód SUKL je správný (7místný číselný řetězec, např. "0052520")
2. Nejprve použijte `sukl_drug_searcher` pro vyhledání podle názvu a získání
   správného kódu
3. Pozor, kódy SUKL jsou doplněny nulami zleva: "52520" by mělo být "0052520"

### "Provider not found"

**Příznak:** `nrpzs_provider_getter` vrací chybu "Provider not found".

**Příčina:** ID poskytovatele v registru NRPZS neexistuje, nebo poskytovatel
již není registrován.

**Řešení:**

1. Použijte `nrpzs_provider_searcher` pro nalezení správného ID poskytovatele
2. Data NRPZS se aktualizují měsíčně; poskytovatel mohl být nedávno
   odregistrován

### "Procedure not found"

**Příznak:** `szv_procedure_getter` vrací chybu "Procedure not found".

**Příčina:** Kód výkonu v databázi SZV neexistuje.

**Řešení:**

1. Ověřte, že kód je 5místný řetězec (např. "09513")
2. Nejprve použijte `szv_procedure_searcher` pro vyhledání podle názvu
3. Kódy výkonů se mohou měnit mezi aktualizacemi vyhlášky

### "Codebook entry not found"

**Příznak:** `vzp_codebook_getter` vrací chybu "Codebook entry not found".

**Příčina:** Typ číselníku nebo kód v číselnících VZP neexistuje.

**Řešení:**

1. Ověřte, že typ číselníku je jeden z: `seznam_vykonu`, `diagnoza`,
   `lekarsky_predpis`, `atc`
2. Nejprve použijte `vzp_codebook_searcher` pro nalezení správného kódu
3. Zkontrolujte, zda je položka stále platná (nevypršela)

### Diakritika neodpovídá

**Příznak:** Vyhledávání vrací odlišné výsledky při použití diakritiky
oproti prostému ASCII.

**Příčina:** K tomuto by nemělo docházet -- normalizace diakritiky je
aplikována automaticky. Pokud k tomu dojde, může to indikovat chybu při
zpracování.

**Řešení:**

1. Zkuste obě formy: "zánět" a "zanet" by měly dávat stejné výsledky
2. Pokud se liší, nahlaste to jako chybu
3. Jako dočasné řešení zkuste verzi v ASCII (bez diakritiky)

### Server se nespouští

**Příznak:** `czechmedmcp run` selže nebo server neodpovídá.

**Řešení:**

1. Ověřte, že je nainstalován Python 3.10+: `python3 --version`
2. Ověřte, že je balíček nainstalován: `czechmedmcp --help`
3. Zkontrolujte konflikty portů při použití HTTP režimu:
   `czechmedmcp run --mode streamable_http --port 8081`
4. Zkontrolujte logy pro chybové hlášení

### Claude Desktop nevidí nástroje

**Příznak:** Claude Desktop nezobrazuje nástroje CzechMedMCP.

**Řešení:**

1. Ověřte, že konfigurace v `claude_desktop_config.json` odpovídá formátu
   uvedenému v sekci 3.2
2. Kompletně restartujte Claude Desktop (ukončete a znovu otevřete)
3. Nejprve otestujte server ručně:
   `czechmedmcp czech sukl search --query "test"`
4. Použijte MCP Inspector k ověření registrace nástrojů:
   `npx @modelcontextprotocol/inspector uv run --with czechmedmcp czechmedmcp run`

---

## Příloha: URL datových zdrojů

| Služba | Základní URL | Autentizace | Data |
|--------|-------------|-------------|------|
| SUKL DLP API v1 | `https://prehledy.sukl.cz/` | Žádná | Registr léčiv |
| SUKL Open Data | `https://opendata.sukl.cz/` | Žádná | Hromadné stahování |
| SUKL Swagger dokumentace | `https://prehledy.sukl.cz/docs/` | Žádná | Specifikace API |
| NRPZS API v1 | `https://nrpzs.uzis.cz/api/v1/` | Žádná | Poskytovatelé |
| NRPZS API dokumentace | `https://nrpzs.uzis.cz/api/doc` | Žádná | Specifikace API |
| MKN-10 prohlížeč | `https://mkn10.uzis.cz/` | Žádná | Kódy ICD-10 |
| NZIP Open Data v3 | `https://nzip.cz/` | Žádná | Výkony |
| SZV databáze | `https://szv.mzcr.cz/` | Žádná | Registr výkonů |
| VZP číselníky | `https://www.vzp.cz/` | Žádná | Pojistná data |

Všechna API jsou veřejná. Pro čtecí přístup používaný CzechMedMCP nejsou
vyžadovány žádné API klíče, certifikáty ani registrace.

---

## Příloha: Validační pravidla

| Entita | Formát | Příklad |
|--------|--------|---------|
| Kód SUKL | 7místný číselný řetězec (doplněný nulami) | `0052520` |
| Kód MKN-10 | `[A-Z]\d{2}(\.\d{1,2})?` | `J06.9`, `I21.0`, `C34` |
| Blok MKN-10 | `[A-Z]\d{2}-[A-Z]\d{2}` | `J00-J06`, `I20-I25` |
| Kód výkonu SZV | 5místný číselný řetězec | `09513` |
| ID poskytovatele NRPZS | Číselný identifikátor | `12345` |
| Typ číselníku VZP | Jeden z: `seznam_vykonu`, `diagnoza`, `lekarsky_predpis`, `atc` | `seznam_vykonu` |

---

*Tato příručka pokrývá CzechMedMCP v0.4.6, založený na BioMCP se 14 českými
zdravotnickými MCP nástroji v 5 modulech. Pro globální nástroje BioMCP
(články, studie, varianty, geny atd.) viz
[hlavní dokumentaci BioMCP](index.md).*
