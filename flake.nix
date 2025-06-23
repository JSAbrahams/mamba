{
description = "A Nix flake providing a Rust dev environment with rustup, cargo tooling, and nushell";

inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
};

outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
    let
        pkgs = import nixpkgs { inherit system; };
    in {
        devShell = pkgs.mkShell {
            buildInputs = with pkgs; [
                git               # Version control tool
                rustup            # Rust toolchain manager
                cargo-nextest     # Fast, parallel test runner
                cargo-sort        # Sorts Cargo.toml entries
                nushell           # Nu Shell                   <https://wiki.nixos.org/wiki/Nushell>
                starship          # Display relevant info      <https://wiki.nixos.org/wiki/Starship>
            ];

            shellHook = ''
                # make sure to configure githooks path
                git config core.hooksPath .githooks
                
                if [ -z "$NU_VERSION" ]; then
                    exec "${pkgs.nushell}/bin/nu" --login --config "$(pwd)/.config/nushell/config.nu"
                fi
            '';
            };
        }
    );
}
