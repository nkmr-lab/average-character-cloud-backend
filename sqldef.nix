{ lib, buildGoModule, fetchFromGitHub, installShellFiles }:

buildGoModule rec {
  pname = "sqldef";
  version = "0.11.35";

  src = fetchFromGitHub {
    owner = "k0kubun";
    repo = pname;
    rev = "v${version}";
    sha256 = "sha256-T8BhGO860G8mlvVzspR2XbHsWrVWYmsTW+tc86Bskr4=";
  };

  vendorSha256 = "sha256-f7f7VGWu8PDZhb5FlqJRfR9zPutpmN3UYuKLxvh4RDE=";

  doCheck = false;
}
