{
  description = "A flake for shell-snoop";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = (import nixpkgs { inherit system; }).pkgsStatic;
    in
    {

      packages."${system}".shell-snoop = pkgs.callPackage ./default.nix { };

      defaultPackage."${system}" = self.packages."${system}".shell-snoop;

      devShell."${system}" = pkgs.mkShell {
        buildInputs = with pkgs; [ libcap_ng ];
      };

    };
}
