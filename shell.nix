let
  src = builtins.fetchTarball {
    # nixpkgs-19.09 as of 2020/01/02.
    url = "https://github.com/NixOS/nixpkgs/archive/eab4ee0c27c5c6f622aa0ca55091c394a9e33edd.tar.gz";
    sha256 = "sha256:1h2z8fp3plm3if9692rp1xdjicxwbvp5vl8pm5cg0gb2r3l7rwy7";
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

    pkgconfig

    # For creating the UI.
    gnome3.glade
  ];
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
