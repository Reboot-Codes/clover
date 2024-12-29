# SystemUI

The job of SystemUI (and its various APIs) is to create a structured higherachy of attention to ensure that the instance's user is not overwhelmed, and has complete control of Clover.

SystemUI has complete control over the content sent to displays through a direct connection to the GPU, or via a bus controlled by Modman. SystemUI will then compose a Composition of Views following X/Y/Z position resolved to relative to the origin of the display, blending modes when handling transparency, and more. SystemUI is heavily based on Bevy's ECS and learning the basics to how that works is suggested before working with CloverHub's default SystemUI implementation.

## Compositions

SystemUI as a compositor must analyze the positions of Views within the context of one or more displays. (The latter scenario of multiple displays being handled by one composition is known as a virtual display, similar to ones in desktop operating systems.) A composition has a context that holds the theme, view positions, and global user input. Each client on EvtBuzz that is authorized to do so may own an arbitrary number of top level Composition Views in which it holds an absolute control of context within said view. Drawing over a view is possible if authorized, and Users may choose to permit the view covered to be informed of this in its context.

## Views

Views are containers for graphics and context for a specific EvtBuzz Client and/or its inheritants. To save on system resources, views are split into the following types, ordered in least to most overhead.

### Composition Views

Composition views use the Component API to render canned graphics commands in commonly used manners (like Buttons, Prompts, etc) and should automatically respect the User's theme unless authorized. (Verified using 3rd party ratings on app listings.) Composition views use the least resources as they are using known values (even when using custom components) which makes rendering easier on SystemUI as it already uses this system internally when composing all top-level views and their subviews when possible. They may also compose other views within themselves as previously mentioned with no extra overhead other than the new content to be rendered with or without passed in context when authorized by the framed view (minus when used as a top level view in a SystemUI implementation, of course).

### Canvas Views

Canvas views act as a frame for direct graphics commands to the underlying Bevy library. For security, they are provided with their own ECS world which is composed into the one for the composition as a whole. They have more overhead than composition views as they must stream all operations to SystemUI through EvtBuzz (and Modman and/or the App Daemon if needed), but are less bandwidth-heavy than Stream Views as they rely on the GPU to expand the commands into actual graphics. It's possible to compose views inside of canvases using Tesseract to use a similar security flow to Compositon Views' Frames.

### Stream Views

Stream views are used when rendering on the GPU is not possible (e.g. video stream) and all frames must be manually sent to SystemUI. Obviously, this is the heaviest View and is discouraged in favor of Canvas Views when possible. Stream views may compose other views within themselves as authorized via a frame-by-frame stream, however, this should not be used to send View data externally. For that, use the Mirroring API.
