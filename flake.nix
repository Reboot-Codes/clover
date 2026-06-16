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
          libX11
          libxcb
          alsa-lib
          udev
          vulkan-loader
          libXrandr
          libXcursor
          libXi
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
          mesa
          libglvnd
        ];
        # Compile all artifacts
        appDeps = craneLib.buildDepsOnly commonArgs;

        appListings = [
          {
            name = "clover-hub";
            path = ./clover-hub/Cargo.toml;
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
        devShells.default =
          let
            cloverCompose = pkgs.writeShellScriptBin "clover-up" ''
              CURRENT_REPO_ROOT=${pkgs.git}/bin/git rev-parse --show-toplevel 2>/dev/null

              # Fallback to the current working directory if not inside a git repo
              if [ -z "$CURRENT_REPO_ROOT" ]; then
                CURRENT_REPO_ROOT=$(pwd)
              fi

              # Verify the file actually exists before trying to pass it to process-compose
              PROCESS_COMPOSE_FILE="$CURRENT_REPO_ROOT/process-compose.yaml"
              if [ ! -f "$PROCESS_COMPOSE_FILE" ]; then
                echo "Fatal: Could not find process-compose.yaml at root target: $CURRENT_REPO_ROOT"
                exit 1
              fi

              # Ensure that libraries are actually passed to the development setup.
              LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath libraries}"

              exec ${pkgs.process-compose}/bin/process-compose up --no-server -f $PROCESS_COMPOSE_FILE
            '';

            cloverHubCMD = pkgs.writeShellScriptBin "clover-hub" ''
              # Ensure that libraries are actually passed to the development setup.
              LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath libraries}"
              RUST_BACKTRACE=1
              CLOVER_SIMULATED_CONTROLS="true"
              CLOVER_MASTER_PRINT="true"

              exec cargo run --bin="clover-hub" -- "$@"
            '';

            cloverHubZenoh = pkgs.writeShellScriptBin "clover-zenoh" ''
              RUST_LOG=info,zenohd=debug

              exec zenohd -l tcp/0.0.0.0:6699 --adminspace-permissions rw \
                --cfg='adminspace/enabled:true' \
                --cfg='adminspace/permissions/read:true' \
                --cfg='adminspace/permissions/write:true'
            '';
          in
          craneLib.devShell {
            inherit buildInputs;

            packages = [
              toolchain
            ]
            ++ (with pkgs; [
              nodejs_22
              yarn-berry
              rust-analyzer
              tokio-console
              zenoh
              process-compose
              cloverCompose
              cloverHubCMD
              cloverHubZenoh
              flutter
            ])
            ++ libraries;
          };
      }
    ));
}
