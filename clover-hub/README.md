# CloverHub

The main executable that orchestrates CLOVER's operation. Holds the server and management utils provided by the CLI.

You can just use `cargo build` to build for your system. Reproducible docker environments (using nix) coming soon!

Installation can be done with `cargo install --git https://github.com/Reboot-Codes/clover.git`!

Check [the docs](https://clover.reboot-codes.com/docs/clover-hub/intro) for a good breakdown.

## To-Do

- [ ] Check if using raylib and EGL binding crates are a better solution that using FFI directly.
