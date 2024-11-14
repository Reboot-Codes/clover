# Manifest Files

`manifest.clover.jsonc` is the most important file for any repository that has a Clover component inside of it, be it an [Application](/docs/components/clover-hub/server/appd/application-manifest), [Module](/docs/components/clover-hub/server/modman/modules/intro), or [Expression Pack](/docs/components/CORE/expression-packs/intro) if you decide to use CORE.

Here's a basic manifest example for an App, Module, and an Expression pack:

```json
{
  "version": "1.0.0",
  "name": "Test Repo",
  "authors": [
    {
      "name": "Reboot/Fitz",
      "email": "hello@reboot-codes.com"
    }
  ],
  "base": "com.reboot-codes.clover-tutorial",
  "applications": {
    "@base.tutorial-app": {
      "name": "Tutorial App",
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
  },
  "modules": {
    "@base.tutorial-module": {
      "name": "Tutorial Module",
      "location": [
        "wrist"
      ],
      "components": {
        "wrist-light": {
          "type": "sensor",
          "input": "RGBA",
          "bus": {
            "can-fd": {
              "enabled": true
            }
          },
          "output": {
            "timing": "on-request",
            "format": "RGBA"
          }
        }
      }
    }
  },
  "expression-packs": {
    "@base.tutorial-expression-pack": {
      "name": "Tutorial Expression Pack",
      "expressions": {
        "com.reboot-codes.clover.CORE.neutral": {
          "static": "@here/neutral.png"
        }
      }
    }
  }
}
```
