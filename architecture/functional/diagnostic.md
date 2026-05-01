# Diagnostic Functional Note

The `diagnostic` entity is a source-aware local-runtime surface over two
diagnostic bundles plus one opt-in live regulatory overlay:

- NCBI Genetic Testing Registry (GTR) for gene-centric genetic tests
- WHO Prequalified IVD for infectious-disease diagnostic products
- OpenFDA device 510(k) and PMA for optional U.S. regulatory status overlays

## Scope

- `search diagnostic --source <gtr|who-ivd|all> --gene|--disease|--type|--manufacturer`
- `get diagnostic <diagnostic_id> [genes|conditions|methods|regulatory|all]`
- `biomcp gtr sync`
- `biomcp who-ivd sync`
- full `biomcp health` readiness for the GTR and WHO IVD local bundles

Out of scope in this slice:

- a new `--source` flag on `get diagnostic`
- cross-entity diagnostic helper commands
- a full FDA device mirror, sync command, or background cache
- live GTR or WHO IVD API calls beyond local refresh
- persistent processed caches
- any third diagnostic source

## Source lifecycle

BioMCP treats both diagnostic sources as local-runtime inputs, parallel to EMA,
WHO Prequalification, and CDC CVX/MVX.

The GTR runtime root is `BIOMCP_GTR_DIR` or the default platform data
directory. A valid GTR root requires both:

- `test_version.gz`
- `test_condition_gene.txt`

Sync must validate both files before replacing either one. A partial refresh is
considered invalid because diagnostic search/detail joins both files.

The WHO IVD runtime root is `BIOMCP_WHO_IVD_DIR` or the default platform data
directory. A valid WHO IVD root requires:

- `who_ivd.csv`

WHO IVD refresh uses the WHO CSV header contract and replaces the local file
atomically only after the required headers are validated.

## Search contract

Diagnostic search is filter-only and conjunctive, with source-aware matching:

- GTR: `--gene` exact match over joined gene names, `--disease` minimum-length
  word/phrase boundary match over joined condition names, `--type` exact equality on GTR test type, and
  `--manufacturer` substring over manufacturer/lab labels
- WHO IVD: `--disease` minimum-length word/phrase boundary match over
  `Pathogen/Disease/Marker`, `--type` exact match over `Assay Format`, and
  `--manufacturer` substring over `Manufacturer name`

Disease filters must contain at least three alphanumeric characters. The
boundary match applies to the full phrase, so `breast cancer` matches
`Hereditary breast cancer panel`, while short or partial tokens such as `ma`
or `emia` do not act as broad substring scans.

Result ordering is deterministic: normalized test name ascending, then
accession ascending after the source-specific match sets are merged. Pagination
applies only after the global merge. Exact totals remain available for
single-source pages; mixed-source `--source all` pages do not claim an exact
combined total.

Explicit `--source who-ivd --gene ...` is invalid and should return a recovery
hint. The default `--source all` route keeps gene-only searches valid by
skipping the WHO IVD leg.

## Get contract

`get diagnostic <id>` always returns the summary card. Source resolution is
implicit from the identifier: GTR accession regex first, WHO IVD exact product
code lookup second.

Optional public sections are:

- `genes`
- `conditions`
- `methods`
- `regulatory`
- `all`

Section support is source-aware:

- GTR supports `genes`, `conditions`, `methods`, and `regulatory`
- WHO IVD supports `conditions` and `regulatory`

`all` expands only to the local source-native sections and intentionally
excludes `regulatory` because the FDA overlay is live and optional. JSON keeps
the same progressive-disclosure contract by omitting unrequested sections and
preserving requested empty sections as `[]`. WHO IVD cards add source-native
summary fields such as target/marker, regulatory version, and prequalification
year instead of forcing GTR-only detail labels.

The `regulatory` section queries OpenFDA device 510(k) and PMA endpoints
against a bounded set of source-native aliases derived from the resolved
diagnostic record. The base summary card still loads if OpenFDA fails; the
overlay degrades to an empty `regulatory` section instead of failing the whole
diagnostic lookup.

## Disease Diagnostic Pivot Contract

`get disease <name_or_id> diagnostics` is an opt-in only disease section. It is
excluded from default disease cards and excluded from `all` so normal disease
cards do not implicitly prepare or scan local diagnostic bundles. The shipped
surface stays inside `get disease`; it is not a helper command or a new mutable
source operation.

The pivot resolves the disease first, then builds its diagnostic query from
`disease_query_value()`: prefer the resolved disease name and fall back to the
resolved disease ID only when the name is empty. It passes only that disease
filter into diagnostic search with `source=All`, `limit=10`, and `offset=0`.
There is no hidden gene, manufacturer, type, ontology, or synonym filter.

The current semantic boundary is resolved-label matching plus minimum-length
word/phrase boundary matching. The diagnostic disease filter must contain at
least three alphanumeric characters. GTR matches source-native condition names;
WHO IVD matches `Pathogen/Disease/Marker`. No MONDO/OLS traversal, synonym
expansion, ancestor/descendant expansion, GTR condition-ID bridge, or alias
search is promised by this contract. A future replacement must preserve the
cap, row, detail, and error contracts here and prove better recall or ordering
with executable specs before replacing boundary matching.

The top hit is only the first row after source-specific filtering, global merge,
deterministic ordering by normalized diagnostic display name, accession
tiebreak, and pagination. It is not a relevance-ranked best match,
exact-disease boost, preferred-source result, or confidence-ranked row.

Disease diagnostic cards remain summary-sized. The disease pivot has a 10-row
cap, and the focused disease rendering tests enforce the 40 KB ceiling for the
rendered markdown card. When a single-source page reports a larger exact total, markdown
may say how many rows are shown out of that total. When a mixed GTR + WHO IVD
page contributes rows, exact combined totals are unknown, so a full first page
uses the "Showing first 10..." cap note rather than claiming "10 of N".

Rows are shared `DiagnosticSearchResult` search rows: accession, name, type,
manufacturer/lab, source label, genes, and conditions. Markdown `Genes` and
`Conditions` cells show at most five values with a `+N more` overflow marker;
JSON keeps the full arrays from `DiagnosticSearchResult`. Full genes,
conditions, methods, regulatory overlays, and source-native detail fields stay
behind the `get diagnostic <id>` detail boundary.

The pivot preserves distinct empty and unavailable states:

- true no-match diagnostics use `diagnostics = Some(Vec::new())`; markdown
  renders `No diagnostic tests found for this disease.` and JSON serializes an
  empty `diagnostics: []` array with no unavailable note
- local diagnostic data unavailable, source preparation failure, or search
  failure uses `diagnostics = None`; markdown renders the diagnostic-local-data
  unavailable note, and JSON omits `diagnostics` while including
  `diagnostics_note`
- broader zero-result recovery belongs to `search diagnostic`, which renders
  filter-adjustment guidance and `biomcp list diagnostic` suggestions

Dedupe is source-level dedupe only. GTR gene and condition lists are deduped
case-insensitively within the GTR source path, and GTR merged genes strip
`SYMBOL:description` suffixes before row output. WHO IVD contributes
`Pathogen/Disease/Marker` as the condition value and has no gene list. There is
no cross-source diagnostic-row dedupe, no cross-source condition normalization
layer, and no unified GTR/WHO record identity promise.

The pivot remains read-only and MCP-safe through the existing `get disease` and
`search diagnostic` command families. It does not add sync/helper commands,
mutable operations, new environment-variable semantics, or MCP access to local
diagnostic sync. Rendered follow-up commands are shell-quoted; the disease
diagnostics card points to a broader
`search diagnostic --disease ... --source all --limit 50` command without
concatenating unescaped source text into a shell command.

Executable proof now includes the public diagnostic spec in
`spec/entity/diagnostic.md` plus focused tests. The active spec/entity corpus
covers diagnostic and other shipped entities beyond the original gene,
variant, and article bootstrap canaries:

- `src/entities/diagnostic/search.rs::disease_phrase_matches_accepts_word_and_phrase_boundaries`,
  `src/entities/diagnostic/search.rs::disease_phrase_matches_rejects_partial_words_and_keeps_scanning`,
  `src/entities/diagnostic/search.rs::normalized_filters_reject_short_disease_filter`,
  `src/entities/diagnostic/mod.rs::search_page_rejects_short_disease_filter`,
  and
  `src/entities/diagnostic/mod.rs::search_page_disease_filter_requires_word_boundary`
  pin the minimum-length boundary-matching filter
- `src/entities/diagnostic/mod.rs::search_page_applies_conjunctive_filters_and_stable_ordering`
  pins merged deterministic ordering before pagination
- `src/entities/diagnostic/mod.rs::search_page_all_source_uses_unknown_total_when_both_sources_match`
  pins mixed-source unknown totals
- `src/entities/diagnostic/mod.rs::get_diagnostic_genes_returns_full_deduped_broad_panel_list`
  pins full detail lists behind `get diagnostic`
- `src/entities/disease/enrichment/tests.rs::disease_diagnostics_section_populates_from_who_fixture`
  and
  `src/entities/disease/enrichment/tests.rs::disease_diagnostics_unavailable_sets_note`
  pin the disease pivot rows and unavailable-data state
- `src/entities/disease/get/tests.rs::disease_parse_sections_all_keeps_diagnostics_opt_in`
  and
  `src/entities/disease/get/tests.rs::parse_sections_all_keeps_optional_sections_opt_in`
  pin `all` exclusion for diagnostics and adjacent optional sections
- `src/render/markdown/disease/tests/rendering.rs::disease_markdown_renders_diagnostics_note_then_shell_safe_search_command`
  pins cap-note rendering and shell-safe follow-up commands
- `src/render/markdown/diagnostic/tests.rs::diagnostic_search_rows_caps_genes_and_conditions_with_overflow_marker`
  pins five-value row display caps while JSON keeps full arrays

## MCP boundary

`search diagnostic` and `get diagnostic` remain MCP-safe because they stay
read-only. `biomcp gtr sync` and `biomcp who-ivd sync` remain CLI-only because
they mutate local runtime roots.
