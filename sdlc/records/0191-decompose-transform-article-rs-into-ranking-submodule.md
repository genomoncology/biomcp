---
base: e6587e6a48a052a5f55d16a8a4bf4f8c037a1177
head: 861e34b3754c8620778d5cc8dd55bb8597cb62c4
---
`src/transform/article.rs` is 2,210 lines. It houses the hybrid ranking logic `Ws×semantic + Wl×lexical + Wc×citations + Wp×position`, federation merge logic, source position tracking, keyword anchor tokenization, and calibration fixtures. Article ranking is the current brief focus (BioASQ retrieval quality) and will keep getting touched. Shrink it into a `src/transform/article/` submodule so future ranking tickets hit a smaller file.

Imported from March ticket 191. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/191-decompose-transform-article-rs-into-ranking-submodule
