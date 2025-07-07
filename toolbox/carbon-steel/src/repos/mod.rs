use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Repo {
  pub name: String,
  pub url: String,
  pub modules: HashMap<String, Module>,
  pub apps: HashMap<String, App>,
  pub gesture_packs: HashMap<String, GesturePack>,
}

#[derive(Debug, Clone)]
pub struct Module {
  pub name: String,
}

#[derive(Debug, Clone)]
pub struct App {
  pub name: String,
  pub installed: bool,
}

#[derive(Debug, Clone)]
pub struct GesturePack {
  pub name: String,
}
