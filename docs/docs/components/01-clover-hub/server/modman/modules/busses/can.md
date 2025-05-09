# CAN

The Controller Area Network protocol is the suggested I/O protocol for most Clover implementations, and is used extensively in CORE along with Bluetooth LE. Available variants are:

- `can-2.0a`: CAN 2.0 using 11-bit identifiers.
- `can-2.0b`: CAN 2.0 using 29-bit identifiers, supports more devices on a single bus.
- `can-fd`: CAN with flexible frame sizes.
- `can-xl`: CAN with larger frame sizes, compatible with 2.0 and FD, and can support OSI networking. (Probably what you want when setting up Clover.)
