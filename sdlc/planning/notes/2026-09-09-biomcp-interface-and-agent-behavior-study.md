---
date: 2026-09-09
---

# BioMCP interface and agent-behaviour study, 2026-09-01

Consolidated 2026-09-09 from nine working documents that were dissolved out of the notes vault. The raw material is archived and never deleted: the Markdown at `~/workspace/archive/notes-2026-09-09/final-snapshot/biomcp/paper-research-2026-09-01/`, and the command captures, ground-truth files, harness scripts and run JSON at `~/workspace/archive/notes-2026-09-09/non-markdown/biomcp/paper-research-2026-09-01/`.

Ian's ruling, 2026-09-09: any BioMCP paper would have to rerun its measurements anyway, so this is reference rather than live work. Every number below is dated 2026-09-01 and pinned to BioMCP 0.8.25 at git `e127992e`. Trial counts move daily and the interface has changed since.

## What was run

Two studies, one day apart in method.

The first ran roughly 60 shell commands against `biomcp 0.8.25` and live public APIs on 2026-09-01. It measured the tool surface. It did not measure an agent.

The second served the same binary over stdio MCP — four tools — and drove it with Claude Code (`claude-sonnet-5`, harness 2.1.252) across 31 tasks, one `claude -p` process per task, no shared context, shell and network tools disallowed, `--max-turns 40`. 179 tool calls, about 27 minutes wall clock. Ground truth for every task was established from the CLI before any agent ran, and the pass/fail rule was written before the run. One rater, no blinding, one run per task.

## Findings that held up

**The published token-reduction claim is wrong and the replacement is better.** The blog's "36 tools, 16,600 tokens → 800 tokens" does not describe the shipped tool. The measured MCP surface was 22,039 bytes, 5,922 `cl100k_base` tokens across 4 tools — a 65% reduction, not 95%. The trajectory is the real finding: 16,600 tokens at Python v0.7.3 (36 tools), 1,628 at an unreleased snapshot (7 tools), 3,974 at a dev build, 5,599-5,922 at published 0.8.25. A 3.6x regression, because the escape-hatch tool's description is generated at build time from a Markdown reference that changed 24 times and grew from 14,097 to 17,332 bytes during the 0.8.25 window. **Interface compression is a maintenance property, not an architectural one.** The repo has a CI bound on it and the published release was never under that bound.

**Progressive disclosure is intact and understated.** Summary against `all`: gene 485 vs 4,574 tokens (9.4x), variant 386 vs 1,473 (3.8x), disease 651 vs 3,052 (4.7x), trial 422 vs 9,870 (**23.4x**). Nobody has published the trial ratio. Both arms are the shipped product, so ablating it needs no new code.

**Suggestions split into a helpful half and a harmful half.** Static affordance disclosure helps: the `Filters:` line printed on every trial search took a pediatric ALL query from 1-of-5 relevant to 5-of-5 once `--age 10 -s RECRUITING` was added. Dynamic next-hop suggestion hurt in three recorded cases — the medulloblastoma card suggested a synonym returning 110 trials and 3 recruiting against 345 and 36 for the user's own word; a Li-Fraumeni phenotype search suggested opening a wrong top hit; `discover "H3K27M"` suggested a command routing to a histone mark misclassified as a drug. Telling an agent what is available and telling it where to go next are different mechanisms with opposite signs.

**A version string is not an identifier.** `0.8.25` names at least three materially different binaries: the tag declares 3 MCP tools, the installed binary 4, unreleased main 7. `Cargo.toml` held `0.8.25` for five weeks. `manifest.json` advertises 7 tools under a version no published build shipped. Any published measurement must be pinned to a git SHA.

**The flat-catalog control is a real shipped artifact.** The 36-tool deletion (commit `09b789d2`, 2026-02-23) was a Python-to-Rust rewrite, not a refactor. The 36 tools are exactly enumerable at `v0.7.3` — 33 registrations plus `search`, `fetch`, `think` — and `biomcp-python==0.7.3` is still on PyPI, unyanked, with a fully pinned lockfile carrying no VCS dependencies. `tools/list` is served before any network call, so upstream drift does not touch a context-cost comparison.

## What the agent runs changed

**87.1% task success, 27 of 31.** Four hard failures: one intent decomposition, two evidence interpretation, one synthesis. Four failures cannot support a rate; these are instances with mechanisms.

**29.1% of tool calls errored — 52 calls across 21 of 31 runs — and almost none on biology.** 19 invented a section name that does not exist. 15 wrote Lucene syntax into a plain-text field. Those two buckets are 65% of failed calls and both have cheap fixes: enumerate the valid sections in the tool schema, and either accept the Lucene forms or reject them with a worked example. **A grammar that resembles a query language and is not one imposes a measurable per-call tax on every agent that meets it.**

One of those grammar errors produced the study's only wrong clinical number. In D02 the agent inverted verb and noun (`trial search` instead of `search trial`), retried the same inversion, abandoned the trial route, fell back to a disease card, and reported **1 recruiting retinoblastoma trial against a truth of 22**.

**The silent zero costs effort, not accuracy — at this model class.** Agents met a `Found: 0` or `Resolution: Unresolved` response 31 times across 18 of 31 runs. **Zero produced a confidently wrong final answer.** Every alias trap was recovered under the partner symbol, and one run went further and warned that records returned for `ALK F1174L` are annotated `p.F797L`. The cost, on eight trap tasks against six clean controls: 3.1x tool calls (5.62 vs 1.83), 1.9x turns, 2.1x context tokens (125,239 vs 58,341), 1.6x dollars ($0.0713 vs $0.0450). Matched pairs of the identical question give roughly 2x on calls and cost. The defect is a tax on capable consumers and a hazard for literal ones. This result is model-class dependent and no weaker model was tested.

**The sharpest finding is that a capable agent silently repairs a broken record.** `get pgx TPMT recommendations` returns 30 rows collapsing onto 6 `(drug, phenotype)` keys, 5 of which carry four mutually contradictory recommendations, all classified `Strong`; a row labelled Normal Metabolizer carries the Poor Metabolizer advice. CPIC's thiopurine guideline is keyed on the pair (TPMT phenotype, NUDT15 phenotype); the record carries one phenotype string and no field for the partner gene. Given those rows, C02 picked one, gave the clinically correct answer, and gave the reader no signal that the source disagrees with itself. D05 did the same at greater scale and produced a clean phenotype-by-dose briefing from a self-contradictory table with no caveat. C07 read azathioprine rows and asserted the same recommendations are "shown for" mercaptopurine and thioguanine; they are not.

**Grounding relocates confident error, and the relocation is worse than "faithful transmission."** The agent does not transmit the defect faithfully. It repairs the defect, and the repair erases the evidence that anything was wrong. Outcome-only scoring marks that a success. Nobody downstream will ever report the bug. That is an argument for trajectory scoring rather than answer scoring.

**Robustness under perturbation needs no fault injection.** Four upstream faults happened unprompted in one session: PharmGKB connect error, DisGeNET 403, PMC OA 404, and DDInter unavailable. The last is the sharpest: `get drug mercaptopurine all` exits 1 with zero output, discarding `label`, `safety`, `targets` and `approvals`, all of which succeed individually. The agent recovered by dropping the failing section and re-requesting the four that work — good behaviour, and wasted calls.

**One task-authoring warning.** A task was written on the premise that `H3-4` is the H3.1 gene mutated in DIPG. It is not; the DIPG gene is `H3C2`, legacy symbol `HIST1H3B`. The agent caught the false premise and corrected it. Ground truth written from memory is a failure source, and here the system under test caught the human.

## What the study did not cover

One model class, one run per task, so no variance figures. No open-weight model, which is where the silent-zero finding would actually be tested. The curated-skill arm was never run — `biomcp skill install` was not executed, so every result is the bare tool surface with no playbook. The flat-catalog comparison was never run. Suggestions were on in every run with no off-switch. Provenance was not measured. The local cBioPortal analytics lane was not exercised. OncoKB was excluded for want of a token and PharmGKB was down, so both are absent from every finding.

The harness defers tool schemas — it presents names first and loads the full JSON schema on request — so the agent did not hold all four schemas from the first token. That plausibly inflates the query-shape error rate, and the front-loaded control arm was not run.

## Where the actionable half went

The repository absorbed it. Records 1091 through 1100 in `repos/biomcp/sdlc/records/` cover the variant silent zero, the pharmacogenomics phenotype collapse, the false zero from a failed disease-label lookup, an unresolved filter reporting zero, `whyStopped` on stopped trials, one unavailable source discarding the working sections, the MCP schema not naming its sections, and a plain-text query field silently accepting filter syntax. Records 1104, 0899 and 0950 cover the adverse-event percent column and the genome-build labels.

Six findings had no ticket. They are filed as an issue at `repos/biomcp/sdlc/issues/2026-09-09-findings-from-the-2026-09-01-interface-study-with-no-ticket.md`: uniform provenance in the `_meta` envelope, protein-numbering tolerance on `--hgvsp`, a frozen replay mode for evaluation, suggestion ranking that prefers the user's own term, an abstention threshold on literature search, and UTF-8 double-encoding in the GTR bundle.

The St. Jude deck outline built on these findings, including its live-demo risk assessment, moved to `repos/mktg/notes/Presentations/st-jude-2026-paper-findings-deck-2026-09-01.md`.

## The paper advice, kept because it is cheap to keep

Seven hypotheses were assessed for feasibility. Three were runnable with no new code — progressive disclosure, curated skills (the skill-install manifest hashes are a ready-made control), and robustness under perturbation — plus the context-cost half of the flat-catalog comparison. Four needed work: the task-success half of the catalog comparison, the suggestion ablation (no off-switch), provenance (absent from four envelope shapes), and the multi-model comparison (blocked on replay mode and rate limits).

The recommendation was two papers rather than one. A short empirical paper on the three runnable hypotheses, led by the compression-decay result. Then the full benchmark paper, with uniform provenance, the normalization fixes and frozen replay as prerequisites rather than extras. And a standing caution: the "90% accuracy, zero hallucinations against 7 published papers" pilot cannot be defended, because one day with the tool produced three counterexamples.

None of that is a commitment. Ian's ruling stands: the measurements get rerun before anything is published.
