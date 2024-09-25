use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use super::evtbuzz::models::{IPCMessageWithId, Store};

pub async fn appd_main(ipc_tx: UnboundedSender<IPCMessageWithId>, ipc_rx: UnboundedReceiver<IPCMessageWithId>, store: Arc<Store>) {
    // TODO: Add docker crate to manage applications.
}
