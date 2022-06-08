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
              buildInputs = [
                pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
                pkgs.darwin.apple_sdk.frameworks.CoreFoundation
              ];
            };
            "average-character-cloud-backend" = attrs: {
              buildInputs = [
                pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
                pkgs.darwin.apple_sdk.frameworks.Security
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
        };
        devShell = mkShell {
          packages = [
            (rust-toolchain pkgs)
            sqlx-cli
            cargo-make
            cargo-watch
            packages.sqldef
            crate2nix
          ] ++ lib.optionals stdenv.isDarwin [
            libiconv
            darwin.apple_sdk.frameworks.SystemConfiguration
            darwin.apple_sdk.frameworks.CoreFoundation
            darwin.apple_sdk.frameworks.Security
          ] ++ lib.optionals stdenv.isLinux [
            openssl
          ];
        };
      }
    );
}
