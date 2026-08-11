use std::{
  collections::HashMap,
  sync::Arc,
};

use serde::{
  Deserialize,
  Serialize,
};

use crate::server::modman::{
  components::{
    audio::models::{
      AudioInputComponent,
      AudioOutputComponent,
    },
    models::CloverComponentTrait,
    movement::models::MovementComponent,
    sensors::models::{
      IndicatorComponent,
      SensorComponent,
    },
    video::{
      cameras::models::CameraComponent,
      displays::models::{
        PhysicalDisplayComponent,
        VirtualDisplayComponent,
      },
    },
  },
  models::{
    gestures::GestureParameters,
    store::ModManStore,
  },
};

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
  SensorComponent(SensorComponent),
  IndicatorComponent(IndicatorComponent),
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
      CloverComponent::SensorComponent(component) => component.init(store.clone()).await,
      CloverComponent::IndicatorComponent(component) => component.init(store.clone()).await,
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
      CloverComponent::SensorComponent(component) => component.deinit(store.clone()).await,
      CloverComponent::IndicatorComponent(component) => component.deinit(store.clone()).await,
      CloverComponent::CameraComponent(component) => component.deinit(store.clone()).await,
      CloverComponent::PhysicalDisplayComponent(component) => component.deinit(store.clone()).await,
      CloverComponent::VirtualDisplayComponent(component) => component.deinit(store.clone()).await,
    }
  }
}
