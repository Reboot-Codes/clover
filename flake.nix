{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    fenix.url = "github:nix-community/fenix";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      ...
    }@inputs:
    let
      fenix = inputs.fenix.packages;
    in
    # Iterate over Arm, x86 for macOS and Linux
    (flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        crane = inputs.crane.mkLib pkgs;
        # Toolchain
        toolchain = fenix.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-TpJxH//QHhdKvCQEf8XbrliwFhqu5lETkU1w99HZ/+4=";
        };
        craneLib = crane.overrideToolchain toolchain;

        # Deps for all software packages
        buildInputs = with pkgs; [
          openssl.dev
          pkg-config
          wayland
        ];

        src = pkgs.lib.cleanSourceWith {
          src = craneLib.path ./.;
          filter = path: type: (pkgs.lib.hasInfix "/assets" path) || (craneLib.filterCargoSources path type);
        };
        commonArgs = {
          doCheck = false;
          inherit src buildInputs;
          nativeBuildInputs = libraries;
        };

        libraries = with pkgs; [
          openssl
          pkg-config
          vulkan-loader
          libxkbcommon
          wayland
          xorg.libX11
          xorg.libxcb
          alsa-lib
          udev
          vulkan-loader
          xorg.libX11
          xorg.libXrandr
          xorg.libXcursor
          xorg.libXi
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
          fontconfig
          libz
        ];
        # Compile all artifacts
        appDeps = craneLib.buildDepsOnly commonArgs;

        appListings = [
          {
            name = "clover-hub";
            path = ./clover-hub/Cargo.toml;
          }
          {
            name = "ratchet";
            path = ./toolbox/ratchet/Cargo.toml;
          }
        ];

        # Compile
        pkg =
          listing:
          craneLib.buildPackage (
            commonArgs
            // {
              cargoExtraArgs = "-p ${listing.name}";
              cargoArtifacts = appDeps;
              pname = listing.name;
              version = (builtins.fromTOML (builtins.readFile listing.path)).package.version;
            }
          );
        app =
          listing:
          flake-utils.lib.mkApp {
            drv = pkg listing;
          };
      in
      {
        # nix build
        packages = {
          default = pkg (builtins.elemAt appListings 0);
        }
        // builtins.listToAttrs (
          map (listing: {
            name = listing.name;
            value = pkg listing;
          }) appListings
        );

        # nix run
        apps = {
          default = app (builtins.elemAt appListings 0);
        }
        // builtins.listToAttrs (
          map (listing: {
            name = listing.name;
            value = app listing;
          }) appListings
        );

        # nix develop
        devShells.default = craneLib.devShell {
          inherit buildInputs;

          packages = [
            toolchain
          ]
          ++ (with pkgs; [
            nodejs_22
            yarn-berry
            rust-analyzer
            tokio-console
          ])
          ++ libraries;

          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath libraries}";
        };
      }
    ));
}
