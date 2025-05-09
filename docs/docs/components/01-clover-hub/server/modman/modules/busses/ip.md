# IP

Using the IP stack is 100% possible, and is usually the simplest way to get started. Depending on the Clover setup, multiple variants may be available (these are regardless of how the NICs are connected to one another):

- `ip`: Use any OSI compliant networking.
- `ip-l3`: Use any network that has IP addresses available for routing. (This is probably what you want.)
- `ip-l3-external`: Using an externally managed L3 network where IP addresses are available for routing and are handed out by something that is not CloverHub or a CloverHub managed service.
- `ip-l3-internal`: In this situation, CloverHub is permitted to create its own L3 network, and it will handle DHCP leases, etc.
- `ip-l2`: No IP addresses are available, however, modules are connected that have defined MAC addresses.
