with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "jackctl-dev";
  buildInputs = with pkgs; [
    rustc cargo rust-analyzer

    pkg-config clang_12
    alsa-lib cairo pango atk
    gtk3 glib gdk-pixbuf libappindicator-gtk3
    jack2
  ];

  ## libappindicator doesn't build unless we point it at libclang.so ??
  LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
}
