---
base: fd2f40ef
head: 19a98783
---

All seven VAERS/OpenFDA checks now run routinely against the existing local
aggregate server, started once by the routine runner. A fresh CDC WONDER MMR
aggregate response anchors the real provider shape, and a fresh OpenFDA
pembrolizumab reaction-count response drives the count contract. Existing
local CDC responses retain the stable positive, seriousness, and age output.

The page covers help, non-vaccine rejection, positive MMR output, combined
source outcomes, unsupported VAERS filters, invalid count fields, and a valid
OpenFDA count. The focused page passed all seven blocks and 48 receipt and
registry tests passed. No source lines were added against the 140-line
ceiling.
