---
flow: build
priority: 5
---
# Decompose transform article.rs into ranking submodule

`src/transform/article.rs` is 2,210 lines. It houses the hybrid ranking logic `Ws×semantic + Wl×lexical + Wc×citations + Wp×position`, federation merge logic, source position tracking, keyword anchor tokenization, and calibration fixtures. Article ranking is the current brief focus (BioASQ retrieval quality) and will keep getting touched. Shrink it into a `src/transform/article/` submodule so future ranking tickets hit a smaller file.

Completed under March on 2026-04-14, as March ticket 191. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/191-decompose-transform-article-rs-into-ranking-submodule
