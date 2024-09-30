# Command Line Interface

## Logging

CloverHub uses [env_logger](https://docs.rs/env_logger/0.11.3/env_logger/) to provide formatted logging using the rust logging macros. The `CLOVER_LOG` environment variable can be used to adjust the log level of modules individually, or for the whole CLI. For example, during testing, a value such as `clover::server=debug` gives useful debugging info (like the master API key when `CLOVER_MASTER_PRINT` is `true`).

## Run the Server

```bash
clover run server
```

See: [Server Component](/docs/components/clover-hub/server/intro)

## Run the TUI

```bash
clover run tui
```

See: [TUI Component](/docs/components/clover-hub/tui/intro)

## Run Both

```bash
# you may also specify `aio`
clover run
```
