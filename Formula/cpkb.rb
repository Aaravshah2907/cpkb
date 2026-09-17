class Cpkb < Formula
  desc "Competitive Programming Knowledge Base (High-Performance Rust Edition)"
  homepage "https://github.com/Aaravshah2907/cpkb"
  url "https://github.com/Aaravshah2907/cpkb/archive/refs/tags/v3.0.1.tar.gz"
  sha256 "8f4e21c680ff76b4e0097c0ceb1a80b0c000acaa9548eb42da4094671d9c82a8"
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
    assert_match "3.0.1", shell_output("#{bin}/cpkb --version")
    assert_match "Competitive Programming Knowledge Base", shell_output("#{bin}/cpkb --help")
  end
end
