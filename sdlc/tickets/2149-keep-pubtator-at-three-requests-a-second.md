---
flow: build
priority: 2
deps: []
---

# 2149: Keep PubTator at three requests a second

Status: OPEN.

Milestone: 0.9.3

## Outcome

BioMCP sends PubTator3 no more than three requests a second whether or not `NCBI_API_KEY` is set. The keyed ten-a-second budget stays with E-utilities.

## Evidence

Reported 2026-10-16 by the research team.

- `pubtator_min_interval` (`src/sources/rate_limit.rs:263-268`) returns 100 ms when `NCBI_API_KEY` is set and 334 ms otherwise; the comment at `:97` says the key enables a higher PubTator budget, and the test at `:624-626` pins it.
- PubTator3's API documentation asks callers to stay at three requests a second or fewer and names no higher keyed rate. The NCBI key raises the E-utilities limit only.

## Work

- Reproduce on main first, and confirm in PubTator3's current documentation that it grants no keyed rate. If it does, cite it in this ticket and close it.
- Otherwise make PubTator's interval 334 ms in both cases, and correct the comment and test.

## Acceptance

1. The rate-limit test pins 334 ms for PubTator with and without the key, and 100 ms for keyed E-utilities.
2. The changelog carries a bullet for this ticket.
