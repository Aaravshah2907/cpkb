class Cpkb < Formula
  include Language::Python::Virtualenv

  desc "Terminal-first Competitive Programming Knowledge Base"
  homepage "https://github.com/Aaravshah2907/cpkb"
  url "https://files.pythonhosted.org/packages/29/13/9f1b84adab9a0c1bf3d2cf62b3dd18dc18241d539178acfb0daf2a76259a/cpkb-2.0.2.tar.gz"
  sha256 "cd0952bb8b20b54f63a0722834b90f3287482ef782fb3af739ca64ffa082b8bf"
  license "MIT"

  depends_on "python@3.11"
  depends_on "fzf" => :recommended

  resource "textual" do
    url "https://files.pythonhosted.org/packages/0c/63/16cdf4b9efb47366940d8315118c5c6dd6309f5eb2c159d7195b60e2e221/textual-3.5.0.tar.gz"
    sha256 "c4a440338694672acf271c74904f1cf1e4a64c6761c056b02a561774b81a04f4"
  end

  def install
    virtualenv_install_with_resources
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/cpkb --version")
    assert_match "Competitive Programming Knowledge Base", shell_output("#{bin}/cpkb --help")
  end
end
