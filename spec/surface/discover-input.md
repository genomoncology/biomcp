# Discover input contracts

The free-text entry point bounds input before ontology resolution. Invalid input
uses the ordinary script-safe CLI error without exposing backend details.

## Discover rejects oversized input locally

<!-- mustmatch-lint: skip -->

Discover accepts a trimmed free-text query up to 4,096 UTF-8 bytes. Longer text
fails as an invalid argument before ontology clients are constructed, without
exposing an internal backend name. The native
`discover_request_rejects_oversized_query_before_clients` test covers the ASCII
and UTF-8 boundaries; this command keeps the public error shape visible.

```bash run id=oversized-discover exit=2
bash ../fixtures/run-oversized-discover-query.sh
```

```json expect=oversized-discover contains
{
  "error": {
    "code": "invalid_argument"
  }
}
```

```text expect=oversized-discover contains
4,096
```

```text expect=oversized-discover not-contains
OLS4
ols4
```
