use crate::server::modman::components::models::CloverComponent;
use serde::{
  Deserialize,
  Serialize,
};

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

#[derive(Debug, Clone)]
pub struct MovementComponent {
  /// The inital position of this component, also determines how many degrees of freedom it uses. **NON OPTIONAL!**
  inital_position: DegreesOfFreedom,
}

impl CloverComponent for MovementComponent {}
