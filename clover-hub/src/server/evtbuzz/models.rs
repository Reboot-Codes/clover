use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use crate::{server::{arbiter::models::{ApiKey, ApiKeyWithKeyWithoutUID, User}, warehouse::{config::models::Config, repos::models::Manifest}, modman::models::Module, appd::models::Application}, utils::{gen_api_key_with_check, gen_uid_with_check}};

// TODO: Define defaults via `Default` trait impl.

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
pub struct Session {
  pub start_time: String,
  pub end_time: String,
  pub api_key: String,
}

// TODO: Move User and API Key models to Arbiter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreUserConfig {
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

// TODO: Add serialization/deserialization functions...
// TODO: Add options for making certain models ephemeral or persistent.
#[derive(Debug, Clone)]
pub struct Store {
  pub config: Arc<Mutex<Config>>,
  pub users: Arc<Mutex<HashMap<String, User>>>,
  pub api_keys: Arc<Mutex<HashMap<String, ApiKey>>>,
  pub clients: Arc<Mutex<HashMap<String, Client>>>,
  pub messages: Arc<Mutex<HashMap<String, IPCMessage>>>,
  pub modules: Arc<Mutex<HashMap<String, Module>>>,
  pub applications: Arc<Mutex<HashMap<String, Application>>>,
  pub repos: Arc<Mutex<HashMap<String, Manifest>>>,
}

#[derive(Debug, Clone)]
pub struct CoreUserConfigs {
  pub evtbuzz: CoreUserConfig, 
  pub arbiter: CoreUserConfig, 
  pub renderer: CoreUserConfig, 
  pub appd: CoreUserConfig, 
  pub modman: CoreUserConfig, 
  pub inference_engine: CoreUserConfig,
  pub warehouse: CoreUserConfig
}

impl Store {
  pub fn new() -> Self {
    Store {
      config: Arc::new(Mutex::new(Default::default())),
      users: Arc::new(Mutex::new(HashMap::new())),
      api_keys: Arc::new(Mutex::new(HashMap::new())),
      clients: Arc::new(Mutex::new(HashMap::new())),
      messages: Arc::new(Mutex::new(HashMap::new())),
      modules: Arc::new(Mutex::new(HashMap::new())),
      applications: Arc::new(Mutex::new(HashMap::new())),
      repos: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  // Create a new store with a set master user.
  pub async fn new_configured_store() -> (Store, CoreUserConfig, CoreUserConfigs) {
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

  pub async fn create_master_user(self) -> CoreUserConfig {
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

    CoreUserConfig {
      id: master_user_id.clone(), 
      api_key: master_api_key.clone()
    }
  }

  /// Adds all the core user accounts, returns their configurations.
  pub async fn add_all_core_users(self) -> CoreUserConfigs {
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

    // Warehouse
    let warehouse_uid = gen_uid_with_check(&self).await;
    let warehouse_key = gen_api_key_with_check(&self).await;
    self.clone().add_user(UserConfig {
      user_type: "com.reboot-codes.clover.warehouse".to_string(),
      pretty_name: "Warehouse".to_string(),
      id: warehouse_uid.clone(),
      api_keys: vec![ApiKeyWithKeyWithoutUID {
        allowed_events_to: vec![".*".to_string()], 
        allowed_events_from: vec![".*".to_string()],
        key: warehouse_key.clone(),
        echo: true
      }]
    }).await;

    CoreUserConfigs {
      evtbuzz: CoreUserConfig {
        id: evtbuzz_uid.clone(),
        api_key: evtbuzz_key.clone()
      },
      arbiter: CoreUserConfig {
        id: arbiter_uid.clone(),
        api_key: arbiter_key.clone()
      },
      renderer: CoreUserConfig {
        id: renderer_uid.clone(),
        api_key: renderer_key.clone()
      },
      appd: CoreUserConfig {
        id: appd_uid.clone(),
        api_key: appd_key.clone()
      },
      modman: CoreUserConfig {
        id: modman_uid.clone(),
        api_key: modman_key.clone()
      },
      inference_engine: CoreUserConfig {
        id: inference_engine_uid.clone(),
        api_key: inference_engine_key.clone()
      },
      warehouse: CoreUserConfig {
        id: warehouse_uid.clone(),
        api_key: warehouse_key.clone()
      },
    }
  }
}
