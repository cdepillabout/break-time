
let
  nixpkgsSrc = builtins.fetchTarball {
    # nixpkgs-unstable as of 2026-06-30
    url = https://github.com/NixOS/nixpkgs/archive/9c4c05a947a91dc14625265fab505fb695e93218.tar.gz;
    sha256 = "19n40n9skzv9i875r2db1pznf08078wqs7853ydcdmk1585s7q61";
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
        ];

        nativeBuildInputs = with final; [
          pkg-config
          wrapGAppsHook3
        ];

        cargoHash = "sha256-a4i6+zlE9RI1uR0i0nwsTxYHkWZSADA64LsVn8ud4ps=";
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
          #rustup

          # Or, just directly install cargo, rustc, rustfmt, etc within the Nix shell.
          cargo
          rustc
          rustfmt
          clippy

          # Some rust packages use clang to compile c bindings.
          llvmPackages.clang
          llvmPackages.libclang

          # Some rust packages use pkgconfig when building.
          pkg-config

          # For creating the UI.
          # gnome3.glade
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
          libxcb
        ];

        shellHook = ''
          # TODO: This clobbers MANPATH if it is already set.
          export MANPATH=":${final.libxcb.man}/share/man"
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
