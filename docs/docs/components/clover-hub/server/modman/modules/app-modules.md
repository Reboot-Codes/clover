# App Modules

An app module is a module provided by an app managed by the [app daemon](/docs/components/clover-hub/server/appd/intro), and is completely managed by it. The application manifest includes the module config instead of registering it with modman's module directory. This also means that app modules cannot be pre-configured, and will be configured after the app daemon starts up the application registering that module and its components.

This also means that user permission management for the module will actually be handled in the app flows instead of the module flows due to their dynamic nature. This is different from physical modules which are fully managed by modman and arbiter, and their configuration is handled in module flows.
