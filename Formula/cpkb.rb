class Cpkb < Formula
  include Language::Python::Virtualenv

  desc "Terminal-first Competitive Programming Knowledge Base"
  homepage "https://github.com/Aaravshah2907/cpkb"
  url "https://files.pythonhosted.org/packages/source/c/cpkb/cpkb-2.0.1.tar.gz"
  sha256 "REPLACE_WITH_PYPI_SDIST_SHA256"
  license "MIT"

  depends_on "python@3.11"
  depends_on "fzf" => :recommended

  resource "cryptography" do
    url "https://files.pythonhosted.org/packages/source/c/cryptography/cryptography-45.0.4.tar.gz"
    sha256 "REPLACE_WITH_BREW_VENDOR_SHA256"
  end

  resource "textual" do
    url "https://files.pythonhosted.org/packages/source/t/textual/textual-3.5.0.tar.gz"
    sha256 "REPLACE_WITH_BREW_VENDOR_SHA256"
  end

  def install
    virtualenv_install_with_resources
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/cpkb --version")
    assert_match "Competitive Programming Knowledge Base", shell_output("#{bin}/cpkb --help")
  end
end
