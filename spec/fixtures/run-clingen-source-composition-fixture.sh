#!/usr/bin/env bash
# Compose the source-owned ClinGen fixture reporters without copying provider captures.
set -euo pipefail

root="$(cd "${1:-../..}" && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

bash "$root/spec/fixtures/run-clingen-erepo-fixture.sh" "$root" >"$work/erepo.json"
bash "$root/spec/fixtures/run-clingen-cspec-fixture.sh" "$root" >"$work/cspec.json"
bash "$root/spec/fixtures/run-variant-article-identity-fixture.sh" "$root" >"$work/identity.json"

jq -n \
  --slurpfile erepo "$work/erepo.json" \
  --slurpfile cspec "$work/cspec.json" \
  --slurpfile identity "$work/identity.json" \
  '{
    clingen_source_namespaces_are_isolated:
      ($erepo[0].healthy_exact_miss_is_empty_and_complete
       and $erepo[0].cli_and_mcp_have_same_contract
       and $cspec[0].capture_binds_requested_gene_and_selected_iri
       and $cspec[0].missing_capture_is_capture_unavailable
       and $identity[0].canonical_equivalence_is_additive
       and $identity[0].clingen_ldh.empty_coverage_preserves_candidates
       and $identity[0].outage_is_incomplete
       and $identity[0].frozen_positive_statuses.apc)
  }'
