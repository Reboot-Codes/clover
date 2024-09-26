use log::info;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use std::sync::Arc;
use crate::server::evtbuzz::models::{IPCMessageWithId, Store, CoreUserConfig};

pub async fn renderer_main(ipc_tx: UnboundedSender<IPCMessageWithId>, ipc_rx: UnboundedReceiver<IPCMessageWithId>, store: Arc<Store>, user_config: Arc<CoreUserConfig>) {
    info!("Starting Renderer...");
    // TODO: Setup EGL ctx for each display we're handling.
}
