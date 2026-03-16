{
  description = "FOL development shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          strictDeps = true;

          packages = with pkgs; [
            rustc
            cargo
            rustfmt
            clippy
            rust-analyzer
            llvmPackages.lldb
            git
            pkg-config
          ];

          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

          shellHook = ''
            export PATH="$PATH:$PWD:$PWD/target/debug:$PWD/target/release"
          '';
        };
      });
}
