# 0.9.4 release admission

The release procedure lives in [the canonical runbook](../../docs/reference/release-process.md). This record binds proof to source; the execution trail remains in pm job logs and the owning retained raw evidence. No tag, publication or final admission GO is established here.

## Source and gate proof

| Published source | Owning main job | Result |
| --- | --- | --- |
| a5314c569719e0336f5ace6c83134dd9afbd6982 | 2299/1792530993968-62782 | Passed, 1004001 ms |
| 3c9ddc1510c0152a150f7bf86d7466d718c5d6d4 | 2308/1792532689278-14540 | Passed, 879115 ms |
| 39425d8fce157f9aa1a92310af9da60983d2dae6 | 2309/1792534415712-7306 | Passed, 1011661 ms, including repaired-source installed Linux wheel |
| 5eaba3028d0b494d441eafd6d6e7e36d34bcc3f3 | 2310/1792535942921-65624 | Failed, 231447 ms, source archive scan encountered a disappearing temporary directory |

The coordinator independently read these terminal results. Passing landing gates remain in their owning ticket/job records. The source-archive repair's branch routine 2320/1792536475770-292 passed on 52b1782a19011c5ace2bb55be474e476d317f50e. Its post-merge job 1792537535078-50249 was canceled under the single-gate ruling and ended with exit 143; it is not a pass or a new source defect. Historical failures retain their original statuses.

The advisory-scope policy landed at 01c431110d3e44a400663c8dd4459e33e1304db4 with candidate gate 2307/1792538030462-72182 passing in 380323 ms. The focused Python environment landed at badecf8c3a2eae2050f5ef4b47cf9f372065de46 with candidate gate 2311/1792538597761-89232 passing in 402731 ms. The coordinator read both terminal results. Neither landing repeated the full main check: the final candidate receives that check once before rehearsal.

Fresh source reviews cover the published increments through 5eaba3028; their original reviewers remain attributed. The continuing prefix reviewer accepted that bounded scope and made no final GO claim. No new reading of the earlier 1122 commits or every large golden body is claimed. Source fixes and reviews remain in their owning records rather than duplicated here.

A fresh completion reviewer accepted the later source and workflow increments at final main `b91c4d46bb33741e76065dbe81e3b406d0e1596c`. Its non-SDLC tree is identical to the reviewed final composition, including the coverage floor, shared timing policy, profile forwarding, release entry points and nightly target. This source acceptance is separate from final qualification and admission.

Fresh Linux job `2258/j402` failed on that source with exit 100: preparation passed, then the routine Rust suite reported 4,285 passed, two failed and 34 skipped. The stale ClinGen prefetch fixture lacked an expired provider policy, and the stalled citation bridge case observed zero requests instead of one. Later stages did not run. Kernel isolation, unchanged tracked source hashes and owned scratch removal were verified. Full diagnostics and the actual result are retained at `final-qualification/j402/` under the permanent evidence root below; manifest SHA256 `34b36b3ffe372a7204783e4b746c19c74ace63bf00d26f0223cf9aec9cda926c`. Ticket 2322 owns the repair; this run is not qualification success.

The same source passed `make main-gate` in M5 job `2258/j401`, including the routine gate, repository audit, all-feature checks, rendered outputs and installed Linux wheel smoke. The coordinator read exit 0 directly: command runtime 1,428,457 ms, scheduler total 1,436,140 ms. This pass does not override the Linux failure or qualify a later repaired source.

Ticket 2322's fresh reviewer accepted `30bc63d919ef9f25bb9ac62690224fd920f038da`. Its bridge fixture removes unrelated cold-cache work from the existing deadline test; its stale-note fixture seeds the required replies through canonical request plans and the shared cache client. Product assertions, deadlines and source rules are unchanged. Yellow job `2322/j567` demonstrated the original incomplete-cache precondition failure followed by nine passing affected cases; `j587` passed formatting and warning-denied Clippy. The tested private payload and published patch were compared by the reviewer. The earlier intermittent failure remains observed only in j402; the controlled test demonstrates the missing-cache precondition, not that exact schedule. Raw repair evidence is retained under `linux-routine-repair/2322/` with manifest SHA256 `7411d235317559692f15be217966f5041388bfd822f7590a75917c3fb56c8c94`.

The repair landed at `a4918fa3e298b8cc48eb900fb720ea041868850c` after candidate job `2322/1792550468210-73627` passed with exit 0 in 863,570 ms. The completion reviewer accepted the bounded source delta at that published revision. Final qualification must bind to this repaired source; no earlier pass or failure is relabelled.

This repaired source passed the complete `make main-gate` in M5 job `2258/j637`: exit 0, command runtime 1,254,057 ms, scheduler total 1,258,101 ms. It includes the routine gate, repository audit, all-feature checks, rendered outputs and installed Linux wheel smoke. The actual caller log, scheduler record and compiler binding are retained under `final-main/j637/`. Linux qualification and the full untagged rehearsal remain separate requirements.

## Qualification boundaries

Whole Linux job 2258/1792502148739-726187 passed exact source 2ca44d1bd1430d7af2bb01bf98bb82d88aa8415c. It proved the named fresh-clone offline stages and cleanup at that source. Later dependency, runtime, fixture and gate changes require final-source qualification; the earlier result is not relabelled as that final proof. The retained Yellow 493 evidence below preserves its original complete output and manifest. [Record 2233](2233-linux-offline-suites.md) preserves earlier qualification and failed attempts.

Fresh Linux job `2258/j643` passed repaired source `a4918fa3e298b8cc48eb900fb720ea041868850c` with exit 0 and scheduler total 2,783,413 ms. It rebuilt without Rust reuse under verified kernel network isolation with Rust 1.93.1 and checkout-owned Python 3.12.12. Rust routine reported 4,287 passed and 34 skipped; the explicit long profile selected zero cases. The inventory check, full Python suite (1,837 passed, 12 skipped), strict docs, all-target/all-feature Clippy, six AlphaGenome cases, full-feature executable and PNG/SVG/terminal checks all passed. Tracked source identity and owned scratch removal were verified. Raw stage outputs and the actual caller log/result are retained under `final-qualification/j643/`.

Full untagged rehearsal [39540105855](https://github.com/genomoncology/biomcp/actions/runs/39540105855) used that repaired source and failed its initial changelog admission: the release-range check required the missing described internal bullet for 2306. All later builds and publishers were skipped. Branch lint's shipped-behavior coverage had passed, but it does not replace the release-range metadata check. The correction adds that internal bullet under this admission ticket. This failed workflow is retained under `rehearsals/39540105855/`; it is not a successful rehearsal.

PM approved reusing `j637` and `j643` for the correction only while the delta from `a4918fa3e298b8cc48eb900fb720ea041868850c` is confined to `CHANGELOG.md` and `sdlc/` files. The coordinator ran `git diff --stat a4918fa3e298b8cc48eb900fb720ea041868850c c14328f67be196cd6853b4f7eafa5bb298bd6f45`: three files, 33 insertions and three deletions, comprising `CHANGELOG.md`, this record and the 2258 ticket. Product source, tests and workflows are byte-identical. Subsequent proof-only record additions must satisfy the same path boundary. Any other change requires fresh qualification. The complete untagged rehearsal still runs on the corrected final commit.

Metadata job `2258/j813` passed that correction at `c14328f67be196cd6853b4f7eafa5bb298bd6f45` with exit 0 and scheduler total 58,748 ms. It ran the canonical collected release checks, including the full release-range changelog command and repository audit. Only proof and ruling records were added afterward; the checked release metadata inputs remain unchanged.

Full untagged rehearsal [39418376272](https://github.com/genomoncology/biomcp/actions/runs/39418376272) used cffae4af5fb600039137b7c13d6955165ca46823 and failed both Linux installed-wheel smokes. Its successful component stages do not constitute admission. Its run metadata and failed logs remain under the retained 2102 evidence below at `rehearsals/39418376272`. Later wheel repairs retain their own receipts. The final rehearsal must use the fully repaired source and succeed in every required job, including rehearsal-complete.

Permanent raw evidence lives on Yellow under `/home/ian/workspace/data/biomcp/release-evidence/0.9.4/`. Historical experiment evidence is retained at `legacy-experiments/m5/2081-biomcp-2258-research-fixture-retention`, `legacy-experiments/m5/2102-biomcp-2258-final-live-replay` and `legacy-experiments/yellow/493-biomcp-2258-repaired-linux`. Each `source-index.sha256.json` resolves the original root and paths to verified retained files. Exact cited archives and artifact-comparison inputs are preserved; only uncited Cargo intermediates from the failed wheel build are omitted and indexed. Source-ticket candidate receipts are retained under `assessment-landings/`.

## Observations and measurements

[Record 2234](2234-candidate-measurements.md) compares v0.9.3 with 51f5e03b on the same M5. It remains historical context. PM accepted the measured warm-variant and cached-GWAS regressions and fixed SQLite overhead; their timing cause remains unknown. [Record 2284](2284-rebuild-attribution.md) retains its separate negative experiment. PM withdrew the refresh request: run no measurement refresh and claim no final-source speed result.

The approved live proof retains 39 checked/passed observations and exact later captures with their actual revisions. [Recorded command coverage](2236-recorded-command-goldens.md), [format qualification](2293-real-replay-format-qualification.md) and [provider identities](2295-provider-policy-identities.md) state their structural, format and source-policy limits. Offline-ready entries are not live passes. Restricted and rate-limited sources remain explicitly unverified under the PM rulings; no clinical validity or value equality is inferred from status/type/shape checks.

[The security repair](2309-mcp-security-upgrade.md) retains the exact wire-compatibility proof and the original failed wrappers separately. Read-only GitHub responses show all fourteen assigned alerts 80 through 93 fixed; none was manually closed. [The wheel ownership repair](2310-wheel-scratch-ownership.md) retains ownership, symlink and combined-failure controls; its failed actual-main job above remains failed.

## Final proof binding

The final local main and fresh-clone Linux jobs name the repaired source. The PM's bounded metadata-reuse ruling above permits the full successful untagged rehearsal to name its changelog-and-records correction, with both source bindings and the exact delta retained. A later records-only change does not require another rehearsal under the canonical process. Ian alone authorizes the explicit source to tag.

The rehearsal does not exercise GitHub publication, PyPI trusted upload, registry authentication/pushes or latest movement, Pages deployment/live-site verification, or the Homebrew push. It validates the production builds, installed artifacts, container smokes, generated documentation and publication payloads without publishing credentials.

Ian's final publication ruling permits completing and landing the repair, qualification and untagged rehearsal only. No tag, GitHub release, package upload or public announcement is authorized. The coordinator hands the qualified commit to the PM; Ian checks and publishes it himself.
