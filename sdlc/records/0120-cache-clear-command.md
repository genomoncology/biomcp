---
base: 4da82bb9df6e6de146b35e1a869b272dea5b03fc
head: 5af589dc85989c477ad8328ed61ebe204cadc779
---
The cache family CLI (116) and the non-destructive `cache clean` command (143) establish the CLI surface and cleanup patterns. This ticket adds the destructive `biomcp cache clear` subcommand — a full wipe of the managed HTTP cache with TTY confirmation safety.

Imported from March ticket 120. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/120-cache-clear-command
