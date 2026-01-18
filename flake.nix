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
        packages = {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "logitech-g600-rs";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs = with pkgs; [ libratbag ];
          };
        };

        devShells = {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              rustup
              cargo
              libratbag
            ];

            shellHook = "zsh";
          };
        };
      });
}
