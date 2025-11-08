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
          config.allowUnfree = true;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            openssl
            pkg-config
            (rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
              extensions = [ "rust-src" ];
              targets = [ "arm-unknown-linux-gnueabihf" "aarch64-unknown-linux-gnu" "x86_64-pc-windows-gnu" ];
            }))
            alsa-lib
            udev
            vulkan-loader
            xorg.libX11
            xorg.libXrandr
            xorg.libXcursor
            xorg.libXi
            kdePackages.qtbase
            nodejs_22
            yarn-berry
            trunk-io
            rust-analyzer
            at-spi2-atk
            atkmm
            cairo
            gdk-pixbuf
            glib
            gtk3
            harfbuzz
            librsvg
            libsoup_3
            pango
            webkitgtk_4_1
            openssl
            wayland
            waylandpp
            fontconfig
          ];

          shellHook = ''
              export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
                pkgs.lib.makeLibraryPath [
                  pkgs.udev
                  pkgs.alsa-lib
                  pkgs.vulkan-loader
                  pkgs.libxkbcommon
                  pkgs.wayland
                ]
              }"
              RUST_SRC_PATH = "${pkgs.rust-bin.nightly.latest.default}/lib/rustlib/src/rust/library";
              '';
        };
      }
    );
}
