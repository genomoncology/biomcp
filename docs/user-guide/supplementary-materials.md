# Supplementary Materials

Retrieve the figures, tables, spreadsheets, and documents that a journal
publishes alongside an article.

Journal full text describes supplementary materials but does not contain
them. In JATS XML a `<supplementary-material>` element carries a label, a
caption, a filename, and a media type — the file itself is a separate
binary stored beside the XML. BioMCP locates those binaries, merges the
copies published by different providers into one list, and returns the
bytes unchanged. It does not convert or parse them.

## Quick start

List what an article has, then download one file:

```bash
biomcp --json get article PMC7857465 assets
biomcp get article PMC7857465 asset TBBE_A_1426496_SM7925.docx > supplement.docx
```

The first command returns a manifest. Each entry in it carries a `handle`,
which is the exact command that retrieves that file.

## Listing assets

```bash
biomcp --json get article <id> assets
```

`<id>` accepts a PMID, a PMCID, or a DOI. The manifest is JSON only;
running it without `--json` returns an error rather than a Markdown card.

The `assets` section must be the only section in the command. Combining it
with `fulltext`, `annotations`, or any other section is rejected.

### Views

The manifest has two lists: `assets`, the files that can be retrieved, and
`coverage`, the files that were named but could not be. Select which to
return with `--asset-view`.

| View | Returns |
| --- | --- |
| `compact` | A short page of both lists. The default. |
| `retrievable` | The `assets` list only, paged. |
| `coverage` | The `coverage` list only, paged. |

Paging applies to whichever list the view selects.

| Option | Meaning |
| --- | --- |
| `--asset-limit <N>` | Page size, 1 to 100. |
| `--asset-offset <N>` | Zero-based offset into the list. |

Place these options before the article identifier:

```bash
biomcp --json get article --asset-view retrievable --asset-limit 100 PMC7857465 assets
```

### Manifest entries

Each entry in `assets` describes one file:

| Field | Meaning |
| --- | --- |
| `filename` | The publisher's filename. |
| `asset_key` | The identifier used to retrieve the file. |
| `kind` | `figure-image`, `supplementary-file`, or `other`. |
| `media_type` | The declared MIME type, when the provider supplies one. |
| `size_bytes` | Size of the retrieved bytes. |
| `sha256` | Digest of the retrieved bytes. |
| `provider` | The source that supplied the bytes. |
| `reuse` | License state for this file. See Licensing below. |
| `jats` | Label, caption, and source id from the article XML, when present. |
| `discovery_routes` | Every route that named this file. |
| `handle` | The command that retrieves this file. |

A file published through several providers appears once. Its
`discovery_routes` lists each route that found it, and `provider` names the
one that supplied the bytes.

## Downloading an asset

```bash
biomcp get article <id> asset <asset-key>
```

The command writes the file to standard output with no conversion. Redirect
it to a file or pipe it to another process:

```bash
biomcp get article PMC7857465 asset TBBE_A_1426496_SM7925.docx > supplement.docx
```

Verify the download against the manifest's `sha256` if integrity matters to
your workflow.

Asset keys come from the manifest. An unknown key returns `not_found` when
the manifest resolved successfully, and `source_unavailable` when no
provider could be reached.

!!! warning "Redirect binary downloads"

    Standard output receives the raw bytes. Running the command in a
    terminal without redirecting it prints binary content directly to the
    terminal, which can leave the terminal in an unusable state.

## Reading coverage

`coverage` names files BioMCP found references to but did not return. It is
how you tell an article with no supplementary materials from an article
whose supplementary materials could not be reached.

```bash
biomcp --json get article --asset-view coverage <id> assets
```

| Outcome | Meaning |
| --- | --- |
| `retrievable` | The file is available; it also appears in `assets`. |
| `healthy_absent` | The provider confirmed the file does not exist. |
| `access_or_licence_denied` | The provider refused access. |
| `pmc_proof_of_work` | PMC answered with a challenge page instead of the file. |
| `unsupported_origin` | The link pointed somewhere BioMCP will not fetch from. |
| `source_unavailable` | The provider could not be reached, or returned unusable content. |

Entries with a `pmc_proof_of_work` outcome carry no `asset_key` and no
`handle`. The file is named because the article references it, and BioMCP
will not present a challenge page as the file's contents.

An article whose supplementary materials are deposited only as an author
manuscript is the common case for this outcome. When Europe PMC holds its
own copy the file is returned normally; when it does not, the file is named
and not delivered.

## Where assets come from

BioMCP consults these routes for a validated PMCID and merges the results:

| Route | Contents |
| --- | --- |
| PMC OA Archive | Media objects declared by the article's S3 package metadata. |
| Europe PMC | The article's supplementary files ZIP, validated in memory. |
| JATS XML | Supplement links declared in the article XML. |
| PMC HTML | Supplement links found on the article page. |
| Figshare | Records discovered through Semantic Scholar for supported article URLs. |

The `source_attempts` array reports what each route returned, independently
of whether any file was found:

| Outcome | Meaning |
| --- | --- |
| `data` | The route returned usable content. |
| `degraded` | The route returned content with a recoverable problem. |
| `healthy_absent` | The route confirmed it holds nothing for this article. |
| `source_unavailable` | The route failed. |
| `timed_out` | The route exceeded its budget. |

Optional routes run under a shorter budget than the primary route, so a slow
provider cannot withhold files another provider has already returned.

Provider URLs stay internal. Handles are BioMCP commands, never download
links.

## Licensing

Each asset carries its own `reuse` block. A file's license is not implied by
the article's license, and is frequently unknown:

- `license_present` — whether BioMCP holds a license fact for this file.
- `license` — the license, when known.
- `license_source` — which provider supplied the license fact.
- `reuse_warning` — present when no license fact is available.

BioMCP does not enforce reuse terms. Review the returned license context and
the provider's terms before redistributing a downloaded file. The durable
inventory of provider terms is in
[Source Licensing](../reference/source-licensing.md).

## Limits

Retrieval is bounded. Content beyond these limits is reported as
unavailable rather than truncated:

- PMC OA objects: 8 MiB each, 256 objects, 64 MiB total.
- Europe PMC ZIP: 64 MiB compressed, 8 MiB per member, 64 MiB expanded,
  256 members. Members are validated and never extracted to disk.
- Linked supplements: 256 candidates, 64 MiB total.

A resolved manifest is reused for five minutes, including paging
continuations. Pass `--no-cache` to bypass that reuse.

## MCP clients

The `assets` manifest is available over MCP. Retrieving asset bytes is not —
`asset <asset-key>` returns an error directing the caller to the CLI. Binary
payloads are outside what the tool interface returns.

## Troubleshooting

**`Article asset manifests are JSON-only; rerun with --json`**

Add `--json`. The manifest has no Markdown rendering.

**`assets is a standalone JSON-only article section; do not combine it with
other sections`**

Either another section was requested in the same command, or an option was
placed after the identifier and parsed as a second section. Put
`--asset-view`, `--asset-limit`, and `--asset-offset` before the
identifier.

**The manifest is empty and coverage lists unfamiliar names**

Those entries are references BioMCP found and would not fetch. Check the
`outcome` on each; `unsupported_origin` means the link pointed outside the
article's own provider.

**Binary content printed to the terminal**

`asset <asset-key>` writes to standard output. Redirect it.

**A file is named in coverage but has no handle**

The outcome explains why. `pmc_proof_of_work`, `access_or_licence_denied`,
and `source_unavailable` all name a file that exists without being able to
deliver it.

## See also

- [Article](article.md) — search, retrieval, and full text.
- [PubMed](../sources/pubmed.md) — the literature source.
- [Data Sources](../reference/data-sources.md) — resolution order and bounds.
- [Source Licensing](../reference/source-licensing.md) — provider terms.
