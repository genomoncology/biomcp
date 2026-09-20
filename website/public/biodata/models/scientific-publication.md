# ScientificPublication

One shared publication value preserving source\-labeled bibliographic assertions without reconciliation\.

The stable catalog identity is `model:ScientificPublication`.

Title and abstract values preserve source-observed states without invented text. Authorship assertions stay opaque and completeness stays unknown. Named-date assertions keep their source names; equal raw text does not assert date equivalence.

## Support and exclusions

- **provisional** `support:scientific\-publication`: proof:scientific\-publication\-document\-roundtrip
- **implemented** `support:pubtator3\-pmid\-projection`: proof:pubtator3\-recorded\-pmid\-projection
- **implemented** `support:europepmc\-lite\-pmid\-projection`: proof:europepmc\-lite\-recorded\-pmid\-projection
- **unsupported** `support:europepmc\-core`: No Europe PMC CORE plan, adapter, or admitted capture exists; only the recorded LITE PMID row is supported\.

The PubTator3 PMID projection and the Europe PMC LITE PMID projection render as implemented only because the adopted catalog carries executed proof. Europe PMC CORE is unsupported.

## Fields and absence rules

| Field | Absence | Description | Constraints |
| --- | --- | --- | --- |
| `abstract\_text` | `required\_member\_non\_null\_value` | No separate catalog description\. | `\{&quot;$ref&quot;:&quot;\#/$defs/abstract&quot;\}` |
| `authorships` | `required\_member\_non\_null\_value` | No separate catalog description\. | `\{&quot;items&quot;:\{&quot;$ref&quot;:&quot;\#/$defs/authorship&quot;\},&quot;maxItems&quot;:32,&quot;type&quot;:&quot;array&quot;\}` |
| `identifiers` | `required\_member\_non\_null\_value` | No separate catalog description\. | `\{&quot;allOf&quot;:\[\{&quot;contains&quot;:\{&quot;properties&quot;:\{&quot;authority&quot;:\{&quot;const&quot;:&quot;pmid&quot;\}\},&quot;required&quot;:\[&quot;authority&quot;\],&quot;type&quot;:&quot;object&quot;\},&quot;maxContains&quot;:1,&quot;minContains&quot;:0\},\{&quot;contains&quot;:\{&quot;properties&quot;:\{&quot;authority&quot;:\{&quot;const&quot;:&quot;pmcid&quot;\}\},&quot;required&quot;:\[&quot;authority&quot;\],&quot;type&quot;:&quot;object&quot;\},&quot;maxContains&quot;:1,&quot;minContains&quot;:0\},\{&quot;contains&quot;:\{&quot;properties&quot;:\{&quot;authority&quot;:\{&quot;const&quot;:&quot;doi&quot;\}\},&quot;required&quot;:\[&quot;authority&quot;\],&quot;type&quot;:&quot;object&quot;\},&quot;maxContains&quot;:1,&quot;minContains&quot;:0\}\],&quot;items&quot;:\{&quot;$ref&quot;:&quot;\#/$defs/identifier&quot;\},&quot;maxItems&quot;:3,&quot;minItems&quot;:1,&quot;type&quot;:&quot;array&quot;\}` |
| `journals` | `required\_member\_non\_null\_value` | No separate catalog description\. | `\{&quot;items&quot;:\{&quot;$ref&quot;:&quot;\#/$defs/labeledRaw&quot;\},&quot;maxItems&quot;:32,&quot;type&quot;:&quot;array&quot;\}` |
| `named\_dates` | `required\_member\_non\_null\_value` | No separate catalog description\. | `\{&quot;items&quot;:\{&quot;$ref&quot;:&quot;\#/$defs/labeledRaw&quot;\},&quot;maxItems&quot;:32,&quot;type&quot;:&quot;array&quot;\}` |
| `title` | `required\_member\_non\_null\_value` | No separate catalog description\. | `\{&quot;$ref&quot;:&quot;\#/$defs/title&quot;\}` |

Every listed member follows its stated absence rule. Required nullable members distinguish an explicit null from a missing member. Rust validation remains authoritative.

## Component models

### PublicationIdentifier

An identifier qualified by its authority; pmid, pmcid, and doi remain distinct authorities\.

| Field | Absence | Description | Constraints |
| --- | --- | --- | --- |
| `authority` | `required\_member\_non\_null\_value` | No separate catalog description\. | `\{&quot;oneOf&quot;:\[\{&quot;const&quot;:&quot;pmid&quot;\},\{&quot;const&quot;:&quot;pmcid&quot;\},\{&quot;const&quot;:&quot;doi&quot;\}\]\}` |
| `identifier` | `required\_member\_non\_null\_value` | No separate catalog description\. | `\{&quot;oneOf&quot;:\[\{&quot;maxLength&quot;:16,&quot;pattern&quot;:&quot;^\[1\-9\]\[0\-9\]\{0,15\}\(?\!\[&\#92;&\#92;s&\#92;&\#92;S\]\)&quot;,&quot;type&quot;:&quot;string&quot;\},\{&quot;maxLength&quot;:19,&quot;pattern&quot;:&quot;^PMC\[1\-9\]\[0\-9\]\{0,15\}\(?\!\[&\#92;&\#92;s&\#92;&\#92;S\]\)&quot;,&quot;type&quot;:&quot;string&quot;\},\{&quot;maxLength&quot;:1024,&quot;pattern&quot;:&quot;^10&\#92;&\#92;\.\[0\-9\]\+\(?:&\#92;&\#92;\.\[0\-9\]\+\)\*/\[^&\#92;&\#92;u0000\-&\#92;&\#92;u0020&\#92;&\#92;u007f\-&\#92;&\#92;u009f&\#92;&\#92;u00a0&\#92;&\#92;u1680&\#92;&\#92;u2000\-&\#92;&\#92;u200a&\#92;&\#92;u2028\-&\#92;&\#92;u2029&\#92;&\#92;u202f&\#92;&\#92;u205f&\#92;&\#92;u3000A\-Z\]\+\(?\!\[&\#92;&\#92;s&\#92;&\#92;S\]\)&quot;,&quot;type&quot;:&quot;string&quot;\}\]\}` |

### PublicationTitle

A source\-labeled title with missing, blank, and usable states; a title is never null\.

| Field | Absence | Description | Constraints |
| --- | --- | --- | --- |
| `label` | `required\_member\_nullable\_value` | No separate catalog description\. | `\{&quot;oneOf&quot;:\[\{&quot;type&quot;:&quot;null&quot;\},\{&quot;$ref&quot;:&quot;\#/$defs/label&quot;\}\]\}` |
| `raw` | `required\_member\_nullable\_value` | No separate catalog description\. | `\{&quot;oneOf&quot;:\[\{&quot;type&quot;:&quot;null&quot;\},\{&quot;$ref&quot;:&quot;\#/$defs/raw&quot;\}\]\}` |
| `state` | `required\_member\_non\_null\_value` | No separate catalog description\. | `\{&quot;oneOf&quot;:\[\{&quot;const&quot;:&quot;missing&quot;\},\{&quot;const&quot;:&quot;blank&quot;\},\{&quot;const&quot;:&quot;usable&quot;\}\]\}` |

### PublicationAbstract

A source\-labeled abstract with missing, null, blank, and usable states; missing never confirms unavailability\.

| Field | Absence | Description | Constraints |
| --- | --- | --- | --- |
| `label` | `required\_member\_nullable\_value` | No separate catalog description\. | `\{&quot;oneOf&quot;:\[\{&quot;type&quot;:&quot;null&quot;\},\{&quot;$ref&quot;:&quot;\#/$defs/label&quot;\}\]\}` |
| `raw` | `required\_member\_nullable\_value` | No separate catalog description\. | `\{&quot;oneOf&quot;:\[\{&quot;type&quot;:&quot;null&quot;\},\{&quot;$ref&quot;:&quot;\#/$defs/raw&quot;\}\]\}` |
| `state` | `required\_member\_non\_null\_value` | No separate catalog description\. | `\{&quot;oneOf&quot;:\[\{&quot;const&quot;:&quot;missing&quot;\},\{&quot;const&quot;:&quot;null&quot;\},\{&quot;const&quot;:&quot;blank&quot;\},\{&quot;const&quot;:&quot;usable&quot;\}\]\}` |

### JournalAssertion

An opaque source\-labeled journal assertion; no journal identity or equivalence is inferred\.

| Field | Absence | Description | Constraints |
| --- | --- | --- | --- |
| `label` | `required\_member\_non\_null\_value` | No separate catalog description\. | `\{&quot;$ref&quot;:&quot;\#/$defs/label&quot;\}` |
| `raw` | `required\_member\_non\_null\_value` | No separate catalog description\. | `\{&quot;$ref&quot;:&quot;\#/$defs/raw&quot;\}` |

### AuthorshipAssertion

An opaque source\-labeled authorship assertion whose completeness stays unknown\.

| Field | Absence | Description | Constraints |
| --- | --- | --- | --- |
| `completeness` | `required\_member\_non\_null\_value` | No separate catalog description\. | `\{&quot;const&quot;:&quot;unknown&quot;\}` |
| `label` | `required\_member\_non\_null\_value` | No separate catalog description\. | `\{&quot;$ref&quot;:&quot;\#/$defs/label&quot;\}` |
| `ordered\_display\_names` | `required\_member\_nullable\_value` | No separate catalog description\. | `\{&quot;oneOf&quot;:\[\{&quot;items&quot;:\{&quot;$ref&quot;:&quot;\#/$defs/raw&quot;\},&quot;maxItems&quot;:1024,&quot;type&quot;:&quot;array&quot;\},\{&quot;type&quot;:&quot;null&quot;\}\]\}` |
| `raw` | `required\_member\_nullable\_value` | No separate catalog description\. | `\{&quot;oneOf&quot;:\[\{&quot;$ref&quot;:&quot;\#/$defs/raw&quot;\},\{&quot;type&quot;:&quot;null&quot;\}\]\}` |

### NamedDateAssertion

An opaque source\-labeled named\-date assertion; equal raw text asserts no date meaning or equivalence\.

| Field | Absence | Description | Constraints |
| --- | --- | --- | --- |
| `label` | `required\_member\_non\_null\_value` | No separate catalog description\. | `\{&quot;$ref&quot;:&quot;\#/$defs/label&quot;\}` |
| `raw` | `required\_member\_non\_null\_value` | No separate catalog description\. | `\{&quot;$ref&quot;:&quot;\#/$defs/raw&quot;\}` |

## Direct relationship diagram

![ScientificPublication fields connect to six component models in adopted catalog order.](/downloads/biodata/scientific-publication-relationships.svg)

[Download the accessible relationship diagram](/downloads/biodata/scientific-publication-relationships.svg).

## Direct relationship text

This text is equivalent to the diagram and remains available without images.

- `publication\-identifiers`: `model:ScientificPublication/field:identifiers` contains `model:PublicationIdentifier`. The publication retains one to three authority\-qualified identifiers in canonical order\.
- `publication\-title`: `model:ScientificPublication/field:title` contains `model:PublicationTitle`. The publication preserves the source\-observed title state without inventing text\.
- `publication\-abstract`: `model:ScientificPublication/field:abstract\_text` contains `model:PublicationAbstract`. The publication preserves the source\-observed abstract state without inventing text\.
- `publication\-journals`: `model:ScientificPublication/field:journals` contains `model:JournalAssertion`. Opaque journal assertions retain source order\.
- `publication\-authorships`: `model:ScientificPublication/field:authorships` contains `model:AuthorshipAssertion`. Opaque authorship assertions retain source order and unknown completeness\.
- `publication\-named\-dates`: `model:ScientificPublication/field:named\_dates` contains `model:NamedDateAssertion`. Opaque named\-date assertions retain source order\.

## Component relationships

- `publication\-identifiers`: `model:ScientificPublication/field:identifiers` contains `model:PublicationIdentifier`. The publication retains one to three authority\-qualified identifiers in canonical order\.
- `publication\-title`: `model:ScientificPublication/field:title` contains `model:PublicationTitle`. The publication preserves the source\-observed title state without inventing text\.
- `publication\-abstract`: `model:ScientificPublication/field:abstract\_text` contains `model:PublicationAbstract`. The publication preserves the source\-observed abstract state without inventing text\.
- `publication\-journals`: `model:ScientificPublication/field:journals` contains `model:JournalAssertion`. Opaque journal assertions retain source order\.
- `publication\-authorships`: `model:ScientificPublication/field:authorships` contains `model:AuthorshipAssertion`. Opaque authorship assertions retain source order and unknown completeness\.
- `publication\-named\-dates`: `model:ScientificPublication/field:named\_dates` contains `model:NamedDateAssertion`. Opaque named\-date assertions retain source order\.

## BioMCP product behavior

BioMCP resolves article detail through its existing tested command surfaces. A PMID normally uses the PubTator detail path. DOI and PMCID inputs use Europe PMC resolution and may reuse a resolved PMID. Europe PMC also supplies enrichment and HTTP fallback. BioMCP owns transport, fallback, reconciliation, and display.

Identifier authorities remain distinct: a `pmid`, `pmcid`, or `doi` value does not prove cross-provider equivalence.

Retained variant, graph, search, annotation, and compatibility paths are outside the shared model.

For example, `biomcp get article 39325770 --json` invokes the tested generic PMID detail surface. That invocation is illustrative, not a recorded offline case, and this reference neither freezes nor claims its live output.

## Provenance

This reference comes from catalog format 1 and the recorded example `export\_39325770\.json`. Levchenko M, Parkin M, McEntyre J, Harrison M \(2024\), Enabling preprint discovery, evaluation, and analysis with Europe PMC, PLOS ONE 19\(9\): e0303005, https://doi\.org/10\.1371/journal\.pone\.0303005; PubTator3 and the U\.S\. National Library of Medicine.

The receipt records the request `https://www\.ncbi\.nlm\.nih\.gov/research/pubtator3\-api/publications/export/biocjson?pmids=39325770` under `The article title and abstract are from © 2024 Levchenko et al\., licensed CC BY 4\.0 at https://creativecommons\.org/licenses/by/4\.0/\. The three captured annotation text and infons\.name values are each COVID\-19, which is also present in that licensed abstract; this scoped text determination does not assert provider derivation or general annotation\-name licensing\. Article CC BY does not blanket\-license provider additions; source\-generated annotations, identifiers, and technical metadata use the separately recorded factual and provider basis\.` with attribution to Levchenko M, Parkin M, McEntyre J, Harrison M \(2024\), Enabling preprint discovery, evaluation, and analysis with Europe PMC, PLOS ONE 19\(9\): e0303005, https://doi\.org/10\.1371/journal\.pone\.0303005; PubTator3 and the U\.S\. National Library of Medicine. Origin: Exact response body captured directly from the public PubTator3 API under Ian Maurer&\#x27;s authorization on 2026\-09\-15. Transformation: None; the committed response body bytes are untouched.

The recorded input SHA\-256 is `4ab5cb9a467342d0e7c902a5f6328652c36f67d255c5d3c147b04cfebf0e2f40`. The adapter and catalog are pinned to BioData revision `c9938b99bd091ab4bf6da8b909ed239826e5ab6d`.

The raw recorded PubTator3 response remains provider evidence retained by BioData and is not republished here. The only publication example download is the strict adapter-generated `biodata/scientific-publication` document embedded in the bundle.

## Reviewed crosswalks

No reviewed crosswalks are recorded for ScientificPublication. The empty crosswalk list is an explicit absence, not an invented mapping.

## Downloads

- [ScientificPublication schema](/downloads/biodata/scientific-publication.schema.json)
- [PubTator3 ScientificPublication example](/downloads/biodata/pubtator3-scientific-publication.json)
- [ScientificPublication relationship diagram](/downloads/biodata/scientific-publication-relationships.svg)
- [Adopted catalog bundle](/downloads/biodata/scientific-publication-v1.bundle.json)
- [ScientificPublication discovery contract](/biodata/discovery/scientific-publication.json)

Artifact digests from the adopted catalog:

- `schema:scientific\-publication`: `sha256:3cd4928fda50e080ab7e05fff66eedc92dd8240fd86bdd12d96240b73823055b`
- `example:pubtator3\-scientific\-publication`: `sha256:33df8e820b97eb7e7e2d708182ccffc0ed77bc5945f8cef40b22c04549a95639`

## Scope

This page renders only the claims and exclusions in the validated adopted catalog.
