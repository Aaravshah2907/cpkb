class Cpkb < Formula
  desc "Terminal-first Competitive Programming Knowledge Base"
  homepage "https://github.com/Aaravshah2907/cpkb"
  url "https://github.com/Aaravshah2907/cpkb/archive/refs/tags/v2.2.1.tar.gz"
  sha256 "961c1a5a0e15b7b66fff4777816ea90a26214be4788bf8bbfa008cf35c57c342"
  license "MIT"

  depends_on "python@3.11"
  depends_on "fzf" => :recommended

  def install
    libexec.install "src"

    (bin/"cpkb").write <<~EOS
      #!/bin/sh
      export PYTHONPATH="#{libexec}/src${PYTHONPATH:+:$PYTHONPATH}"
      exec python3 -m cpkb.cli "$@"
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/cpkb --version")
    assert_match "Competitive Programming Knowledge Base", shell_output("#{bin}/cpkb --help")
    assert_match "setup", shell_output("#{bin}/cpkb --help")
  end
end
