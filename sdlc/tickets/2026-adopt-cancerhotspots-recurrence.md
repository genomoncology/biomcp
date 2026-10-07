# 2026 — Adopt shared Cancer Hotspots responses and recurrence

Status: COMPLETE.
Owner: Root; assigned consumer builder.
Phase: Both variant callers use the shared producer; reviewed and landed on the dedicated 1.0 line.
Milestone: 1.0
Component: backend

## Outcome

Replace the local Cancer Hotspots response decoder, recurrence mapper and authoritative values with BioData0714 in both variant detail and variant structure. Preserve existing CLI/MCP JSON/Markdown behavior. The [design](../planning/cancerhotspots-2026/design.md) defines exact caller, test-retirement and measurement claims.

## Evidence

- Starts from: dedicated1.0 `56efdc4509e7e471a5539d4b6165c4565d86196b`, paired BioData `0ed99b4e70c3d8f5dddb6781a33a2711f27416ac` version0.0.39. Completed2011/2012/2014/2015/2025 already migrated whole MyVariant/cached evidence/direct ClinVar/direct population/direct constraint. Record0664 and retained capture receipts establish Hotspots examples. Actual get and structure callers still use the same local mapper and values.
- Keeps: source rows/order, alternate selection/counts/transcript, explicit null recurrence fields, requested change precedence, get all-section policy, structure applicability, source outcomes/provenance, retry/cache/body limits and timeouts. No command, MCP tool, rendering text or source route changes.
- Changes: both callers delegate original-byte decoding and selection to BioData0714 and store its immutable recurrence beneath local source metadata. Borrowed flattened encoding and directional target restoration preserve output. The consumer record names the exact implementation files and displaced code/test claims.
- Proof: one actual get transport/channel owner and one structure execution owner; producer0714 owns admission/selection/target semantics. Preserve independent expected answers and unique existing renderer/request/HTTP claims. Record build/test costs and lean suite growth during BUILD.
- Defers: live calls, hosted CI, scratch deletion, workers and release acceptance. Root owns fresh CODE review, dedicated 1.0 landing and paired closure. Other enrichments and whole Variant ownership remain open.

## Review

PM allocated2026 in the global repository sequence and stamped active1.0. Backend names this component. This design writer claims only this ticket and its linked design. Root assigns fresh paired DESIGN review of both exact candidates before any BUILD release. Cost of delay: replacing both finite active callers now advances1.0; a broad source audit or completed-slice redesign does not.

Review: fresh paired DESIGN ACCEPT at BioData1e7c6172 and BioMCPd191a20b. Root adopts the nonblocking correction that removes dedicated benchmark ceremony. Functional contracts remain unchanged. Root released BUILD after producer0714 landing and the dedicated target merge.

## BUILD release

Root released BUILD on October 7 after BioData0714 landed at `edd138da0ba104039885fe213107003dfb626246`, version `0.0.41`. This branch merged dedicated target `582bbd492e64d0227517dcaef828425ab84b2e10` at `692bf15f`; the target incorporates maintenance `1c774def`. Both callers use the shared original-byte response and immutable recurrence. New caller owners passed before the source mapping claims retired. Root owns fresh review, dedicated 1.0 landing and paired closure. The ticket remains open until that outcome.

Final qualification: both callers and channels passed. The [consumer record](../records/2026-adopt-cancerhotspots-recurrence.md) names the exact producer, target, candidate, 14 affected Rust tests, 147 Python checks, retired claims, measured costs and inherited gate limits. Root retains closure authority.

CODE review: ACCEPT at d97fd21b by fresh read-only reviewer; no concrete findings.
Landed: e87bb5981665c3d077e576058b9cf079aa7e3af0
