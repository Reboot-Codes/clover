use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::server::evtbuzz::models::Session;

#[derive(Debug, Clone)]
pub struct User {
  /// A vector of API keys associated with this user.
  pub api_keys: Vec<String>,
  pub sessions: Arc<Mutex<HashMap<String, Session>>>,
  pub user_type: String,
  pub pretty_name: String
}

impl User {
  pub fn to_user_with_id(self, id: String) -> UserWithId {
    UserWithId { 
      id, 
      api_keys: self.api_keys, 
      sessions: self.sessions,
      user_type: self.user_type,
      pretty_name: self.pretty_name
    }
  }
}

#[derive(Debug, Clone)]
pub struct UserWithId {
  pub id: String,
  pub api_keys: Vec<String>,
  pub sessions: Arc<Mutex<HashMap<String, Session>>>,
  pub user_type: String,
  pub pretty_name: String
}

impl Into<User> for UserWithId {
  fn into(self) -> User {
    User {
      api_keys: self.api_keys,
      sessions: self.sessions,
      user_type: self.user_type,
      pretty_name: self.pretty_name
    }
  }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiKey {
  pub allowed_events_to: Vec<String>,
  pub allowed_events_from: Vec<String>,
  pub user_id: String,
  pub echo: bool,
}

impl ApiKey {
  pub fn to_api_key_with_key(self, key: String) -> ApiKeyWithKey {
    ApiKeyWithKey {
      key,
      allowed_events_to: self.allowed_events_to,
      allowed_events_from: self.allowed_events_from,
      user_id: self.user_id,
      echo: self.echo,
    }
  }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiKeyWithKey {
  pub key: String,
  pub allowed_events_to: Vec<String>,
  pub allowed_events_from: Vec<String>,
  pub user_id: String,
  pub echo: bool,
}

impl Into<ApiKey> for ApiKeyWithKey {
  fn into(self) -> ApiKey {
    ApiKey {
      allowed_events_to: self.allowed_events_to,
      allowed_events_from: self.allowed_events_from,
      user_id: self.user_id,
      echo: self.echo,
    }
  }
}

impl Into<ApiKeyWithKeyWithoutUID> for ApiKeyWithKey {
  fn into(self) -> ApiKeyWithKeyWithoutUID {
    ApiKeyWithKeyWithoutUID {
      key: self.key,
      allowed_events_to: self.allowed_events_to,
      allowed_events_from: self.allowed_events_from,
      echo: self.echo,
    }
  }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiKeyWithKeyWithoutUID {
  pub key: String,
  pub allowed_events_to: Vec<String>,
  pub allowed_events_from: Vec<String>,
  pub echo: bool,
}
