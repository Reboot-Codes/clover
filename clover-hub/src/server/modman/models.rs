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
        DisplayComponent,
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
use tokio::sync::Mutex;

// TODO: Define defaults via `Default` trait impl.

#[derive(Debug, Clone)]
pub struct Module {
  pub module_type: String,
  pub module_name: String,
  pub custom_name: Option<String>,
  pub initialized: bool,
  pub components: HashMap<String, Arc<(CloverComponentMeta, CloverComponent)>>,
  pub registered_by: String,
}

impl Module {
  pub fn get_name(self: &Self) -> String {
    match self.custom_name.clone() {
      Some(name) => name.clone(),
      None => self.module_name.clone(),
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
  DisplayComponent(DisplayComponent),
  VirtualDisplayComponent(VirtualDisplayComponent),
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
  pub config: Arc<Mutex<Config>>,
}

impl ModManStore {
  pub fn new(optional_config: Option<Arc<Mutex<Config>>>) -> Self {
    let config = match optional_config {
      Some(cfg) => cfg,
      None => Arc::new(Mutex::new(Config::default())),
    };

    ModManStore {
      modules: Arc::new(Mutex::new(HashMap::new())),
      config,
    }
  }
}
