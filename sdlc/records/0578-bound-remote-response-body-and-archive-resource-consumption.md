---
base: c32d49f29af44c30e3b4192cc966e5e16d13bd99
head: f1032a330521c87d777527aef7f6e10aa3595478
---
Remote responses are buffered and copied without consistent bounds in the transport/download layer. The shared cached HTTP client applies its body-size limit **after** materializing the response into the cache, so an oversized body is fully buffered before rejection. cBioPortal study archive download and expansion are unbounded (a zip-bomb / oversized-archive class exposure). And after bounded reads the CTGov/PubMed paths still make redundant `.to_vec()` copies of the body. Together these are a resource-consumption and DoS-resistance gap in the layer every source flows through.

Imported from March ticket 578. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/578-bound-remote-response-body-and-archive-resource-consumption
