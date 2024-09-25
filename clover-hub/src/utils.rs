use std::{hash::{DefaultHasher, Hash, Hasher}, ops::Deref};

use api_key::types::{ApiKeyResults, Default, StringGenerator};
use async_recursion::async_recursion;
use chrono::prelude::{DateTime, Utc};
use futures::{future::BoxFuture, FutureExt};
use uuid::Uuid;
use crate::server::evtbuzz::models::Store;

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

#[async_recursion]
async fn gen_api_key_with_check_with_set_key(store: &Store, src_key: String) -> String {
  let mut key = src_key;

  match store.api_keys.lock().await.get(&key.clone()) {
    Some(_) => {
      key = gen_api_key_with_check_with_set_key(store, gen_api_key()).await;
    },
    None => {}
  }

  key
}

/// Generates a new API key after checking that it is not currently in the Store.
pub async fn gen_api_key_with_check(store: &Store) -> String {
  gen_api_key_with_check_with_set_key(store, gen_api_key()).await
}

#[async_recursion]
async fn gen_uid_with_check_with_set_key(store: &Store, src_key: String) -> String {
  let mut key = src_key;

  match store.api_keys.lock().await.get(&key.clone()) {
    Some(_) => {
      key = gen_api_key_with_check_with_set_key(store, Uuid::new_v4().to_string()).await;
    },
    None => {}
  }

  key
}

/// Generates a new UID after checking that it is currently not in the Store.
pub async fn gen_uid_with_check(store: &Store) -> String {
  gen_uid_with_check_with_set_key(store, Uuid::new_v4().to_string()).await
}
