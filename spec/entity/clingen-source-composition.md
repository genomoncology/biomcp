# ClinGen source composition

BioMCP exposes ClinGen’s distinct upstream services as separate, opt-in workflows.
The shared organization name must not collapse their purposes: ERepo reports expert
assertions, CSpec retrieves source documents, CAR normalizes explicit HGVS input,
and LDH only augments requested article identity verification.

## Four ClinGen surfaces remain independently discoverable

Users can discover the ERepo helper beside the article workflow without making
article identity verification part of the default command. The same command tree
keeps the separate CSpec and CAR routes available for their own inputs.

```bash
biomcp variant --help | mustmatch like 'articles
erepo'
```

```bash
biomcp gene --help | mustmatch like 'cspec
ClinGen Criteria Specification Registry'
```

```bash
biomcp variant normalize --help | mustmatch like 'CAR is available as car
biomcp variant normalize car'
```

```bash
biomcp variant articles --help | mustmatch like 'Verify article identity from captured provider evidence
--verify-identity'
```

## Source results stay isolated

A requested identity-verification run must retain a healthy result from one ClinGen
source when another source reports an unavailable or empty result. The routine
fixture will report that public consequence after it composes the source-owned
fixtures; its detailed provider payloads remain owned by those source contracts.

```bash
bash ../fixtures/run-variant-article-identity-fixture.sh ../.. | mustmatch like '"clingen_source_namespaces_are_isolated": true'
```
