# How to: find trials

This guide shows common trial-searching patterns.

## Baseline search

```bash
biomcp search trial -c melanoma --status recruiting --limit 10
```

Add intervention:

```bash
biomcp search trial -c melanoma -i pembrolizumab --status recruiting --limit 10
```

## Biomarker-aware filters (best effort)

```bash
biomcp search trial -c melanoma --mutation "BRAF V600E" --limit 10
biomcp search trial -c melanoma --biomarker BRAF --limit 10
```

## Geographic search

```bash
biomcp search trial -c melanoma --lat 42.36 --lon -71.06 --distance 50 --limit 10
```

## Switch data sources

ClinicalTrials.gov (default):

```bash
biomcp search trial -c melanoma --source ctgov --limit 10
```

NCI CTS:

```bash
export NCI_API_KEY="..."
biomcp search trial -c melanoma --source nci --limit 10
```
