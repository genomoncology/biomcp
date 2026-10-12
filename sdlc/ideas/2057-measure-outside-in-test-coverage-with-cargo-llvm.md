# Measure outside-in test coverage with cargo llvm-cov: outside-in tests alone (CLI, MCP, spec), then with unit tests, and the share of unit-test lines outside tests already reach; report the largest gaps

Status: open.

Kind: idea
When: after the 0.9.2 tag ships

## Problem

Measure outside-in test coverage with cargo llvm-cov: outside-in tests alone (CLI, MCP, spec), then with unit tests, and the share of unit-test lines outside tests already reach; report the largest gaps. shows which unit tests duplicate outside-in proof and where behavior has no outside test.
