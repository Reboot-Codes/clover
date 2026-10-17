pub trait Interpolation {}

/// Smooth interpolation is best suited for movement components, or components where momentum matters to ensure target accuracy and efficency.
pub struct SmoothInterpolation {}

impl SmoothInterpolation {
  /// Create a new, default smoothing curve interpolation.
  pub fn new() -> Self {
    SmoothInterpolation {}
  }
}

impl Interpolation for SmoothInterpolation {}

/// Linear interpolation is best suited for components who's rate of change is not affected by momentum, for example, an LED-based IndicatorComponent.
pub struct LinearInterpolation {}

impl LinearInterpolation {
  pub fn new() -> Self {
    LinearInterpolation {}
  }
}

impl Interpolation for LinearInterpolation {}

/// This struct allows for a custom cubic bezier curve
pub struct CubicInterpolation {
  pub x1: u16,
  pub y1: u16,
  pub x2: u16,
  pub y2: u16,
}

impl CubicInterpolation {
  pub fn new(x1: u16, y1: u16, x2: u16, y2: u16) -> Self {
    CubicInterpolation { x1, y1, x2, y2 }
  }
}

impl Interpolation for CubicInterpolation {}

pub struct CustomInterpolation<F>(pub F);

impl<F> CustomInterpolation<F> {
  pub fn new(f: F) -> Self {
    CustomInterpolation(f)
  }
}

impl<F> Interpolation for CustomInterpolation<F> {}
