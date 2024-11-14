# App Modules

An app module is a module provided by an app managed by the [app daemon](/docs/components/clover-hub/server/appd/intro), and is completely managed by it. The application manifest includes allowed module permissions per module type. This also means that app modules cannot be pre-configured, and will be configured after the app daemon starts up the application registering that module and its components.

App modules are still managed in module flows, which can be started from the application side as well.
