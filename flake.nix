# Slighty modified example of example flake at <https://nixos.wiki/wiki/Rust> to install Rust.
# We opt for the variant where we outsource Rust toolchain management to rustup.
# This is so that those who wish to develop without Nix (for any number of reasons) still benefit from Rustup.

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
        # Read the rust toolchain config relative to the flake's root
        overrides = (builtins.fromTOML (builtins.readFile (self + "/rust-toolchain.toml")));
        libPath = with pkgs; lib.makeLibraryPath [
          # load external libraries that you need in your rust project here
        ];
    in {
        devShell = pkgs.mkShell {
            buildInputs = with pkgs; [
                git               # Version control tool
                
                clang                 # C++ tooling
                llvmPackages.bintools #
                rustup                # Manage rust toolchains
                
                nushell           # Nu Shell                   <https://wiki.nixos.org/wiki/Nushell>
                starship          # Display relevant info      <https://wiki.nixos.org/wiki/Starship>

                openssh           # SSH agent
            ];

            RUSTC_VERSION = overrides.toolchain.channel;
            # https://github.com/rust-lang/rust-bindgen#environment-variables
            LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
            # Add precompiled library to rustc search path
            RUSTFLAGS = (builtins.map (a: ''-L ${a}/lib'') [
                # add libraries here (e.g. pkgs.libvmi)
            ]);
            LD_LIBRARY_PATH = libPath;
            # Add glibc, clang, glib, and other headers to bindgen search path
            # Includes normal include path
            BINDGEN_EXTRA_CLANG_ARGS = (builtins.map (a: ''-I"${a}/include"'') [
                # add dev libraries here (e.g. pkgs.libvmi.dev)
                pkgs.glibc.dev
            ])
            # Includes with special directory paths
            ++ [
                ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
                ''-I"${pkgs.glib.dev}/include/glib-2.0"''
                ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
            ];

            shellHook = ''
                export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
                export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/

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
