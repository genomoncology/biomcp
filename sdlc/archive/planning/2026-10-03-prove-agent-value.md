# Prove that BioMCP helps agents

Started 2026-10-03 on Ian's direction. The BioMCP owner drives this programme and answers for its results. It serves ideal-state gap 4 (nobody measures whether an agent succeeds with the tool) and gap 10 (no paper or benchmark backs the claims). It also serves ThinkThen gaps 2, 3, 4 and 6: an outside user, labeled cases, long-text search, and the missing functions.

## The claim under test

A model remembers the frozen past. BioMCP serves the present. Tools help an agent where memory cannot answer:

1. Answers that changed after the model's training cutoff: approvals, reclassifications, trial status.
2. Complete lists, such as every recruiting trial for one mutation and cancer.
3. Exact identifiers and codes.
4. Ranked documents, where the answer is a list of PMIDs or trials and memory cannot invent them.
5. Answers whose cited sources actually support them.

Experiment 421 showed the opposite case. On old BioASQ recall questions, tools tied the bare model and sometimes talked it out of right answers. That result stays in the paper as the motivating negative.

## Rules for every experiment

- Each experiment gets a numbered folder under `~/workspace/experiments/` with a README naming the risk, inputs, stop rule, spend cap and the decision it can change.
- Answer keys come from the raw provider, never from BioMCP, so a tool cannot grade itself.
- Every JSON shape the harness reads gets an assertion before the data reaches a model. Experiment 421's empty-abstract bug is the reason.
- Exact matching never grades free text alone. A grader is calibrated against a hand-checked sample before any number leaves the folder.
- GLM-5.3 runs on the Z.ai coding endpoint. ThinkThen runs on a local host. Each experiment caps ThinkThen spend at $2 with `--max-requests-total` and records actual spend.
- BioMCP under test is the published 0.9.1 wheel: `uvx --from biomcp-cli==0.9.1 biomcp`.
- Work stays on cancer care decisions for clinicians.

## Who does what

- The BioMCP owner designs experiments, reviews results, and writes every conclusion.
- Implementers build harnesses and data pulls, one per experiment folder.
- Fresh researchers review designs and code. A fresh reviewer checks clinical question wording and any claim headed for the paper.
- The GLM-5.3 agent runs the agent legs through `pi-job`. The owner verifies every report against raw output.

## Experiment queue

| No. | Question | Status |
| --- | --- | --- |
| 431 | Can we build a question set whose answers changed after the model's cutoff, with keys from ClinVar, FDA and ClinicalTrials.gov? | started |
| 432 | Does an agent answer better with BioMCP, with BioMCP plus its skill, and with ThinkThen added, than with no tools? | started |
| 433 | On TREC Precision Medicine cancer topics, does ThinkThen ranking of BioMCP results beat BioMCP's own order against judged relevance? | started |
| 434 | Does meaning search over windows of one full-text paper find answer passages that grep misses, and which window shape works for papers? | queued |
| 435 | Can ThinkThen link gene, variant and disease mentions to BioMCP candidates, including "none", scored against `article entities`? | queued |
| 436 | Do an article's claims match the abstracts they cite? | queued |
| 437 | Does a local knowledge base on the LLM wiki pattern with QMD search gain anything from ThinkThen judgments over QMD alone? | queued |

Published benchmarks to adopt after wave 1, in order:

1. TREC Precision Medicine 2017 to 2019 (433): cancer topics, judged abstracts and trials.
2. SkillsBench healthcare tasks (Li et al., arXiv 2602.12670). The blog post `docs/blog/skillbench-biomcp-skills.md` cites its skill gain, but nobody has measured BioMCP's own skill on it.
3. TREC Clinical Trials 2021 to 2023: patient descriptions matched to trials.
4. GeneTuring and GeneHop: gene name, location and multi-step lookups, with GeneGPT as the published tool baseline.
5. LAB-Bench DbQA and LitQA2, restricted to clinical and genomic items.

The remaining documentation asks (decorate JSON, Markdown cleanup) fold into 434 and 437 as steps, not separate experiments.

## The paper

Working claim: evidence beats memory when the question has a date on it.

Shape: a tool paper for BioMCP with a four-part evaluation.

1. Static recall on BioASQ: no gain, with the reason (experiment 421).
2. Dated questions: bare model against tool-using agent (431, 432).
3. Ranking on judged cancer topics (433).
4. Source faithfulness and cost (432, 436).

The outline and drafts live in `repos/mktg/content/biomcp/papers/` once 431 to 433 report.

## Articles

Each article comes from a recorded experiment and links its folder's numbers. The backlog stays in `notes/ideas/biomcp-marketing-thread.md` and drafts stay in `repos/mktg/content/biomcp/drafts/`. ThinkThen pairs follow the joint documentation message, `repos/sdlc/inbox/biomcp/2026-10-03-docs-joint-marketing-with-thinkthen.md`.

## Where results go

- BioMCP defects and features become tickets in this repository for the next 0.9 release.
- ThinkThen feature evidence goes to `repos/sdlc/inbox/thinkthen/`.
- The knowledge base becomes its own repository once experiment 437 works. Ian decides whether it goes public.
