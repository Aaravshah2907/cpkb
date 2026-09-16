class Cpkb < Formula
  desc "Competitive Programming Knowledge Base (High-Performance Rust Edition)"
  homepage "https://github.com/Aaravshah2907/cpkb"
  url "https://github.com/Aaravshah2907/cpkb/archive/refs/tags/v3.0.0.tar.gz"
  sha256 "f6e77c8cbb00132f1ddaaa7e11492ec4dbc3808e11f33f096444f95c4ae04b02"
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
    assert_match "3.0.0", shell_output("#{bin}/cpkb --version")
    assert_match "Competitive Programming Knowledge Base", shell_output("#{bin}/cpkb --help")
  end
end
