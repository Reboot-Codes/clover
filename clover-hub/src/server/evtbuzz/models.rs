use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

use crate::utils::{gen_api_key_with_check, gen_uid_with_check};

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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IPCMessage {
  pub author: String,
  pub kind: String,
  pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IPCMessageWithId {
  pub author: String,
  pub kind: String,
  pub message: String,
  pub id: String,
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClientWithId {
  pub id: String,
  pub api_key: String,
  pub user_id: String,
  pub active: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterUserConfig {
  pub id: String,
  pub api_key: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
  pub user_type: String,
  pub pretty_name: String,
  pub id: String,
  pub api_keys: Vec<ApiKeyWithKeyWithoutUID>
}

// TODO: Add module models!!!!

// TODO: Create application manifest schema/models

// TODO: Add serialization/deserialization functions.... ough.
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
  pub async fn new_configured_store() -> (Store, MasterUserConfig, (MasterUserConfig, MasterUserConfig, MasterUserConfig, MasterUserConfig, MasterUserConfig, MasterUserConfig)) {
    let ret = Store::new();

    let master_user_config = ret.clone().create_master_user().await;
    let core_users_config = ret.clone().add_all_core_users().await;

    (ret, master_user_config, core_users_config)
  }

  pub async fn add_user(self, user_config: UserConfig) {
    let mut key_ids: Vec<String> = vec![];
    for key_config in user_config.api_keys.iter() { key_ids.push(key_config.key.clone()); };
    self.users.lock().await.insert(user_config.id.clone(), User { 
      pretty_name: user_config.pretty_name, 
      user_type: user_config.user_type, 
      api_keys: key_ids, 
      sessions: Arc::new(Mutex::new(HashMap::new()))
    });

    for key_config in user_config.api_keys.iter() {
      self.api_keys.lock().await.insert(key_config.key.clone(), ApiKey { 
        allowed_events_to: key_config.allowed_events_to.clone(), 
        allowed_events_from: key_config.allowed_events_from.clone(), 
        user_id: user_config.id.clone(), 
        echo: key_config.echo.clone()
      });
    };
  }

  pub async fn create_master_user(self) -> MasterUserConfig {
    let master_user_id = gen_uid_with_check(&self).await;
    let master_api_key = gen_api_key_with_check(&self).await;

    
    self.add_user(UserConfig {
      id: master_user_id.clone(), 
      pretty_name: "Master User".to_string(), 
      user_type: "com.reboot-codes.clover.master".to_string(), 
      api_keys: vec![ApiKeyWithKeyWithoutUID { 
        allowed_events_to: vec![".*".to_string()], 
        allowed_events_from: vec![".*".to_string()],
        key: master_api_key.clone(),
        echo: true
      }]
    }).await;

    MasterUserConfig {
      id: master_user_id.clone(), 
      api_key: master_api_key.clone()
    }
  }

  /// Adds all the core user accounts, returns their configurations in the following order:
  /// 
  /// - EvtBuzz
  /// - Arbiter
  /// - Renderer
  /// - AppD
  /// - ModMan
  /// - and Inference Engine
  pub async fn add_all_core_users(self) -> (MasterUserConfig, MasterUserConfig, MasterUserConfig, MasterUserConfig, MasterUserConfig, MasterUserConfig) {
    // EvtBuzz
    let evtbuzz_uid = gen_uid_with_check(&self).await;
    let evtbuzz_key = gen_api_key_with_check(&self).await;
    self.clone().add_user(UserConfig {
      user_type: "com.reboot-codes.clover.evtbuzz".to_string(),
      pretty_name: "EvtBuzz".to_string(),
      id: evtbuzz_uid.clone(),
      api_keys: vec![ApiKeyWithKeyWithoutUID {
        allowed_events_to: vec![".*".to_string()], 
        allowed_events_from: vec![".*".to_string()],
        key: evtbuzz_key.clone(),
        echo: true
      }]
    }).await;

    // Arbiter
    let arbiter_uid = gen_uid_with_check(&self).await;
    let arbiter_key = gen_api_key_with_check(&self).await;
    self.clone().add_user(UserConfig {
      user_type: "com.reboot-codes.clover.arbiter".to_string(),
      pretty_name: "Arbiter".to_string(),
      id: arbiter_uid.clone(),
      api_keys: vec![ApiKeyWithKeyWithoutUID {
        allowed_events_to: vec![".*".to_string()], 
        allowed_events_from: vec![".*".to_string()],
        key: arbiter_key.clone(),
        echo: true
      }]
    }).await;

    // Renderer
    let renderer_uid = gen_uid_with_check(&self).await;
    let renderer_key = gen_api_key_with_check(&self).await;
    self.clone().add_user(UserConfig {
      user_type: "com.reboot-codes.clover.renderer".to_string(),
      pretty_name: "Renderer".to_string(),
      id: renderer_uid.clone(),
      api_keys: vec![ApiKeyWithKeyWithoutUID {
        allowed_events_to: vec![".*".to_string()], 
        allowed_events_from: vec![".*".to_string()],
        key: renderer_key.clone(),
        echo: true
      }]
    }).await;

    // AppD
    let appd_uid = gen_uid_with_check(&self).await;
    let appd_key = gen_api_key_with_check(&self).await;
    self.clone().add_user(UserConfig {
      user_type: "com.reboot-codes.clover.appd".to_string(),
      pretty_name: "appd".to_string(),
      id: appd_uid.clone(),
      api_keys: vec![ApiKeyWithKeyWithoutUID {
        allowed_events_to: vec![".*".to_string()], 
        allowed_events_from: vec![".*".to_string()],
        key: appd_key.clone(),
        echo: true
      }]
    }).await;

    // ModMan
    let modman_uid = gen_uid_with_check(&self).await;
    let modman_key = gen_api_key_with_check(&self).await;
    self.clone().add_user(UserConfig {
      user_type: "com.reboot-codes.clover.modman".to_string(),
      pretty_name: "ModMan".to_string(),
      id: modman_uid.clone(),
      api_keys: vec![ApiKeyWithKeyWithoutUID {
        allowed_events_to: vec![".*".to_string()], 
        allowed_events_from: vec![".*".to_string()],
        key: modman_key.clone(),
        echo: true
      }]
    }).await;

    // Inference Engine
    let inference_engine_uid = gen_uid_with_check(&self).await;
    let inference_engine_key = gen_api_key_with_check(&self).await;
    self.clone().add_user(UserConfig {
      user_type: "com.reboot-codes.clover.inference-engine".to_string(),
      pretty_name: "Inference Engine".to_string(),
      id: inference_engine_uid.clone(),
      api_keys: vec![ApiKeyWithKeyWithoutUID {
        allowed_events_to: vec![".*".to_string()], 
        allowed_events_from: vec![".*".to_string()],
        key: inference_engine_key.clone(),
        echo: true
      }]
    }).await;

    (
      MasterUserConfig {
        id: evtbuzz_uid.clone(),
        api_key: evtbuzz_key.clone()
      },
      MasterUserConfig {
        id: arbiter_uid.clone(),
        api_key: arbiter_key.clone()
      },
      MasterUserConfig {
        id: renderer_uid.clone(),
        api_key: renderer_uid.clone()
      },
      MasterUserConfig {
        id: appd_uid.clone(),
        api_key: appd_key.clone()
      },
      MasterUserConfig {
        id: modman_uid.clone(),
        api_key: modman_key.clone()
      },
      MasterUserConfig {
        id: inference_engine_uid.clone(),
        api_key: inference_engine_key.clone()
      }
    )
  }
}
