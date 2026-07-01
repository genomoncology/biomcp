class Biomcp < Formula
  desc "Biomedical Model Context Protocol command-line interface"
  homepage "https://biomcp.org"
  version "__VERSION__"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/genomoncology/biomcp/releases/download/__TAG__/biomcp-darwin-arm64.tar.gz"
      sha256 "__DARWIN_ARM64_SHA256__"
    else
      url "https://github.com/genomoncology/biomcp/releases/download/__TAG__/biomcp-darwin-x86_64.tar.gz"
      sha256 "__DARWIN_X86_64_SHA256__"
    end
  end

  def install
    bin.install "biomcp"
  end

  test do
    system "#{bin}/biomcp", "--version"
  end
end
