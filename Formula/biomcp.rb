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
    bin.install_symlink "biomcp" => "biomcp-cli"
  end

  test do
    output = shell_output("#{bin}/biomcp --json version")
    assert_match version.to_s, output
    assert_match "__REVISION__", output
    expected_sha256 = Hardware::CPU.arm? ? "__DARWIN_ARM64_BINARY_SHA256__" : "__DARWIN_X86_64_BINARY_SHA256__"
    assert_equal expected_sha256, Digest::SHA256.file(bin/"biomcp").hexdigest
    assert_predicate bin/"biomcp-cli", :symlink?
  end
end
