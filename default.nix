{ lib, rustPlatform }:

rustPlatform.buildRustPackage rec {
  pname = "shell-snoop";
  version = "0.3.0";

  src = lib.cleanSource ./.;

  cargoSha256 = "sha256-wUzwQbopT3BstKo6UiIQj4sEgfJbMUDivSL9FqAr/WY=";

  postInstall = "strip $out/bin/shell-snoop";

  meta = with lib; {
    homepage = "https://github.com/michaeladler/shell-snoop";
    description = "figure out the exact command which was used to run a child process in a shell";
    platforms = platforms.all;
    license = licenses.asl20;
  };

}
