//! # Clover ModMan Data Structures
//!
//! [Modules](Module) are comprised of [Components](CloverComponent) and their [Metadata](CloverComponentMeta).
//!

use crate::server::{
  modman::{
    components::{
      audio::models::{
        AudioInputComponent,
        AudioOutputComponent,
      },
      movement::models::MovementComponent,
      sensors::models::{
        InputSensorComponent,
        OutputSensorComponent,
      },
      video::{
        cameras::models::CameraComponent,
        displays::models::{
          PhysicalDisplayComponent,
          VirtualDisplayComponent,
        },
      },
    },
    connections::ModuleConnection,
  },
  warehouse::config::models::Config,
};
use serde::{
  Deserialize,
  Serialize,
};
use std::collections::HashMap;
use std::sync::Arc;
use strum::VariantNames;
use tokio::sync::Mutex;

use super::components::models::CloverComponentTrait;

// TODO: Define defaults via `Default` trait impl.

/// Modules are comprised of [Components](CloverComponent) and their [Metadata](CloverComponentMeta).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
  /// RFQDN of the module definition from the manifest database.
  pub module_type: String,
  /// Manifest-defined name for this module (e.g. recognizable model number).
  pub module_name: String,
  /// User defined pretty name with manifest-defined default.
  pub custom_name: Option<String>,
  /// Has communication been established and self-test run?
  pub initialized: bool,
  /// Vec of Component IDs and if they're critical.
  pub components: Vec<(String, bool)>,
  /// Either `com.reboot-codes.clover.hub` or the RFQDN of the app that manages this module.
  pub registered_by: String,
  /// How is this module connected to modman?
  pub connection: ModuleConnection,
}

impl Module {
  pub fn get_name(self: &Self) -> String {
    match self.custom_name.clone() {
      Some(name) => name.clone(),
      Option::None => self.module_name.clone(),
    }
  }
}

/// Metadata for components, mostly useful for gesture configurations and security.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloverComponentMeta {
  /// Friendly name for this component to be shown to the User in any UI.
  pub name: String,
  /// Is this component required for the module to work? Default: yes.
  /// If any critical component fails to initalize, the module will fail to initalize entirely.
  pub critical: bool,
  /// Where this component is on/in the user. RFQDN formatted, e.g. `com.reboot-codes.CORE.head.eyes.internal` for a HUD display
  pub location: String,
  /// Parameters used for gesture events to synthesize commands to send to this component if it supports RX from Nexus.
  /// This is also used to determine if a gesture is supported by this component.
  /// Ignored if the component does not support recv.
  pub base_gesture_parameters: HashMap<String, GestureParameters>,
  /// If the component is internal, usually inferenced from the `location` parameter. Used by the permissions/privacy rules model.
  pub internal: bool,
}

/// Enum with all known clover component types, technically a valid "component" ([see the Component Trait](CloverComponentTrait)) itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloverComponent {
  AudioInputComponent(AudioInputComponent),
  AudioOutputComponent(AudioOutputComponent),
  MovementComponent(MovementComponent),
  InputSensorComponent(InputSensorComponent),
  OutputSensorComponent(OutputSensorComponent),
  CameraComponent(CameraComponent),
  PhysicalDisplayComponent(PhysicalDisplayComponent),
  VirtualDisplayComponent(VirtualDisplayComponent),
}

impl CloverComponentTrait for CloverComponent {
  /// Passes the context to the inner-component function implementation.
  async fn init(&mut self, store: Arc<ModManStore>) -> Result<(), anyhow::Error> {
    match self {
      CloverComponent::AudioInputComponent(component) => component.init(store.clone()).await,
      CloverComponent::AudioOutputComponent(component) => component.init(store.clone()).await,
      CloverComponent::MovementComponent(component) => component.init(store.clone()).await,
      CloverComponent::InputSensorComponent(component) => component.init(store.clone()).await,
      CloverComponent::OutputSensorComponent(component) => component.init(store.clone()).await,
      CloverComponent::CameraComponent(component) => component.init(store.clone()).await,
      CloverComponent::PhysicalDisplayComponent(component) => component.init(store.clone()).await,
      CloverComponent::VirtualDisplayComponent(component) => component.init(store.clone()).await,
    }
  }

  /// Passes the context to the inner-component function implementation.
  async fn deinit(&mut self, store: Arc<ModManStore>) -> Result<(), anyhow::Error> {
    match self {
      CloverComponent::AudioInputComponent(component) => component.deinit(store.clone()).await,
      CloverComponent::AudioOutputComponent(component) => component.deinit(store.clone()).await,
      CloverComponent::MovementComponent(component) => component.deinit(store.clone()).await,
      CloverComponent::InputSensorComponent(component) => component.deinit(store.clone()).await,
      CloverComponent::OutputSensorComponent(component) => component.deinit(store.clone()).await,
      CloverComponent::CameraComponent(component) => component.deinit(store.clone()).await,
      CloverComponent::PhysicalDisplayComponent(component) => component.deinit(store.clone()).await,
      CloverComponent::VirtualDisplayComponent(component) => component.deinit(store.clone()).await,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Turns a gesture intensity (-1.0 to 1.0) into a value that the component can understand in accordance with the following formula
/// ```rust
/// let value = (
///   (
///     (
///       (intensity + 1) * (max - min)
///     ) + min
///   ) * multiplier
/// ) + offset
/// ```
pub struct GestureParameters {
  pub min: f64,
  pub max: f64,
  pub multiplier: f64,
  pub offset: f64,
}

impl GestureParameters {
  pub fn calculate_intensity(&self, intensity: f64) -> f64 {
    return ((((intensity + 1.0) * (self.max - self.min)) + self.min) * self.multiplier)
      + self.offset;
  }
}

/// In memory data-store for components, modules, and any needed configuration.
#[derive(Debug, Clone)]
pub struct ModManStore {
  pub modules: Arc<Mutex<HashMap<String, Module>>>,
  pub components: Arc<Mutex<HashMap<String, Arc<(CloverComponentMeta, CloverComponent)>>>>,
  pub config: Arc<Mutex<Config>>,
  pub gesture_states: Arc<Mutex<HashMap<String, GestureStates>>>,
  pub foreground_gesture_priority: Arc<Mutex<Vec<String>>>,
  pub background_gesture_priority: Arc<Mutex<Vec<String>>>,
  /// Used for [Bus](super::busses::models::Bus) statuses, etc
  pub port_statuses: PortStatuses,
}

impl ModManStore {
  pub fn new(optional_config: Option<Arc<Mutex<Config>>>) -> Self {
    let config = match optional_config {
      Some(cfg) => cfg,
      Option::None => Arc::new(Mutex::new(Config::default())),
    };

    ModManStore {
      modules: Arc::new(Mutex::new(HashMap::new())),
      components: Arc::new(Mutex::new(HashMap::new())),
      gesture_states: Arc::new(Mutex::new(HashMap::new())),
      foreground_gesture_priority: Arc::new(Mutex::new(Vec::new())),
      background_gesture_priority: Arc::new(Mutex::new(Vec::new())),
      port_statuses: PortStatuses {
        uart: Arc::new(Mutex::new(HashMap::new())),
      },
      config,
    }
  }
}

/// Used for [Bus](super::busses::models::Bus) statuses, etc
#[derive(Debug, Clone)]
pub struct PortStatuses {
  /// Used by the [UART Bus](super::busses::proxies::uart::UARTBus)
  pub uart: Arc<Mutex<HashMap<String, PortStatus>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortStatus {
  /// Available but unused.
  #[serde(rename = "available")]
  Available,
  /// Requested by $MODULE_ID, but the UART bus isn't initalized yet
  #[serde(rename = "requested")]
  Requested(String),
  /// Currently being used by $MODULE_ID
  #[serde(rename = "bound")]
  Bound(String),
  /// Unavailable, but still requested by $MODULE_ID
  #[serde(rename = "unavailable")]
  Unavailable(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, VariantNames, PartialEq)]
#[serde(tag = "command")]
pub enum GestureState {
  #[serde(rename = "begin")]
  #[strum(serialize = "begin")]
  Begin {
    /// Multiplier over the value (y) axis of the gesture's smoothing curve.
    intensity: f64,
    /// Multiplier over the time (x) axis of the gesture's smoothing curve.
    speed: f64,
    /// Background gestures will be calculated before foreground gestures. Foreground gestures will always override background gestures.
    ///
    /// For example, you can have a background gesture that displays intermittent blinking, but you can start a temporary foreground gesture to show a different shape of the eyes to display an emotion.
    background: Option<bool>,
  },
  /// Resume calculation of module commands.
  ///
  /// This state is not saved at all.
  #[serde(rename = "unpause")]
  #[strum(serialize = "unpause")]
  UnPause,
  /// Temporarilly freeze the values that were already calculated for module commands as a result of this gesture.
  ///
  /// This state is not saved. Upon shutdown, if a foreground gesture is paused, the gesture will be removed from the stack upon shutdown; otherwise, background gestures will be saved in the `Begin` state to be automatically resumed upon startup.
  #[serde(rename = "pause")]
  #[strum(serialize = "end")]
  Pause,
  /// Removes the gesture from module command calculations entierly.
  #[serde(rename = "end")]
  #[strum(serialize = "end")]
  End,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GestureCommand {
  pub state: GestureState,
  pub auto_switch: Option<f64>,
  pub is_from_system: bool,
  pub areas: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureConfig {
  /// The primary gesture pack to use for this component
  pub primary_gesture_pack: Option<String>,
  /// The default gesture for the component to use when idle.
  pub default_gesture: Option<String>,
  pub gesture_parameters: Option<GestureParameters>,
  /// Override gestures (use different pack, adjust gesture parameters, etc)
  pub gesture_overrides: HashMap<String, GestureOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureOverride {
  /// Gesture id in `gesture_RFQDN@gesture_pack_RFQDN` format, where `@` and everything after can be ommitted to use the default gesture pack.
  pub gesture_preset_id: String,
  pub gesture_parameters: Option<GestureParameters>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManConfig {
  /// All ports available for modman to use to connect to modules.
  pub uart_ports: Vec<String>,
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

      use crate::server::modman::components::video::{
        displays::{
          self,
          models::{
            DirectConnection,
            DisplaySize,
          },
        },
        VideoResolution,
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
      restart_gestures: Default::default(),
      gesture_states: Default::default(),
      gestures_bg_by_default: Default::default(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureStates {
  /// Previous state of the gesture if we'd like to temporarily move back.
  pub prev_state: Option<GestureState>,
  /// Is this gesture paused?
  pub paused: bool,
  /// The current state of the gesture.
  pub current_state: GestureState,
  /// The next state to switch to (pre-loaded into Renderer and Modules if it loads resources), with a delay if we want to switch automatically.
  pub next_state: Option<(f64, GestureState)>,
}
