//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
  let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());

  // By default, Cargo will re-run a build script whenever
  // any file in the project changes. By specifying `memory.x`
  // here, we ensure the build script is only re-run when
  // `memory.x` is changed.
  if env::var("CARGO_FEATURE_PICO_1").is_ok() {
    fs::write(out.join("memory.x"), include_bytes!("rp2040_memory.x")).unwrap();
  } else if env::var("CARGO_FEATURE_PICO_2").is_ok() {
    fs::write(out.join("memory.x"), include_bytes!("rp235x_memory.x")).unwrap();
  }

  println!("cargo:rustc-link-search={}", out.display());

  println!("cargo:rerun-if-env-changed=CARGO_FEATURE_PICO_1");
  println!("cargo:rerun-if-env-changed=CARGO_FEATURE_PICO_2");
  println!("cargo:rerun-if-changed=rp2040_memory.x");
  println!("cargo:rerun-if-changed=rp235x_memory.x");

  println!("cargo:rustc-link-arg-bins=--nmagic");
  println!("cargo:rustc-link-arg-bins=-Tlink.x");
  if env::var("CARGO_FEATURE_PICO_1").is_ok() {
    println!("cargo:rustc-link-arg-bins=-Tlink-rp.x");
  }
  println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}
