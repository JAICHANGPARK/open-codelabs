class Oc < Formula
  desc "Open Codelabs command-line interface"
  homepage "https://github.com/JAICHANGPARK/open-codelabs"
  version "0.0.0"
  license "Apache-2.0"

  # Draft formula:
  # 1. Replace `version`
  # 2. Replace the URLs to point at a real `oc-v<version>` GitHub release tag
  # 3. Replace all SHA256 placeholders with published artifact checksums

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/JAICHANGPARK/open-codelabs/releases/download/oc-v0.0.0/oc-0.0.0-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_MACOS_ARM64_SHA256"
    else
      url "https://github.com/JAICHANGPARK/open-codelabs/releases/download/oc-v0.0.0/oc-0.0.0-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_MACOS_AMD64_SHA256"
    end
  end

  on_linux do
    url "https://github.com/JAICHANGPARK/open-codelabs/releases/download/oc-v0.0.0/oc-0.0.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "REPLACE_WITH_LINUX_AMD64_SHA256"
  end

  def install
    bin.install "oc"
    bin.install "local_bench"
    bin.install "ops_bench"
    bin.install "ws_bench"
    prefix.install_metafiles
  end

  test do
    assert_match "Open Codelabs CLI", shell_output("#{bin}/oc --help")
  end
end
