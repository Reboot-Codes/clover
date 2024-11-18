use std::collections::HashMap;
use serde::Deserialize;
use simple_error::SimpleError;
use os_path::OsPath;
use super::{models::*, resolve_entry_value, resolve_list_entry};

impl ManifestCompilationFrom<Option<String>> for OptionalString {
  fn compile(spec: Option<String>, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized {
    let mut err = None;

    let res = match spec.clone() {
      Some(raw_str) => {
        match resolve_entry_value(raw_str, resolution_ctx.clone()) {
          Ok(resolution) => {
            match resolution {
              Resolution::ImportedMultiple(_) => {
                err = Some(SimpleError::new("Glob import not supported at this level!"));
                OptionalString::None
              },
              Resolution::ImportedSingle(imported) => {
                match serde_jsonc::from_str::<String>(&imported) {
                  Ok(val) => {
                    OptionalString::Some(val)
                  },
                  Err(e) => {
                    err = Some(SimpleError::from(e));
                    OptionalString::None
                  }
                }
              },
              Resolution::NoImport(val) => {
                OptionalString::Some(val)
              }
            }
          },
          Err(e) => {
            err = Some(e);
            OptionalString::None
          }
        }
      },
      None => { OptionalString::None }
    };

    match err {
      Some(e) => { Err(e) },
      None => { Ok(res) }
    }
  }
}

impl ManifestCompilationFrom<String> for RequiredString {
  fn compile(spec: String, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized {
    let mut err = None;

    let res = match resolve_entry_value(spec.clone(), resolution_ctx.clone()) {
      Ok(resolution) => {
        match resolution {
          Resolution::ImportedMultiple(_) => {
            err = Some(SimpleError::new("Glob import not supported at this level!"));
            RequiredString(String::new())
          },
          Resolution::ImportedSingle(imported) => {
            match serde_jsonc::from_str::<String>(&imported) {
              Ok(val) => {
                RequiredString(val)
              },
              Err(e) => {
                err = Some(SimpleError::from(e));
                RequiredString(String::new())
              }
            }
          },
          Resolution::NoImport(val) => {
            RequiredString(val)
          }
        }
      },
      Err(e) => {
        err = Some(e);
        RequiredString(String::new())
      }
    };

    match err {
      Some(e) => { Err(e) },
      None => { Ok(res) }
    }
  }
}

impl ManifestCompilationFrom<OptionalStringListManifestSpecEntry> for OptionalStrStrHashMap {
  fn compile(spec: OptionalStringListManifestSpecEntry, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized {
    let mut err = None;

    let ret: OptionalStrStrHashMap = match spec.clone() {
      OptionalStringListManifestSpecEntry::Some(raw_intents) => {
        let mut entries = HashMap::new();

        for (intent_id, raw_intent) in raw_intents {
          match resolve_entry_value(raw_intent.try_into().unwrap(), resolution_ctx.clone()) {
            Ok(resolution) => {
              match resolution {
                Resolution::ImportedMultiple(_) => {
                  err = Some(SimpleError::new("Glob import not supported at this level!"));
                  break;
                },
                Resolution::ImportedSingle(imported) => {
                  match serde_jsonc::from_str::<String>(&imported) {
                    Ok(val) => {
                      entries.insert(intent_id, val);
                    },
                    Err(e) => {
                      err = Some(SimpleError::from(e));
                      break;
                    }
                  }
                },
                Resolution::NoImport(val) => {
                  entries.insert(intent_id, val);
                }
              }
            },
            Err(e) => {
              err = Some(e);
              break
            }
          }
        }

        match err {
          Some(_) => {
            OptionalStrStrHashMap::None
          },
          None => {
            OptionalStrStrHashMap::Some(entries)
          }
        }
      },
      OptionalStringListManifestSpecEntry::ImportString(import_str) => {
        match resolve_entry_value(import_str, resolution_ctx.clone()) {
          Ok(resolution) => {
            match resolution {
              Resolution::ImportedSingle(raw_val) => {
                match serde_jsonc::from_str::<HashMap<String, String>>(&raw_val) {
                  Ok(val) => {
                    OptionalStrStrHashMap::Some(val)
                  },
                  Err(e) => {
                    err = Some(SimpleError::from(e));
                    OptionalStrStrHashMap::None
                  }
                }
              },
              Resolution::ImportedMultiple(raw_vals) => {
                let mut entries = HashMap::new();

                for (val_key, raw_val) in raw_vals {
                  match serde_jsonc::from_str::<String>(&raw_val) {
                    Ok(val) => {
                      entries.insert(val_key, val);
                    },
                    Err(e) => {
                      err = Some(SimpleError::from(e));
                      break;
                    }
                  }
                }

                match err {
                  Some(_) => {
                    OptionalStrStrHashMap::None
                  },
                  None => {
                    OptionalStrStrHashMap::Some(entries)
                  }
                }
              },
              Resolution::NoImport(raw_val) => {
                match serde_jsonc::from_str::<HashMap<String, String>>(&raw_val) {
                  Ok(val) => {
                    OptionalStrStrHashMap::Some(val)
                  },
                  Err(e) => {
                    err = Some(SimpleError::from(e));
                    OptionalStrStrHashMap::None
                  }
                }
              }
            }
          },
          Err(e) => {
            err = Some(e);
            OptionalStrStrHashMap::None
          }
        }
      },
      OptionalStringListManifestSpecEntry::None => { OptionalStrStrHashMap::None }
    };

    match err {
      Some(e) => { Err(e) },
      None => { Ok(ret) }
    }
  }
}

impl<T, K> ManifestCompilationFrom<OptionalListManifestSpecEntry<T>> for OptionalStrTHashMap<K> where
  K: ManifestCompilationFrom<T>, T: for<'a> Deserialize<'a>
{
  fn compile(spec: OptionalListManifestSpecEntry<T>, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized {
    let mut err = None;
    let mut entries = OptionalStrTHashMap::None;

    match spec {
        OptionalListManifestSpecEntry::Some(hash_map) => {
          match resolve_list_entry(hash_map, resolution_ctx.clone()) {
            Ok(list) => {
              entries = OptionalStrTHashMap::Some(list);
            },
            Err(e) => {
              err = Some(e);
            }
          }
        },
        OptionalListManifestSpecEntry::ImportString(raw_str) => {
          match serde_jsonc::from_str(&raw_str) {
            Ok(hash_map) => {
              match resolve_list_entry(hash_map, resolution_ctx.clone()) {
                Ok(list) => {
                  entries = OptionalStrTHashMap::Some(list);
                },
                Err(e) => {
                  err = Some(e);
                }
              }
            },
            Err(e) => {
              err = Some(SimpleError::from(e));
            }
          }
        },
        OptionalListManifestSpecEntry::None => {}, // No-op.
    }

    match err {
      Some(e) => { Err(e) },
      None => { Ok(entries) }
    }
  }
}

//* ----------------------------

impl Manifest {
  pub fn compile(spec: ManifestSpec, spec_path: OsPath) -> Result<Manifest, SimpleError> {
    let mut err = None;

    let base = match OptionalString::compile(spec.base.clone(), ResolutionCtx { base: None, here: spec_path.clone() }) {
      Ok(val) => { val },
      Err(e) => {
        err = Some(e);
        OptionalString::None
      }
    };

    let resolution_ctx = ResolutionCtx { 
      base: match base.clone() {
        OptionalString::Some(val) => { Some(val) },
        OptionalString::None => { None }
      }, 
      here: spec_path.clone() 
    };
    
    let name = match OptionalString::compile(spec.name.clone(), resolution_ctx.clone()) {
      Ok(val) => { val },
      Err(e) => {
        err = Some(e);
        OptionalString::None
      }
    };

    let version = match RequiredString::compile(spec.version.clone(), resolution_ctx.clone()) {
      Ok(val) => { val },
      Err(e) => {
        err = Some(e);
        RequiredString(String::new())
      }
    };

    let modules = match OptionalStrTHashMap::compile(spec.modules.clone(), resolution_ctx.clone()) {
      Ok(val) => {
        val
      },
      Err(e) => {
        err = Some(e);
        OptionalStrTHashMap::None
      }
    };

    let applications = match OptionalStrTHashMap::compile(spec.applications.clone(), resolution_ctx.clone()) {
      Ok(val) => {
        val
      },
      Err(e) => {
        err = Some(e);
        OptionalStrTHashMap::None
      }
    };

    #[cfg(feature = "core")]
    let expression_packs = match OptionalStrTHashMap::compile(spec.expression_packs.clone(), resolution_ctx.clone()) {
      Ok(val) => {
        val
      },
      Err(e) => {
        err = Some(e);
        OptionalStrTHashMap::None
      }
    };

    match err {
      Some(e) => { Err(e) },
      None => { 
        Ok(Manifest {
          name,
          version,
          base,
          modules,
          applications,
          #[cfg(feature = "core")]
          expression_packs
        })
      }
    }
  }
}

impl ManifestCompilationFrom<RawApplicationSpec> for ApplicationSpec {
  fn compile(spec: RawApplicationSpec, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized {
    let mut err = None;

    let name = match RequiredString::compile(spec.name.clone(), resolution_ctx.clone()) {
      Ok(val) => { val },
      Err(e) => {
        err = Some(e);
        RequiredString(String::new())
      }
    };

    let version = match RequiredString::compile(spec.version.clone(), resolution_ctx.clone()) {
      Ok(val) => { val },
      Err(e) => {
        err = Some(e);
        RequiredString(String::new())
      }
    };

    let intents = match OptionalStrStrHashMap::compile(spec.intents.clone(), resolution_ctx.clone()) {
      Ok(val) => {
        val
      },
      Err(e) => {
        err = Some(e);
        OptionalStrStrHashMap::None
      }
    };

    let containers  = match OptionalStrTHashMap::compile(spec.containers.clone(), resolution_ctx.clone()) {
      Ok(val) => {
        val
      },
      Err(e) => {
        err = Some(e);
        OptionalStrTHashMap::None
      }
    };

    match err {
      Some(e) => { Err(e) },
      None => {
        Ok(Self {
          name,
          version,
          intents,
          containers
        })
      }
    }
  }
}

impl ManifestCompilationFrom<RawModuleSpec> for ModuleSpec {
  fn compile(spec: RawModuleSpec, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized {
    let mut err = None;

    let name = match OptionalString::compile(spec.name.clone(), resolution_ctx.clone()) {
      Ok(val) => {
        val
      },
      Err(e) => {
        err = Some(e);
        OptionalString::None
      }
    };

    match err {
      Some(e) => { Err(e) },
      None => {
        Ok(Self {
          name
        })
      }
    }
  }
}
