---
base: 7cce6ed0d3e8e732825a2494b4fa4c2ac2bdfcbb
head: 1aa174c1cfadcb24fa321d0ad483f9df627f12fe
---

# Stop flag order from silently changing a command

Article asset paging options now parse before or after the `assets` section.
Asset-request errors omit `assets` so failures cannot look like empty manifests.
