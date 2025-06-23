{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    cargo2nix = {
      url = "github:cargo2nix/cargo2nix/main";
      inputs.rust-overlay.follows = "rust-overlay";
    };
    cargo2nix-ifd = {
      url = "github:kgtkr/cargo2nix-ifd";
      inputs.cargo2nix.follows = "cargo2nix";
    };
  };

  outputs = { self, nixpkgs, flake-utils, cargo2nix, cargo2nix-ifd, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ cargo2nix.overlays.default ];
        pkgsArgs = {
          inherit system overlays;
        };
        pkgs = import nixpkgs pkgsArgs;
        cargo2nix-ifd-lib = cargo2nix-ifd.mkLib pkgs;
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        projectName = "average-character-cloud-backend";
        src = ./.;
        filteredSrc = cargo2nix-ifd-lib.filterSrc {
          orFilter = path: _type:
            let
              parentDirs = [ "migrations" ".sqlx" ];
            in
            pkgs.lib.any (dir: baseNameOf (dirOf path) == dir) parentDirs;
          inherit projectName src;
        };
        generatedSrc = cargo2nix-ifd-lib.generateSrc {
          src = filteredSrc;
          inherit projectName rustToolchain;
        };
        rustPkgs = pkgs.rustBuilder.makePackageSet {
          packageFun = import "${generatedSrc}/Cargo.nix";
          packageOverrides = pkgs: pkgs.rustBuilder.overrides.all ++
            [
              (pkgs.rustBuilder.rustLib.makeOverride {
                name = "average-character-cloud-backend";
                overrideAttrs = drv:
                  {
                    SQLX_OFFLINE = 1;
                  };
              })
              # https://github.com/launchbadge/sqlx/issues/2911 の問題でnix buildのときビルドしてしまうためworkaround
              (pkgs.rustBuilder.rustLib.makeOverride {
                name = "sqlx-sqlite";
                overrideAttrs = drv:
                  {
                    propagatedBuildInputs = drv.propagatedBuildInputs or [ ] ++ [
                      pkgs.sqlite
                    ];
                    patchPhase = ''
                      echo "" > src/lib.rs
                    '';
                  };
              })
            ];
          inherit rustToolchain;
        };
      in
      {
        packages = {
          default = self.packages.${system}.average-character-cloud-backend;
          average-character-cloud-backend = (rustPkgs.workspace.average-character-cloud-backend { }).bin;
          average-character-cloud-backend-docker = pkgs.dockerTools.buildImage {
            name = "average-character-cloud-backend";
            contents = [
              pkgs.coreutils
              pkgs.bash
              pkgs.cacert
              self.packages.${system}.average-character-cloud-backend
            ];
            config = {
              Env = [ "AVCC_HOST=0.0.0.0" ];
              Entrypoint = [ "average-character-cloud-backend" ];
            };
          };
        };
        devShell = pkgs.mkShell {
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          packages = [
            rustToolchain
            pkgs.sqlx-cli
            pkgs.just
            pkgs.sqldef
            pkgs.cargo-watch
            pkgs.minio-client
            cargo2nix.packages.${system}.cargo2nix
          ];
        };
      }
    );
}
