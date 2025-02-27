{
  description = "flake rmk with rust toolchain";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils }:
    let
      systemBuildInputs = system: pkgs:
        [ pkgs.iconv pkgs.openssl ]
        ++ (if pkgs.lib.strings.hasInfix "$system" "darwin" then [
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
        ] else
          [ ]);
    in utils.lib.eachDefaultSystem (system:
      let pkgs = import nixpkgs { inherit system; };
      in {
        # defaultPackage = naersk-lib.buildPackage {
        #   src = ./.;
        #   buildInputs = (systemBuildInputs system pkgs).${system};
        # };

        devShell = with pkgs;
          mkShell {
            shellHook = "export PATH=${"$"}PATH:~/.cargo/bin;";
            buildInputs = [
              rustup
              rustfmt
              cargo-make
              openocd-rp2040
              probe-rs-tools
              # pre-commit
              # rustPackages.clippy
            ] ++ (systemBuildInputs system pkgs);
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
      });
}
