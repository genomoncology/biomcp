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
  if [[ -f "$tmpdir/biomcp" ]]; then
    bin_path="$tmpdir/biomcp"
  elif [[ -f "$tmpdir/bin/biomcp" ]]; then
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
  if [[ -f "$tmpdir/biomcp.exe" ]]; then
    bin_path="$tmpdir/biomcp.exe"
  else
    echo "Could not find biomcp.exe in archive" >&2
    exit 1
  fi
else
  echo "Unsupported archive format: $ASSET" >&2
  exit 1
fi

mkdir -p "$INSTALL_DIR"
chmod +x "$bin_path" || true
installed_bin="$INSTALL_DIR/$(basename "$bin_path")"
mv -f "$bin_path" "$installed_bin"

echo "Installed biomcp to $installed_bin"

if ! installed_version="$("$installed_bin" version 2>/dev/null | head -n 1)"; then
  echo "Install verification failed: $installed_bin version" >&2
  exit 1
fi
if [[ -n "$installed_version" ]]; then
  echo "Verified installation: $installed_version"
else
  echo "Verified installation: biomcp version returned successfully"
fi

if [[ "$INSTALL_DIR" == "$HOME/.local/bin" ]]; then
  if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    shell_name="$(basename "${SHELL:-}")"
    if [[ "$shell_name" == "zsh" ]]; then
      rc="$HOME/.zshrc"
      line='export PATH="$HOME/.local/bin:$PATH"'
      grep -Fqs "$line" "$rc" 2>/dev/null || printf "\n%s\n" "$line" >> "$rc"
      echo "Updated PATH in $rc"
    elif [[ "$shell_name" == "bash" ]]; then
      rc="$HOME/.bashrc"
      line='export PATH="$HOME/.local/bin:$PATH"'
      grep -Fqs "$line" "$rc" 2>/dev/null || printf "\n%s\n" "$line" >> "$rc"
      echo "Updated PATH in $rc"
    elif [[ "$shell_name" == "fish" ]]; then
      rc="$HOME/.config/fish/config.fish"
      mkdir -p "$(dirname "$rc")"
      line='set -gx PATH $HOME/.local/bin $PATH'
      grep -Fqs "$line" "$rc" 2>/dev/null || printf "\n%s\n" "$line" >> "$rc"
      echo "Updated PATH in $rc"
    else
      printf 'Add to PATH:\n  export PATH="$HOME/.local/bin:$PATH"\n' >&2
    fi
  fi
else
  echo "Ensure $INSTALL_DIR is on your PATH"
fi

printf "Verify:\\n  biomcp version\\n"
