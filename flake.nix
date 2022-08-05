{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgsArgs = {
          inherit system overlays;
        };
        pkgs = import nixpkgs pkgsArgs;
        rust-toolchain = pkgs: pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        buildRustCrateForPkgs = pkgs: pkgs.buildRustCrate.override {
          rustc = rust-toolchain pkgs;
          defaultCrateOverrides = pkgs.defaultCrateOverrides // {
            "sqlx-macros" = attrs: {
              buildInputs = with pkgs; lib.optionals stdenv.isDarwin [
                darwin.apple_sdk.frameworks.SystemConfiguration
                darwin.apple_sdk.frameworks.CoreFoundation
              ];
            };
            "average-character-cloud-backend" = attrs: {
              buildInputs = with pkgs; lib.optionals stdenv.isDarwin [
                darwin.apple_sdk.frameworks.SystemConfiguration
                darwin.apple_sdk.frameworks.Security
              ];
              SQLX_OFFLINE = "true";
            };
          };
        };
        crate = import ./Cargo.nix {
          inherit pkgs buildRustCrateForPkgs;
        };
      in
      with pkgs; rec {
        defaultPackage = crate.rootCrate.build;
        packages = {
          sqldef = callPackage ./sqldef.nix { };
          average-character-cloud-backend = defaultPackage;
          average-character-cloud-backend-docker = dockerTools.buildImage {
            name = "average-character-cloud-backend";
            contents = [
              pkgs.coreutils
              pkgs.bash
              pkgs.cacert
              defaultPackage
            ];
            config = {
              Cmd = [ "average-character-cloud-backend" ];
            };
          };
        };
        devShell = mkShell {
          packages = [
            (rust-toolchain pkgs)
            sqlx-cli
            cargo-make
            packages.sqldef
            crate2nix
          ] ++ lib.optionals stdenv.isDarwin [
            libiconv
            darwin.apple_sdk.frameworks.SystemConfiguration
            darwin.apple_sdk.frameworks.CoreFoundation
            darwin.apple_sdk.frameworks.Security
          ] ++ lib.optionals stdenv.isLinux [
            openssl
            pkg-config
            glibc
          ];
        };
      }
    );
}
