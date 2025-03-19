use super::config::models::Config;

pub struct WarehouseStore {
  pub repos: Arc<Mutex<HashMap<String, Manifest>>>,
  pub config: Config
}
