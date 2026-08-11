use std::sync::Arc;
use std::time::Duration;
use std::{
  collections::HashMap,
  thread::sleep as std_sleep,
};

use crate::server::modman::busses::proxies::group::can_2::CAN2Bus;

use nix::net::if_::if_nameindex;
use tokio::sync::mpsc::{
  unbounded_channel,
  UnboundedReceiver,
  UnboundedSender,
};
use tokio_util::sync::CancellationToken;
use tracing::{
  debug,
  error,
  info,
  instrument,
};

#[derive(Debug, Clone)]
pub enum CanLookoutEvent {
  IFaceCreate((String, u32)),
  IFaceDestroy(String),
}

#[instrument(skip(ctx))]
pub async fn can_lookout_thread(ctx: Arc<CAN2Bus>) {
  let (lookout_tx, lookout_rx) = unbounded_channel::<CanLookoutEvent>();

  let registrar_ctx = ctx.clone();
  let lookout_ctx = ctx.clone();

  // Something something tokio task pool is limited.
  std::thread::spawn(|| can_interface_lookout(lookout_ctx, lookout_tx));
  tokio::task::spawn(async move { can_bus_registrar(registrar_ctx, lookout_rx).await }).await;
}

/// Detects network interfaces to try and bind.
#[instrument(skip(ctx, channel))]
pub fn can_interface_lookout(ctx: Arc<CAN2Bus>, channel: UnboundedSender<CanLookoutEvent>) {
  let mut known_ifaces: HashMap<String, u32> = HashMap::new();
  let mut retries = 0;

  while !ctx.cancellation_token.is_cancelled() {
    match if_nameindex() {
      Ok(detected_ifaces) => {
        let mut ifaces_to_save = Vec::new();

        // Detect new interfaces, and mark known ones to be preserved.
        for detected_iface in detected_ifaces.iter() {
          let detected_iface_name = detected_iface.name().to_string_lossy().to_string();

          match known_ifaces.get(&detected_iface_name) {
            Some(_iface) => {
              ifaces_to_save.push(detected_iface_name);
            }
            None => {
              debug!("Detected new network interface: {}!", &detected_iface_name);
              match channel.send(CanLookoutEvent::IFaceCreate((
                detected_iface_name.clone(),
                detected_iface.index(),
              ))) {
                Ok(_) => {}
                Err(err) => {
                  error!("Failed to inform async registry thread that the '{}' interface was discovered, due to:\n{err}", detected_iface_name.clone());
                }
              }

              known_ifaces.insert(detected_iface_name.clone(), detected_iface.index());
              ifaces_to_save.push(detected_iface_name);
            }
          }
        }

        // Copy the current status into a new hashmap to avoid borrow errors.
        let mut known_ifaces_snapshot = HashMap::new();

        for known_iface in known_ifaces.iter() {
          known_ifaces_snapshot.insert(known_iface.0.clone(), known_iface.1.clone());
        }

        // Remove all interfaces that shouldn't be saved.
        for known_iface in known_ifaces_snapshot.iter() {
          let mut should_save = false;

          for iface_to_save in ifaces_to_save.iter() {
            if iface_to_save == known_iface.0 {
              should_save = true;
            }
          }

          if !should_save {
            debug!(
              "Interface: {}, dissapeared, removing it...",
              known_iface.0.clone()
            );
            known_ifaces.remove(known_iface.0);
            match channel.send(CanLookoutEvent::IFaceDestroy(known_iface.0.clone())) {
              Ok(_) => {}
              Err(err) => {
                error!("Failed to inform async registry thread that the '{}' interface was destroyed, due to:\n{err}", known_iface.0.clone());
              }
            }
          }
        }
      }
      Err(err) => {
        if retries == 5 {
          error!("Continously failed to get network interfaces even after initial check, did something happen to the network manager?");
          debug!("{err}");
          ctx.cancellation_token.cancel();
          break;
        } else {
          std_sleep(Duration::from_millis(500));
          retries += 1;
        }
      }
    }
  }
}

#[instrument(skip(ctx, channel))]
pub async fn can_bus_registrar(ctx: Arc<CAN2Bus>, mut channel: UnboundedReceiver<CanLookoutEvent>) {
  let mut bus_registry: HashMap<String, CancellationToken> = HashMap::new();
  // Prevent the mutex from being locked for the entire time we run the bus proxy.
  let config = ctx.store.config.lock().await;
  let can2_config = config.modman.group_busses.can_2.clone();
  drop(config);

  while !ctx.cancellation_token.is_cancelled() {
    if let Some(lookout_event) = channel.recv().await {
      match lookout_event {
        CanLookoutEvent::IFaceCreate(iface_details) => {
          let (iface_name, iface_index) = iface_details;

          for permitted_iface in can2_config.permitted_interfaces.clone() {
            if permitted_iface == iface_name {
              match bus_registry.get(&iface_name) {
                Some(_) => {}
                None => {
                  info!(
                    "Found configured interface: {}, starting up a CAN bus listener...",
                    iface_name.clone()
                  );
                }
              }
            }
          }
        }
        CanLookoutEvent::IFaceDestroy(iface_name) => todo!(),
      }
    }
  }
}
