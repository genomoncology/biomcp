# 2026 — Adopt shared Cancer Hotspots responses and recurrence

Status: OPEN.
Owner: Root; assigned consumer builder.
Phase: BUILD candidate; Root fresh CODE review and focused qualification pending.
Milestone: 1.0
Component: backend

## Outcome

Replace the local Cancer Hotspots response decoder, recurrence mapper and authoritative values with BioData0714 in both variant detail and variant structure. Preserve existing CLI/MCP JSON/Markdown behavior. The [design](../planning/cancerhotspots-2026/design.md) defines exact caller, test-retirement and measurement claims.

## Evidence

- Starts from: dedicated1.0 `56efdc4509e7e471a5539d4b6165c4565d86196b`, paired BioData `0ed99b4e70c3d8f5dddb6781a33a2711f27416ac` version0.0.39. Completed2011/2012/2014/2015/2025 already migrated whole MyVariant/cached evidence/direct ClinVar/direct population/direct constraint. Record0664 and retained capture receipts establish Hotspots examples. Actual get and structure callers still use the same local mapper and values.
- Keeps: source rows/order, alternate selection/counts/transcript, explicit null recurrence fields, requested change precedence, get all-section policy, structure applicability, source outcomes/provenance, retry/cache/body limits and timeouts. No command, MCP tool, rendering text or source route changes.
- Changes: DESIGN claims `sdlc/tickets/2026-adopt-cancerhotspots-recurrence.md` and `sdlc/planning/cancerhotspots-2026/design.md` only. Later reviewed BUILD supplies direct original-byte shared decoding, shared response return and shared recurrence storage beneath local source metadata; borrowed flattened encoding and directional target restoration. Retire displaced local models/helpers and redundant mapping tests after both callers pass.
- Proof: one actual get transport/channel owner and one structure execution owner; producer0714 owns admission/selection/target semantics. Preserve independent expected answers and unique existing renderer/request/HTTP claims. Record build/test costs and lean suite growth during BUILD.
- Defers: BUILD, tests, measurements, sources/live calls, hosted CI, deletion, workers and landing. Other enrichments, whole Variant ownership, live provider qualification and release acceptance remain open.

## Review

PM allocated2026 in the global repository sequence and stamped active1.0. Backend names this component. This design writer claims only this ticket and its linked design. Root assigns fresh paired DESIGN review of both exact candidates before any BUILD release. Cost of delay: replacing both finite active callers now advances1.0; a broad source audit or completed-slice redesign does not.

Review: fresh paired DESIGN ACCEPT at BioData1e7c6172 and BioMCPd191a20b. Root adopts the nonblocking correction that removes dedicated benchmark ceremony. Functional contracts remain unchanged. Build follows producer0713 landing and then-current main.

## BUILD release

Root released BUILD on October 7 after BioData0714 landed at `edd138da0ba104039885fe213107003dfb626246`, version `0.0.41`. This branch merged dedicated target `582bbd492e64d0227517dcaef828425ab84b2e10` at `692bf15f`; the target incorporates maintenance `1c774def`. Both callers use the shared original-byte response and immutable recurrence. New caller owners passed before the source mapping claims retired. Root owns fresh review, dedicated 1.0 landing and paired closure. The ticket remains open until that outcome.
