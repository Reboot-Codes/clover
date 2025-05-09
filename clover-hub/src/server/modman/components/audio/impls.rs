use super::models::{
  AudioInputComponent,
  AudioOutputComponent,
  ConnectionType,
  DirectConnection,
};
use crate::server::modman::components::models::CloverComponentTrait;
use anyhow::anyhow;
use log::{
  debug,
  error,
};
use rodio::{
  cpal::{
    self,
    traits::HostTrait,
  },
  DeviceTrait,
};
use std::sync::Arc;

impl CloverComponentTrait for AudioInputComponent {
  async fn init(
    &mut self,
    store: Arc<crate::server::modman::models::ModManStore>,
  ) -> Result<(), anyhow::Error> {
    let mut ret = Ok(());

    match self.connection.clone() {
      super::models::ConnectionType::Direct(direct_connection) => {
        let host = cpal::default_host();

        match host.input_devices() {
          Ok(devices) => {
            let mut num_devices = 0;
            let mut found_device = None;

            for device in devices {
              num_devices += 1;

              match device.name() {
                Ok(name) => {
                  if direct_connection.device_id == name {
                    match device.supported_input_configs() {
                      Ok(input_configs) => {
                        let mut config_num = 0;

                        for conf in input_configs {
                          config_num += 1;
                          debug!("Supported Input config found: {:#?}", conf);
                        }

                        if config_num > 1 {
                          found_device = Some(name.clone());
                          self.connection = ConnectionType::Direct(DirectConnection {
                            device_id: name.clone(),
                            connection: Some(Arc::new(device)),
                          });
                        } else {
                          ret = Err(anyhow!("Supported input configs are ZERO, this isn't a valid device for an INPUT component."));
                        }
                      }
                      Err(e) => ret = Err(e.into()),
                    }
                  }
                }
                Err(e) => {
                  error!("Failed to get ALSA device id... let's hope that wasn't the one we wanted, kay?");
                  ret = Err(e.into());
                }
              }
            }

            if num_devices < 1 {
              ret = Err(anyhow!("No audio devices available!"));
            } else {
              match found_device {
                Some(_) => {}
                Option::None => match ret {
                  Ok(_) => {
                    ret = Err(anyhow!(
                      "Unable to find device {}, just doesn't exist!",
                      direct_connection.device_id
                    ));
                  }
                  Err(e) => {
                    ret = Err(anyhow::Error::from(e).context(format!(
                      "Unable to find device {}",
                      direct_connection.device_id
                    )));
                  }
                },
              }
            }
          }
          Err(e) => {
            error!("Unable to list ALSA audio devices. Are you sure ALSA is installed correctly?");
            ret = Err(e.into());
          }
        }
      }
      super::models::ConnectionType::ModManProxy(proxied_connection) => todo!(),
      super::models::ConnectionType::Stream(streaming_connection) => todo!(),
    }

    return ret;
  }

  async fn deinit(
    &mut self,
    store: Arc<crate::server::modman::models::ModManStore>,
  ) -> Result<(), anyhow::Error> {
    todo!()
  }
}

impl CloverComponentTrait for AudioOutputComponent {
  async fn init(
    &mut self,
    store: Arc<crate::server::modman::models::ModManStore>,
  ) -> Result<(), anyhow::Error> {
    let mut ret = Ok(());

    match self.connection.clone() {
      super::models::ConnectionType::Direct(direct_connection) => {
        let host = cpal::default_host();

        match host.output_devices() {
          Ok(devices) => {
            let mut num_devices = 0;
            let mut found_device = None;

            for device in devices {
              num_devices += 1;

              match device.name() {
                Ok(name) => {
                  if direct_connection.device_id == name {
                    match device.supported_output_configs() {
                      Ok(output_configs) => {
                        let mut config_num = 0;

                        for conf in output_configs {
                          config_num += 1;
                          debug!("Supported Output config found: {:#?}", conf);
                        }

                        if config_num > 1 {
                          found_device = Some(name.clone());
                          self.connection = ConnectionType::Direct(DirectConnection {
                            device_id: name.clone(),
                            connection: Some(Arc::new(device)),
                          });
                        } else {
                          ret = Err(anyhow!("Supported output configs are ZERO, this isn't a valid device for an OUTPUT component."));
                        }
                      }
                      Err(e) => ret = Err(e.into()),
                    }
                  }
                }
                Err(e) => {
                  error!("Failed to get ALSA device id... let's hope that wasn't the one we wanted, kay?");
                  ret = Err(e.into());
                }
              }
            }

            if num_devices < 1 {
              ret = Err(anyhow!("No audio devices available!"));
            } else {
              match found_device {
                Some(_) => {}
                Option::None => match ret {
                  Ok(_) => {
                    ret = Err(anyhow!(
                      "Unable to find device {}, just doesn't exist!",
                      direct_connection.device_id
                    ));
                  }
                  Err(e) => {
                    ret = Err(anyhow::Error::from(e).context(format!(
                      "Unable to find device {}",
                      direct_connection.device_id
                    )));
                  }
                },
              }
            }
          }
          Err(e) => {
            error!("Unable to list ALSA audio devices. Are you sure ALSA is installed correctly?");
            ret = Err(e.into());
          }
        }
      }
      super::models::ConnectionType::ModManProxy(proxied_connection) => todo!(),
      super::models::ConnectionType::Stream(streaming_connection) => todo!(),
    }

    return ret;
  }

  async fn deinit(
    &mut self,
    store: Arc<crate::server::modman::models::ModManStore>,
  ) -> Result<(), anyhow::Error> {
    todo!()
  }
}
