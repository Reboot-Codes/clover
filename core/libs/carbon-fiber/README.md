# CarbonFiber

Module and Component library for Clover.

## For Modules

Enable the `mcu` feature, and optionally `alloc` if you have an allocator. 
CarbonFiber includes many convinience features, especially for the officially supported bus types and configurations. 

For supported configurations and MCUs, most module code will be very simple and rely on the builder architecture. For example, a blinkie can be written with:

```rust
!#[no_std]

use carbon_fiber::module::*;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
  let p = embassy_rp::init(Default::default());
  let bus = CAN2::new(p.SPI0, p.PIN_16, p.PIN_17, p.PIN_18, p.PIN_19);

  let led = IndicatorComponent::new(0)
    .neopixel_pin(p.PIN_3, NeopixelConfig { num_pixels: 1 })
    .interpolate_linear();

  let (comms_worker, mut module_runner) = Module::init(bus, (led, stage1, stage2));

  spawner.spawn(comms_task(comms_worker)).unwrap();
  module_runner.run().await;
}
```

Note that this assumes that you're using a proper build system!

## For Hubs

While we would love if you use CloverHUB, you don't need to. CloverHUB *should* use the same exact types as found in CarbonFiber, and this can all be treated as a standardized, and versioned protocol.

Enable the `std` feature. CarbonFiber includes the exact types that components will be expecting and sending, so parsing should be pretty easy.
