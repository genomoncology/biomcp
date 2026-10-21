---
priority: 1
---

# 2331 — Check keyed Semantic Scholar recordings safely

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 274f36b3539ec387900b994fc259010f1220bcb3, accept

Reviews: revision e6baee3acbeae287aaf1cbd49dd27d3c5df8175d, accept

## Outcome

Changes: `tools/check-replay-live`, `tests/test_replay_live_drift.py`, `docs/reference/release-process.md`, `CHANGELOG.md`.

Let the existing replay live checker check exactly the five Semantic Scholar recordings using the inherited S2_API_KEY, read-only and paced at one second, under Ian's ruling relayed in PM m3019. Preserve existing source restrictions and safe logging. Confirm the current checker limitation before changing it; reuse an existing supported path if one exists.

- Proof: outside-in controls show explicit source selection, inherited authentication, pacing and failure handling without printing or persisting credentials or authorization headers. Run the full routine on the coherent branch composed with current main before fresh review. The actual five-recording check belongs to release admission 2330.

## Evidence

- PM m3019 authorizes existing build-machine keys without agent inspection, copying or storage. Committed fixtures remain minimized. KEGG stays uncalled; this ticket grants no other source access.
- Carry the decision from tickets/pm-plan in the next landing. Keep this source change separate from admission records; add it as an admission dependency if needed.
