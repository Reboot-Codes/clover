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
        components::{
          sensors::models::{
            ConnectionType as SensorConnectionType,
            SensorComponent,
          },
          video::{
            displays::{
              self,
              models::{
                DisplaySize,
                PhysicalDisplayComponent,
              },
            },
            VideoResolution,
          },
        },
        connections::ModuleConnection,
      };

      static_modules.insert(
        "debug_display_module".to_string(),
        Module {
          module_type: "com.reboot-codes.clover.debug-display".to_string(),
          module_name: "Debug Displays".to_string(),
          custom_name: Default::default(),
          initialized: false,
          components: vec![
            ("debug_internal_display".to_string(), true),
            ("debug_external_display".to_string(), true),
          ],
          registered_by: "com.reboot-codes.clover.modman.default".to_string(),
          connection: ModuleConnection::Simulated(
            "com.reboot-codes.clover.debug-display:0".to_string(),
          ),
        },
      );

      static_components.insert(
        "debug_internal_display".to_string(),
        (
          CloverComponentMeta {
            name: "Debug Internal Display (HUD OLED)".to_string(),
            critical: true,
            location: "head.internal".to_string(),
            base_gesture_parameters: HashMap::new(),
            internal: true,
          },
          CloverComponent::PhysicalDisplayComponent(PhysicalDisplayComponent {
            resolution: VideoResolution {
              width: NonZero::new(128).unwrap(),
              height: NonZero::new(64).unwrap(),
            },
            size: DisplaySize {
              height: Some(0.96),
              width: None,
            },
            connection: displays::models::ConnectionType::Simulated,
            virtual_display: None,
            gesture_config: None,
            internal: true,
            target_fps: Some(30),
            mirror: Some(false),
          }),
        ),
      );

      static_components.insert(
        "debug_external_display".to_string(),
        (
          CloverComponentMeta {
            name: "Debug External Display (Eye LCD)".to_string(),
            critical: true,
            location: "head.eye.left".to_string(),
            base_gesture_parameters: HashMap::new(),
            internal: false,
          },
          CloverComponent::PhysicalDisplayComponent(PhysicalDisplayComponent {
            resolution: VideoResolution {
              width: NonZero::new(240).unwrap(),
              height: NonZero::new(240).unwrap(),
            },
            size: DisplaySize {
              height: Some(1.28),
              width: None,
            },
            connection: displays::models::ConnectionType::Simulated,
            virtual_display: None,
            gesture_config: None,
            internal: false,
            target_fps: Some(30),
            mirror: Some(false),
          }),
        ),
      );

      let sim_btn_ids = [
        ("sim_btn_up", "Simulated Button Up"),
        ("sim_btn_down", "Simulated Button Down"),
        ("sim_btn_left", "Simulated Button Left"),
        ("sim_btn_right", "Simulated Button Right"),
        ("sim_btn_1", "Simulated Button 1"),
        ("sim_btn_2", "Simulated Button 2"),
        ("sim_btn_back", "Simulated Button Back"),
        ("sim_btn_home", "Simulated Button Home"),
      ];

      static_modules.insert(
        "debug_controls_module".to_string(),
        Module {
          module_type: "com.reboot-codes.clover.simulated-controls".to_string(),
          module_name: "Simulated Controls".to_string(),
          custom_name: Default::default(),
          initialized: false,
          components: sim_btn_ids
            .iter()
            .map(|(id, _)| (id.to_string(), false))
            .collect(),
          registered_by: "com.reboot-codes.clover.modman.default".to_string(),
          connection: ModuleConnection::Simulated(
            "com.reboot-codes.clover.simulated-controls:0".to_string(),
          ),
        },
      );

      for (id, name) in sim_btn_ids {
        static_components.insert(
          id.to_string(),
          (
            CloverComponentMeta {
              name: name.to_string(),
              critical: false,
              location: "control_panel".to_string(),
              base_gesture_parameters: HashMap::new(),
              internal: true,
            },
            CloverComponent::SensorComponent(SensorComponent {
              connection: SensorConnectionType::Simulated,
            }),
          ),
        );
      }
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
