class Cpkb < Formula
  desc "Terminal-first Competitive Programming Knowledge Base"
  homepage "https://github.com/Aaravshah2907/cpkb"
  url "https://github.com/Aaravshah2907/cpkb/releases/download/v2.0.3/cpkb-2.0.3.tar.gz"
  sha256 "4536d62192b4a2489db32e1f71a2f63efc852b1aa27d9ba8748513b481a4489b"
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
