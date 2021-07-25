{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nmattia/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";
        naersk-lib = naersk.lib."${system}";
      in
      rec {
        # `nix build`
        packages.shell-snoop = naersk-lib.buildPackage {
          pname = "shell-snoop";
          version = "0.3.0";
          root = ./.;

          meta = with pkgs.lib; {
            homepage = "https://github.com/michaeladler/shell-snoop";
            description = "figure out the exact command which was used to run a child process in a shell";
            platforms = platforms.linux;
            license = licenses.asl20;
          };
        };
        defaultPackage = packages.shell-snoop;

        # `nix run`
        apps.shell-snoop = utils.lib.mkApp {
          drv = packages.shell-snoop;
        };
        defaultApp = apps.shell-snoop;

        # `nix develop`
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ rustc cargo ];
        };
      });
}
