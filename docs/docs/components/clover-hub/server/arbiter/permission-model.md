# Permission Model

Arbiter manages Clover's permission model. Due to the intimate nature of a Clover installation, Clover is built with attention to security and consent. This also means that all Modules, Applications, and Configuration Interfaces are constrained by an explicit declaration of intent before they're allowed to interface with [evtbuzz](/docs/components/clover-hub/server/evtbuzz/intro) and are only permitted to take specific actions as defined by that declaration.

Declarations are described in manifests. Everything is required to expose a manifest that can be accessed by CloverHub. Custom permissions can be registered and handled by arbiter as well for external APIs that use EvtBuzz.

## Permission types

| identifier |
|-|
| `com.reboot-codes.clover.arbiter.display.out` |
| `com.reboot-codes.clover.arbiter.video.in` |
| `com.reboot-codes.clover.arbiter.sensor.in` |
| `com.reboot-codes.clover.arbiter.sensor.out` |
| `com.reboot-codes.clover.arbiter.movement.in` |
| `com.reboot-codes.clover.arbiter.movement.out` |
| `com.reboot-codes.clover.arbiter.application.status` |
| `com.reboot-codes.clover.arbiter.application.register` |
| `com.reboot-codes.clover.arbiter.application.manage` |
| `com.reboot-codes.clover.arbiter.application.remove` |
| `com.reboot-codes.clover.arbiter.user.list` |
| `com.reboot-codes.clover.arbiter.user.add` |
| `com.reboot-codes.clover.arbiter.user.remove` |
| `com.reboot-codes.clover.arbiter.user.manage` |
| `com.reboot-codes.clover.arbiter.api-key.list` |
| `com.reboot-codes.clover.arbiter.api-key.keys` |
| `com.reboot-codes.clover.arbiter.api-key.add` |
| `com.reboot-codes.clover.arbiter.api-key.remove` |
| `com.reboot-codes.clover.arbiter.api-key.manage` |
| `com.reboot-codes.clover.arbiter.session.list` |
| `com.reboot-codes.clover.arbiter.session.add` |
| `com.reboot-codes.clover.arbiter.session.remove` |
| `com.reboot-codes.clover.arbiter.session.manage` |
| `com.reboot-codes.clover.arbiter.session.change-key` |
| `com.reboot-codes.clover.arbiter.client.list` |
| `com.reboot-codes.clover.arbiter.client.authorize` |
| `com.reboot-codes.clover.arbiter.client.unauthorize` |
| `com.reboot-codes.clover.arbiter.client.disconnect` |
