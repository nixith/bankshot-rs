{
  description = "A java flake with csc 116 dev tools";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
    };

    crane.url = "github:ipetkov/crane";
  };
  outputs = {
    self,
    nixpkgs,
    fenix,
    crane,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        config.allowUnfree = true;
      };
      devToolchain = fenix.packages."${system}".stable; # NOTE stable
      craneLib = crane.lib.${system};
    in {
      # For `nix build` & `nix run`:
      packages.default = craneLib.buildPackage {
        src = craneLib.cleanCargoSource (craneLib.path ./.);
        nativeBuildInputs = [pkgs.pkg-config pkgs.sqlite pkgs.openssl];
      };
      devShells.default = pkgs.mkShell {
        packages = with pkgs; [
          (devToolchain.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rustc"
            "rustfmt"
          ])
          rust-analyzer
          docker-compose
          sqlitebrowser
          sqlite
          openssl
          pkg-config
        ];
      };
    });
}
