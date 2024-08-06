use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::gen_api_key;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiKey {
  pub allowed_events_to: Vec<String>,
  pub allowed_events_from: Vec<String>,
  pub user_id: String,
}

impl ApiKey {
  pub fn to_api_key_with_key(self, key: String) -> ApiKeyWithKey {
    ApiKeyWithKey {
      key,
      allowed_events_to: self.allowed_events_to,
      allowed_events_from: self.allowed_events_from,
      user_id: self.user_id,
    }
  }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiKeyWithKey {
  pub key: String,
  pub allowed_events_to: Vec<String>,
  pub allowed_events_from: Vec<String>,
  pub user_id: String,
}

impl ApiKeyWithKey {
  pub fn to_api_key(self) -> ApiKey {
    self.into()
  }
}

impl Into<ApiKey> for ApiKeyWithKey {
  fn into(self) -> ApiKey {
    ApiKey {
      allowed_events_to: self.allowed_events_to,
      allowed_events_from: self.allowed_events_from,
      user_id: self.user_id,
    }
  }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IPCMessage {
  pub author: String,
  pub kind: String,
  pub message: String,
}

impl IPCMessage {
  pub fn to_message_with_id(self, id: String) -> IPCMessageWithId {
    IPCMessageWithId {
      id,
      author: self.author,
      kind: self.kind,
      message: self.message,
    }
  }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IPCMessageWithId {
  pub author: String,
  pub kind: String,
  pub message: String,
  pub id: String,
}

impl IPCMessageWithId {
  pub fn to_message(self) -> IPCMessage {
    self.into()
  }
}

impl Into<IPCMessage> for IPCMessageWithId {
  fn into(self) -> IPCMessage {
    IPCMessage {
      author: self.author,
      kind: self.kind,
      message: self.message,
    }
  }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Client {
  pub api_key: String,
  pub user_id: String,
  pub active: bool,
}

impl Client {
  pub fn to_client_with_id(self, id: String) -> ClientWithId {
    ClientWithId {
      id,
      api_key: self.api_key,
      user_id: self.user_id,
      active: self.active,
    }
  }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClientWithId {
  pub id: String,
  pub api_key: String,
  pub user_id: String,
  pub active: bool,
}

impl ClientWithId {
  pub fn to_client(self) -> Client {
    self.into()
  }
}

impl Into<Client> for ClientWithId {
  fn into(self) -> Client {
    Client {
      api_key: self.api_key,
      user_id: self.user_id,
      active: self.active,
    }
  }
}

#[derive(Debug, Clone)]
pub struct User {
  /// A vector of API keys associated with this user.
  pub api_keys: Vec<String>,
  pub sessions: Arc<Mutex<HashMap<String, Session>>>,
}

impl User {
  pub fn new() -> Self {
    User {
      api_keys: Vec::new(),
      sessions: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  pub fn to_user_with_id(self, id: String) -> UserWithId {
    UserWithId { 
      id, 
      api_keys: self.api_keys, 
      sessions: self.sessions,
    }
  }
}

#[derive(Debug, Clone)]
pub struct Session {
  pub start_time: String,
  pub end_time: String,
  pub api_key: String,
}

#[derive(Debug, Clone)]
pub struct UserWithId {
  pub id: String,
  pub api_keys: Vec<String>,
  pub sessions: Arc<Mutex<HashMap<String, Session>>>,
}

impl UserWithId {
  pub fn to_user(self) -> User {
    self.into()
  }
}

impl Into<User> for UserWithId {
  fn into(self) -> User {
    User {
      api_keys: self.api_keys,
      sessions: self.sessions
    }
  }
}

#[derive(Debug, Clone)]
pub struct Store {
  pub users: Arc<Mutex<HashMap<String, User>>>,
  pub api_keys: Arc<Mutex<HashMap<String, ApiKey>>>,
  pub clients: Arc<Mutex<HashMap<String, Client>>>,
  pub messages: Arc<Mutex<HashMap<String, IPCMessage>>>,
}

impl Store {
  pub fn new() -> Self {
    Store {
      users: Arc::new(Mutex::new(HashMap::new())),
      api_keys: Arc::new(Mutex::new(HashMap::new())),
      clients: Arc::new(Mutex::new(HashMap::new())),
      messages: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  // Create a new store with a set master user.
  pub async fn new_with_set_master_user(master_user_id: String) -> Self {
    let ret = Store::new();

    ret.clone().create_set_master_user(master_user_id).await;

    ret
  }

  pub async fn create_master_user(self) -> String {
    let master_user_id = Uuid::new_v4().to_string();

    self.create_set_master_user(master_user_id.clone()).await;

    master_user_id
  }

  pub async fn create_set_master_user(self, master_user_id: String) {
    let master_api_key = gen_api_key();

    self.users.lock().await.insert(master_user_id.clone(), User { api_keys: vec![master_api_key.clone()], sessions: Arc::new(Mutex::new(HashMap::new())) });
    self.api_keys.lock().await.insert(master_api_key.clone(), ApiKey { allowed_events_to: vec![".*".to_string()], allowed_events_from: vec![".*".to_string()], user_id: master_user_id.clone() });
  }
}
