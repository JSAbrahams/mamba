# Slighty modified example of example flake at <https://nixos.wiki/wiki/Rust> to install Rust.
#
# Toolchain sourcing is deliberately split in two, so each path stays simple on its own:
# - Nix-shell users get rustc/cargo/rustfmt/clippy straight from nixpkgs (pinned via flake.lock,
#   so still fully reproducible per-commit). No rustup involved, nothing to configure.
# - Non-nix users keep using rustup exactly as before: `rust-toolchain.toml` at the repo root is
#   picked up automatically by rustup, independent of this file.
# `RUSTC_VERSION` below only exists to warn (not block) if the two ever drift apart.

{
  description = "Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        # Read the file relative to the flake's root
        overrides = (builtins.fromTOML (builtins.readFile (self + "/rust-toolchain.toml")));
        libPath = with pkgs; lib.makeLibraryPath [
          # load external libraries that you need in your rust project here
        ];
      in
      {
        devShells.default = pkgs.lib.warnIf
          (pkgs.rustc.version != overrides.toolchain.channel)
          "nixpkgs rustc (${pkgs.rustc.version}) differs from rust-toolchain.toml's pinned channel (${overrides.toolchain.channel}) -- non-nix (rustup) contributors will be building with a different compiler version."
          (pkgs.mkShell rec {
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = with pkgs; [
            aspell    # Spell checker
            coreutils # Dircolors support (via coreutils)
            direnv    # Direnv for environment switching

            git      # Version control tool
            less     # Used under the hood by git
            more     # Nice to have next to 'less'

            clang                    # C++ tooling
            llvmPackages.bintools    #
            llvmPackages_latest.llvm # LLVM build tools (also provides llvm-cov/llvm-profdata, see LLVM_COV below)

            rustc   # Rust toolchain straight from the nix store, see top-of-file note
            cargo
            rustfmt
            clippy
            rust-analyzer # Language server, for editor integration
            cargo-llvm-cov # `cargo llvm-cov` coverage tooling, see CONTRIBUTING.md

            python310 # required by the test suite (tests_util::PYTHON on Linux); see CLAUDE.md

            nushell  # Nu Shell                   <https://wiki.nixos.org/wiki/Nushell>
            starship # Display relevant info      <https://wiki.nixos.org/wiki/Starship>
            jq       # Commandline json processor

            vim      # Text editor
            nano     # Text editor

            openssh  # SSH
          ];

          RUSTC_VERSION = overrides.toolchain.channel;

          # https://github.com/rust-lang/rust-bindgen#environment-variables
          LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];

          # Consumed by `cargo llvm-cov`; sourced from the same nixpkgs llvmPackages_latest as
          # LIBCLANG_PATH above, rather than a system/arch-specific path, so this works on every
          # system the flake targets (not just x86_64-linux).
          LLVM_COV = "${pkgs.llvmPackages_latest.llvm}/bin/llvm-cov";
          LLVM_PROFDATA = "${pkgs.llvmPackages_latest.llvm}/bin/llvm-profdata";

          shellHook = ''
            # coreutuils in path
            # direnv: initialize auto-env loading for nushell
            eval "$(direnv hook bash)"

            # make sure to configure githooks path
            git config core.hooksPath .githooks

            if [ -z "$NU_VERSION" ]; then
                # workspace is where we called flake from
                export WORKSPACE="$PWD"
                export STARSHIP_CONFIG="$WORKSPACE/.config/startship.toml"

                # first check for user-specific config
                if [ -f "$(pwd)/.config/nushell/config.nu" ]; then
                    exec "${pkgs.nushell}/bin/nu" --login --config "$(pwd)/.config/nushell/config.nu"

                # else default to default version-controlled config
                elif [ -f "$(pwd)/.config/nushell/config.example.nu" ]; then
                    exec "${pkgs.nushell}/bin/nu" --login --config "$(pwd)/.config/nushell/config.example.nu"

                # else just no config, failsafe
                else
                    exec "${pkgs.nushell}/bin/nu" --login
                fi
            fi
          '';

          # Add precompiled library to rustc search path
          RUSTFLAGS = (builtins.map (a: ''-L ${a}/lib'') [
            # add libraries here (e.g. pkgs.libvmi)
          ]);
          
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (buildInputs ++ nativeBuildInputs);

          # Add glibc, clang, glib, and other headers to bindgen search path
          BINDGEN_EXTRA_CLANG_ARGS =
          # Includes normal include path
          (builtins.map (a: ''-I"${a}/include"'') [
            # add dev libraries here (e.g. pkgs.libvmi.dev)
            pkgs.glibc.dev
          ])
          # Includes with special directory paths
          ++ [
            ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
            ''-I"${pkgs.glib.dev}/include/glib-2.0"''
            ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
          ];
        });
      }
    );
}
