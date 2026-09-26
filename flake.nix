{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-manifest = {
      url = "https://static.rust-lang.org/dist/2026-09-24/channel-rust-nightly.toml";
      flake = false;
    };

    suit-regular = {
      url = "file+https://cdn.jsdelivr.net/gh/sun-typeface/SUIT@2.0.5/fonts/static/woff2/SUIT-Regular.woff2";
      flake = false;
    };

    suit-medium = {
      url = "file+https://cdn.jsdelivr.net/gh/sun-typeface/SUIT@2.0.5/fonts/static/woff2/SUIT-Medium.woff2";
      flake = false;
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      fenix,
      rust-manifest,
      suit-regular,
      suit-medium,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        fenixPkgs = fenix.packages.${system};
        mkToolchain = (fenixPkgs.fromManifestFile rust-manifest).withComponents;
        wasmStd = (fenixPkgs.targets.wasm32-unknown-unknown.fromManifestFile rust-manifest).rust-std;
        stableToolchain = components: fenixPkgs.combine [
          (fenixPkgs.stable.withComponents components)
          fenixPkgs.targets.wasm32-unknown-unknown.stable.rust-std
        ];

        fonts = pkgs.linkFarm "suit-woff2" [
          {
            name = "SUIT-Regular.woff2";
            path = suit-regular;
          }
          {
            name = "SUIT-Medium.woff2";
            path = suit-medium;
          }
        ];

        coreComponents = [
          "cargo"
          "rustc"
        ];
        devComponents = [
          "rust-src"
          "rustfmt"
          "rust-analyzer"
          "clippy"
          "miri"
        ];

        coreTools = [
          pkgs.trunk
          pkgs.wasm-bindgen-cli_0_2_127
          pkgs.binaryen
          pkgs.tailwindcss_4
        ];
        devTools = [
          pkgs.llvmPackages.llvm
        ];
      in
      {
        devShells = {
          default = pkgs.mkShell {
            packages =
              coreTools
              ++ devTools
              ++ [
                (fenixPkgs.combine [
                  (mkToolchain (coreComponents ++ devComponents))
                  wasmStd
                ])
              ];
            WEB_FONTS = fonts;
          };

          release = pkgs.mkShell {
            packages = coreTools ++ [ (stableToolchain coreComponents) ];
            WEB_FONTS = fonts;
          };

          miri = pkgs.mkShell {
            packages = [ (mkToolchain (coreComponents ++ [ "rust-src" "miri" ])) ];
          };

          ci = pkgs.mkShell {
            packages = [
              (fenixPkgs.combine [
                (fenixPkgs.stable.withComponents (coreComponents ++ [ "clippy" ]))
                fenixPkgs.targets.wasm32-unknown-unknown.stable.rust-std
                (mkToolchain [ "rustfmt" ])
              ])
            ];
          };
        };
      }
    );
}
