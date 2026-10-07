# 2027 — Adopt the shared InterPro domain response

Status: COMPLETE.
Milestone: 1.0
Component: backend
Owner: Root owns the repository queue and producer0715; Codex owns consumer2027 BUILD.
Phase: Reviewed, checked and landed through both callers.
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

DESIGN review correction: the same caller cases extend through existing CLI/MCP harnesses and JSON/Markdown outputs. Assert omission/order/source attribution, failures and serialized recovery. No duplicate decoder matrix or new framework is proposed. Root obtains closure of this named public-output gap before BUILD release.

## BUILD release

Ian released consumer BUILD against accepted design `475bcfb20eb468059daea5ff8dd0e53769f1d18f` and paired producer design `e031a07cc72091e6cfe519f71497321048ef692b`. Root owns producer0715, repository queues, fresh review, landing, push and paired closure. The consumer builder owns only2027. Producer LAND is `29e01881b24604a3d0ea079b79b0e60c2f506964`, BioData0.0.42. Sartre accepted corrected CODE `61bb5da2de09ee9652a968a067c3062055cefc87`. Its six public cases, lint and spec passed. The original whole test receipt applies to `ec60c80ae1b4271d153253ab53fe2e31d11d1d93`. The builder merged dedicated target `b46064c42cf94a9b4142f0922a49e0893d0a127d` without rebasing. GitHub internal server failures do not block authorized local implementation. Cargo uses the declared Git source and immutable full producer revision through the existing host Git cache. No source policy exception applies. Cleanup belongs to another agent.

## CODE handoff

Checked consumer candidate: `ef17e572ec5f5f812e71e46799bdd191c9b5a3b5`. Exact producer pin: `29e01881b24604a3d0ea079b79b0e60c2f506964`. All12 affected Rust checks and147 Python checks passed. The established Linux-only check is explicitly deselected on Darwin. Four affected ratchet audits passed. The [build record](../records/2027-adopt-interpro-domain-response.md) gives exact costs, retired claims, cache state and inherited findings. Runtime code is unchanged from early candidate `51c81ac0`; subsequent commits strengthen caller proof, register checks and retire the displaced parser claim. All candidate fixes are pushed. Root reports fresh Medium CODE review has started on the checked retirement candidate. This ticket stays OPEN until Root accepts, lands and records paired closure.

## Landing

Reviews: accept
Fresh Medium CODE review accepted ef17e572ec5f5f812e71e46799bdd191c9b5a3b5. Twelve affected Rust checks and147 Python checks passed, with one established Darwin deselection. Inherited whole-gate findings stay separate.
Landed: 22945b5ee0c31c97c79f6b2511068ca68f8c7dbb
