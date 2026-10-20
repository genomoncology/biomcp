---
base: 2cc785a7d3a9849025c7de89372a33139c1aef5b
head: 8e721ae1
---

Moved all five server-starting run wrappers onto the generalized fixture
supervisor from 0896. Each wrapper now owns a canonical cache root, carries an
authenticated kind/token marker, records PID and process-start identity, and
uses shared record validation for normal cleanup and stale recovery.

Wrapper supervision deliberately binds to the wrapper process itself rather
than the longer-lived specification coordinator. Killing a wrapper therefore
reaps its complete server process group, owned root, socket, and port while the
rest of the specification run can continue.

Five behavioral tests run the real wrappers through their exported ownership
records, SIGKILL each wrapper, and prove that the recorded group and root are
gone. All five pass; Bash syntax and Python lint also pass. Production `src/`
did not change.
