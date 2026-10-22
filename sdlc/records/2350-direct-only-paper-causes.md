# Direct-only paper causes

Research commit `561cb47` supplies 56 search traces in `0041-direct-only-findings.tsv` and 27 observations in `0041-tool-issues.tsv`. The traces contain 42 distinct papers: 39 PMIDs and three PMC-only records. There are 43 case/paper pairs because PMID 42674815 appears in both the FANCL and ABRAXAS1 cases. The two separate PMC-only ABRAXAS1 records remain separate. A returned search row proves discovery, not supplied-allele identity.

The current-main probes below used `f7165ac3b6e2c25367f31f25393343313564b5e7`. The question was whether fields, aliases, dates, page size, ranking or source availability explain each observation. The investigation stops at an evidenced cause, an owning fix, or an explicit limit on what the observation proves. It adds no product implementation.

## Paper findings

M5 job j4346 repeated all nine distinct recorded Europe PMC queries and contrasted each with the same words and date bounds restricted to `TITLE_ABS`. All 42 candidate identities remained in their corresponding unrestricted replies. None occurred in the corresponding title/abstract control. The largest unrestricted result had 40 rows, so asking for 1000 instead of 50 does not explain any of these 42 source-level differences. These controls preserve the recorded aliases; they do not prove that every alias or paper identifies the supplied allele.

| Study query | Current unrestricted count | Title/abstract count | Difference retained by the control |
|---|---:|---:|---|
| c001/0006 BRCA1 P1749R/Pro1749Arg/coding/rsID union | 27 | 4 | Indexed field scope |
| c001/0015 extended BRCA1 alias union | 32 | 4 | Indexed field scope; broader aliases remain distinct |
| c002/0031 FANCL/long-name dated gene query | 16 | 4 | Indexed field scope; no frameshift constraint |
| c003/0004 CHEK2 G167R/Gly167Arg | 22 | 0 | Indexed field scope |
| c003/0007 same CHEK2 query with date bound | 1 | 0 | Indexed field scope; date preserved |
| c003/0016 CHEK2 coding/Gly210Arg/G210R | 40 | 2 | Indexed field scope; alias identity not inferred |
| c003/0039 expanded CHEK2 dated query | 1 | 0 | Indexed field scope; aliases and date preserved |
| c004/0016 CDKN2A/p16/INK4A alias union | 5 | 1 | Indexed field scope; aliases remain distinct |
| c005/0019 dated ABRAXAS1/FAM175A/CCDC98/ABRAXAS | 14 | 2 | Indexed field scope; no G39V constraint |

The table lists each distinct paper. Every row has current unrestricted membership and lacks corresponding title/abstract membership. The last column says what an equivalent BioMCP query should do without upgrading a literal match to verified identity. Query references identify the exact commands and provider strings in the study TSV and j4346 evidence.

| Paper | Case and direct call(s) | Cause and expected query behavior |
|---|---|---|
| 10486320 | c001/0006,0015 | Confirmed supplied-allele paper. Supported `-g BRCA1 -k Pro1749Arg --source europepmc` finds it; default variant union misses it. |
| 10811118 | c001/0006,0015 | Broader-field BRCA1 alias candidate. Lexical discovery may retain it as unverified; supplied-allele relevance remains unassessed. |
| 11016951 | c001/0006,0015 | Broader-field BRCA1 alias candidate. Lexical discovery may retain it as unverified; supplied-allele relevance remains unassessed. |
| 11389159 | c001/0006,0015 | Broader-field BRCA1 alias candidate. Lexical discovery may retain it as unverified; supplied-allele relevance remains unassessed. |
| 12707373 | c001/0015 | Broader-field BRCA1 alias candidate. Lexical discovery may retain it as unverified; supplied-allele relevance remains unassessed. |
| 18688287 | c001/0015 | Broader-field BRCA1 alias candidate. Lexical discovery may retain it as unverified; supplied-allele relevance remains unassessed. |
| 19509300 | c001/0006,0015 | Broader-field BRCA1 alias candidate. Lexical discovery may retain it as unverified; supplied-allele relevance remains unassessed. |
| 20516115 | c001/0006,0015 | Recovered by current default union in j4307. Historical direct-only classification no longer describes that command. |
| 22262852 | c001/0006,0015 | Broader-field BRCA1 alias candidate. Lexical discovery may retain it as unverified; supplied-allele relevance remains unassessed. |
| 31788995 | c001/0006,0015 | Confirmed supplied-allele paper. Supported `-g BRCA1 -k c.5246C>G --source europepmc` finds it; default variant union misses it. |
| 37377590 | c001/0015 | Broader-field BRCA1 alias candidate. Lexical discovery may retain it as unverified; supplied-allele relevance remains unassessed. |
| 38552003 | c001/0006,0015 | Broader-field BRCA1 alias candidate. Lexical discovery may retain it as unverified; supplied-allele relevance remains unassessed. |
| 40074693 | c001/0015 | Broader-field BRCA1 alias candidate. Lexical discovery may retain it as unverified; supplied-allele relevance remains unassessed. |
| 40864753 | c001/0015 | Broader-field BRCA1 alias candidate. Lexical discovery may retain it as unverified; supplied-allele relevance remains unassessed. |
| 9159158 | c001/0006,0015 | Broader-field BRCA1 alias candidate. Lexical discovery may retain it as unverified; supplied-allele relevance remains unassessed. |
| 42636061 | c002/0031 | Dated literal/long-name gene candidate, not an equivalent typed annotation-index or T367fs query. Keep the explicit filter explanation in 2348. |
| 42674815 | c002/0031; c005/0019 | Both dated queries search gene mentions across indexed fields. Neither constrains FANCL T367fs or ABRAXAS1 G39V; no variant-recall failure follows. |
| 42753176 | c002/0031 | Dated literal/long-name gene candidate, not an equivalent typed annotation-index or T367fs query. Keep the explicit filter explanation in 2348. |
| 42763538 | c002/0031 | Dated literal/long-name gene candidate, not an equivalent typed annotation-index or T367fs query. Keep the explicit filter explanation in 2348. |
| 42884094 | c002/0031 | Dated literal/long-name gene candidate, not an equivalent typed annotation-index or T367fs query. Keep the explicit filter explanation in 2348. |
| 12533788 | c003/0004 | Broader-field CHEK2 exact-name candidate. Discovery may retain it as unverified; supplied-allele relevance remains unassessed. |
| 18547414 | c003/0016 | Broader-field alternate coding/G210R candidate. No supplied G167R equivalence is inferred; expansion needs the existing authoritative allele rule. |
| 27504877 | c003/0016 | Broader-field alternate coding/G210R candidate. No supplied G167R equivalence is inferred; expansion needs the existing authoritative allele rule. |
| 28281021 | c003/0016 | Broader-field alternate coding/G210R candidate. No supplied G167R equivalence is inferred; expansion needs the existing authoritative allele rule. |
| 33111630 | c003/0016 | Broader-field alternate coding/G210R candidate. No supplied G167R equivalence is inferred; expansion needs the existing authoritative allele rule. |
| 34009545 | c003/0016 | Broader-field alternate coding/G210R candidate. No supplied G167R equivalence is inferred; expansion needs the existing authoritative allele rule. |
| 34672684 | c003/0016 | Broader-field alternate coding/G210R candidate. No supplied G167R equivalence is inferred; expansion needs the existing authoritative allele rule. |
| 35132179 | c003/0004 | Broader-field CHEK2 exact-name candidate. Discovery may retain it as unverified; supplied-allele relevance remains unassessed. |
| 40943312 | c003/0004,0016 | Broader-field CHEK2 exact-name candidate. Discovery may retain it as unverified; supplied-allele relevance remains unassessed. |
| 41646265 | c003/0016 | Broader-field alternate coding/G210R candidate. No supplied G167R equivalence is inferred; expansion needs the existing authoritative allele rule. |
| PMC13665518 | c003/0004,0007,0039 | Broader-field CHEK2 exact-name candidate. Discovery may retain it as unverified; supplied-allele relevance remains unassessed. |
| 24294354 | c004/0016 | Broader-field CDKN2A alias candidate. No title/abstract match; full-text lexical discovery stays unverified without identity evidence. |
| 42693354 | c005/0019 | Dated broad-gene/alias candidate. The query contains no G39V constraint, so it cannot establish a missing variant-specific result. |
| 42702687 | c005/0019 | Dated broad-gene/alias candidate. The query contains no G39V constraint, so it cannot establish a missing variant-specific result. |
| 42741651 | c005/0019 | Dated broad-gene/alias candidate. The query contains no G39V constraint, so it cannot establish a missing variant-specific result. |
| 42792985 | c005/0019 | Dated broad-gene/alias candidate. The query contains no G39V constraint, so it cannot establish a missing variant-specific result. |
| 42793752 | c005/0019 | Dated broad-gene/alias candidate. The query contains no G39V constraint, so it cannot establish a missing variant-specific result. |
| 42799484 | c005/0019 | Dated broad-gene/alias candidate. The query contains no G39V constraint, so it cannot establish a missing variant-specific result. |
| 42812597 | c005/0019 | Dated broad-gene/alias candidate. The query contains no G39V constraint, so it cannot establish a missing variant-specific result. |
| 42882778 | c005/0019 | Dated broad-gene/alias candidate. The query contains no G39V constraint, so it cannot establish a missing variant-specific result. |
| PMC13661460 | c005/0019 | Dated broad-gene/alias candidate. The query contains no G39V constraint, so it cannot establish a missing variant-specific result. |
| PMC13666186 | c005/0019 | Dated broad-gene/alias candidate. The query contains no G39V constraint, so it cannot establish a missing variant-specific result. |

## Confirmed shared recall cause

Current default variant union misses two clinically confirmed papers that its supported lower-level article search can find. Job j4307 ran `biomcp --json variant articles 'BRCA1 P1749R' --limit 50 --debug-plan`: 35 results, 63 pre-dedup candidates, incomplete/truncated, both expert papers absent even from `candidate_trace`. It reached the invocation deadline with work allowances remaining. PMID 20516115 was recovered, so the saved omission list must not be copied as a current failure list.

Job j4346 then ran the supported commands below with `--json --no-cache`, limit 50 and `--debug-plan`:

```
biomcp search article -g BRCA1 -k P1749R --source europepmc --limit 50
biomcp search article -g BRCA1 -k Pro1749Arg --source europepmc --limit 50
biomcp search article -g BRCA1 -k 'c.5246C>G' --source europepmc --limit 50
biomcp search article -g BRCA1 -k rs80357462 --source europepmc --limit 50
```

The Pro1749Arg command returned PMID 10486320. The coding command returned 31788995 and 32257056. Both commands completed. No ranking or page-size explanation can account for the experts' absence from the default union's candidate pool. Its strict Europe PMC queries restrict title/abstract. `VariantArticleExecutionContext::includes_boolean_alias_sources` in `src/entities/article/variant_search.rs` suppresses Europe PMC lexical work for aliases covered by Boolean strict queries; `acquire_federated_article_rows_with_context` in `src/entities/article/search.rs` applies that decision. Title/abstract coverage does not cover an all-indexed-field search. The normalized overflow aliases also appear in strict title/abstract plans, not an equivalent all-field recovery. The deadline explains why later work stopped; a disclosed deadline alone is not a separate bug or authority to raise the guard.

Ticket 2355 owns the shared field-coverage repair. One fix should recover compatible all-indexed-field alias candidates within the existing provider/deadline rules. One outside-in regression should recover both actual expert papers from the default union while preserving source/alias provenance, unverified candidate status, typed-annotation identity rules and truthful incomplete replies. The other broader-field matches in the paper table are candidates, not additional clinically confirmed same-allele papers.

### Clinical adjudication

The clinical reviewer confirmed both expert papers with high confidence from their primary full text and authoritative allele records:

- [Gayther 1999, PMID 10486320](https://pmc.ncbi.nlm.nih.gov/articles/PMC1288234/) reports the variant in a study family. Its Results and Table 1 pair BRCA1 Pro1749Arg with historical nucleotide 5365C>G. [ClinVar VCV000055479.6](https://www.ncbi.nlm.nih.gov/clinvar/variation/55479/) maps U14680.1:n.5365C>G to NM_007294.4:c.5246C>G and NP_009225.1:p.Pro1749Arg.
- [Lai 2020, PMID 31788995, Table 3](https://www.ebi.ac.uk/europepmc/webservices/rest/PMC6918895/fullTextXML) directly pairs BRCA1 c.5246C>G and p.Pro1749Arg in a sequenced study participant.
- [ClinGen CA003387](https://reg.genome.network/allele/CA003387) supplies the current allele bridge: GRCh38 NC_000017.11:g.43057083G>C and GRCh37 NC_000017.10:g.41209100G>C. The historical papers do not themselves specify versioned transcript/build metadata. The curated mapping supplies that context; the review does not attribute it retroactively to the papers.

These are substantive allele observations. Neither conclusion rests on a gene token, a protein string alone, or a reference-list mention.

## All 27 tool observations

Row numbers below count observations, not physical TSV lines; physical line equals row plus one. Each case/call identifies its exact command and saved evidence paths in `0041-tool-issues.tsv`. Existing fix evidence establishes its stated behavior; it does not assert that an old live provider response repeats today.

| Row | Case/call | Current disposition and evidence |
|---:|---|---|
| 1 | c001/0003 | Coding/route recovery belongs to 2327 and 2337. j4307 plans the coding forms and recovers 20516115. The confirmed remaining all-field expert-paper cause is identified above. |
| 2 | c001/0008 | Metadata fallback has 2269/2329 in-band status and 2336 bounded retry coverage. A provider HTTP 400 alone proves no malformed product request. Without `--date-to`, later issue dates do not violate the requested lower bound. The guarded viewer's byte pagination belongs to the study harness. |
| 3 | c001/0013 | The direct ESearch query broadened coding HGVS to generic All Fields text and was excluded by research. It is not a BioMCP request or valid variant-recall proof. |
| 4 | c001/0018 | Harness prelaunch rejection launched zero BioMCP children. No product defect. |
| 5 | c001/0032 | Reproduced on current main in j4335: initial fulltext retrieval reports a 65243-byte, 336-line PMC HTML conversion and path; immediate same-cache ranges 1:60 and 61:100 both return not_found. The job did not independently stat the path before the first range. 2354 owns the returned-conversion repair; diagnose reuse, not article-wide absence. |
| 6 | c001/0034 | Official PMC XML returns a publisher-restricted abstract without body. Source-local availability and access rules apply; 2253 documents the ladder. No restriction bypass or provider-wide outage follows. |
| 7 | c001/0009 | Provider entity highlights can name different substitutions, and the recorded BRCA1 sentence has an ACACA annotation. Keep literal naming unverified under 2139. Zero assets and one missing abstract are availability observations. No gene-token fallback or clinical identity conclusion. |
| 8 | c002/0003 | 2334 owns FANCL p.T367fs parser behavior; exact outside-in cases passed in j2213, with j2191 landing proof. Short frameshift identity remains partial. |
| 9 | c002/0004 | 2334 owns the exact RefSeq duplication spelling, build/range controls and parsed-but-absent distinction. No inferred genomic duplication follows from protein-only input. |
| 10 | c002/0007 | j4335 repeats the exact dated FANCL command. PubTator still reports timeout with retry_attempted=true; five rows from other sources remain, exit 0, incomplete reply. This keyword-free request is not evidence of the CHEK2 OR-precedence cause. 2353 owns bounded diagnosis and completion of this persistent dated search. The observation establishes explicit source degradation, not its parser/request cause or authority to raise the guard. |
| 11 | c002/0016 | 2327/2337 cover coding aliases, attempted routes and finite remaining-work disclosure. A historical capped request does not establish a current missing route. |
| 12 | c002/0014 | 2348 explains the empty typed GENE_PROTEIN annotation-index result and offers a literal alternative in JSON/Markdown. j4152 RED to j4204 PASS. The dated direct query changes indexed fields and includes a long name; the paper table retains its unverified candidates. |
| 13 | c003/0003 | 2337 covers recovery and honest remaining caps. Its j2227 RED to j2484 controls recover the missing authored paper. Ambiguous CHEK2 identity still prevents a verified/completely resolved claim. Broader-field study candidates remain distinct above. |
| 14 | c003/0006 | 2346 owns the demonstrated PubTator conjunction defect: j4159 contrasts the malformed 17471-hit request with the correctly grouped 15-hit request; j4205 proves the affected repair. Keep the existing guard. |
| 15 | c003/0009 | j4335's authored local-source command accepts NC_000022.11:g.28725070C>T and requests `/variant/chr22:g.28725070C%3ET`. Its deliberately unavailable resolver then yields source_unavailable, not invalid_argument. No clinical protein/genomic equivalence follows. |
| 16 | c003/0014 | Same exact query and cause as row 14; no second defect or fix ticket. |
| 17 | c003/0027 | The source ladder legitimately reaches PMC HTML after XML sources lack bodies. Source-local reasons do not claim a global outage. j4148 rechecks the two named PMIDs; 2253/2269 own explanation. |
| 18 | c003/0033 | 2347's current-main j4148 did not reproduce an unreadable returned conversion: 22419737 returned an existing 42475-byte/195-line conversion and read successfully; 12533788 initially returned no path during a bot challenge and later read successfully. This does not settle the distinct row 5 input, which j4335 reproduces. |
| 19 | c003/0037 | Incomplete/ambiguous union replies intentionally retain unknown total and has_more even when the requested offset yields zero. This is not proof of infinite pagination. 2337 owns route recovery; different direct queries are not equivalent exhaustion proof. |
| 20 | c004/0003 | 2327/2337 cover aliases and remaining routes. Missing PubTator entity stays unresolved; no fabricated fallback identity. The pm viewer's origin timeout belongs to infrastructure. |
| 21 | c004/0026 | A missing provider body and failed open-PDF download are access outcomes. 2183 distinguishes download failure from unusable text; 2253 documents it. More granular URL/status output is not an established retrieval bug or approved new interface requirement. |
| 22 | c004/0021 | Body reading succeeded. Supplement uncertainty does not invalidate it. 2349 owns the actual retained empty Europe PMC XML case: j4166/j4174 RED to j4203 PASS, independent j4210 PASS. It preserves source-listed-none versus universal absence and names the browser alternative. |
| 23 | c004/0003 | 2345's j4144 and the clinical ruling establish the GRCh37/GRCh38 pair as the same allele. A bare serialized coordinate omits context but does not justify shifting normalization by one. The authoritative ruling records the build-specific forms. |
| 24 | c005/0003 | 2327/2337 own alias recovery, including the authored ABRAXAS1 alternate-protein control in j2484. The direct dated gene-only candidates do not contain a G39V constraint and cannot establish missing supplied-variant papers. |
| 25 | c005/0033 | Harness prelaunch rejection, zero BioMCP children. No product provider failure. |
| 26 | c005/0033 | Corrected duplicate of row 25; call 0035 also launched zero children. Direct call 0036 succeeding proves no BioMCP child ran. |
| 27 | c005/0011 | Different direct one-/zero-result queries do not exhaust the variant union. Unknown total/has_more is intentional when work or identity is incomplete. 2337 owns recovery. The paper-level comparison above supplies the narrower field-scope explanation. |

## Reproduction evidence and limits

The raw study replies remain read-only in research's protected store at commit 561cb47. This record uses the TSV identifiers and preserves all trace links through their case/call IDs. The two TSV working copies were checked against their 561cb47 Git blobs before the final inventory. New raw outputs are archived on Yellow under `data/biomcp/ticket-evidence/2350/`. The `recipes/` directory holds the exact three probe drivers, source-query inputs and three pm job logs:

- `first-probe/`: j4307 default BRCA1 union JSON and stderr. Its two initial article probes incorrectly requested limit 100 and returned invalid_argument before provider work; they establish no search behavior. j4346 replaces them with supported limit-50 single-alias commands.
- `followup-probe/`: j4335 exact FANCL dated request, fulltext retrieval/ranges and local-source RefSeq parse control. Its complex keyword field comparison is not a raw provider-query comparison: BioMCP's provider-neutral keyword escaping changes parentheses/quotes. Those two keyword attempts are excluded from the field finding.
- `field-comparison/`: j4346's nine unrestricted/title-abstract reply pairs, comparison-summary.json and four supported BRCA1 CLI command outputs. Provider comparisons keep the recorded dates and aliases. The dated FANCL count changed from the saved 15 to the current 16; source indexing can change.

The three passing job exits mean their evidence recipes completed. They do not mean every product command passed: the fulltext range failures and deliberately unavailable local resolver are explicitly retained. No standalone Semantic Scholar qualification check or fixture refresh ran. No clinical identity claim was made for the other 40 papers. No product files, provider limits or shared rules changed.
