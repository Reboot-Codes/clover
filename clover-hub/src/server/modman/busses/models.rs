#[derive(Debug, Clone)]
pub enum BusTypes {
  App,
  #[cfg(feature = "can_fd")]
  CANFD,
  #[cfg(feature = "can_2")]
  CAN2,
  #[cfg(feature = "bt_classic")]
  BTClassic,
  #[cfg(feature = "bt_le")]
  BTLE,
  #[cfg(feature = "spi")]
  SPI,
  #[cfg(feature = "i2c")]
  I2C,
  #[cfg(feature = "uart")]
  UART
}

pub trait Bus {
  
}
