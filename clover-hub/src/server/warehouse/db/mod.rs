use super::models::WarehouseStore;
use core::time::Duration;
use log::info;
use sea_orm::{
  ConnectOptions,
  Database,
};
use std::sync::Arc;

pub async fn connect_db(store: WarehouseStore) -> Result<(), anyhow::Error> {
  // TODO: Support other DB drivers like Postgresql and MySQL/MariaDB
  let config = store.config.lock().await;
  let db_file_path = config.data_dir.join("db.sqlite");

  info!("Connecting to DB: {}...", db_file_path.clone());

  let mut opt = ConnectOptions::new(format!("sqlite://{}?mode=rwc", db_file_path));
  std::mem::drop(config);

  opt
    .max_connections(100)
    .min_connections(5)
    .connect_timeout(Duration::from_secs(8))
    .acquire_timeout(Duration::from_secs(8))
    .idle_timeout(Duration::from_secs(8))
    .max_lifetime(Duration::from_secs(8))
    .sqlx_logging(true)
    .sqlx_logging_level(log::LevelFilter::Info);

  match Database::connect(opt).await {
    Ok(db) => {
      info!("Connected to DB!");
      store.config.lock().await.db = Some(Arc::new(db));
      Ok(())
    }
    Err(e) => Err(e.into()),
  }
}
