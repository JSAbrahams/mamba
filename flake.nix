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
        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = with pkgs; [
            aspell    # Spell checker
            coreutils # Dircolors support (via coreutils)
            direnv    # Direnv for environment switching

            git      # Version control tool
            less     # Used under the hood by git
            more     # Nice to have next to 'less'
            
            rustup                # Manage rust toolchains
            clang                 # C++ tooling
            llvmPackages.bintools #

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
          
          shellHook = ''
            # point to correct cargo and rustup toolchain version
            export PATH=''${CARGO_HOME:-~/.cargo}/bin:$PATH
            export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/

            # coreutuils in path
            # direnv: initialize auto-env loading for nushell
            eval "$(direnv hook bash)"

            # make sure to configure githooks path
            git config core.hooksPath .githooks

            # for now we rely on cargo to install external tooling, in future we might put it in the above flake which may be more idiomatic
            # quiet to avoid annoying warning that this is already installed (again, we committed to using the rustup toolchain)
            cargo install cargo-llvm-cov@0.6.16 --quiet

            if [ -z "$NU_VERSION" ]; then
                # export as absolute paths relative to project
                export WORKSPACE="${toString ./.}/"
                export STARSHIP_CONFIG="$WORKSPACE/.config/startship.toml"

                # first check for user-specific config
                if [ -f "$(pwd)/.config/nushell/config.nu" ]; then
                    exec "${pkgs.nushell}/bin/nu" --login --config "$(pwd)/.config/nushell/config.nu"

                # else default to default version-controlled config
                elif [ -f "$(pwd)/.config/nushell/config.example.nu" ]; then
                    exec "${pkgs.nushell}/bin/nu" --login --config "$(pwd)/.config/nushell/config.example.nu"
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
        };
      }
    );
}
