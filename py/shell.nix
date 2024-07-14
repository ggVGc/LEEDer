let
  pkgs = (import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-23.05.tar.gz") { });
  nixpkgs-python = import  (fetchTarball "https://github.com/cachix/nixpkgs-python/archive/refs/heads/main.zip");
in
  pkgs.mkShell {
    buildInputs = [
      (nixpkgs-python.packages.x86_64-linux."2.7")
    ];
  }
