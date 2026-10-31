use crate::server::modman::models::gestures::{
  GestureConfig,
  GestureParameters,
};
use serde::{
  Deserialize,
  Serialize,
};
use strum::VariantNames;

#[derive(Debug, Clone, Serialize, Deserialize, VariantNames)]
pub enum ConnectionType {
  #[serde(rename = "modman-proxy")]
  #[strum(serialize = "modman-proxy")]
  ModManProxy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position2D {
  pub x: f64,
  pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position3D {
  pub x: f64,
  pub y: f64,
  pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position4D {
  pub x: f64,
  pub y: f64,
  pub z: f64,

  pub u: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position5D {
  pub x: f64,
  pub y: f64,
  pub z: f64,

  pub u: f64,
  pub v: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position6D {
  pub x: f64,
  pub y: f64,
  pub z: f64,

  pub u: f64,
  pub v: f64,
  pub w: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DegreesOfFreedom {
  OneDegree(f64),
  TwoDegrees(Position2D),
  ThreeDegrees(Position3D),
  FourDegrees(Position4D),
  FiveDegrees(Position5D),
  SixDegrees(Position6D),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureParamsPosition2D {
  pub x: GestureParameters,
  pub y: GestureParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureParamsPosition3D {
  pub x: GestureParameters,
  pub y: GestureParameters,
  pub z: GestureParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureParamsPosition4D {
  pub x: GestureParameters,
  pub y: GestureParameters,
  pub z: GestureParameters,

  pub u: GestureParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureParamsPosition5D {
  pub x: GestureParameters,
  pub y: GestureParameters,
  pub z: GestureParameters,

  pub u: GestureParameters,
  pub v: GestureParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureParamsPosition6D {
  pub x: GestureParameters,
  pub y: GestureParameters,
  pub z: GestureParameters,

  pub u: GestureParameters,
  pub v: GestureParameters,
  pub w: GestureParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DegreesOfFreedomGestureParams {
  OneDegree(GestureParameters),
  TwoDegrees(GestureParamsPosition2D),
  ThreeDegrees(GestureParamsPosition3D),
  FourDegrees(GestureParamsPosition4D),
  FiveDegrees(GestureParamsPosition5D),
  SixDegrees(GestureParamsPosition6D),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementComponent {
  /// The initial position of this component, also determines how many degrees of freedom it uses. **NON OPTIONAL!**
  pub initial_position: DegreesOfFreedom,
  pub gesture_config: GestureConfig,
  pub connection: ConnectionType,
}
