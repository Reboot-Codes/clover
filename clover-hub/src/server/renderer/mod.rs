mod display;

use log::{debug, error, info};
use tokio::sync::{mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender}, Mutex};
use tokio_util::sync::CancellationToken;
use url::Url;
use std::{collections::HashMap, sync::Arc};
use crate::{server::evtbuzz::models::{CoreUserConfig, IPCMessageWithId, Store}, utils::send_ipc_message};
use display::{register_display, DisplaySpec};

pub async fn renderer_main(
  ipc_tx: UnboundedSender<IPCMessageWithId>, 
  mut ipc_rx: UnboundedReceiver<IPCMessageWithId>, 
  store: Arc<Store>, 
  user_config: Arc<CoreUserConfig>,
  cancellation_tokens: (CancellationToken, CancellationToken)
) {
  info!("Starting Renderer...");

  let display_map = Arc::new(Mutex::new(HashMap::new()));
  // let display_handles = Arc::new(HashMap::new());
  let (from_tx, mut from_rx) = unbounded_channel::<IPCMessageWithId>();
  let instance = Arc::new(wgpu::Instance::new(wgpu::InstanceDescriptor {
    backends: wgpu::Backends::PRIMARY,
    ..Default::default()
  }));
  let adapter = Arc::new(instance
    .request_adapter(&wgpu::RequestAdapterOptions {
      power_preference: wgpu::PowerPreference::default(),
      compatible_surface: None,
      force_fallback_adapter: false
    })
    .await
    .unwrap());
  
  let adapter_device = Arc::new(adapter
    .request_device(&Default::default(), None)
    .await
    .unwrap());

  let init_store = Arc::new(store.clone());
  let init_user = Arc::new(user_config.clone());
  let init_from_tx = from_tx.clone();
  cancellation_tokens.0.run_until_cancelled(async move {
    let _ = send_ipc_message(
      &init_store, 
      &init_user, 
      Arc::new(init_from_tx), 
      "clover://renderer.clover.reboot-codes.com/status".to_string(), 
      "finished-init".to_string()
    ).await;
  }).await;

  let ipc_recv_store = Arc::new(store.clone());
  let ipc_recv_user = Arc::new(user_config.clone());
  let all_displays_store = Arc::new(store.clone());
  let all_displays_user = Arc::new(user_config.clone());
  let ipc_recv_token = cancellation_tokens.0.clone();
  let display_token = cancellation_tokens.0.clone();
  let all_displays_tx = ipc_tx.clone();
  let ipc_recv_from_tx = from_tx.clone();
  let ipc_recv_device = adapter_device.clone();
  let ipc_recv_handle = tokio::task::spawn(async move {
    tokio::select! {
      _ = ipc_recv_token.cancelled() => {
        debug!("ipc_recv exited");
      },
      _ = async move {
        while let Some(msg) = ipc_rx.recv().await {
          let kind = Url::parse(&msg.kind.clone()).unwrap();

          // Verify that we care about this event.
          if kind.host().unwrap() == url::Host::Domain("renderer.clover.reboot-codes.com") {
            debug!("Processing: {}", msg.kind.clone());

            if kind.path() == "/register-display" {
              match serde_jsonc::from_str::<DisplaySpec>(&msg.message) {
                Ok(display_spec) => {
                  match register_display(
                    ipc_recv_device.clone(), 
                    display_map.clone(), 
                    display_spec.clone(), 
                    display_token.child_token()
                  ).await {
                    Ok(mut frames_rx) => {
                      info!("Registered display: {}!", display_spec.id.clone());
                      let _ = send_ipc_message(
                        &ipc_recv_store,
                        &ipc_recv_user,
                        Arc::new(ipc_recv_from_tx.clone()),
                        "clover://renderer.clover.reboot-codes.com/register-display/succeeded".to_string(), 
                        display_spec.id.clone()
                      ).await;

                      let frame_tx_token = display_token.child_token();
                      let display_store = all_displays_store.clone();
                      let display_user = all_displays_user.clone();
                      let display_tx = Arc::new(all_displays_tx.clone());
                      tokio::task::spawn(async move {
                        tokio::select! {
                          _ = async {
                            while let Some(frame) = frames_rx.recv().await {
                              let _ = send_ipc_message(
                                &display_store, 
                                &display_user, 
                                display_tx.clone(), 
                                format!("clover://renderer.clover.reboot-codes.com/display/{}/frame", display_spec.id.clone()),
                                serde_jsonc::to_string(&frame).unwrap()
                              );
                            }
                          } => {},
                          _ = frame_tx_token.cancelled() => {}
                        }
                      });
                    },
                    Err(e) => {
                      error!("Failed to register display, due to\n{}", e);
                      let _ = send_ipc_message(
                        &ipc_recv_store,
                        &ipc_recv_user,
                        Arc::new(ipc_recv_from_tx.clone()),
                        "clover://renderer.clover.reboot-codes.com/register-display/failed/registration".to_string(), 
                        serde_jsonc::to_string(&(display_spec.id.clone(), e.to_string())).unwrap()
                      ).await;
                    }
                  }
                },
                Err(e) => {
                  error!("Failed to register display, due to\n{}", e);
                  let _ = send_ipc_message(
                    &ipc_recv_store,
                    &ipc_recv_user,
                    Arc::new(ipc_recv_from_tx.clone()),
                    "clover://renderer.clover.reboot-codes.com/register-display/failed/parsing".to_string(), 
                    serde_jsonc::to_string(&e.to_string()).unwrap()
                  ).await;
                }
              }
            }
          }
        }
      } => {}
    }
  });

  let ipc_trans_token = cancellation_tokens.0.clone();
  let ipc_trans_tx = Arc::new(ipc_tx.clone());
  let ipc_trans_handle = tokio::task::spawn(async move {
    tokio::select! {
      _ = async move {
        while let Some(msg) = from_rx.recv().await {
          debug!("Sending message {} to IPC bus!", msg.kind.clone());
          match ipc_trans_tx.send(msg) {
            Ok(_) => {
              debug!("Sent message to IPC bus!");
            },
            Err(_) => {
              debug!("Failed to send message to IPC bus!");
            }
          }
        }
      } => {},
      _ = ipc_trans_token.cancelled() => {
        debug!("ipc_trans exited");
      }
    }
  });

  let cleanup_token = cancellation_tokens.0.clone();
  tokio::select! {
    _ = cleanup_token.cancelled() => {
      ipc_recv_handle.abort();
      ipc_trans_handle.abort();

      info!("Cleaning up displays...");
      // TODO: Clean up registered displays when server is shutting down.

      std::mem::drop(store);

      cancellation_tokens.1.cancel();
    }
  }

  info!("Renderer has stopped!");
}
