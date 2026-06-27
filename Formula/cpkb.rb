class Cpkb < Formula
  include Language::Python::Virtualenv

  desc "Terminal-first Competitive Programming Knowledge Base"
  homepage "https://github.com/Aaravshah2907/cpkb"
  url "https://files.pythonhosted.org/packages/8e/dc/28e023d71c93daeb2d8c58a41fafa82a40a659a3de120c3d0b86f67af5fe/cpkb-2.0.1.tar.gz"
  sha256 "f9cc6cc985d8fcdf857ab4a9d1fdbb97d07d9a5d3824f7a9880064eaf53433d7"
  license "MIT"

  depends_on "rust" => :build
  depends_on "python@3.11"
  depends_on "fzf" => :recommended

  resource "cryptography" do
    url "https://files.pythonhosted.org/packages/fe/c8/a2a376a8711c1e11708b9c9972e0c3223f5fc682552c82d8db844393d6ce/cryptography-45.0.4.tar.gz"
    sha256 "7405ade85c83c37682c8fe65554759800a4a8c54b2d96e0f8ad114d31b808d57"
  end

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
