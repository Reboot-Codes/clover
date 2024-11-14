# Security

Supported modules are required to use an asynchronous signature for all communications with EvtBuzz. An asymmetric encryption key is registered during the registration flow <!-- TODO: add registration flow documentation --> to ensure that all messages are legitimate. 

This is due to Clover's security first design. *No security is not an option.*

## Security Levels

Level 1 and 2 are designed for simple modules that do not have movement components. Regardless, Level 3 or 4 (using post-quantum encryption algorithms) is suggested for production modules and especially for modules with movement components.

If a module does not use Post-Quantum Level 3 or 4 security and have a movement component, users will be warned of this fact using a non-dismissible UI component if the configuration application is CORE/Spanner compliant!

### Level 1

The async key provided to Clover is private to a single instance and should be deleted after registration, or be hidden during normal usage if that is not feasible. (such as when using an embossed QR code on a module without a built-in display.)

### Level 2

Clover will generate an async key and provide it to the module during the registration flow to ensure that messages *from* Clover are legitimate.

### Level 3

Similar to Level 2, however, all messages are encrypted using symmetric encryption, with the key attached to the message, encrypted using the private key of the transmitting party, a.k.a. Hybrid Cryptography.

### Level 4

Similar to Level 3, however, the asymmetric keys are changed constantly to ensure perfect forward secrecy.
