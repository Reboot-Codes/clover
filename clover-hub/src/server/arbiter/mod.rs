use std::sync::Arc;
use log::info;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use super::evtbuzz::models::{IPCMessageWithId, CoreUserConfig, Store};

pub async fn arbiter_main(ipc_tx: UnboundedSender<IPCMessageWithId>, ipc_rx: UnboundedReceiver<IPCMessageWithId>, store: Arc<Store>, user_config: Arc<CoreUserConfig>) {
    info!("Starting Arbiter...");
}
