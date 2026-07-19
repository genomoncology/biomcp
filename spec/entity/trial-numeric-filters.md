# Trial numeric filter contracts

Patient age and geographic coordinates are validated before trial source work.
Invalid values remain client errors rather than being attributed to a registry.

## Trial ages stay within a human range

<!-- mustmatch-lint: skip -->

Trial eligibility accepts fractional patient ages from zero through 150 years.
Non-finite or out-of-range ages fail before count or search work, so they cannot
silently change a result page.

| str:value | str:label |
|---|---|
| NaN | not a number |
| +inf | positive infinity |
| -inf | negative infinity |
| 1e309 | overflow |
| -1 | below zero |
| -1e-50 | tiny negative |
| 151 | above 150 |
| 150.000001 | just above 150 |

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

| str:coordinates | str:flag | str:label |
|---|---|---|
| --lat=NaN --lon=0 | --lat | latitude not a number |
| --lat=+inf --lon=0 | --lat | latitude infinity |
| --lat=-91 --lon=0 | --lat | latitude below -90 |
| --lat=91 --lon=0 | --lat | latitude above 90 |
| --lat=0 --lon=NaN | --lon | longitude not a number |
| --lat=0 --lon=+inf | --lon | longitude infinity |
| --lat=0 --lon=-181 | --lon | longitude below -180 |
| --lat=0 --lon=181 | --lon | longitude above 180 |

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
