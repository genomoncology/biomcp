---
flow: build
priority: 4
---
# Broaden the dual-build collision corpus to the measured 41 coordinates

Carried over from March ticket 688 when BioMCP moved to the sdlc
factory. The body below is March's, unchanged; it was already written to
stand alone. Work products from any earlier attempt:

    /home/ian/workspace/planning/biomcp/artifacts/688-broaden-the-dual-build-collision-corpus-to-the-measured-41-coordinates
## Why

Ticket 687 shipped the build-inference runtime and one captured dual-build collision
(`chr10:g.87933119A>C` -> GRID1/rs1212585646 in GRCh37, PTEN/rs759485888 in GRCh38).
That single pair proves the *behavior*. It does not prove the *breadth*.

687's code step correctly aborted rather than invent provider payloads for a corpus the
repository never named. The coordinate list below is now measured, so the capture work is
unblocked. Method, so it can be re-run: query MyVariant for `chr10:87863113-87971930`
under `assembly=hg38`, take the substitutions, batch-POST the same IDs under
`assembly=hg19`, and keep the ones that resolve in both builds with a different rsID.

Measured 2026-08-07 against live MyVariant:

- 904 GRCh38 substitutions tested in the PTEN/GRID1 region
- 46 resolve in **both** builds
- **41 of those 46 return a genuinely different variant** — same coordinate string,
  different rsID, roughly 4.5% of the region

This is the silent-wrong-answer case. A caller passing a bare GRCh38 coordinate gets a
real, well-formed record for a different variant, with no error and nothing to notice.

## Scope

- Capture each coordinate below under both builds, **through the program** (point the
  source base URL at a recording proxy and run the real command; never hand-build the
  request — see 666).
- Register receipts and fixture routes for every capture.
- Land deterministic `spec/*.md` assertions over the corpus.
- Do not change committed 687 runtime behavior unless a real capture contradicts it.
  If one does, that is a finding: report it rather than editing the fixture to agree.

## Success Checklist

- [ ] Every coordinate below has a receipt-backed capture under both GRCh37 and GRCh38.
- [ ] Each one asserts `build_ambiguous: true` and lists both candidate identities.
- [ ] Each one returns the **preferred-build** record by default, and the other build's
      identity appears as the competing candidate.

      **Which build is preferred depends on whether 690 has landed.** Before 690 the default
      is GRCh37; after 690 it is GRCh38. Read the current behavior from
      `spec/entity/variant.md` rather than hardcoding an expectation from this ticket — it
      was written while GRCh37 was still preferred. Do not "fix" a corpus mismatch by
      editing runtime; if the corpus and the shipped default disagree, that is a sequencing
      question for the operator.
- [ ] A single assertion covers the whole corpus rather than 41 near-identical blocks.
- [ ] `make lint`, `make test`, and `make spec` pass.

## Corpus

| coordinate | GRCh37 rsID | GRCh38 rsID |
|---|---|---|
| `chr10:g.87865864C>T` | rs2131948317 | rs1440948756 |
| `chr10:g.87867015G>A` | rs1467902434 | rs577822594 |
| `chr10:g.87868266A>G` | rs2131950044 | rs1858503468 |
| `chr10:g.87869876C>T` | rs1000625745 | rs866903609 |
| `chr10:g.87869935T>C` | rs1207310255 | rs1038453644 |
| `chr10:g.87870681T>C` | rs1589374693 | rs1054105194 |
| `chr10:g.87874074C>T` | rs1844539015 | rs980904944 |
| `chr10:g.87879123A>G` | rs962324342 | rs933494472 |
| `chr10:g.87880198T>C` | rs915530974 | rs972047291 |
| `chr10:g.87882203C>T` | rs1844690556 | rs1400626542 |
| `chr10:g.87885960C>T` | rs1844750171 | rs1382442706 |
| `chr10:g.87886411C>T` | rs1483139583 | rs1052317502 |
| `chr10:g.87889241A>G` | rs1353526487 | rs1007379450 |
| `chr10:g.87889641A>G` | rs1361054747 | rs1024504550 |
| `chr10:g.87892320T>G` | rs562130542 | rs1313146820 |
| `chr10:g.87899090C>T` | rs1195252547 | rs1397616237 |
| `chr10:g.87900828T>C` | rs924217576 | rs1047824665 |
| `chr10:g.87910430T>A` | rs371781972 | rs989715172 |
| `chr10:g.87910810T>C` | rs72835292 | rs543306163 |
| `chr10:g.87911712C>T` | rs894323277 | rs1385750403 |
| `chr10:g.87914255G>T` | rs990849427 | rs535770733 |
| `chr10:g.87916405A>G` | rs904189456 | rs191540731 |
| `chr10:g.87920429T>C` | rs1281838176 | rs1859697942 |
| `chr10:g.87921963G>A` | rs138676836 | rs936724288 |
| `chr10:g.87922163A>G` | rs1845336336 | rs1859737795 |
| `chr10:g.87923392C>G` | rs566045743 | rs533697484 |
| `chr10:g.87925590A>G` | rs1589394378 | rs1259353611 |
| `chr10:g.87926283T>C` | rs1845401210 | rs1205348242 |
| `chr10:g.87929122T>G` | rs1845448280 | rs1272746033 |
| `chr10:g.87930033C>A` | rs1256454898 | rs1231887120 |
| `chr10:g.87930470A>G` | rs1845472977 | rs1267010561 |
| `chr10:g.87933402T>C` | rs973243379 | rs1179855179 |
| `chr10:g.87947657C>T` | rs1333193469 | rs200749330 |
| `chr10:g.87951454C>T` | rs1845804595 | rs917997127 |
| `chr10:g.87954957G>A` | rs879269090 | rs981916669 |
| `chr10:g.87957145C>T` | rs780471177 | rs1157771742 |
| `chr10:g.87957402C>T` | rs1445046730 | rs1209274124 |
| `chr10:g.87957862T>C` | rs1272565615 | rs2132276458 |
| `chr10:g.87959124G>A` | rs1589407747 | rs1409117005 |
| `chr10:g.87965429A>T` | rs1229753642 | rs2132290180 |
| `chr10:g.87968571C>T` | rs1379816103 | rs929158248 |
## Dependencies
- 690-make-grch38-the-preferred-build-for-bare-coordinates (sequencing: capture the corpus
  against the final default, not the interim one, so the captures do not need redoing)
- 687-remaining-infer-and-disambiguate-myvariant-genome-builds (runtime and the single
  proven collision must merge first)

## Notes
The five collisions that resolve in both builds but return the *same* variant are
deliberately excluded — they are indistinguishable to a caller and prove nothing.
