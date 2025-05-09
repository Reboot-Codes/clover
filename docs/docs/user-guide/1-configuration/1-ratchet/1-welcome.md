# Welcome

Opening ratchet for the first time, a user will see the welcome screen, showing any last used connections, and/or the inital setup buttons.

## Inital Setup...

### From Scratch

This option will guide the user in choosing parts, fabrication, flashing, and then connecting to the new Clover instance.

#### Choosing Parts

The basics for a clover setup are:

- A control module (running [clover-hub](/docs/components/clover-hub/server/intro))
- A power source (an off the shelf battery pack can be used, or something custom, up to you, [CORE has examples])
- Some modules, depending on use-case, some examples include, ...
  - For Furries:
    - A head (examples with animatronic ears, eyes using displays, even facial micro-expressions are available, HUDs optional but suggested for use with smart-gloves)
    - Tail (moves, obviously)
    - Gloves (paws, these can be used to control clover without the need of a companion device)
  - Cosplayers
    - Headbands with ears/antennae
    - Tails
    - Light up accessories/props
  - For Cyborgs:
    - Wrist interfaces
    - Heads up displays
    - Externally facing displays
    - Sensors
    - Extra limbs
- And optionally, a companion device (running [Spanner](/docs/user-guide/configuration/spanner/intro)) for faster, on the fly configuration.

#### Building and Flashing

Some of these parts may need to be ordered and/or manually assembled, so CORE provides known working examples with 3D models and firmware if needed. Ratchet is capable of using system tools to flash firmware if needed, and picking out a set of compatible models if requested.

If parts were already chosen, Ratchet will compose a configuration for the new instance that is already informed of all the modules and extra parts the user (you!) chose to facilitate setup. The wizard will then move to trying to connect to the running instance automatically once flashing is completed.

### for A Running Instance

A connection is usually formed by Wi-Fi/Ethernet, Bluetooth, or Serial over USB. If the instance was just configured, authentication will most likely not be required (a key was generated and stored automatically), otherwise, a code will be output to the Console (available in the TUI, or via the control module's logging service like `journalctl`) for pairing.
