# ClinGen CSpec commands

ClinGen's Criteria Specification Registry publishes versioned VCEP source
specifications. BioMCP exposes the CSpec command family locally; retrieval itself
uses the real provider only in the explicit live verification lane.

## Inspect CSpec retrieval options

<!-- mustmatch-lint: skip -->

Use the CSpec command's help before choosing a full resource IRI. The command
accepts a version selector and bounded page controls, and its raw capture reader is
a distinct `document` subcommand.

```bash run id=clingen-cspec-help exit=0
biomcp gene cspec --help
```

```text expect=clingen-cspec-help contains
Usage: biomcp gene cspec
document
--version
--offset
--limit
```
