
let
  break-time-overlay = self: super: {
    break-time =
      self.rustPlatform.buildRustPackage rec {
        pname = "break-time";
        version = "0.1.2";

        src = self.nix-gitignore.gitignoreSource [] ./.;

        buildInputs = [
          self.glib
          self.gtk3
          self.openssl
        ];

        nativeBuildInputs = [
          self.pkg-config
          self.python3 # needed for Rust xcb package
          self.wrapGAppsHook
        ];

        cargoSha256 = "08slryr9dnciz4y5hqpfdvjv3g915qgivjdq74qp31z2bvw7jnxr";
      };
  };

  nixpkgsSrc = builtins.fetchTarball {
    # nixpkgs master as of 2020-07-17
    url = https://github.com/NixOS/nixpkgs/archive/e6d81a9b89e8dd8761654edf9dc744660a6bef0a.tar.gz;
    sha256 = "0lmw1zy00l89b0x7l5f85bvxdd2w245iqf9smyiyxvl1j03b0zyq";
  };

  nixpkgs = import nixpkgsSrc {
    overlays = [ break-time-overlay ];
  };
in

nixpkgs.break-time
