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

download_optional_checksum() {
  local url="$1"
  local dest="$2"
  local status=""

  if command -v curl >/dev/null 2>&1; then
    status="$(curl -sSL -o "$dest" -w '%{http_code}' "$url")" || return 1
  elif command -v wget >/dev/null 2>&1; then
    # Wget writes HTTP status lines to stderr; use the final status after redirects.
    local headers
    headers="$(wget -qS -O "$dest" "$url" 2>&1 || true)"
    status="$(printf '%s\n' "$headers" | awk '/^  HTTP\// { code=$2 } END { print code }')"
  else
    echo "curl or wget is required to download biomcp" >&2
    return 1
  fi

  if [[ "$status" == "200" ]]; then
    return 0
  fi

  rm -f "$dest"
  if [[ "$status" == "404" ]]; then
    return 2
  fi

  if [[ -n "$status" ]]; then
    echo "Failed to download checksum file (HTTP $status): $url" >&2
  else
    echo "Failed to download checksum file: $url" >&2
  fi
  return 1
}

compute_sha256() {
  local file="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" | awk '{print tolower($1)}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$file" | awk '{print tolower($1)}'
  else
    return 1
  fi
}

verify_checksum() {
  local archive="$1"
  local checksum_file="$2"
  local expected
  local actual

  expected="$(awk 'NF {print tolower($1); exit}' "$checksum_file" | tr -d '\r')"
  if [[ ! "$expected" =~ ^[0-9a-f]{64}$ ]]; then
    echo "Checksum file is invalid: $checksum_file" >&2
    return 1
  fi

  if ! actual="$(compute_sha256 "$archive")"; then
    echo "Warning: no SHA256 tool available; skipping checksum verification." >&2
    return 0
  fi

  if [[ "$actual" != "$expected" ]]; then
    echo "Checksum verification failed for $(basename "$archive")" >&2
    echo "Expected: $expected" >&2
    echo "Actual:   $actual" >&2
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
  DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
else
  TAG="${VERSION#v}"
  DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${TAG}/${ASSET}"
fi

tmpdir="$(mktemp -d)"
cleanup() { rm -rf "$tmpdir"; }
trap cleanup EXIT

mkdir -p "$INSTALL_DIR"

echo "Downloading ${ASSET} from ${REPO} (version: ${VERSION})..."

archive_path="$tmpdir/$ASSET"
download "$DOWNLOAD_URL" "$archive_path"

checksum_path="$archive_path.sha256"
if download_optional_checksum "${DOWNLOAD_URL}.sha256" "$checksum_path"; then
  echo "Verifying checksum..."
  verify_checksum "$archive_path" "$checksum_path"
else
  checksum_status=$?
  if [[ $checksum_status -eq 2 ]]; then
    echo "Warning: checksum file not found for this release; continuing without checksum verification." >&2
  else
    echo "Checksum download failed; aborting installation." >&2
    exit 1
  fi
fi

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
