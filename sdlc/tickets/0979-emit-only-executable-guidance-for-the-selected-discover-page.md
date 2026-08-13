---
flow: build
priority: 8
---
# Emit only executable guidance for the selected discover page

Later `discover` pages can classify weak UMLS labels such as `erbB1 Genes` as
genes and emit `biomcp get gene erbB1 Genes`. The command is not shell-safe and
Clap parses `Genes` as a section, so following BioMCP's own guidance fails.
Mutation-like UMLS labels can similarly be presented as exact gene identities.
At an offset beyond the final page, `concepts` is empty but guidance derived
from concepts hidden by pagination is still returned.

Discovery guidance must be derived from the concepts actually returned on the
selected page, use the shared typed command renderer, and claim an exact entity
command only when the candidate has a suitable canonical identity. Weak or
mutation-like labels may offer a bounded search or article path, but not a
fabricated exact gene lookup.

## Done when

- Every next command emitted for a selected discovery page is safely rendered
  and accepted by the current CLI parser as one command.
- Weak UMLS-only gene labels containing spaces or punctuation do not become
  exact `get gene` commands without canonical gene resolution.
- An empty selected page has no guidance derived from unreturned concepts;
  any query-level fallback is explicitly identified and independent of hidden
  results.
- Deterministic tests cover weak gene labels, mutation-like labels, shell
  metacharacters, and an offset beyond the final page without public network
  access.

## Authorized test changes

Design may restate discovery selection and command assertions in
`src/entities/discover.rs`, parser-validity assertions in
`src/cli/tests/next_commands_validity.rs`, and executable discover contracts in
`spec/surface/discover.md` and `spec/surface/cli-contract-ratchet.md`.
