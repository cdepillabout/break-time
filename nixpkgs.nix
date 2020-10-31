
let
  nixpkgsSrc = builtins.fetchTarball {
    # nixpkgs-20.09 as of 2020-10-31
    url = https://github.com/NixOS/nixpkgs/archive/edb26126d98bc696f4f3e206583faa65d3d6e818.tar.gz;
    sha256 = "1cl4ka4kk7kh3bl78g06dhiidazf65q8miyzaxi9930d6gwyzkci";
  };

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

        cargoSha256 = "08slryr9dnciz4y5hqpfdvjv3g915qgivjdq74qp31z2bvw7jnxr";
      };

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