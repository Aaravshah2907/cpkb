class Cpkb < Formula
  desc "Terminal-first Competitive Programming Knowledge Base"
  homepage "https://github.com/Aaravshah2907/cpkb"
  url "https://github.com/Aaravshah2907/cpkb/archive/refs/tags/v2.2.12.tar.gz"
  sha256 "b9f83df43f3ad0856cd2cb0bf9f7b697119fea141f3bfbbb1cde432ce200942d"
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
