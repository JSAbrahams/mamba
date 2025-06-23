    {
    description = "A Nix flake providing a Rust dev environment with cargo-nextest, cargo-sort, rustup, zsh, and Oh My Zsh";

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

                    # point Nu config dir our repo `.config` folder
                    export XDG_CONFIG_HOME="$(pwd)/.config"

                    # If not already in Nu Shell, switch to an interactive Nu session
                    if [ -z "$NU_VERSION" ]; then
                        exec "${pkgs.nushell}/bin/nu" --login
                    fi
                '';
                };
            }
        );
    }
