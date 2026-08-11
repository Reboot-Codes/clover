use serde::{
  Deserialize,
  Serialize,
};

use crate::server::modman::connections::ModuleConnection;

/// Modules are comprised of [Components](CloverComponent) and their [Metadata](CloverComponentMeta).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
  /// RFQDN of the module definition from the manifest database.
  pub module_type: String,
  /// Manifest-defined name for this module (e.g. recognizable model number).
  pub module_name: String,
  /// User defined pretty name with manifest-defined default.
  pub custom_name: Option<String>,
  /// Has communication been established and self-test run?
  pub initialized: bool,
  /// Vec of Component IDs and if they're critical.
  pub components: Vec<(String, bool)>,
  /// Either `com.reboot-codes.clover.hub` or the RFQDN of the app that manages this module.
  pub registered_by: String,
  /// How is this module connected to modman?
  pub connection: ModuleConnection,
}

impl Module {
  pub fn get_name(self: &Self) -> String {
    match self.custom_name.clone() {
      Some(name) => name.clone(),
      Option::None => self.module_name.clone(),
    }
  }
}
