let
  src = builtins.fetchTarball {
    # nixpkgs master as of 2020-07-17
    url = https://github.com/NixOS/nixpkgs/archive/e6d81a9b89e8dd8761654edf9dc744660a6bef0a.tar.gz;
    sha256 = "0lmw1zy00l89b0x7l5f85bvxdd2w245iqf9smyiyxvl1j03b0zyq";
  };
in

with import src {};

stdenv.mkDerivation {
  name = "break-time-rust-env";

  nativeBuildInputs = [
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
  LIBCLANG_PATH = "${llvmPackages.libclang}/lib";

  buildInputs = [
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
    export MANPATH=":${xorg.libxcb.man}/share/man"
  '';

  # Set Environment Variables
  #RUST_BACKTRACE = 1;
}
