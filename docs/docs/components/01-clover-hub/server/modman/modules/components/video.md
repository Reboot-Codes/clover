# Video

Video components take in a video stream from the outside for processing and/or reproduction. If you'd like to display video, please see the [display component docs](/docs/components/clover-hub/server/modman/modules/components/display). Video components are managed by modman, [CarbonFiber](/docs/components/toolbox/carbon-steel/intro) and [Tesseract](/docs/components/core/libs/tesseract/intro) will handle this for you when given the proper permissions.

Video components can be defined as a block device via video4linux, or as an RTSP/RTMP stream that Clover is authorized to reproduce. If your component needs extra authentication, or a specific process to authenticate, create an application that performs those steps, then exposes one of those streams, then register the component as an [app module](/docs/components/clover-hub/server/modman/modules/module-types/app-modules).
