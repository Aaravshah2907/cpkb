class Cpkb < Formula
  desc "Competitive Programming Knowledge Base (High-Performance Rust Edition)"
  homepage "https://github.com/Aaravshah2907/cpkb"
  url "https://github.com/Aaravshah2907/cpkb/archive/refs/tags/v3.0.2.tar.gz"
  sha256 "c72e611ab303c765a030b3ae974ef0234e0a30fc3f9cac283693ca85f61d0600"
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
    assert_match "3.0.2", shell_output("#{bin}/cpkb --version")
    assert_match "Competitive Programming Knowledge Base", shell_output("#{bin}/cpkb --help")
  end
end
