//! # Cameras
//!
//! Camera components take in a video stream from the outside for processing and/or reproduction. If you'd like to display video, please see the [display component](super::displays) docs. Camera components are managed by modman, CarbonFiber and Tesseract will handle this for you when given the proper permissions.
//!
//! Camera components can be defined as a block device via video4linux, or as an RTSP/RTMP stream that Clover is authorized to reproduce. If your component needs extra authentication, or a specific process to authenticate, create an application that performs those steps, then exposes one of those streams, then register the component with an [App Module](crate::server::modman::modules#configuration).
//!

pub mod impls;
pub mod models;
