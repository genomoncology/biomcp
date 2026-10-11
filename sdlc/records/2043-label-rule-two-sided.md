# 2043 build record — drug labels match the brand or the plain ingredient

Landed 2026-10-10: a brand query takes its own brand's label and an ingredient query takes the plain single-ingredient product's, under Ian's clean-release ruling that superseded the earlier deferral. Twelve biting tests fail on main's behavior; the record dates were corrected before landing. Reviews: branch head bf8861a2a (with the content head 41acc461f reviewed and its P1 dates fixed), the landed head on main at 7c1faf778 accepted by a gate-running reviewer who ran the label-choice suite.
