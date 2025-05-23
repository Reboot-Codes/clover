# Applications

The application manifest is used to provide permission consent, and for the application daemon to know how to interface with the application in a higher level manner.

## Basics

The simplest application manifest for an app that only takes [basic input] and does not require any other access to resources like the network, sensors, external displays, internal displays other than the primary one (defined by the primary intent provider, this may not actually be a display when using something like a passthrough internal display, but this application is unaware of that context).

```json
{
  "version": "1.0.0",
  "applications": {
    "com.reboot-codes.clover.tutorial": {
      "name": "Tutorial Application",
      "intents": {
        "com.reboot-codes.clover.from-launcher": "ws-intent://@self/from-launcher"
      },
      "containers": {
        "main": {
          "interface": true,
          "build": {
            "url": "@here/Dockerfile"
          }
        }
      }
    }
  }
}
```

Permissions for writing to the primary app display segment, basic input, etc are provided automatically by the `com.reboot-codes.clover.from-launcher` intent. Specifically a `ws-intent`. Not the most optimized way to interface with an app for it's main intent connection, but certainly the simplest.
