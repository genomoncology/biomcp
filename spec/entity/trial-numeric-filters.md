# Trial numeric filter contracts

Patient age and geographic coordinates are validated before trial source work.
Invalid values remain client errors rather than being attributed to a registry.

## Trial ages stay within a human range

<!-- mustmatch-lint: skip -->

Trial eligibility accepts fractional patient ages from zero through 150 years.
Non-finite or out-of-range ages fail before count or search work, so they cannot
silently change a result page.

The native table-driven test
`trial_numeric_filters_are_validated_before_request_construction` covers the
full non-finite and out-of-range age matrix. This command keeps the CLI's
script-safe error shape visible to users.

| str:value | str:label |
|---|---|
| -1 | below zero |

```bash run id=invalid-trial-age exit=2 each_row="Trial ages stay within a human range"
biomcp --json search trial --age={{value}} --count-only
```

```json expect=invalid-trial-age contains each_row="Trial ages stay within a human range"
{
  "error": {
    "code": "invalid_argument"
  }
}
```

```text expect=invalid-trial-age contains each_row="Trial ages stay within a human range"
--age
```

## Trial coordinates stay on Earth

<!-- mustmatch-lint: skip -->

Geographic trial search uses finite latitude from -90 through 90 and longitude
from -180 through 180. Values outside those bounds are client errors, not
ClinicalTrials.gov or NCI outages.

The native table-driven test
`trial_numeric_filters_are_validated_before_request_construction` covers the
full non-finite and out-of-range coordinate matrix. This command keeps the
CLI's script-safe error shape visible to users.

| str:coordinates | str:flag | str:label |
|---|---|---|
| --lat=-91 --lon=0 | --lat | latitude below -90 |

```bash run id=invalid-trial-coordinate exit=2 each_row="Trial coordinates stay on Earth"
biomcp --json search trial -c melanoma {{coordinates}} --distance=1 --limit=1
```

```json expect=invalid-trial-coordinate contains each_row="Trial coordinates stay on Earth"
{
  "error": {
    "code": "invalid_argument"
  }
}
```

```text expect=invalid-trial-coordinate contains each_row="Trial coordinates stay on Earth"
{{flag}}
```
