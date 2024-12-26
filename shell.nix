{ pkgs ? import <nixpkgs> {
  config = {
    android_sdk.accept_license = true;
    allowUnfree = true;
  };

  overlays = [
    (import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz"))
  ];
} }: /*
based on
https://discourse.nixos.org/t/how-can-i-set-up-my-rust-programming-environment/4501/9
*/
let
  # Toolchain.
  gccVersion = "13";
  gccPkg = pkgs."gcc${gccVersion}"; # pkgs.wrapCCMulti $GCC
  llvmVersion = "17";
  llvmPkgs = pkgs."llvmPackages_${llvmVersion}";
  clangTools = pkgs."clang-tools_${llvmVersion}";
  clanggccPkg = pkgs.gccPkgAdapters.overrideCC llvmPkgs.gccPkg (llvmPkgs.clang.override {gccForLibs = gccPkg;});
  libTriple = "x86_64-unknown-linux-gnu";

  compilerLinks = pkgs.runCommand "clang-links" {} ''
    mkdir -p $out/bin
    mkdir -p $out/usr/${libTriple}/usr/lib/
    ln -s ${llvmPkgs.clang}/bin/clang $out/bin/clang-${llvmVersion}
    ln -s ${llvmPkgs.clang}/bin/clang++ $out/bin/clang++-${llvmVersion}
    ln -s ${llvmPkgs.llvm}/bin/llvm-as $out/bin/llvm-as-${llvmVersion}
    ln -s ${gccPkg.cc}/lib/gcc/${libTriple}/${gccPkg.cc.version}/crtbeginS.o $out/usr/${libTriple}/usr/lib/crtbeginS.o
    ln -s ${gccPkg.cc}/lib/gcc/${libTriple}/${gccPkg.cc.version}/crtendS.o $out/usr/${libTriple}/usr/lib/crtendS.o
  '';

  androidBuildToolsVersion = "30.0.3";
  androidCMakeVersion = "3.10.2";

  androidComposition = pkgs.androidenv.composeAndroidPackages {
    cmdLineToolsVersion = "8.0";
    toolsVersion = "26.1.1";
    platformToolsVersion = "35.0.2";
    buildToolsVersions = [ androidBuildToolsVersion ];
    includeEmulator = true;
    emulatorVersion = "35.2.5";
    platformVersions = [ "26" ];
    abiVersions = [ "x86" "x86_64" "arm64-v8a" "armeabi-v7a" ];
    includeSources = false;
    includeSystemImages = true;
    systemImageTypes = [ "google_apis_playstore" ];
    cmakeVersions = [ androidCMakeVersion ];
    includeNDK = true;
    ndkVersions = ["22.0.7026061"];
    useGoogleAPIs = false;
    includeExtras = [
      "extras;google;gcm"
    ];
  };

  androidsdk = androidComposition.androidsdk;

  dependencies = with pkgs; [
    # Dependencies
    fmt
    protobuf
    gtk3
    harfbuzz
    atk
    cairo
    pango
    gdk-pixbuf
    webkitgtk_4_1
    openssl
    wayland
    libxkbcommon
    libGL
    xorg.libX11
    SDL2
    SDL2_image
    egl-wayland
    eglexternalplatform
    alsa-lib
    udev
    vulkan-loader
    xorg.libX11
    xorg.libXrandr
    xorg.libXcursor
    xorg.libXi
    shaderc
    kdePackages.full
  ];

  nativeBuildInputs = with pkgs; [
    (lib.hiPrio llvmPkgs.bintools-unwrapped)
    lldb_17
    ninja
    cmake
    mold-wrapped

    # Toolchain
    deno
    nodejs_23
    yarn-berry
    android-tools
    # Do not use the clangd from this package as it does not work correctly with
    # stdlib headers.
    llvmPkgs.lld
    llvmPkgs.lldb

    # Compiler Links
    compilerLinks

    pkg-config
  ] ++ dependencies ++ [
    clangTools
  ];

  rustVersion = "2024-12-20";
  rust = pkgs.rust-bin.nightly.${rustVersion}.default.override {
    extensions = [
      "rust-src" # for rust-analyzer
      "rust-analyzer"
    ];

    targets = [
      "x86_64-unknown-linux-gnu"
      "x86_64-pc-windows-gnu"
      "i686-unknown-linux-gnu"
      "x86_64-apple-darwin"
      "aarch64-apple-darwin"
      "aarch64-unknown-linux-gnu"
      "aarch64-linux-android"
      "arm-linux-androideabi"
      "armv7-linux-androideabi"
      "i686-linux-android"
      "x86_64-linux-android"
      "aarch64-apple-ios"
      "x86_64-apple-ios"
    ];
  };

  hostPlatform = "x86_64-unknown-linux-gnu";
in pkgs.mkShell.override {
  stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.clangStdenv;
} {
  name = "clover-dev";

  _propagatePkgConfigDepends = false;

  buildInputs = [
    rust
    gccPkg.cc
    llvmPkgs.libclang
    androidsdk
  ] ++ (with pkgs; [
    clang
    glslang
  ]);

  shellHook = ''
    export LIBCLANG_PATH="${llvmPkgs.libclang.lib}/lib";
    export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath ([ gccPkg.cc.lib ] ++ dependencies)}";
    export RUST_BACKTRACE=1;
    # $(< ${gccPkg.cc}/nix-support/libc-crt1-cflags) $(< ${gccPkg.cc}/nix-support/libc-cflags) $(< ${gccPkg.cc}/nix-support/cc-cflags) $(< ${gccPkg.cc}/nix-support/libcxx-cxxflags)
    export BINDGEN_EXTRA_CLANG_ARGS="-idirafter ${gccPkg.cc}/lib/clang/${pkgs.lib.getVersion gccPkg.cc}/include ${pkgs.lib.optionalString gccPkg.cc.isGNU "-isystem ${gccPkg.cc}/include/c++/${pkgs.lib.getVersion gccPkg.cc} -isystem ${gccPkg.cc}/include/c++/${pkgs.lib.getVersion gccPkg.cc}/${hostPlatform} -idirafter ${gccPkg.cc}/lib/gcc/${hostPlatform}/${pkgs.lib.getVersion gccPkg.cc}/include"}"
    export RUSTFLAGS="-C link-arg=-fuse-ld=${pkgs.mold-wrapped}/bin/mold -Zshare-generics=y"
    export PATH="$(echo "$ANDROID_SDK_ROOT/cmake/${androidCMakeVersion}".*/bin):$PATH"
  '';

  GRADLE_OPTS = "-Dorg.gradle.project.android.aapt2FromMavenOverride=${androidsdk}/libexec/android-sdk/build-tools/${androidBuildToolsVersion}/aapt2";
  ANDROID_SDK_ROOT = "${androidsdk}/libexec/android-sdk";
  ANDROID_HOME = "${androidsdk}/libexec/android-sdk";
  ANDROID_NDK_ROOT = "${androidsdk}/libexec/android-sdk/ndk-bundle";

  inherit nativeBuildInputs;
}
