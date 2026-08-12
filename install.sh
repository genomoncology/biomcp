#!/usr/bin/env bash
set -euo pipefail

REPO="${BIOMCP_GITHUB_REPO:-genomoncology/biomcp}"
INSTALL_DIR="${BIOMCP_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${BIOMCP_VERSION:-latest}"

usage() {
  cat <<'EOF'
Usage: install.sh [--version <tag>] [--help]

Options:
  -V, --version  Install a specific release version (e.g., 0.4.1 or v0.4.1)
  -h, --help     Show this help text
EOF
}

download() {
  local url="$1"
  local dest="$2"

  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$dest"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$dest" "$url"
  else
    echo "curl or wget is required to download biomcp" >&2
    return 1
  fi
}

download_checksum() {
  local url="$1"
  local dest="$2"

  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$dest"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$dest" "$url"
  else
    echo "curl or wget is required to download biomcp" >&2
    return 1
  fi
}

compute_sha256() {
  local file="$1"

  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" | awk '{print tolower($1)}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$file" | awk '{print tolower($1)}'
  elif command -v openssl >/dev/null 2>&1; then
    openssl dgst -sha256 "$file" | awk '{print tolower($NF)}'
  else
    echo "No SHA-256 tool found; install sha256sum, shasum, or openssl." >&2
    return 1
  fi
}

verify_checksum() {
  local archive="$1"
  local checksum_file="$2"
  local asset_name
  local expected=""
  local actual
  local line
  local records=0

  asset_name="$(basename "$archive")"
  while IFS= read -r line || [[ -n "$line" ]]; do
    line="${line%$'\r'}"
    [[ "$line" =~ ^[[:space:]]*$ ]] && continue
    records=$((records + 1))
    if [[ $records -ne 1 ]]; then
      echo "Checksum file must contain exactly one record." >&2
      return 1
    fi

    if [[ "$line" =~ ^([[:xdigit:]]{64})$ ]]; then
      expected="${BASH_REMATCH[1],,}"
    elif [[ "$line" =~ ^([[:xdigit:]]{64})[[:space:]]+\*?([^[:space:]]+)$ ]] && [[ "${BASH_REMATCH[2]}" == "$asset_name" ]]; then
      expected="${BASH_REMATCH[1],,}"
    else
      echo "Checksum file is invalid for ${asset_name}." >&2
      return 1
    fi
  done < "$checksum_file"

  if [[ $records -ne 1 ]]; then
    echo "Checksum file must contain exactly one record." >&2
    return 1
  fi

  if ! actual="$(compute_sha256 "$archive")" || [[ ! "$actual" =~ ^[0-9a-f]{64}$ ]]; then
    echo "Could not compute SHA-256 for ${asset_name}; refusing unverified installation." >&2
    return 1
  fi

  if [[ "$actual" != "$expected" ]]; then
    echo "Checksum verification failed for ${asset_name}." >&2
    return 1
  fi

  echo "Checksum verified."
}

json_escape() {
  local value="$1"
  value="${value//\\/\\\\}"
  value="${value//\"/\\\"}"
  printf '%s' "$value"
}

sync_path() {
  sync -f "$1" 2>/dev/null || sync
}

receipt_value() {
  local key="$1" file="$2" line
  while IFS= read -r line; do
    if [[ "$line" =~ ^[[:space:]]*\"${key}\"[[:space:]]*:[[:space:]]*\"([^\"]*)\" ]]; then
      printf '%s' "${BASH_REMATCH[1]}"
      return 0
    fi
  done < "$file"
  return 1
}

receipt_is_true() {
  local key="$1" file="$2" line
  while IFS= read -r line; do
    [[ "$line" =~ ^[[:space:]]*\"${key}\"[[:space:]]*:[[:space:]]*true ]] && return 0
  done < "$file"
  return 1
}

write_receipt() {
  local receipt="$1" state="$2" version="$3" sha="$4"
  local transaction_nonce="${5:-}" old_version="${6:-}" old_sha="${7:-}"
  local new_version="${8:-}" new_sha="${9:-}" staged_receipt
  staged_receipt="$(mktemp "$INSTALL_DIR/.biomcp.install.json.XXXXXX")"
  receipt_stage_path="$staged_receipt"
  chmod 600 "$staged_receipt"
  {
    printf '{\n'
    printf '  "schema_version": 1,\n'
    printf '  "installer": "biomcp-standalone-installer",\n'
    printf '  "state": "%s",\n' "$state"
    printf '  "executable_path": "%s",\n' "$(json_escape "$installed_bin")"
    printf '  "version": "%s",\n' "$(json_escape "$version")"
    printf '  "sha256": "%s"' "$sha"
    if [[ "$state" == "pending" ]]; then
      printf ',\n  "transaction_nonce": "%s",\n' "$(json_escape "$transaction_nonce")"
      if [[ -n "$old_sha" ]]; then
        printf '  "old_version": "%s",\n' "$(json_escape "$old_version")"
        printf '  "old_sha256": "%s",\n' "$old_sha"
      else
        printf '  "old_absent": true,\n'
      fi
      printf '  "new_version": "%s",\n' "$(json_escape "$new_version")"
      printf '  "new_sha256": "%s"' "$new_sha"
    fi
    printf '\n}\n'
  } > "$staged_receipt"
  sync_path "$staged_receipt"
  mv "$staged_receipt" "$receipt"
  receipt_stage_path=""
  sync_path "$INSTALL_DIR"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -V|--version)
      if [[ $# -lt 2 ]]; then
        echo "--version requires a value" >&2
        usage >&2
        exit 1
      fi
      VERSION="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux) OS_ID="linux" ;;
  Darwin) OS_ID="darwin" ;;
  MINGW*|MSYS*|CYGWIN*) OS_ID="windows" ;;
  *)
    echo "Unsupported OS: $OS" >&2
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64|amd64) ARCH_ID="x86_64" ;;
  arm64|aarch64) ARCH_ID="arm64" ;;
  *)
    echo "Unsupported architecture: $ARCH" >&2
    exit 1
    ;;
esac

ASSET=""
case "$OS_ID/$ARCH_ID" in
  linux/x86_64) ASSET="biomcp-linux-x86_64.tar.gz" ;;
  linux/arm64) ASSET="biomcp-linux-arm64.tar.gz" ;;
  darwin/x86_64) ASSET="biomcp-darwin-x86_64.tar.gz" ;;
  darwin/arm64) ASSET="biomcp-darwin-arm64.tar.gz" ;;
  windows/x86_64) ASSET="biomcp-windows-x86_64.zip" ;;
  *)
    echo "Unsupported platform: $OS_ID $ARCH_ID" >&2
    exit 1
    ;;
esac

if [[ "$VERSION" == "latest" ]]; then
  # Resolve to the most recent release that has our platform binary.
  # A newly-created release may not have assets yet (builds take minutes),
  # so we skip releases without the required asset file.
  RESOLVED_TAG=""
  api_url="https://api.github.com/repos/${REPO}/releases"
  if command -v jq >/dev/null 2>&1 && releases_json="$(curl -fsSL "$api_url" 2>/dev/null)"; then
    RESOLVED_TAG="$(printf '%s' "$releases_json" | \
      jq -r --arg asset "$ASSET" \
        '[.[] | select(.draft==false and .prerelease==false) | select(.assets[]?.name == $asset)][0].tag_name // empty' 2>/dev/null)" || true
  fi
  if [[ -z "$RESOLVED_TAG" ]]; then
    # API unavailable or no release with assets — fall back to GitHub redirect
    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
  else
    echo "Resolved latest release with assets: ${RESOLVED_TAG}"
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${RESOLVED_TAG}/${ASSET}"
  fi
else
  TAG="${VERSION#v}"
  DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${TAG}/${ASSET}"
fi

tmpdir="$(mktemp -d)"
cleanup() { rm -rf "$tmpdir"; }
trap cleanup EXIT

echo "Downloading ${ASSET} from ${REPO} (version: ${VERSION})..."

archive_path="$tmpdir/$ASSET"
download "$DOWNLOAD_URL" "$archive_path"

checksum_path="$archive_path.sha256"
if ! download_checksum "${DOWNLOAD_URL}.sha256" "$checksum_path"; then
  echo "Could not download release checksum; refusing unverified installation." >&2
  exit 1
fi

echo "Verifying checksum..."
verify_checksum "$archive_path" "$checksum_path"

bin_path=""
if [[ "$ASSET" == *.tar.gz ]]; then
  tar -xzf "$archive_path" -C "$tmpdir"
  if [[ -f "$tmpdir/biomcp" && ! -L "$tmpdir/biomcp" ]]; then
    bin_path="$tmpdir/biomcp"
  elif [[ -f "$tmpdir/bin/biomcp" && ! -L "$tmpdir/bin/biomcp" ]]; then
    bin_path="$tmpdir/bin/biomcp"
  else
    echo "Could not find biomcp binary in archive" >&2
    exit 1
  fi
elif [[ "$ASSET" == *.zip ]]; then
  if ! command -v unzip >/dev/null 2>&1; then
    echo "unzip is required to install on Windows shells" >&2
    exit 1
  fi
  unzip -q "$archive_path" -d "$tmpdir"
  if [[ -f "$tmpdir/biomcp.exe" && ! -L "$tmpdir/biomcp.exe" ]]; then
    bin_path="$tmpdir/biomcp.exe"
  else
    echo "Could not find biomcp.exe in archive" >&2
    exit 1
  fi
else
  echo "Unsupported archive format: $ASSET" >&2
  exit 1
fi

if [[ -L "$INSTALL_DIR" ]]; then
  echo "Refusing symbolic-link install directory: $INSTALL_DIR" >&2
  exit 1
fi
mkdir -p "$INSTALL_DIR"
INSTALL_DIR="$(cd "$INSTALL_DIR" && pwd -P)"
installed_bin="$INSTALL_DIR/$(basename "$bin_path")"
receipt="$INSTALL_DIR/biomcp.install.json"
if [[ -L "$installed_bin" || -L "$receipt" ]]; then
  echo "Refusing symbolic-link installation path." >&2
  exit 1
fi

if [[ -f "$receipt" ]] && [[ "$(receipt_value state "$receipt" || true)" == "pending" ]]; then
  current_sha=""
  [[ -f "$installed_bin" ]] && current_sha="$(compute_sha256 "$installed_bin" || true)"
  pending_old="$(receipt_value old_sha256 "$receipt" || true)"
  pending_new="$(receipt_value new_sha256 "$receipt" || true)"
  pending_installer="$(receipt_value installer "$receipt" || true)"
  pending_path="$(receipt_value executable_path "$receipt" || true)"
  if [[ "$pending_installer" != "biomcp-standalone-installer" || "$pending_path" != "$installed_bin" || ! "$pending_new" =~ ^[0-9a-f]{64}$ ]]; then
    echo "Pending install receipt has an invalid identity; refusing to continue." >&2
    exit 1
  fi
  if [[ -z "$current_sha" ]] && receipt_is_true old_absent "$receipt"; then
    :
  elif [[ -z "$current_sha" || ( "$current_sha" != "$pending_old" && "$current_sha" != "$pending_new" ) ]]; then
    echo "Pending install receipt does not match the installed binary; refusing to continue." >&2
    exit 1
  fi
fi

stage_path="$(mktemp "$INSTALL_DIR/.biomcp-stage.XXXXXX")"
cleanup_stage() {
  if [[ -n "${stage_path:-}" && -e "$stage_path" ]]; then
    rm -f "$stage_path"
  fi
  if [[ -n "${receipt_stage_path:-}" && -e "$receipt_stage_path" ]]; then
    rm -f "$receipt_stage_path"
  fi
  return 0
}
trap 'cleanup_stage; cleanup' EXIT
cp "$bin_path" "$stage_path"
chmod 755 "$stage_path"
sync_path "$stage_path"

if ! installed_version="$("$stage_path" version 2>/dev/null | head -n 1)"; then
  echo "Install verification failed before replacement: staged biomcp version" >&2
  exit 1
fi
reported_version="${installed_version##* }"
requested_version="${VERSION#v}"
if [[ "$VERSION" != "latest" && "${reported_version#v}" != "$requested_version" ]]; then
  echo "Install verification failed: requested $requested_version but staged binary reported $reported_version" >&2
  exit 1
fi
new_sha="$(compute_sha256 "$stage_path")"
old_sha=""
old_version=""
if [[ -f "$installed_bin" ]]; then
  old_sha="$(compute_sha256 "$installed_bin")"
  old_output="$("$installed_bin" version 2>/dev/null | head -n 1 || true)"
  old_version="${old_output##* }"
fi
transaction_nonce="${stage_path##*.biomcp-stage.}"
write_receipt "$receipt" pending "$old_version" "$old_sha" "$transaction_nonce" "$old_version" "$old_sha" "$reported_version" "$new_sha"
mv "$stage_path" "$installed_bin"
sync_path "$INSTALL_DIR"
write_receipt "$receipt" installed "$reported_version" "$new_sha"

echo "Installed biomcp to $installed_bin"
echo "Verified installation: $installed_version"

if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
  printf 'Add BioMCP to PATH:\n  export PATH="%s:$PATH"\n' "$INSTALL_DIR" >&2
fi

printf "Verify:\\n  biomcp version\\n"
