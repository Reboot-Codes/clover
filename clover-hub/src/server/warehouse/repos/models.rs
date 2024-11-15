use std::collections::HashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use simple_error::SimpleError;
use os_path::OsPath;
use crate::server::appd::models::BuildConfig;
use super::{ResolutionCtx, resolve_entry_value, Resolution};

// TODO: Define defaults via `Default` trait impl.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequiredSingleManifestEntry<T> {
  Some(T),
  ImportString(String)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptionalSingleManifestSpecEntry<T> {
  Some(T),
  ImportString(String),
  None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptionalListManifestSpecEntry<T> {
  Some(HashMap<String, ManifestEntry<T>>),
  ImportString(String),
  None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequiredListManifestSpecEntry<T> {
  Some(HashMap<String, ManifestEntry<T>>),
  ImportString(String)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ManifestEntry<T> {
  RequiredSingleManifestEntry(RequiredSingleManifestEntry<T>),
  OptionalSingleManifestEntry(OptionalSingleManifestSpecEntry<T>),
  RequiredListManifestEntry(RequiredListManifestSpecEntry<T>),
  OptionalListManifestEntry(OptionalListManifestSpecEntry<T>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestSpec {
  pub name: Option<String>,
  pub version: String,
  pub base: Option<String>,
  pub modules: OptionalListManifestSpecEntry<RawModuleSpec>,
  pub applications: OptionalListManifestSpecEntry<RawApplicationSpec>,
  #[cfg(feature = "core")]
  pub expression_packs: OptionalListManifestSpecEntry<RawExpressionPackSpec>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawModuleSpec {
  pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawApplicationSpec {
  pub intents: OptionalListManifestSpecEntry<String>,
  pub containers: OptionalListManifestSpecEntry<RawContainerSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawContainerSpec {
  pub interface: OptionalSingleManifestSpecEntry<bool>,
  pub build: OptionalSingleManifestSpecEntry<BuildConfig>,
}

#[cfg(feature = "core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawExpressionPackSpec {
  pub name: Option<String>,
  pub expressions: RequiredListManifestSpecEntry<RawExpressionSpec>
}

#[cfg(feature = "core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RawExpressionSpec {
  RawStaticExpressionSpec(RawStaticExpressionSpec),
}

#[cfg(feature = "core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawStaticExpressionSpec {
  pub static_url: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
  pub name: Option<String>,
  pub version: String,
  pub base: Option<String>,
  pub modules: Option<HashMap<String, ModuleSpec>>,
  pub applications: Option<HashMap<String, ApplicationSpec>>,
  #[cfg(feature = "core")]
  pub expression_packs: Option<HashMap<String, ExpressionPackSpec>>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSpec {
  pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationSpec {
  pub intents: Option<HashMap<String, String>>,
  pub containers: Option<HashMap<String, ContainerSpec>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSpec {
  pub interface: Option<bool>,
  pub build: Option<BuildConfig>,
}

#[cfg(feature = "core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionPackSpec {
  pub name: Option<String>,
  pub expressions: HashMap<String, ExpressionSpec>
}

#[cfg(feature = "core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpressionSpec {
  StaticExpressionSpec,
}

#[cfg(feature = "core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticExpressionSpec {
  pub static_url: String
}

trait ManifestCompilationFrom<T> {
  fn compile(spec: T, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized;
}

pub enum KeyToResolve {
  Import
}

impl Manifest {
  fn resolve_list_entry<T, K>(raw_list: HashMap<String, String>, resolution_ctx: ResolutionCtx) -> Result<HashMap<String, K>, SimpleError> 
    where K: ManifestCompilationFrom<T>, T: for<'a> Deserialize<'a>
  {
    let mut err = None;
    let mut entries = HashMap::new();
    let glob_import_key_re = Regex::new("^(?<base>[^\\*\\n\\r]+)(\\*)$").unwrap();

    for (key, raw_entry) in raw_list {
      let is_glob = glob_import_key_re.is_match(&key);
      let mut entry_err = None;

      match resolve_entry_value(raw_entry, resolution_ctx) {
        Ok(resolution) => {
          match resolution {
            Resolution::ImportedSingle(raw_obj) => {
              if is_glob {
                err = Some(SimpleError::new("Resolved only one file for glob key import, import the root key instead!"));
              } else {
                match serde_jsonc::from_str::<T>(&raw_obj) {
                  Ok(obj_spec) => {
                    match K::compile(obj_spec, resolution_ctx.clone()) {
                      Ok(obj) => {
                        entries.insert(key.clone(), obj);
                      },
                      Err(e) => {
                        entry_err = Some(e);
                      }
                    } 
                  },
                  Err(e) => {
                    entry_err = Some(SimpleError::from(e));
                  }
                }
              }
            },
            Resolution::ImportedMultiple(raw_objs) => {
              if is_glob {
                for (obj_key_seg, raw_obj) in raw_objs {
                  match serde_jsonc::from_str::<T>(&raw_obj) {
                    Ok(obj_spec) => {
                      match K::compile(obj_spec, resolution_ctx.clone()) {
                        Ok(obj) => {
                          entries.insert([glob_import_key_re.captures(&key).unwrap().name("base").unwrap().as_str().to_string(), obj_key_seg].join("."), obj);
                        },
                        Err(e) => {
                          entry_err = Some(e);
                        }
                      }
                    },
                    Err(e) => {
                      entry_err = Some(SimpleError::from(e));
                    }
                  }
                }
              }
            }
          }
        },
        Err(e) => {
          err = Some(e);
        }
      }
    }

    match err {
      Some(e) => { Err(e) },
      None => { Ok(entries) }
    }
  }

  pub fn compile(spec: ManifestSpec, spec_path: OsPath) -> Result<Manifest, SimpleError> {
    let mut err = None;
    
    let name: Option<String> = match spec.name {
      Some(raw_spec_val) => {
        match resolve_entry_value(raw_spec_val, ResolutionCtx { base: spec.base.clone(), here: spec_path.clone()}) {
          Ok(name) => {
            match name {
              Resolution::ImportedSingle(val) => {
                Some(val)
              },
              Resolution::ImportedMultiple(_) => {
                err = Some(SimpleError::new("This field does not support glob imports."));
                None
              },
              Resolution::NoImport(val) => {
                Some(val)
              }
            }
          },
          Err(e) => {
            err = Some(e);
            None
          }
        }
      },
      None => { None }
    };

    let base: Option<String> = match spec.base {
      Some(raw_spec_val) => {
        match resolve_entry_value(raw_spec_val, ResolutionCtx { base: spec.base.clone(), here: spec_path.clone()}) {
          Ok(name) => {
            match name {
              Resolution::ImportedSingle(val) => {
                Some(val)
              },
              Resolution::ImportedMultiple(_) => {
                err = Some(SimpleError::new("This field does not support glob imports."));
                None
              },
              Resolution::NoImport(val) => {
                Some(val)
              }
            }
          },
          Err(e) => {
            err = Some(e);
            None
          }
        }
      },
      None => { None }
    };

    let modules: Option<HashMap<String, ModuleSpec>> = match spec.modules {
      OptionalListManifestSpecEntry::Some(raw_spec_val) => {
        
      },
      OptionalListManifestSpecEntry::ImportString(import_str) => {
        match resolve_entry_value(import_str, ResolutionCtx { base: spec.base.clone(), here: spec_path.clone()}) {
          Ok(name) => {
            match name {
              Resolution::ImportedSingle(val) => {
                match serde_jsonc::from_str::<HashMap<String, RawModuleSpec>>(&val) {
                  Ok(module_specs) => {

                  },
                  Err(e) => {
                    err = Some(SimpleError::from(e));
                    None
                  }
                }
              },
              Resolution::ImportedMultiple(_) => {
                err = Some(SimpleError::new("This field does not support glob imports."));
                None
              },
              Resolution::NoImport(val) => {
                match serde_jsonc::from_str::<HashMap<String, ModuleSpec>>(&val) {
                  Ok(module_specs) => {

                  },
                  Err(e) => {
                    err = Some(SimpleError::from(e));
                    None
                  }
                }
              }
            }
          },
          Err(e) => {
            err = Some(e);
            None
          }
        }
      },
      OptionalListManifestSpecEntry::None => { None }
    };

    // pub modules: OptionalListManifestSpecEntry<ModuleSpec>,
    // pub applications: OptionalListManifestSpecEntry<ApplicationSpec>,
    // #[cfg(feature = "core")]
    // pub expression_packs: OptionalListManifestSpecEntry<ExpressionPackSpec>

    match err {
      Some(e) => { Err(e) },
      None => { 
        Ok(Manifest {
          name,
          version: spec.version.clone(),
          base,
          modules
        })
      }
    }
  }
}
