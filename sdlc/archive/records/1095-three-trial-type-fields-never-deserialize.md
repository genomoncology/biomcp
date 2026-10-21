---
base: f8a6d8caa6956376ffc7f1553895b8e49d3d3dbb
head: e82bd986690b0b43783a942b83e5bf14da037647
---

# Preserve ClinicalTrials.gov type fields

BioMCP now deserializes provider type keys for interventions, arms, and
references. Default trial details request intervention types, and Markdown and
JSON expose supplied values so trial cards no longer silently lose them.
