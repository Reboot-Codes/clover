# Master Users

| Name | Allowed Events From | Allowed Events To | Type |
|-|-|-|-|
| Master | `[".*"]` | `[".*"]` | `com.reboot-codes.clover.master` |
| EvtBuzz | `` | `` | `com.reboot-codes.clover.evtbuzz`|
| Arbiter | `` | `` | `com.reboot-codes.clover.arbiter` |
| Renderer | `` | `` | `com.reboot-codes.clover.renderer` |
| AppD | `` | `` | `com.reboot-codes.clover.appd` |
| ModMan | `` | `` | `com.reboot-codes.clover.modman` |
| Inference Engine | `` | `` | `com.reboot-codes.inference-engine` |

## Master User

The master user is like `root` on a Unix system, it has access to all facets of Clover. It's mostly a debug tool and access is determined by an environment variable: `CLOVER_MASTER_PRINT`. If set to `true`, the master user's credentials will be printed to the console if the log level for `clover::server` is `debug`.
