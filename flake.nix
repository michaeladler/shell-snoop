{
  description = "A flake for shell-snoop";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }:

    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages.${system}; in
      rec {

        packages = {
          shell-snoop = pkgs.callPackage ./default.nix { };
        };

        defaultPackage = packages.shell-snoop;

      });

}
