# Rychlý start: CzechMedMCP

## Předpoklady

- Python >= 3.10
- uv (doporučeno) nebo pip

## Instalace

```bash
# Klonování repozitáře
git clone https://github.com/digimedic/czechmedmcp.git
cd czechmedmcp

# Instalace pomocí uv (doporučeno)
uv pip install -e ".[dev]"

# Nebo pomocí pip
pip install -e ".[dev]"
```

## Nastavení dat MKN-10

Stáhněte český ClaML XML soubor MKN-10:

```bash
# Vytvořte adresář pro data (vytvoří se automaticky)
mkdir -p data/mkn10
# Stáhněte z UZIS (manuální krok - viz https://mkn10.uzis.cz/o-mkn)
# Umístěte ClaML XML soubor jako: data/mkn10/mkn10.xml
```

## Spuštění MCP serveru

```bash
# Výchozí režim STDIO (pro Claude Desktop, Cursor)
biomcp run

# HTTP režim (pro produkci/Docker)
biomcp run --mode streamable_http --port 8080
```

## Konfigurace Claude Desktop

Přidejte do nastavení Claude Desktop:

```json
{
  "mcpServers": {
    "czechmedmcp": {
      "command": "uv",
      "args": ["run", "--with", "biomcp-python", "biomcp", "run"]
    }
  }
}
```

## Rychlý test - České nástroje

```bash
# Vyhledání léku
biomcp czech sukl search --query "Ibuprofen"

# Detail léku podle SUKL kódu
biomcp czech sukl get "0001234"

# Kontrola dostupnosti léku
biomcp czech sukl availability "0001234"

# Vyhledání diagnózy podle kódu
biomcp czech mkn search --query "J06.9"

# Vyhledání diagnózy volným textem
biomcp czech mkn search --query "infarkt"

# Procházení kapitol MKN-10
biomcp czech mkn browse

# Vyhledání poskytovatele
biomcp czech nrpzs search --city "Praha" --specialty "kardiologie"

# Vyhledání zdravotního výkonu
biomcp czech szv search --query "EKG"

# Vyhledání v číselnících VZP
biomcp czech vzp search --query "antibiotika"
```

## Rychlý test - Globální BioMCP nástroje (beze změn)

```bash
# Ověření, že globální nástroje fungují
biomcp article search --gene BRAF --disease Melanoma
biomcp trial search --condition "Lung Cancer"
biomcp variant search --gene TP53
```

## Ověření pomocí MCP Inspector

```bash
npx @modelcontextprotocol/inspector uv run --with biomcp-python biomcp run
```

Otevře webové rozhraní pro prozkoumání a testování všech nástrojů.

## Docker

```bash
docker build -t czechmedmcp:latest .
docker run -p 8080:8080 czechmedmcp:latest biomcp run --mode streamable_http
```

## Očekávaný výstup

Po nastavení by `biomcp run` měl zaregistrovat všechny MVP nástroje:
- 21+ globálních BioMCP nástrojů (články, klinické studie, varianty, geny atd.)
- 14 českých MVP nástrojů (5 SUKL, 3 MKN-10, 2 NRPZS, 2 SZV, 2 VZP)
- 3 základní nástroje (think, search, fetch)

## Řešení problémů

| Problém | Řešení |
|---------|--------|
| `ModuleNotFoundError: biomcp` | Spusťte `uv pip install -e ".[dev]"` |
| `lxml` chyba instalace | Na macOS: `brew install libxml2 libxslt` |
| SUKL API neodpovídá | Zkontrolujte připojení k internetu; cachovaná data budou použita automaticky |
| MKN-10 "No data loaded" | Stáhněte ClaML XML do `data/mkn10/mkn10.xml` |
| NRPZS vrací prázdné výsledky | Zkuste jiný dotaz; API vyžaduje alespoň jeden filtr |
