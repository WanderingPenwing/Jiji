{ pkgs ? import <nixpkgs> { overlays = [ (import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz)) ]; },
  unstable ? import <nixos-unstable> { config = { allowUnfree = true; }; }
}:
with pkgs;

let
  # Specify Pillow as a build input
  pillow = python3Packages.pillow;
in

mkShell {
  nativeBuildInputs = with xorg; [
    libxcb
    libXcursor
    libXrandr
    libXi
    pkg-config
  ] ++ [
    cargo
    unstable.rustc
    python3
    libGL
    libGLU
    libxkbcommon
    gtk3-x11
    gnome.zenity
  ];
  buildInputs = [
    latest.rustChannels.stable.rust
    xorg.libX11
    wayland
    libxkbcommon
  ];

  shellHook = ''
      export LD_LIBRARY_PATH=/run/opengl-driver/lib/:${lib.makeLibraryPath ([libGL libGLU libxkbcommon])}
  '';
}
