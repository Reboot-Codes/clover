use std::collections::HashMap;

// TODO: Define defaults via `Default` trait impl.

#[derive(Debug, Clone)]
pub struct Module {
  pub module_type: String,
  pub pretty_name: String,
  pub initialized: bool,
  pub components: HashMap<String, Component>,
  pub registered_by: String,
}

#[derive(Debug, Clone)]
pub enum Component {
  Audio,
  Video,
  Sensor,
  Movement
}
