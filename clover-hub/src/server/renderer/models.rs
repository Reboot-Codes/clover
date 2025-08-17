use std::{
  collections::HashMap,
  num::NonZero,
};

use serde::{
  Deserialize,
  Serialize,
};

use crate::server::modman::components::video::{
  displays::models::{
    DisplayPosition,
    VirtualDisplayComponent,
  },
  VideoResolution,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererConfig {
  pub virtual_displays: HashMap<String, VirtualDisplayComponent>,
}

impl Default for RendererConfig {
  /// Ensure that there is a display if the compositor was compiled in
  /// and there wasn't a display defined in the config/disabled explicitly.
  fn default() -> Self {
    let mut virtual_displays = HashMap::new();

    #[cfg(feature = "compositor")]
    {
      let mut displays = HashMap::new();

      displays.insert("0".to_string(), DisplayPosition { x: 0.0, y: 0.0 });

      virtual_displays.insert(
        "debug_display".to_string(),
        VirtualDisplayComponent {
          displays,
          resolution: VideoResolution {
            height: NonZero::new(200).unwrap(),
            width: NonZero::new(500).unwrap(),
          },
          gesture_config: None,
        },
      );
    }

    Self { virtual_displays }
  }
}
