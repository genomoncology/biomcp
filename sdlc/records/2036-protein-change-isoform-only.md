# 2036 build record — protein-change requests never resolve to an isoform-only match

Landed 2026-10-10 through the landcheck-B chain onto main; the canonical-residue check runs before any non-MANE annotation is accepted, so TP53 S183Y and BRCA1 S1587F refuse naming the MANE spelling instead of returning an isoform-only answer. Reviews: the branch head review at b6ae3fe4a (read-only) and the landed-head verdict on main at 7c1faf778 by a gate-running reviewer, both accept. The 2047 lane completed its residuals (caching, refusal text, no-MANE arm).
