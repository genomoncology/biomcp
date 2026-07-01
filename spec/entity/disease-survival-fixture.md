# Disease Survival Fixture

Disease survival cards combine disease resolution with SEER Explorer survival
summaries. This routine spec keeps that workflow deterministic by using local
MyDisease and SEER fixtures while still driving the shipped CLI.

## Disease Survival Commands Exit After Rendering With SEER Fixture
<!-- mustmatch-lint: skip -->

Both disease-get forms should render the SEER survival card and then exit. The
fixture grounds chronic myeloid leukemia to `MONDO:0011996` and serves the CML
SEER payload locally, so the routine gate proves bounded CLI behavior without
public upstream availability.

```bash run id=disease-survival-name exit=0 timeout=25
timeout 20s ../../tools/biomcp-ci get disease --name "chronic myeloid leukemia" survival
```

```text expect=disease-survival-name contains
## Survival (SEER Explorer)
Source: Chronic Myeloid Leukemia (CML)
| Sex | Latest observed year | 5-year relative survival | 95% CI | Cases | Latest modeled |
Both Sexes
```

```bash run id=disease-survival-positional exit=0 timeout=25
timeout 20s ../../tools/biomcp-ci get disease "chronic myeloid leukemia" survival
```

```text expect=disease-survival-positional contains
## Survival (SEER Explorer)
Source: Chronic Myeloid Leukemia (CML)
| Sex | Latest observed year | 5-year relative survival | 95% CI | Cases | Latest modeled |
Both Sexes
```
