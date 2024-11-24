{ pkgs ? import <nixpkgs> {} }: /*
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
];

nativeBuildInputs = with pkgs;
  [
    llvmPkgs.bintools-unwrapped
    lldb_17
    ninja
    cmake

    # Toolchain

    # Do not use the clangd from this package as it does not work correctly with
    # stdlib headers.
    llvmPkgs.lld
    llvmPkgs.lldb

    # Compiler Links
    compilerLinks

    pkg-config
    gccPkg.cc
  ]
  ++ dependencies ++ [ clangTools ];
  rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
  rustVersion = "latest";
  #rustVersion = "1.62.0";
  rust = pkgs.rust-bin.nightly.${rustVersion}.default.override {
    extensions = [
      "rust-src" # for rust-analyzer
      "rust-analyzer"
    ];
  };

  hostPlatform = "x86_64-unknown-linux-gnu";
in pkgs.mkShell {
  name = "clover-dev";

  buildInputs = [
    rust
    gccPkg.cc
    llvmPkgs.libclang
  ] ++ (with pkgs; [
    clang
  ]);

  shellHook = ''
    export LIBCLANG_PATH="${llvmPkgs.libclang.lib}/lib";
    export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath ([ gccPkg.cc.lib ] ++ dependencies)}";
    export RUST_BACKTRACE=1;
    # $(< ${gccPkg.cc}/nix-support/libc-crt1-cflags) $(< ${gccPkg.cc}/nix-support/libc-cflags) $(< ${gccPkg.cc}/nix-support/cc-cflags) $(< ${gccPkg.cc}/nix-support/libcxx-cxxflags)
    export BINDGEN_EXTRA_CLANG_ARGS="-idirafter ${gccPkg.cc}/lib/clang/${pkgs.lib.getVersion gccPkg.cc}/include ${pkgs.lib.optionalString gccPkg.cc.isGNU "-isystem ${gccPkg.cc}/include/c++/${pkgs.lib.getVersion gccPkg.cc} -isystem ${gccPkg.cc}/include/c++/${pkgs.lib.getVersion gccPkg.cc}/${hostPlatform} -idirafter ${gccPkg.cc}/lib/gcc/${hostPlatform}/${pkgs.lib.getVersion gccPkg.cc}/include"}"
  '';

  inherit nativeBuildInputs;
}
