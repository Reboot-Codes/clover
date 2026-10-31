//! # Interpolation Strategies for Command Components
//!
//! Interpolators transform normalized progress `0..=u16::MAX` (representing 0.0 to 1.0)
//! across component transition windows without floating-point math, ensuring efficiency
//! and deterministic execution on resource-constrained embedded microcontrollers.

/// Trait defining an easing curve and value interpolation.
pub trait Interpolation {
  /// Transforms normalized progress in `0..=u16::MAX` (representing 0.0 to 1.0)
  /// according to the interpolation curve, returning an eased factor in `0..=u16::MAX`.
  fn transform(&self, progress: u16) -> u16;

  /// Linearly interpolates a `u32` value from `start` to `end` using this curve's factor.
  fn interpolate_u32(&self, start: u32, end: u32, progress: u16) -> u32 {
    let factor = self.transform(progress) as u64;
    if end >= start {
      let diff = (end - start) as u64;
      start + (((diff * factor) + 32767) / 65535) as u32
    } else {
      let diff = (start - end) as u64;
      start - (((diff * factor) + 32767) / 65535) as u32
    }
  }

  /// Linearly interpolates a `u16` value from `start` to `end` using this curve's factor.
  fn interpolate_u16(&self, start: u16, end: u16, progress: u16) -> u16 {
    let factor = self.transform(progress) as u32;
    if end >= start {
      let diff = (end - start) as u32;
      start + (((diff * factor) + 32767) / 65535) as u16
    } else {
      let diff = (start - end) as u32;
      start - (((diff * factor) + 32767) / 65535) as u16
    }
  }

  /// Linearly interpolates a `u8` value from `start` to `end` using this curve's factor.
  fn interpolate_u8(&self, start: u8, end: u8, progress: u16) -> u8 {
    let factor = self.transform(progress) as u32;
    if end >= start {
      let diff = (end - start) as u32;
      start + (((diff * factor) + 32767) / 65535) as u8
    } else {
      let diff = (start - end) as u32;
      start - (((diff * factor) + 32767) / 65535) as u8
    }
  }
}

/// Evaluates a 1D cubic Bézier polynomial at parameter `u` in `[0..=65535]` with start `0`,
/// control points `p1`, `p2`, and end `65535`.
///
/// Formula: B(u) = (3 * (1-t)^2 * t * p1 + 3 * (1-t) * t^2 * p2 + t^3 * 1.0)
/// Scaled to integer arithmetic with zero floating-point operations.
fn sample_cubic(u: u16, p1: u16, p2: u16) -> u16 {
  const M: u128 = 65535;
  let u_val = u as u128;
  let w = M - u_val;
  let p1_val = p1 as u128;
  let p2_val = p2 as u128;
  let num = 3 * w * w * u_val * p1_val + 3 * w * u_val * u_val * p2_val + u_val * u_val * u_val * M;
  let denom = M * M * M;
  ((num + denom / 2) / denom).min(M) as u16
}

/// A custom cubic Bézier curve defined by control points `(x1, y1)` and `(x2, y2)`,
/// where coordinates are scaled to `0..=u16::MAX` (`0..=65535`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CubicInterpolation {
  pub x1: u16,
  pub y1: u16,
  pub x2: u16,
  pub y2: u16,
}

impl CubicInterpolation {
  pub const fn new(x1: u16, y1: u16, x2: u16, y2: u16) -> Self {
    CubicInterpolation { x1, y1, x2, y2 }
  }
}

impl Interpolation for CubicInterpolation {
  fn transform(&self, progress: u16) -> u16 {
    if progress == 0 {
      return 0;
    }
    if progress == 65535 {
      return 65535;
    }
    // Fast path: if X(u) == Y(u) everywhere (e.g. linear diagonal), progress maps directly.
    if self.x1 == self.y1 && self.x2 == self.y2 {
      return progress;
    }

    // 16-step binary search solves for parameter u in [0..=65535] where sample_cubic(u, x1, x2) ~= progress.
    let mut low: u32 = 0;
    let mut high: u32 = 65535;

    for _ in 0..16 {
      let mid = ((low + high) / 2) as u16;
      let x = sample_cubic(mid, self.x1, self.x2);
      if x < progress {
        low = mid as u32 + 1;
      } else {
        high = mid as u32;
      }
    }

    let u = low.min(65535) as u16;
    sample_cubic(u, self.y1, self.y2)
  }
}

/// Smooth interpolation using an S-curve cubic Bézier profile.
///
/// Ideal for actuators, motors, and components with physical momentum to ensure smooth
/// acceleration and deceleration without jerk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SmoothInterpolation {
  pub cubic: CubicInterpolation,
}

impl SmoothInterpolation {
  /// Create a new S-curve smoothing curve interpolation (cubic Bézier `(0.42, 0.0, 0.58, 1.0)`).
  pub const fn new() -> Self {
    // 0.42 * 65535 = 27525, 0.58 * 65535 = 38010
    SmoothInterpolation {
      cubic: CubicInterpolation::new(27525, 0, 38010, 65535),
    }
  }
}

impl Default for SmoothInterpolation {
  fn default() -> Self {
    Self::new()
  }
}

impl Interpolation for SmoothInterpolation {
  fn transform(&self, progress: u16) -> u16 {
    self.cubic.transform(progress)
  }
}

/// Linear interpolation without acceleration or deceleration curves.
///
/// Ideal for components whose rate of change is not affected by momentum,
/// such as LED dimmers, RGB indicators, and speaker volume setpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LinearInterpolation {
  pub cubic: CubicInterpolation,
}

impl LinearInterpolation {
  /// Create a new linear interpolation.
  pub const fn new() -> Self {
    LinearInterpolation {
      cubic: CubicInterpolation::new(0, 0, 65535, 65535),
    }
  }
}

impl Default for LinearInterpolation {
  fn default() -> Self {
    Self::new()
  }
}

impl Interpolation for LinearInterpolation {
  fn transform(&self, progress: u16) -> u16 {
    self.cubic.transform(progress)
  }
}

/// Custom interpolation wrapping a user-supplied closure or function.
///
/// The closure receives normalized tick progress in `0..=u16::MAX` (`0..=65535`)
/// and returns the desired eased factor in `0..=u16::MAX`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CustomInterpolation<F>(pub F);

impl<F: Fn(u16) -> u16> CustomInterpolation<F> {
  pub const fn new(f: F) -> Self {
    CustomInterpolation(f)
  }
}

impl<F: Fn(u16) -> u16> Interpolation for CustomInterpolation<F> {
  fn transform(&self, progress: u16) -> u16 {
    (self.0)(progress)
  }
}
