use core::cell::Cell;

use carbon_fiber::{
  adoption::{
    hub::HubAdoptionContext,
    models::AdoptionState,
    module::ModuleAdoptionContext,
  },
  alerts::{
    models::components::{
      sensors::{
        AnalogResistorSensor,
        BinarySensor,
        CapacitiveSensor,
        RotaryEncoderSensor,
        SensorComponent,
        SensorDriver,
      },
      AlertPayload,
    },
    module::{
      components::AlertComponentPipeline,
      sensitivity::{
        EdgeTrigger,
        Sensitivity,
      },
    },
  },
  commands::{
    models::components::{
      indicators::{
        BinaryIndicator,
        IndicatorComponent,
      },
      movement::{
        MovementComponent,
        MovementTarget,
      },
      speaker::SpeakerComponent,
      ComponentTarget,
    },
    module::{
      interpolation::{
        CubicInterpolation,
        CustomInterpolation,
        Interpolation,
        LinearInterpolation,
        SmoothInterpolation,
      },
      CommandComponent,
      CommandComponentBuilder,
    },
  },
  module::{
    comms::can2::CAN2,
    Module,
  },
};

// Mock pin implementing embedded_hal::digital::InputPin and OutputPin
struct MockPin {
  pub high: bool,
}

impl embedded_hal::digital::ErrorType for MockPin {
  type Error = core::convert::Infallible;
}

impl embedded_hal::digital::InputPin for MockPin {
  fn is_high(&mut self) -> Result<bool, Self::Error> {
    Ok(self.high)
  }

  fn is_low(&mut self) -> Result<bool, Self::Error> {
    Ok(!self.high)
  }
}

impl embedded_hal::digital::OutputPin for MockPin {
  fn set_low(&mut self) -> Result<(), Self::Error> {
    self.high = false;
    Ok(())
  }

  fn set_high(&mut self) -> Result<(), Self::Error> {
    self.high = true;
    Ok(())
  }
}

#[test]
fn test_adoption_handshake() {
  let module_id = "blinky";
  let rfqdn = "com.reboot-codes.clover.CORE.blinky";

  let mut module = ModuleAdoptionContext::new(module_id, rfqdn);
  let mut hub = HubAdoptionContext::new(module_id, rfqdn, module.pin);

  assert_eq!(module.state, AdoptionState::Initial);
  assert_eq!(hub.state, AdoptionState::Initial);

  // 1. Hub sends Probe
  let probe = hub.gen_probe();
  assert!(module.parse_probe(&probe).is_ok());
  assert_eq!(module.state, AdoptionState::Probed);

  // 2. Module sends Hello
  let hello = module.gen_hello();
  assert!(hub.parse_hello(&hello).is_ok());
  assert_eq!(hub.state, AdoptionState::Probed);

  // 3. Hub sends KeyHOMI
  let homi = hub.gen_key_homi();
  assert!(module.parse_key_homi(&homi).is_ok());
  assert_eq!(module.state, AdoptionState::KeyExchanged);

  // 4. Module sends KeyHIMO
  let himo = module.gen_key_himo();
  assert!(hub.parse_key_himo(&himo).is_ok());
  assert_eq!(hub.state, AdoptionState::KeyExchanged);

  // 5. Hub sends Confirm
  let confirm = hub.gen_confirm();
  assert!(module.parse_confirm(&confirm).is_ok());
  assert_eq!(module.state, AdoptionState::Confirmed);
}

#[test]
fn test_module_build_and_dispatch() {
  let pin = MockPin { high: false };
  let mut indicator = IndicatorComponent::new(42, BinaryIndicator::new(pin, false));

  let (comms, mut worker) = Module::new("blinky", "com.reboot-codes.clover.CORE.blinky")
    .add_command_component(&mut indicator)
    .build(CAN2::new(()));

  assert_eq!(comms.adoption.module_id, "blinky");

  // Dispatch command to component 42
  let res = worker.dispatch_command(
    42,
    ComponentTarget::IndicatorBinary(true),
    100,
    None,
    [1u8; 16],
  );
  assert!(res.is_ok());

  // Dispatch to non-existent component should return error
  let res_err = worker.dispatch_command(
    99,
    ComponentTarget::IndicatorBinary(true),
    100,
    None,
    [1u8; 16],
  );
  assert!(res_err.is_err());

  // Step worker
  worker.step(50);
  worker.step(60);
}

#[test]
fn test_speaker_component() {
  let mut speaker = SpeakerComponent::new(1);
  assert_eq!(speaker.id(), 1);

  // Push tone command
  let push_res = speaker.on_target(
    ComponentTarget::SpeakerTone {
      frequency_hz: 440,
      volume_percent: 80,
    },
    100,
    None,
    [2u8; 16],
  );
  assert!(push_res.is_ok());

  // Step through transition
  let _ = speaker.step(50);
  assert_eq!(speaker.current_freq_hz, 440);
  assert_eq!(speaker.current_volume, 80);
  assert!(speaker.active);

  // Step to completion
  let telemetry = speaker.step(60);
  assert!(telemetry.is_some());
}

#[test]
fn test_movement_component() {
  let mut movement = MovementComponent::new(2);
  assert_eq!(movement.id(), 2);

  let push_res = movement.on_target(
    ComponentTarget::Movement(MovementTarget::TwoDof(100, 200)),
    100,
    None,
    [3u8; 16],
  );
  assert!(push_res.is_ok());

  // Step halfway
  let _ = movement.step(50);
  assert!(movement.positions[0] > 0);
  assert!(movement.positions[1] > 0);

  // Step to completion
  let _ = movement.step(60);
  assert_eq!(movement.positions[0], 100);
  assert_eq!(movement.positions[1], 200);
}

#[test]
fn test_sensor_binary() {
  let pin = MockPin { high: true };
  let sensor_driver = BinarySensor::new(pin, 0, false, EdgeTrigger::Both);
  let mut sensor = SensorComponent::new(10, sensor_driver, Sensitivity::Realtime);

  // Pin is high, realtime debounce = 0ms
  let alert = sensor.driver.poll_payload(&mut sensor.pipeline, 10);
  assert_eq!(alert, Some(AlertPayload::DigitalContact { pin_index: 0, active: true }));
}

#[test]
fn test_rotary_encoder_sensor() {
  let pin_a = MockPin { high: false };
  let pin_b = MockPin { high: false };
  let mut encoder = RotaryEncoderSensor::new(pin_a, pin_b);
  let mut pipeline = AlertComponentPipeline::new();

  // Quadrature transition: 00 -> 01 (forward step)
  encoder.pin_b.high = true;
  let payload = encoder.poll_payload(&mut pipeline, 10);
  assert_eq!(payload, Some(AlertPayload::SpatialTrigger { event_flags: 1, magnitude: 1 }));
}

#[test]
fn test_analog_resistor_sensor() {
  let val = Cell::new(50i16);
  let mut sensor = AnalogResistorSensor::new(|| val.get(), 5, 20, 80);
  let mut pipeline = AlertComponentPipeline::new();

  // First read: 50 -> delta 50 >= deadband 5, between low_trip and high_trip -> None
  let p1 = sensor.poll_payload(&mut pipeline, 10);
  assert_eq!(p1, None);

  // Trip high limit
  val.set(85);
  let p2 = sensor.poll_payload(&mut pipeline, 10);
  assert_eq!(p2, Some(AlertPayload::ThresholdCrossed { reading_raw: 85, threshold: 80 }));
}

#[test]
fn test_capacitive_sensor() {
  let pin = MockPin { high: true };
  let mut cap = CapacitiveSensor::new(pin, 3).with_threshold(100);
  let mut pipeline = AlertComponentPipeline::new().with_debounce(0);

  // No raw proximity, acts as digital contact pin
  let p = cap.poll_payload(&mut pipeline, 10);
  assert_eq!(p, Some(AlertPayload::DigitalContact { pin_index: 3, active: true }));

  // With threshold crossed
  cap.set_proximity(150);
  let p_thresh = cap.poll_payload(&mut pipeline, 10);
  assert_eq!(p_thresh, Some(AlertPayload::ThresholdCrossed { reading_raw: 150, threshold: 100 }));
}

#[test]
fn test_linear_interpolation() {
  let linear = LinearInterpolation::new();
  assert_eq!(linear.transform(0), 0);
  assert_eq!(linear.transform(65535), 65535);
  assert_eq!(linear.transform(32768), 32768);
  assert_eq!(linear.interpolate_u32(100, 200, 32768), 150);
  assert_eq!(linear.interpolate_u16(1000, 2000, 32768), 1500);
  assert_eq!(linear.interpolate_u8(10, 20, 32768), 15);
}

#[test]
fn test_smooth_interpolation_s_curve() {
  let smooth = SmoothInterpolation::new();
  assert_eq!(smooth.transform(0), 0);
  assert_eq!(smooth.transform(65535), 65535);

  // S-curve symmetry: at midpoint ~50%
  let mid = smooth.transform(32767);
  assert!((mid as i32 - 32767).abs() <= 50);

  // S-curve slow start (acceleration): at 10% time (6553), progress should be < 5% (3276)
  let early = smooth.transform(6553);
  assert!(early < 3276, "early was {}", early);

  // S-curve slow finish (deceleration): at 90% time (58981), progress should be > 95% (62258)
  let late = smooth.transform(58981);
  assert!(late > 62258, "late was {}", late);
}

#[test]
fn test_cubic_interpolation() {
  // Linear cubic: (0, 0) and (65535, 65535)
  let cubic_linear = CubicInterpolation::new(0, 0, 65535, 65535);
  assert_eq!(cubic_linear.transform(32768), 32768);

  // Fast ease-out: starts fast, ends slow
  let ease_out = CubicInterpolation::new(0, 0, 38010, 65535);
  assert_eq!(ease_out.transform(0), 0);
  assert_eq!(ease_out.transform(65535), 65535);
  assert!(ease_out.transform(32768) > 32768);
}

#[test]
fn test_custom_interpolation() {
  // Custom step curve: returns 0 if < 32768, otherwise 65535
  let step = CustomInterpolation::new(|p| if p < 32768 { 0 } else { 65535 });
  assert_eq!(step.transform(10000), 0);
  assert_eq!(step.transform(40000), 65535);
  assert_eq!(step.interpolate_u32(100, 200, 10000), 100);
  assert_eq!(step.interpolate_u32(100, 200, 40000), 200);
}

#[test]
fn test_movement_with_custom_interpolator() {
  // Create a movement component and configure with linear interpolation
  let mut movement = MovementComponent::new(3).interpolate_linear();
  let push_res = movement.on_target(
    ComponentTarget::Movement(MovementTarget::OneDof(100)),
    100,
    None,
    [4u8; 16],
  );
  assert!(push_res.is_ok());

  // Step halfway (50ms of 100ms) with linear interpolation -> exactly 50
  movement.step(50);
  assert_eq!(movement.positions[0], 50);

  // Step to completion -> exactly 100
  movement.step(60);
  assert_eq!(movement.positions[0], 100);
}

