with import <nixpkgs> {};

with rustPlatform;

buildRustPackage rec {
  name = "tml-${version}";
  version = "0.1.0";

  depsSha256 = "1x31zlzxc7bkab8li26p3zrdk36q1l5lc1k2xrgdsmdaww3ml1ra";

  src = ./.;
}
