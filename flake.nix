{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachSystem flake-utils.lib.defaultSystems (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };

      in {
        devShells = {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              rustup
              cargo
            ];

            shellHook = "zsh";
          };
        };
      });
}
