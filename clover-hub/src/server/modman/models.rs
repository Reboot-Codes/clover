use crate::server::{
  modman::components::{
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

#[derive(Debug, Clone)]
pub struct Module {
  pub module_type: String,
  pub module_name: String,
  pub custom_name: Option<String>,
  pub initialized: bool,
  pub components: Vec<(String, bool)>,
  pub registered_by: String,
}

impl Module {
  pub fn get_name(self: &Self) -> String {
    match self.custom_name.clone() {
      Some(name) => name.clone(),
      Option::None => self.module_name.clone(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloverComponentMeta {
  pub name: String,
  /// Is this component required for the module to work? Default: yes.
  /// If any critical component fails to initalize, the module will fail to initalize entirely.
  pub critical: bool,
  /// Where this component is on/in the user. RFQDN formatted, e.g. `com.reboot-codes.CORE.head.eyes.internal`
  pub location: String,
  /// Parameters used for gesture events to synthesize commands to send to this component if it supports RX from Nexus.
  /// This is also used to determine if a gesture is supported by this component.
  /// Ignored if the component does not support recv.
  pub base_gesture_parameters: HashMap<String, GestureParameters>,
}

#[derive(Debug, Clone)]
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
  min: f64,
  max: f64,
  multiplier: f64,
  offset: f64,
}

#[derive(Debug, Clone)]
pub struct ModManStore {
  pub modules: Arc<Mutex<HashMap<String, Module>>>,
  pub components: Arc<Mutex<HashMap<String, Arc<(CloverComponentMeta, CloverComponent)>>>>,
  pub config: Arc<Mutex<Config>>,
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
      port_statuses: PortStatuses {
        uart: Arc::new(Mutex::new(HashMap::new())),
      },
      config,
    }
  }
}

#[derive(Debug, Clone)]
pub struct PortStatuses {
  pub uart: Arc<Mutex<HashMap<String, PortStatus>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortStatus {
  /// Available but unused.
  #[serde(rename = "available")]
  Available,
  /// Requested by $COMPONENT_ID, but the UART bus isn't initalized yet
  #[serde(rename = "requested")]
  Requested(String),
  /// Currently being used by $COMPONENT_ID
  #[serde(rename = "bound")]
  Bound(String),
  /// Unavailable, but still requested by $COMPONENT_ID
  #[serde(rename = "unavailable")]
  Unavailable(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, VariantNames)]
pub enum GestureState {
  #[serde(rename = "begin")]
  #[strum(serialize = "begin")]
  Begin,
  #[serde(rename = "continue")]
  #[strum(serialize = "continue")]
  Continue,
  #[serde(rename = "pause")]
  #[strum(serialize = "end")]
  Pause,
  #[serde(rename = "end")]
  #[strum(serialize = "end")]
  End,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GestureCommand {
  state: GestureState,
  intensity: f64,
  speed: f64,
  areas: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GestureConfig {
  Static(String),
  Reactive(ReactiveGestureConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactiveGestureConfig {
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModManConfig {
  pub uart_ports: Vec<(String, u32)>,
}
