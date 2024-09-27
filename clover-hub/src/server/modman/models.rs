use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Module {
  pub module_type: String,
  pub pretty_name: String,
  pub initialized: bool,
  pub components: HashMap<String, Component>
}

#[derive(Debug, Clone)]
pub struct Component {
  pub component_type: String
}

// TODO: Add component types
