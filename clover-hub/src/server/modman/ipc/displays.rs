use std::sync::Arc;

use tokio_util::sync::CancellationToken;
use tracing::{
  debug,
  error,
  instrument,
};

use crate::server::{
  modman::{
    models::{
      CloverComponent,
      ModManStore,
    },
    MODULE_EVT_ID,
  },
  renderer::system_ui::AnyDisplayComponent,
};

#[instrument(skip(store, session, cancellation_token))]
pub async fn display_queryable(
  store: ModManStore,
  session: Arc<zenoh::Session>,
  cancellation_token: CancellationToken,
) {
  let key_expr = format!("{MODULE_EVT_ID}/components/by-type/video/displays/all");

  let queryable = session.declare_queryable(&key_expr).await.unwrap();

  debug!("Listening on {key_expr}!");
  while !cancellation_token.is_cancelled() {
    match queryable.recv_async().await {
      Ok(query) => {
        debug!("Replying with all displays...");

        let mut res = vec![];

        for (_module_id, module_config) in store.modules.lock().await.iter() {
          if module_config.initialized {
            for (component_id, _is_critical) in &module_config.components {
              match store.components.lock().await.get(component_id) {
                Some(component_entry) => {
                  let component_config = component_entry.1.clone();

                  match component_config {
                    CloverComponent::PhysicalDisplayComponent(physical_display_config) => {
                      res.push(AnyDisplayComponent::Physical(physical_display_config));
                    }
                    CloverComponent::VirtualDisplayComponent(virtual_display_config) => {
                      res.push(AnyDisplayComponent::Virtual(virtual_display_config));
                    }
                    _ => {}
                  }
                }
                None => todo!(),
              }
            }
          }
        }

        match query
          .reply(&key_expr, &serde_json_lenient::to_string(&res).unwrap())
          .await
        {
          Ok(_) => debug!("Successfully replied with all displays."),
          Err(err) => error!("Failed to send reply with all registered displays, due to:\n{err}"),
        }
      }
      Err(err) => {
        error!("{err}")
      }
    }
  }
}
