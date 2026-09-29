# Yellow build overlaps another repository gate

Status: resolved. Reported 2026-09-24 during the other repository's ticket 0171 verification; the decision and lock protocol below supersede the open status.

## Observation

A BioMCP spec artifact build ran on Yellow while the other repository's reviewed container runner held the shared gate lock. The observer found BioMCP Cargo PID 1425695 and rustc PID 1425720 in the BioMCP checkout. Cargo stdout targeted `biomcp-gates-1240.log`. The Cargo command was `cargo build --locked --profile spec --no-default-features --bin biomcp --example rmcp_streamable_http_contract`. the other repository's concurrent runner PID was 1420565 with Docker child 1420591.

The other repository's run used pushed revision `ed42252b461287c67bc17e052e2731490755e78b`. Its partial formatting and fixture-policy failures are preliminary evidence. The run was stopped, its temporary checkout was cleaned up, and its results were not accepted as the required gate. The BioMCP processes were left untouched.

## Needed repair

Determine how both jobs entered the shared host concurrently. Check the lock path and identity, acquisition timing, lock lifetime, and every BioMCP entrypoint used by the spec build. The observation establishes overlap; it does not establish which launcher omitted or released a lock.

All manual Rust gate and artifact-preparation work on Yellow must share the same host lock for the whole run. A one-time process check does not prevent another job from starting later. Add a reproducible concurrency check that shows the second job waits or refuses before Cargo starts. Preserve ordinary hosted CI behavior and the release owner's active work.

This issue records the defect only. It authorizes no interruption of another job and changes no runtime code or release decision.

Edit 2026-09-24 (queue owner): the original text named the other
repository, which this repository's zero-coupling gate forbids; the
name is now the generic phrase "the other repository". No observation changed.

## Shared-lock protocol (2026-09-26, Ian's direction; wording
## corrected 2026-09-27)

Correction (2026-09-27 review): the paragraphs below first claimed
BioMCP's gate scripts already take the lock. They do not. The
flock-acquisition code lives in the dotfiles-side runner, and the
dotfiles issue
`sdlc/issues/2026-09-24-yellow-lock-available-during-active-checks.md`
records the lock free during BioMCP checks twice, including a
recurrence on 2026-09-26 involving an agent's `run-gates-*.sh`
wrapper. The protocol below is the recorded rule for when host
coordination resumes; it is not mechanically wired into BioMCP's
own scripts, and the ad-hoc agent gates run since 2026-09-26 relied
on Ian's "you own yellow for now" grant, not on the lock.

The protocol itself, when coordination resumes: take
`~/.yellow-gate.lock` for the ENTIRE job — acquire before
`make lint` and release after `GATES-DONE` — using `flock`:

    exec 9>"$HOME/.yellow-gate.lock"
    flock 9
    ... make lint / test / spec / stress ...
    flock -u 9

Any future BioMCP gate runner (agent or human) uses this shape; the
one-gate-at-a-time check by pgrep is no longer the primary guard,
only a fallback. The other repository's runner holds the same lock,
so a job that starts while the lock is taken waits instead of
overlapping.

## Resolved

The process rule is in force and recorded here: one gate at a time,
checked before launch; when host coordination resumes the runner
takes `~/.yellow-gate.lock` for the entire job (flock), so a
starting job waits rather than overlaps. Until the lock is wired
into the runner (dotfiles issue
`2026-09-24-yellow-lock-available-during-active-checks.md`), the
grant "you own yellow for now" is the working control and the pgrep
idle check is the fallback. The 2026-09-24 overlap this file
records was the last observed; none since.
