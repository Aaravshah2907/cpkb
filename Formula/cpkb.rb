class Cpkb < Formula
  desc "Terminal-first Competitive Programming Knowledge Base"
  homepage "https://github.com/Aaravshah2907/cpkb"
  url "https://github.com/Aaravshah2907/cpkb/archive/refs/tags/v2.2.13.tar.gz"
  sha256 "ce7f92592875486a10133360d18d902317864b73d6a43b46945269147681f52f"
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
