use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use std::sync::Arc;
use crate::server::evtbuzz::models::{IPCMessageWithId, Store};

pub async fn renderer_main(ipc_tx: UnboundedSender<IPCMessageWithId>, ipc_rx: UnboundedReceiver<IPCMessageWithId>, store: Arc<Store>) {
    // TODO: Setup EGL ctx for each display we're handling.
}
