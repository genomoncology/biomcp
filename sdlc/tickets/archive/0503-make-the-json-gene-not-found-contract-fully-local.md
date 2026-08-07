---
flow: quickfix
priority: 5
---
# Make the JSON gene-not-found contract fully local

Routine `make test` must be deterministic and must not depend on public biomedical services. `tests/json_error_contract.rs::json_mode_gene_not_found_error_writes_json_stdout_and_exit_1` currently runs `biomcp --json get gene ZZZNOTAREALGENE` against public MyGene.info with a 10-second child-process deadline. During ticket 498 it timed out after 2,382 other tests passed, failing an unrelated health JSON change. A focused rerun later passed in 4.3 seconds, confirming a live-service timing dependency rather than a product regression.

Completed under March on 2026-07-11, as March ticket 503. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/503-make-the-json-gene-not-found-contract-fully-local
