use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use super::evtbuzz::models::{IPCMessageWithId, Store};

pub async fn modman_main(ipc_tx: UnboundedSender<IPCMessageWithId>, ipc_rx: UnboundedReceiver<IPCMessageWithId>, store: Arc<Store>) {

}
