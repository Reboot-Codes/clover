use std::collections::HashMap;

use serde::{
  Deserialize,
  Serialize,
};

use crate::server::modman::{
  busses::proxies::group::GroupBusConfigs,
  models::{
    components::{
      CloverComponent,
      CloverComponentMeta,
    },
    gestures::GestureStates,
    modules::Module,
  },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManConfig {
  /// All ports available for modman to use to connect to modules.
  pub uart_ports: Vec<String>,
  pub group_busses: GroupBusConfigs,
  /// Whether to restart paused gestures automatically on startup.
  pub restart_gestures: bool,
  pub gesture_states: HashMap<String, GestureStates>,
  pub gestures_bg_by_default: bool,
  pub static_modules: HashMap<String, Module>,
  pub static_components: HashMap<String, (CloverComponentMeta, CloverComponent)>,
}

impl Default for ModManConfig {
  /// Ensure that there is a display if the compositor was compiled in
  /// and there wasn't a display defined in the config/disabled explicitly.
  fn default() -> Self {
    let mut static_modules = HashMap::new();
    let mut static_components = HashMap::new();

    #[cfg(feature = "compositor")]
    {
      use std::num::NonZero;

      use crate::server::modman::{
        components::video::{
          displays::{
            self,
            models::{
              DirectConnection,
              DisplaySize,
              PhysicalDisplayComponent,
            },
          },
          VideoResolution,
        },
        connections::ModuleConnection,
      };

      let internal_display_id = uuid::Uuid::new_v4().to_string();
      let external_display_id = uuid::Uuid::new_v4().to_string();

      static_modules.insert(
        uuid::Uuid::new_v4().to_string(),
        Module {
          module_type: "com.reboot-codes.clover.debug-display".to_string(),
          module_name: "Debug Displays".to_string(),
          custom_name: Default::default(),
          initialized: false,
          components: vec![
            (internal_display_id.clone(), true),
            (external_display_id.clone(), true),
          ],
          registered_by: "com.reboot-codes.clover.modman.default".to_string(),
          connection: ModuleConnection::Simulated(
            "com.reboot-codes.clover.debug-display:0".to_string(),
          ),
        },
      );

      static_components.insert(
        internal_display_id.clone(),
        (
          CloverComponentMeta {
            name: "Debug Internal Display".to_string(),
            critical: true,
            location: "none".to_string(),
            base_gesture_parameters: HashMap::new(),
            internal: true,
          },
          CloverComponent::PhysicalDisplayComponent(PhysicalDisplayComponent {
            resolution: VideoResolution {
              width: NonZero::new(500).unwrap(),
              height: NonZero::new(200).unwrap(),
            },
            size: DisplaySize {
              height: Some(2.0),
              width: None,
            },
            connection: displays::models::ConnectionType::Direct(DirectConnection {
              display_id: "@primary".to_string(),
              windowed: true,
            }),
            virtual_display: None,
            gesture_config: None,
            internal: true,
          }),
        ),
      );

      static_components.insert(
        external_display_id.clone(),
        (
          CloverComponentMeta {
            name: "Debug External Display".to_string(),
            critical: true,
            location: "none".to_string(),
            base_gesture_parameters: HashMap::new(),
            internal: false,
          },
          CloverComponent::PhysicalDisplayComponent(PhysicalDisplayComponent {
            resolution: VideoResolution {
              width: NonZero::new(500).unwrap(),
              height: NonZero::new(200).unwrap(),
            },
            size: DisplaySize {
              height: Some(2.0),
              width: None,
            },
            connection: displays::models::ConnectionType::Direct(DirectConnection {
              display_id: "@primary".to_string(),
              windowed: true,
            }),
            virtual_display: None,
            gesture_config: None,
            internal: false,
          }),
        ),
      );
    }

    Self {
      static_components,
      static_modules,
      uart_ports: Default::default(),
      group_busses: Default::default(),
      restart_gestures: Default::default(),
      gesture_states: Default::default(),
      gestures_bg_by_default: Default::default(),
    }
  }
}
