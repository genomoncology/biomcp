#!/usr/bin/env bash
set -euo pipefail

manifest="${1:?candidate manifest is required}"
candidate_root="${2:?candidate root is required}"
inventory="${3:?promotion inventory is required}"
tap_repo="${BIOMCP_HOMEBREW_TAP_REPO:-genomoncology/homebrew-biomcp}"
repo="${GITHUB_REPOSITORY:-genomoncology/biomcp}"

version="$(jq -er .version "$manifest")"
source_sha="$(jq -er .source_sha "$manifest")"
tag="v$version"
notes_file="${RUNNER_TEMP:?}/release-notes-$version.md"

jq -e '.manual_windows_desktop_smoke.result == "passed"
  and (.updater_result | type == "string" and length > 0)
  and (.updater_transition | type == "object")
  and (.prior_public_release.tag_name | type == "string" and length > 1)
  and (.public_release_inventory_sha256 | test("^[0-9a-f]{64}$"))' \
  "$inventory" >/dev/null
python3 release/release_notes.py --changelog CHANGELOG.md --version "$version" \
  --output "$notes_file"

existing_ref="$(git ls-remote --tags origin "refs/tags/$tag^{}" | cut -f1)"
if [[ -z "$existing_ref" ]]; then
  existing_ref="$(git ls-remote --tags origin "refs/tags/$tag" | cut -f1)"
fi
if [[ -n "$existing_ref" && "$existing_ref" != "$source_sha" ]]; then
  printf 'release tag conflict: %s points at %s\n' "$tag" "$existing_ref" >&2
  exit 2
fi
if [[ -z "$existing_ref" ]]; then
  git tag "$tag" "$source_sha"
  git push origin "refs/tags/$tag"
fi
if ! gh release view "$tag" --repo "$repo" >/dev/null 2>&1; then
  gh release create "$tag" --repo "$repo" --verify-tag --latest=false \
    --title "BioMCP $version" --notes-file "$notes_file"
fi

publish_github_file() {
  local file="$1" name temporary remote_hash
  name="$(basename "$file")"
  temporary="$(mktemp)"
  if gh release download "$tag" --repo "$repo" --pattern "$name" --output "$temporary" \
    >/dev/null 2>&1; then
    remote_hash="$(sha256sum "$temporary" | cut -d' ' -f1)"
    rm -f "$temporary"
    if [[ "$remote_hash" != "$(sha256sum "$file" | cut -d' ' -f1)" ]]; then
      printf 'public GitHub asset conflict: %s\n' "$name" >&2
      exit 2
    fi
    return
  fi
  rm -f "$temporary"
  gh release upload "$tag" "$file" --repo "$repo"
}

while IFS= read -r artifact_id; do
  file="$candidate_root/$(jq -er --arg id "$artifact_id" '.files[$id]' "$inventory")"
  publish_github_file "$file"
  sidecar="${RUNNER_TEMP:?}/$(basename "$file").sha256"
  printf '%s  %s\n' "$(jq -er --arg id "$artifact_id" '.artifacts[$id].sha256' "$manifest")" \
    "$(basename "$file")" > "$sidecar"
  publish_github_file "$sidecar"
done < <(jq -er '.channels.github[]' "$inventory")

mapfile -t wheels < <(
  jq -er '.channels.pypi[]' "$inventory" | while read -r artifact_id; do
    printf '%s/%s\n' "$candidate_root" "$(jq -er --arg id "$artifact_id" '.files[$id]' "$inventory")"
  done
)
uv publish --check-url https://pypi.org/simple/biomcp-cli/ \
  --token "$BIOMCP_PYPI_TOKEN" "${wheels[@]}"

oci="$candidate_root/$(jq -er '.files["oci-index"]' "$inventory")"
skopeo copy --all --dest-creds "$GITHUB_ACTOR:$BIOMCP_GHCR_TOKEN" \
  "oci-archive:$oci" "docker://ghcr.io/genomoncology/biomcp:$version"

formula="$candidate_root/$(jq -er '.files["homebrew-formula"]' "$inventory")"
tap_dir="$(mktemp -d)"
trap 'rm -rf "$tap_dir"' EXIT
git clone -q "https://x-access-token:${BIOMCP_HOMEBREW_TAP_TOKEN}@github.com/${tap_repo}.git" "$tap_dir"
if git -C "$tap_dir" rev-parse --verify -q "refs/tags/$tag" >/dev/null; then
  remote_formula_hash="$(git -C "$tap_dir" show "refs/tags/$tag:Formula/biomcp.rb" | sha256sum | cut -d' ' -f1)"
  test "$remote_formula_hash" = "$(sha256sum "$formula" | cut -d' ' -f1)" || {
    printf 'public Homebrew formula conflict: %s\n' "$tag" >&2
    exit 2
  }
  formula_commit="$(git -C "$tap_dir" rev-list -n1 "$tag")"
else
  mkdir -p "$tap_dir/Formula"
  cp "$formula" "$tap_dir/Formula/biomcp.rb"
  git -C "$tap_dir" add Formula/biomcp.rb
  git -C "$tap_dir" -c user.name='BioMCP release' -c user.email='release@biomcp.org' \
    commit -q -m "BioMCP $version"
  formula_commit="$(git -C "$tap_dir" rev-parse HEAD)"
  git -C "$tap_dir" tag "$tag"
  git -C "$tap_dir" push origin "refs/tags/$tag"
fi
printf '%s\n' "$formula_commit" > formula-commit.txt
