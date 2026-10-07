# 2027 — Adopt the shared InterPro domain response

Status: OPEN.
Milestone: 1.0
Component: backend
Owner: Root; DESIGN author Codex.
Phase: DESIGN drafted; fresh paired review and BUILD release belong to Root.
Risk: Cap ordering, first overlapping fragment and failure classification affect existing public answers.

## Outcome

Use producer0715 through both `entities::protein::get` and `entities::variant::structure::structure`. Preserve the public protein-domain and variant-structure answers and retire displaced source ownership. The [design](../planning/2027-interpro-domain-adoption.md) specifies the actual callers and distinct proof.

## Evidence

- Starts from: dedicated1.0 `c8e3c190`; producer `42c0a7eba67870185dbeca25d0209597ffdd3c20`. The milestone1.0/backend queue has no ready or waiting duplicate. Global `pm ticket new` allocated2027. Allocation used isolated Git metadata because local maintenance main differs from remote; no shared main ref changed.
- Keeps: protein cap20, structure cap25, take-before-filter, trimmed labels, first inclusive overlap, duplicate occurrence order, output omission/null rules, source labels, outcomes, warnings and CLI/MCP channels.
- Changes: InterPro client returns a shared admitted response; both callers consume borrowing domain views. Retire local source decoder/types and displaced parsing proof after producer coverage and real caller checks pass.
- Proof: one real protein-get case and one real structure case, plus focused transport failure checks. Keep existing outcome/rendering owners. Producer0715 owns source admission and range shaping. A library-only transfer is incomplete.
- Defers: hosted jobs, live requests, production data, source copies, source licensing changes, broad audits, publication and deletion. Completed0714/2026 remains complete. DESIGN claims only this ticket and its linked note.

## Review

Root assigns a fresh read-only review of the exact pair and releases BUILD separately. No implementation or independent acceptance is claimed. Root reconciles maintenance before consumer BUILD/review without rebasing the dedicated line. M5 is authorized; newer host strategies do not override the no-hosted-jobs boundary. Another agent owns cleanup.
