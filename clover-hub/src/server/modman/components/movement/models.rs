use crate::server::modman::{
  components::models::CloverComponentTrait,
  models::GestureParameters,
};
use serde::{
  Deserialize,
  Serialize,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position2D {
  x: f64,
  y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position3D {
  x: f64,
  y: f64,
  z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position4D {
  x: f64,
  y: f64,
  z: f64,

  u: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position5D {
  x: f64,
  y: f64,
  z: f64,

  u: f64,
  v: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position6D {
  x: f64,
  y: f64,
  z: f64,

  u: f64,
  v: f64,
  w: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DegreesOfFreedom {
  OneDegree(f64),
  TwoDegrees(Position2D),
  ThreeDegrees(Position3D),
  FourDegrees(Position4D),
  SixDegrees(Position6D),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureParamsPosition2D {
  x: GestureParameters,
  y: GestureParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureParamsPosition3D {
  x: GestureParameters,
  y: GestureParameters,
  z: GestureParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureParamsPosition4D {
  x: GestureParameters,
  y: GestureParameters,
  z: GestureParameters,

  u: GestureParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureParamsPosition5D {
  x: GestureParameters,
  y: GestureParameters,
  z: GestureParameters,

  u: GestureParameters,
  v: GestureParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureParamsPosition6D {
  x: GestureParameters,
  y: GestureParameters,
  z: GestureParameters,

  u: GestureParameters,
  v: GestureParameters,
  w: GestureParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DegreesOfFreedomGestureParams {
  OneDegree(GestureParameters),
  TwoDegrees(GestureParamsPosition2D),
  ThreeDegrees(GestureParamsPosition3D),
  FourDegrees(GestureParamsPosition4D),
  SixDegrees(GestureParamsPosition6D),
}

#[derive(Debug, Clone)]
pub struct MovementComponent {
  /// The inital position of this component, also determines how many degrees of freedom it uses. **NON OPTIONAL!**
  inital_position: DegreesOfFreedom,
  /// Gesture synthesis parameters to use for each coordinate value for each gesture.
  gesture_params: HashMap<String, DegreesOfFreedomGestureParams>,
}

impl CloverComponentTrait for MovementComponent {}
