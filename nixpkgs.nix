
let
  nixpkgsSrc = builtins.fetchTarball {
    # nixpkgs-21.05 as of 2021-06-05
    url = https://github.com/NixOS/nixpkgs/archive/4c2e84394c0f372c019e941e95d6fbe21835719b.tar.gz;
    sha256 = "099f5cgjrmkqbgdlhynghbbr32jlxr0fqigqfg2w421gk9vkhp4d";
  };

  # This is a derivation for actually building break-time using the Rust
  # infrastructure in nixpkgs.  If break-time was upstreamed, something similar
  # to this could be put directly into nixpkgs.
  break-time-overlay = final: prev: {
    break-time =
      final.rustPlatform.buildRustPackage rec {
        pname = "break-time";
        version = "0.1.2";

        src = final.nix-gitignore.gitignoreSource [] ./.;

        buildInputs = with final; [
          glib
          gtk3
          openssl
        ];

        nativeBuildInputs = with final; [
          pkg-config
          python3 # needed for Rust xcb package
          wrapGAppsHook
        ];

        cargoSha256 = "0q6xdsd2bxc6y7d1f4c7i1a2fsh0wqmpxvp8397zmdqnqaszk58f";
      };

    # This is a development shell.  It should be run with nix-shell.  It
    # provides rustup, which can be used to install things like cargo, rustc,
    # clippy, etc.
    #
    # This is just for development, and could never be upstreamed
    break-time-shell =
      final.stdenv.mkDerivation {
        name = "break-time-rust-env";

        nativeBuildInputs = with final; [
          # Things like cargo, rustc, rustfmt, and clippy can be installed with commands like
          #
          # $ rustup component add clippy
          rustup

          # Some rust packages use clang to compile c bindings.
          llvmPackages.clang
          llvmPackages.libclang

          # Some rust packages use pkgconfig when building.
          pkgconfig

          # For creating the UI.
          gnome3.glade
        ];

        # libappindicator-sys generates bindings with bindgen, which uses LLVM and
        # requires LIBCLANG_PATH be set.
        LIBCLANG_PATH = "${final.llvmPackages.libclang}/lib";

        buildInputs = with final; [
          openssl

          # GTK libraries
          glib
          gtk3

          # Xorg libraries
          python3 # xcb crate uses python
          xorg.libxcb
        ];

        shellHook = ''
          # TODO: This clobbers MANPATH if it is already set.
          export MANPATH=":${final.xorg.libxcb.man}/share/man"
        '';

        # These phases need to be set to noops so this shell file can actually be
        # built with `nix-build shell.nix`.
        unpackPhase = "true";
        installPhase = "touch $out";

        # Set Environment Variables
        #RUST_BACKTRACE = 1;
      };
  };

  nixpkgs = import nixpkgsSrc {
    overlays = [ break-time-overlay ];
  };
in

nixpkgs
