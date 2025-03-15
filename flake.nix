{
  description = "CLOVER's Flake";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            openssl
            pkg-config
            rust-bin.nightly.latest.default
            alsa-lib
            udev
            vulkan-loader
            xorg.libX11
            xorg.libXrandr
            xorg.libXcursor
            xorg.libXi
            kdePackages.qtbase
            nodejs_23
            yarn-berry
          ];

          shellHook = ''
              export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
                pkgs.lib.makeLibraryPath [
                  pkgs.udev
                  pkgs.alsa-lib
                  pkgs.vulkan-loader
                  pkgs.libxkbcommon
                ]
              }"'';
        };
      }
    );
}
