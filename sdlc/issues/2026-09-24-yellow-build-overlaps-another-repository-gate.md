# Yellow build overlaps another repository gate

Status: open. Reported 2026-09-24 during BD ticket 0171 verification.

## Observation

A BioMCP spec artifact build ran on Yellow while BD's reviewed container runner held the shared gate lock. The observer found BioMCP Cargo PID 1425695 and rustc PID 1425720 in the BioMCP checkout. Cargo stdout targeted `biomcp-gates-1240.log`. The Cargo command was `cargo build --locked --profile spec --no-default-features --bin biomcp --example rmcp_streamable_http_contract`. BD's concurrent runner PID was 1420565 with Docker child 1420591.

The BD run used pushed revision `ed42252b461287c67bc17e052e2731490755e78b`. Its partial formatting and fixture-policy failures are preliminary evidence. The run was stopped, its temporary checkout was cleaned up, and its results were not accepted as the required gate. The BioMCP processes were left untouched.

## Needed repair

Determine how both jobs entered the shared host concurrently. Check the lock path and identity, acquisition timing, lock lifetime, and every BioMCP entrypoint used by the spec build. The observation establishes overlap; it does not establish which launcher omitted or released a lock.

All manual Rust gate and artifact-preparation work on Yellow must share the same host lock for the whole run. A one-time process check does not prevent another job from starting later. Add a reproducible concurrency check that shows the second job waits or refuses before Cargo starts. Preserve ordinary hosted CI behavior and the release owner's active work.

This issue records the defect only. It authorizes no interruption of another job and changes no runtime code or release decision.

Edit 2026-09-24 (queue owner): the original text named the other
repository, which this repository's zero-coupling gate forbids; the
name is now "BD" with the same meaning. No observation changed.
