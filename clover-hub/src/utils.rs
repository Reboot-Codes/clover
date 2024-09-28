use std::hash::{DefaultHasher, Hash, Hasher};
use api_key::types::{ApiKeyResults, Default, StringGenerator};
use chrono::prelude::{DateTime, Utc};
use uuid::Uuid;
use crate::server::evtbuzz::models::{CoreUserConfig, IPCMessageWithId, Store};
use tokio;

/// formats like "2001-07-08T00:34:60.026490+09:30"
pub fn iso8601(st: &std::time::SystemTime) -> String {
  let dt: DateTime<Utc> = st.clone().into();
  format!("{}", dt.format("%+"))
}

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
  let mut s = DefaultHasher::new();
  t.hash(&mut s);
  s.finish()
}

/// Generates a new api key. Please use [`gen_api_key_with_check`] to ensure that its' unique!
pub fn gen_api_key() -> String {
  let options = StringGenerator {
    prefix: "CLOVER:".to_string(),
    length: 50,
    ..StringGenerator::default()
  };
  let key: ApiKeyResults = api_key::string(options);
  
  match key {
    ApiKeyResults::String(res) => res,
    ApiKeyResults::StringArray(res_vec) => res_vec.join(""),
  }
}

/// Generates a new API key after checking that it is not currently in the Store.
pub async fn gen_api_key_with_check(store: &Store) -> String {
  loop {
    let api_key = gen_api_key();
    match store.api_keys.lock().await.get(&api_key.clone()) {
      Some(_) => {},
      None => {
        break api_key;
      }
    }
  }
}

/// Generates a new UID after checking that it is currently not in the Store.
pub async fn gen_uid_with_check(store: &Store) -> String {
  loop {
    let uid = Uuid::new_v4().to_string();
    match store.users.lock().await.get(&uid.clone()) {
      Some(_) => {},
      None => {
        break uid;
      }
    }
  }
}

pub async fn gen_message_id_with_check(store: &Store) -> String {
  loop {
    let message_id = Uuid::new_v4().to_string();
    match store.messages.lock().await.get(&message_id.clone()) {
      Some(_) => {},
      None => {
        break message_id;
      }
    }
  }
}

pub async fn gen_ipc_message(store: &Store, user_config: &CoreUserConfig, kind: String, message: String) -> IPCMessageWithId {
  let message_id = gen_message_id_with_check(&store.clone()).await;
  IPCMessageWithId { 
    id: message_id.clone(),
    author: user_config.id.to_string(),
    kind,
    message
  }
}

pub async fn gen_cid_with_check(store: &Store) -> String {
  loop {
    let client_id = Uuid::new_v4().to_string();
    match store.clients.lock().await.get(&client_id.clone()) {
      Some(_) => {},
      None => {
        break client_id;
      }
    }
  }
}

// TODO: Create a Util function for core users to send messages with their MasterUserConfig
