---
base: 819bfbd01506ab4ba974b21aa06283b44995ce22
head: b7270cd141dca33389da59d9d66e697c8ce0289a
---
Routine `make test` must be deterministic and must not depend on public biomedical services. `tests/json_error_contract.rs::json_mode_gene_not_found_error_writes_json_stdout_and_exit_1` currently runs `biomcp --json get gene ZZZNOTAREALGENE` against public MyGene.info with a 10-second child-process deadline. During ticket 498 it timed out after 2,382 other tests passed, failing an unrelated health JSON change. A focused rerun later passed in 4.3 seconds, confirming a live-service timing dependency rather than a product regression.

Imported from March ticket 503. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/503-make-the-json-gene-not-found-contract-fully-local
