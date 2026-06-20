use serde::{
  Deserialize,
  Serialize,
};

/// The Bus Proxy this module is connected through.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModuleConnection {
  /// Device ID
  Simulated(String),
  App(AppConnection),
  /// Bus path and Device ID
  #[cfg(feature = "can_fd")]
  CANFD(CANFDConnection),
  /// Bus path and Device ID
  #[cfg(feature = "can_2")]
  CAN2(CAN2Connection),
  /// Device ID
  #[cfg(feature = "bt_classic")]
  BT(String),
  /// Device ID
  #[cfg(feature = "bt_le")]
  BTLE(String),
  /// Bus path and Device ID
  #[cfg(feature = "spi")]
  SPI(SPIConnection),
  /// Bus path and Device ID
  #[cfg(feature = "i2c")]
  I2C(I2CConnection),
  /// Bus path
  #[cfg(feature = "uart")]
  UART(UARTConnection),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConnection {
  pub app_id: String,
  pub device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CANFDConnection {
  pub bus_id: String,
  pub device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CAN2Connection {
  pub bus_id: String,
  pub device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SPIConnection {
  pub bus_id: String,
  pub device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I2CConnection {
  pub bus_id: String,
  pub device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UARTConnection {
  pub port: String,
  pub baud: u32,
}
