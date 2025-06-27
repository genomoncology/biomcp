# Articles CLI Documentation

The Articles CLI module provides commands for searching and retrieving biomedical research articles using the PubTator3 API, with automatic integration of cBioPortal cancer genomics data when searching for genes.

> **API Documentation**:
>
> - For PubTator3 API details, see the [PubTator3 API Documentation](../apis/pubtator3_api.md)
> - For cBioPortal integration, see the [cBioPortal Integration Documentation](../apis/cbioportal.md)
>
> **Tip**: Use the `--help` flag with any command (e.g., `biomcp article search --help`) to see the most up-to-date options directly from the tool.

## Search Command (`search`)

Search for biomedical research articles based on various filters.

### Usage

```bash
biomcp article search [OPTIONS]
```

#### Options

- `-g, --gene TEXT`: Gene name to search for (e.g., BRAF). Automatically includes cBioPortal data. Can be specified multiple times.
- `-v, --variant TEXT`: Genetic variant to search for (e.g., "BRAF V600E"). Can be specified multiple times.
- `-d, --disease TEXT`: Disease name to search for (e.g., Melanoma). Can be specified multiple times.
- `-c, --chemical TEXT`: Chemical or drug name to search for (e.g., Vemurafenib). Can be specified multiple times.
- `-k, --keyword TEXT`: Additional keyword to search for. Use mutation notation (e.g., "V600E", "F57\*") for mutation-specific cBioPortal data. Can be specified multiple times.
- `-j, --json`: Render output in JSON format instead of Markdown.
- `--help`: Show help message and exit.

**Note**: When using the `--gene` option, cBioPortal cancer genomics data is automatically included in the results.

#### Examples

Search for articles about the BRAF gene (includes cBioPortal summary):

```bash
biomcp article search --gene BRAF
```

Search for articles about the BRAF V600E mutation in melanoma:

```bash
biomcp article search --gene BRAF --variant "BRAF V600E" --disease Melanoma
```

Search for specific mutations with cBioPortal data:

```bash
# BRAF V600E mutation
biomcp article search --gene BRAF --keyword V600E

# SRSF2 F57Y mutation
biomcp article search --gene SRSF2 --keyword F57Y

# Any mutation at SRSF2 position 57 (F57*)
biomcp article search --gene SRSF2 --keyword "F57*"
```

Search with multiple gene filters:

```bash
biomcp article search --gene BRAF --gene KRAS --disease Melanoma
```

Get results as JSON:

```bash
biomcp article search --gene BRAF --json
```

## Get Command (`get`)

Retrieve detailed information (abstract, metadata) for specific articles using their PubMed IDs (PMIDs).

### Usage

```bash
biomcp article get [OPTIONS] PMIDS...
```

#### Arguments

- `PMIDS`: One or more PubMed IDs (integers) of the articles to retrieve. [required]

#### Options

- `-f, --full`: Attempt to retrieve full text if available via the API (Abstract is always retrieved). [default: False]
- `-j, --json`: Render output in JSON format instead of Markdown.
- `--help`: Show help message and exit.

#### Examples

Get article abstract by PMID:

```bash
biomcp article get 21717063
```

Get multiple articles:

```bash
biomcp article get 21717063 22301848
```

Get full text (if available):

```bash
biomcp article get 21717063 --full
```

Get results as JSON:

```bash
biomcp article get 21717063 --json
```
