---
base: 8bfa4537c21a5569be55572f3ef2f15aaafdd87d
head: 4f87115801fd7ffb58eeba8adc91e52d2c8c135a
---

# Isolate CTGov spec request logs

Each CTGov Markdown consumer now receives a private request log and namespaced
fixture URL, while the fixture records canonical provider request paths. This
keeps parallel pages from truncating or observing another page's requests.

The isolation contract discovers mutable-log consumers mechanically and keeps
the zero-distance and provider-request-shape proofs intact. Three consecutive
`make spec` runs passed with parallel workers.
