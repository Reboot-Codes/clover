# Server

The CloverHub server (specifically, EVTBuzz) runs, by default, on port 6699 (yes, you can say "nice", this was intentional), and provides a standardized, secure, websocket-based event bus for all of clover's components to use.

CloverHub also contains a bunch of extra stuff like:

- Arbiter: Permission model and authentication root of trust.
- Renderer: Output independent, 2.5D UI renderer.
- InferenceEngine: TPU manager for Machine Learning models.
- ModMan: Manage module registration and user configuration.
- AppDaemon (a.k.a. AppD): Manage containerized applications.
