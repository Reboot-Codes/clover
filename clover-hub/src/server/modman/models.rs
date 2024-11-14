use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Module {
  pub module_type: String,
  pub pretty_name: String,
  pub initialized: bool,
  pub components: HashMap<String, Component>,
  pub registered_by: String,
}

// TODO: Finish module configuration struct
struct ModuleConfig {
  pub pretty_name: String,
  pub module_type: String,
}

#[derive(Debug, Clone)]
pub struct Component {
  pub component_type: String
}

// TODO: Add component types
