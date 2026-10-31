# CarbonFiber

Module and Component library for Clover.

## For Modules

Enable the `mcu` feature, and optionally `alloc` if you have an allocator. 
CarbonFiber includes many convinience features, especially for the officially supported bus types and configurations. 

For supported configurations and MCUs, most module code will be very simple and rely on the builder architecture.

## For Hubs

CloverHUB *should* use the same exact types as found in CarbonFiber, and this can all be treated as a standardized, and versioned protocol.

Enable the `std` feature. CarbonFiber includes the exact types that components will be expecting and sending, so parsing should be pretty easy.
