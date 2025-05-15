# Renderer

Output independent, hardware accelerated, 2.5D renderer.

The renderer service is only responsible for creating, managing, destroying, and writing to an arbitrary number of OpenGL contexts who's frames are captured and sent to [displays](/docs/components/clover-hub/server/modman/modules/components/display) registered with modman when permitted by the user.
