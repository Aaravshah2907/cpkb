class Cpkb < Formula
  desc "Competitive Programming Knowledge Base (High-Performance Rust Edition)"
  homepage "https://github.com/Aaravshah2907/cpkb"
  url "https://github.com/Aaravshah2907/cpkb/archive/refs/tags/v3.0.3.tar.gz"
  sha256 "56f39b33f1a62fa1de6a846b6d5efb9aac666fc46946ea666ba9e195c7027e07"
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
    assert_match "3.0.3", shell_output("#{bin}/cpkb --version")
    assert_match "Competitive Programming Knowledge Base", shell_output("#{bin}/cpkb --help")
  end
end
