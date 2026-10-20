# 2308 — Security lock updates

Update only the vulnerable locked versions identified by the PM: certifi, urllib3, mkdocs-material and xxhash-rust. Keep dependency and manifest changes to the smallest supported set and preserve application behavior.

Before code, the developer read GitHub alerts 80, 87, 88, 89, 91 and 93. Their first patched versions are certifi 2026.10.19, urllib3 2.8.0, mkdocs-material 9.7.7 and xxhash-rust 0.8.16. Planned changes are uv.lock, Cargo.lock and the changelog. The focused job compares the current lock pins with retained authoritative alert JSON before generating the narrow updates. Initial M5 job 2308/1792530249031-74931 stopped because its Python lacked tomllib; this is a setup failure, not a reproduced advisory failure or a passing proof.

Raw alert inputs, commands, before and after locks and job results belong under M5 experiment 2102-biomcp-2258-final-live-replay/security-pins/2308. Focused red and green results, routine qualification, fresh review and automatic post-landing alert closure remain required. No GitHub alert is closed manually.
