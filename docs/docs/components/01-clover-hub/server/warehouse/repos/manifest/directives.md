# Directives

Manifests can make use of `@` directives which expand to automatically fill areas of manifest strings.

## @base

The top-level `base` key of a manifest can be set to define the RFQDN (reverse fully qualified domain name) of the manifest, which is useful to avoid continuously rewriting the repo's RFQDN. For example:

```json
{
  "base": "com.reboot-codes.clover-tutorial",
  "applications": {
    "@base.tutorial-application": { // resolves to `com.reboot-codes.clover-tutorial.tutorial-application`
      /* Rest of the application spec. */ 
    }
  }
}
```

## @builtin

`@builtin` resolves to the applicable built-in RFQDN for this area of the manifest. For CloverHub, this will usually resolve to `com.reboot-codes.clover`. Or, for CORE (like expression packs), `com.reboot-codes.clover.CORE`. You can also use built-in directives for specific domains:

- `@builtin:clover`: `com.reboot-codes.clover`
- `@builtin:core`: `com.reboot-codes.clover.CORE`

## @import

`@import()` uses `file:` resolution to locate and import other `.clover.jsonc` files. Globs are supported, but only when the key also has a glob. Paths must be wrapped in <code>\`</code>, `'`, or `"`.

### Import One File

When importing a single file as the value of a key, the root of the imported file is that key. For example:

```json title="repo/manifest.clover.jsonc"
{
  "base": "com.reboot-codes.clover-tutorial",
  "modules": "@import('./modules/manifest.clover.jsonc')"
}
```

```json title="repo/modules/manifest.clover.jsonc"
{
  "@base.tutorial-module": {
    // ... contents of the module spec.
  },
  // ... more modules
}
```

### Import Multiple Files

Importing multiple files can be done to automatically add manifest entries as a repository is updated. To do so, the key *and* the import directive must contain a single glob. For example:

```json title="repo/manifest.clover.jsonc"
{
  "base": "com.reboot-codes.clover-tutorial",
  "modules": {
    "@base.*": "@import('./modules/*/manifest.clover.jsonc')"
  }
}
```

```json title="repo/modules/tutorial-module-1/manifest.clover.jsonc"
{
  "name": "Tutorial Module 1",
  "components": {
    // ... static module component specs
  },
  // ... rest of the module
}
```

```json title="repo/modules/tutorial-module-2/manifest.clover.jsonc"
{
  "name": "Tutorial Module 2",
  "components": {
    // ... static module component specs
  },
  // ... rest of the module
}
```

And the ModMan Module Directory would list the following available static module configurations:

- `com.reboot-codes.clover-tutorial.tutorial-module-1`: Tutorial Module 1
- `com.reboot-codes.clover-tutorial.tutorial-module-2`: Tutorial Module 2
