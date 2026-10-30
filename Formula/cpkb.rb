class Cpkb < Formula
  desc "Competitive Programming Knowledge Base (High-Performance Rust Edition)"
  homepage "https://github.com/Aaravshah2907/cpkb"
  url "https://github.com/Aaravshah2907/cpkb/archive/refs/tags/v3.0.4.tar.gz"
  sha256 "a8c3bec1e6bfae6c7cf0774b428650bc8e08ba735468d36671e51d004c5a49bf"
  license "MIT"

  depends_on "rust" => :build

  def install
    cd "rust" do
      system "cargo", "install", *std_cargo_args
    end

    # Install shell completions
    generate_completions_from_executable(bin/"cpkb", "completions")
  end

  test do
    assert_match "3.0.4", shell_output("#{bin}/cpkb --version")
    assert_match "Competitive Programming Knowledge Base", shell_output("#{bin}/cpkb --help")
  end
end
