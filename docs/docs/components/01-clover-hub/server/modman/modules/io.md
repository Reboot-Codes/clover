# I/O

Depending on the Clover configuration different I/O schemes will be available to modules to use. Officially supported protocols are listed here, however, App modules have extra flexibility here by leveraging hardware access via docker.

## Protocols

These protocols have internal components within CloverHub to add support for them without writing an App module, or needing to manage these protocols manually when using an App module.

### IP

Using the IP stack is 100% possible, and is usually the simplest way to get started. Depending on the Clover setup, multiple variants may be available (these are regardless of how the NICs are connected to one another):

- `ip`: Use any OSI compliant networking.
- `ip-l3`: Use any network that has IP addresses available for routing. (This is probably what you want.)
- `ip-l3-external`: Using an externally managed L3 network where IP addresses are available for routing and are handed out by something that is not CloverHub or a CloverHub managed service.
- `ip-l3-internal`: In this situation, CloverHub is permitted to create its own L3 network, and it will handle DHCP leases, etc.
- `ip-l2`: No IP addresses are available, however, modules are connected that have defined MAC addresses.

### CAN

The Controller Area Network protocol is the suggested I/O protocol for most Clover implementations, and is used extensively in CORE along with Bluetooth LE. Available variants are:

- `can-2.0a`: CAN 2.0 using 11-bit identifiers.
- `can-2.0b`: CAN 2.0 using 29-bit identifiers, supports more devices on a single bus.
- `can-fd`: CAN with flexible frame sizes.
- `can-xl`: CAN with larger frame sizes, compatible with 2.0 and FD, and can support OSI networking. (Probably what you want when setting up Clover.)
