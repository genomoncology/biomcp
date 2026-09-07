---
flow: build
priority: 8
deps: []
---

# Read trial eligibility from the shared BioData type

## Goal

BioMCP stops maintaining its own clinical-trial eligibility model and reads the one BioData already publishes. One project owns the parsing of sex, age bounds, and criteria text, so the two cannot drift.

Today `src/entities/trial/mod.rs` defines a local `TrialEligibility` with its own `sex`, `minimum_age`, `maximum_age`, plus separate `age_range` and `eligibility_text` fields on `Trial`. BioData publishes `ClinicalTrialEligibility` covering the same ground. Six BioData trial tickets have already landed — 1169, 1170, 1171, 1175, 1176, and 1177 — so eligibility is the next slice of the same migration, not a new direction.

## Required behavior

`Trial` carries a single `eligibility` field of BioData's `ClinicalTrialEligibility`. The local `TrialEligibility` type, the separate `age_range` field, and the separate `eligibility_text` field are gone.

The trial card still shows an Eligibility section whenever the source supplies eligibility, and that section still opens with a `Sex:` line and an `Eligible Ages:` line when the source supplies them. Those two lines move from `templates/trial.md.j2` into the Rust renderer, which is where the BioData value can be read. The template branches on whether eligibility is present rather than reassembling it.

Age normalization keeps the behavior record 1116 established: a bound that fails to parse is retained as its original string rather than dropped, and a no-limit bound reads as no limit rather than as a number.

The `Cargo.toml` BioData pin moves from `a4c5bec98ab5185cc50bd6bc8c18f833b8d4097f` to `685a830aa634545fae1a93d2025717de30cbb348`, which is the revision publishing the type.

## Success criteria

- `biomcp get trial <id>` prints the same Eligibility section for a trial with sex and age bounds as it printed before this change, including both lines and the criteria text.
- A trial whose source supplies no eligibility prints no Eligibility section.
- A trial with an unparseable age bound still shows that bound's original text.
- A trial with a no-limit upper bound still reads as having no upper limit.
- JSON output carries eligibility under one field, and its shape is documented wherever the trial JSON contract is stated.
- Both trial sources keep working: ClinicalTrials.gov and NCI.
- `grep` finds no remaining definition or use of a BioMCP-local `TrialEligibility`.
- `make lint`, `make test`, and `make spec` pass.

## Boundaries

This ticket moves the eligibility model to BioData and nothing else. It does not change eligibility filtering or search, add eligibility fields the sources do not supply, alter arms, references, or any other trial section, interpret criteria text, or move a further BioData type.

## Implementation status

An implementation exists and predates this ticket. `origin/ticket/1178-implementation` at `589454ca` carries it: 41 files, 1,629 insertions, 543 deletions, including the dependency bump. The identical change also sits on the local branch `manual/1178-biodata-eligibility` at `aaf9739f` on an older base; that copy is redundant and can be dropped.

The branch is one commit ahead of main as of `e95bb7a4` and 33 commits behind current main. It needs a rebase before review.

Writing the ticket after the implementation inverts the normal order, so its acceptance criteria above were derived by reading the diff rather than from a design. The BioData team owns that work and must confirm the criteria describe what they intended, in particular:

1. Whether the trial JSON eligibility shape is meant to change with the type, and whether any consumer depends on the old shape.
2. Whether moving the `Sex:` and `Eligible Ages:` lines out of the template into Rust is intended, or an artifact.
3. Whether dropping the separate `age_range` and `eligibility_text` fields is intended for the public JSON as well as the internal model.
4. Whether the BioData revision bump carries anything besides the eligibility type.

Correcting this ticket before review is expected, not a defect.
