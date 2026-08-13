# ClinGen CSpec captures

`atm-gn020-1.5.1.json` was captured from the real ClinGen CSpec document
`https://cspec.genome.network/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1`
on 2026-07-30. It is minimized to the envelope, document identity fields, and
the first two criteria needed by the paging contract. In particular, it retains
the provider's absent `data.@id`, which distinguishes the real document shape
from the older synthetic fixture.

`pten-gn003-3.2.1.json` is a byte-faithful capture of the public PTEN GN003
version 3.2.1 document from 2026-08-13. It retains the five linked public File
entities used to prove attachment metadata without downloading their content.
