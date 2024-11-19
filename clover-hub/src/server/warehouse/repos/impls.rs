use std::collections::HashMap;
use log::debug;
use serde::Deserialize;
use simple_error::SimpleError;
use os_path::OsPath;
use crate::server::appd::models::{BuildConfig, RepoCreds};
use super::{models::*, resolve_entry_value, resolve_list_entry};

impl ManifestCompilationFrom<Option<String>> for OptionalString {
  async fn compile(spec: Option<String>, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized {
    let mut err = None;

    let res = match spec.clone() {
      Some(raw_str) => {
        match resolve_entry_value(raw_str, resolution_ctx.clone()).await {
          Ok(resolution) => {
            match resolution {
              Resolution::ImportedMultiple(_) => {
                err = Some(SimpleError::new("Glob import not supported at this level!"));
                OptionalString::None
              },
              Resolution::ImportedSingle((here, imported)) => {
                match serde_jsonc::from_str::<String>(&imported) {
                  Ok(val) => {
                    OptionalString::Some(val)
                  },
                  Err(e) => {
                    err = Some(SimpleError::new(format!("ctx: {:?}\nerr: {}", ResolutionCtx { base: resolution_ctx.clone().base, here }, e)));
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

impl<T: Clone + for<'a> Deserialize<'a>, K: ManifestCompilationFrom<T>> ManifestCompilationFrom<OptionalSingleManifestSpecEntry<T>> for Optional<K> {
  async fn compile(spec: OptionalSingleManifestSpecEntry<T>, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized {
    let mut err = None;

    let res = match spec.clone() {
      OptionalSingleManifestSpecEntry::Some(raw_val) => {
        match K::compile(raw_val, resolution_ctx.clone()).await {
          Ok(val) => {
            Optional::Some(val)
          },
          Err(e) => {
            err = Some(e);
            Optional::None
          }
        }
      },
      OptionalSingleManifestSpecEntry::ImportString(raw_str) => {
        match resolve_entry_value(raw_str, resolution_ctx.clone()).await {
          Ok(resolution) => {
            match resolution {
              Resolution::ImportedMultiple(_) => {
                err = Some(SimpleError::new("Glob import not supported at this level!"));
                Optional::None
              },
              Resolution::ImportedSingle((here, imported)) => {
                match serde_jsonc::from_str(&imported) {
                  Ok(raw_val) => {
                    match K::compile(raw_val, ResolutionCtx { base: resolution_ctx.clone().base, here }).await {
                      Ok(val) => {
                        Optional::Some(val)
                      },
                      Err(e) => {
                        err = Some(e);
                        Optional::None
                      }
                    }
                  },
                  Err(e) => {
                    err = Some(SimpleError::new(format!("ctx: {:?}\nerr: {}", ResolutionCtx { base: resolution_ctx.clone().base, here }, e)));
                    Optional::None
                  }
                }
              },
              Resolution::NoImport(val_str) => {
                match serde_jsonc::from_str(&val_str) {
                  Ok(raw_val) => {
                    match K::compile(raw_val, resolution_ctx.clone()).await {
                      Ok(val) => {
                        Optional::Some(val)
                      },
                      Err(e) => {
                        err = Some(e);
                        Optional::None
                      }
                    }
                  },
                  Err(e) => {
                    err = Some(SimpleError::new(format!("ctx: {:?}\nerr: {}", resolution_ctx.clone(), e)));
                    Optional::None
                  }
                }
              }
            }
          },
          Err(e) => {
            err = Some(e);
            Optional::None
          }
        }
      },
      OptionalSingleManifestSpecEntry::None => { Optional::None }
    };

    match err {
      Some(e) => { Err(e) },
      None => { Ok(res) }
    }
  }
}

impl ManifestCompilationFrom<String> for RequiredString {
  async fn compile(spec: String, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized {
    let mut err = None;

    let res = match resolve_entry_value(spec.clone(), resolution_ctx.clone()).await {
      Ok(resolution) => {
        match resolution {
          Resolution::ImportedMultiple(_) => {
            err = Some(SimpleError::new("Glob import not supported at this level!"));
            Default::default()
          },
          Resolution::ImportedSingle((here, imported)) => {
            match serde_jsonc::from_str::<String>(&imported) {
              Ok(val) => {
                RequiredString(val)
              },
              Err(e) => {
                err = Some(SimpleError::new(format!("ctx: {:?}\nerr: {}", ResolutionCtx { base: resolution_ctx.clone().base, here }, e)));
                Default::default()
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
        Default::default()
      }
    };

    match err {
      Some(e) => { Err(e) },
      None => { Ok(res) }
    }
  }
}

impl ManifestCompilationFrom<OptionalStringListManifestSpecEntry> for OptionalStrStrHashMap {
  async fn compile(spec: OptionalStringListManifestSpecEntry, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized {
    let mut err = None;

    let ret: OptionalStrStrHashMap = match spec.clone() {
      OptionalStringListManifestSpecEntry::Some(raw_intents) => {
        let mut entries = HashMap::new();

        for (intent_id, raw_intent) in raw_intents {
          match resolve_entry_value(raw_intent.try_into().unwrap(), resolution_ctx.clone()).await {
            Ok(resolution) => {
              match resolution {
                Resolution::ImportedMultiple(_) => {
                  err = Some(SimpleError::new("Glob import not supported at this level!"));
                  break;
                },
                Resolution::ImportedSingle((here, imported)) => {
                  match serde_jsonc::from_str::<String>(&imported) {
                    Ok(val) => {
                      entries.insert(intent_id, val);
                    },
                    Err(e) => {
                      err = Some(SimpleError::new(format!("ctx: {:?}\nerr: {}", ResolutionCtx { base: resolution_ctx.clone().base, here }, e)));
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
        match resolve_entry_value(import_str, resolution_ctx.clone()).await {
          Ok(resolution) => {
            match resolution {
              Resolution::ImportedSingle((here, raw_val)) => {
                match serde_jsonc::from_str::<HashMap<String, String>>(&raw_val) {
                  Ok(val) => {
                    OptionalStrStrHashMap::Some(val)
                  },
                  Err(e) => {
                    err = Some(SimpleError::new(format!("ctx: {:?}\nerr: {}", ResolutionCtx { base: resolution_ctx.clone().base, here }, e)));
                    OptionalStrStrHashMap::None
                  }
                }
              },
              Resolution::ImportedMultiple((here, raw_vals)) => {
                let mut entries = HashMap::new();

                for (val_key, raw_val) in raw_vals {
                  match serde_jsonc::from_str::<String>(&raw_val) {
                    Ok(val) => {
                      entries.insert(val_key, val);
                    },
                    Err(e) => {
                      err = Some(SimpleError::new(format!("ctx: {:?}\nerr: {}", ResolutionCtx { base: resolution_ctx.clone().base, here }, e)));
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
                    err = Some(SimpleError::new(format!("ctx: {:?}\nerr: {}", resolution_ctx.clone(), e)));
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
  async fn compile(spec: OptionalListManifestSpecEntry<T>, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized {
    let mut err = None;
    let mut entries = OptionalStrTHashMap::None;

    match spec {
        OptionalListManifestSpecEntry::Some(hash_map) => {
          match resolve_list_entry(hash_map, resolution_ctx.clone()).await {
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
              match resolve_list_entry(hash_map, resolution_ctx.clone()).await {
                Ok(list) => {
                  entries = OptionalStrTHashMap::Some(list);
                },
                Err(e) => {
                  err = Some(e);
                }
              }
            },
            Err(e) => {
              err = Some(SimpleError::new(format!("ctx: {:?}\nerr: {}", resolution_ctx.clone(), e)));
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

impl<T: Clone + for<'a> Deserialize<'a>, K: ManifestCompilationFrom<T> + for<'a> Deserialize<'a>> ManifestCompilationFrom<RequiredListManifestSpecEntry<T>> for RequiredStrTHashMap<K> {
  async fn compile(spec: RequiredListManifestSpecEntry<T>, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized {
    let mut err = None;

    let res = match spec.clone() {
      RequiredListManifestSpecEntry::ImportString(spec_str) => {
        match resolve_entry_value(spec_str.clone(), resolution_ctx.clone()).await {
          Ok(resolution) => {
            match resolution {
              Resolution::ImportedMultiple(_) => {
                err = Some(SimpleError::new("Glob import not supported at this level!"));
                RequiredStrTHashMap(HashMap::new())
              },
              Resolution::ImportedSingle((here, imported)) => {
                match serde_jsonc::from_str::<HashMap<String, RequiredSingleManifestEntry<T>>>(&imported) {
                  Ok(spec_list) => {
                    let mut entries = HashMap::new();

                    for (entry_id, raw_entry) in spec_list {
                      match raw_entry {
                        RequiredSingleManifestEntry::Some(raw_entry_obj) => {
                          match K::compile(raw_entry_obj, ResolutionCtx { base: resolution_ctx.clone().base, here: here.clone() }).await {
                            Ok(val) => {
                              entries.insert(entry_id.clone(), RequiredSingleManifestEntry::Some(val));
                            },
                            Err(e) => {
                              err = Some(e);
                            }
                          }
                        },
                        RequiredSingleManifestEntry::ImportString(raw_entry_str) => {
                          match resolve_entry_value(raw_entry_str, ResolutionCtx { base: resolution_ctx.clone().base, here: here.clone() }).await {
                            Ok(resolution) => {
                              match resolution {
                                Resolution::ImportedSingle((here, val_str)) => {
                                  match serde_jsonc::from_str(&val_str) {
                                    Ok(raw_val) => {
                                      match K::compile(raw_val, ResolutionCtx { base: resolution_ctx.clone().base, here: here.clone() }).await {
                                        Ok(val) => {
                                          entries.insert(entry_id.clone(), RequiredSingleManifestEntry::Some(val));
                                        },
                                        Err(e) => {
                                          err = Some(e);
                                        }
                                      }
                                    },
                                    Err(e) => {
                                      err = Some(SimpleError::new(format!("ctx: {:?}\nerr: {}", ResolutionCtx { base: resolution_ctx.clone().base, here: here.clone() }, e)));
                                    }
                                  }
                                },
                                Resolution::ImportedMultiple((here, hash_map)) => {
                                  for (val_id, val_str) in hash_map {
                                    match serde_jsonc::from_str(&val_str) {
                                      Ok(raw_val) => {
                                        match K::compile(raw_val, ResolutionCtx { base: resolution_ctx.clone().base, here: here.clone() }).await {
                                          Ok(val) => {
                                            entries.insert(val_id, RequiredSingleManifestEntry::Some(val));
                                          },
                                          Err(e) => {
                                            err = Some(e);
                                          }
                                        }
                                      },
                                      Err(e) => {
                                        err = Some(SimpleError::new(format!("ctx: {:?}\nerr: {}", ResolutionCtx { base: resolution_ctx.clone().base, here: here.clone() }, e)));
                                      }
                                    }
                                  }
                                },
                                Resolution::NoImport(val_str) => {
                                  match serde_jsonc::from_str(&val_str) {
                                    Ok(raw_val) => {
                                      match K::compile(raw_val, resolution_ctx.clone()).await {
                                        Ok(val) => {
                                          entries.insert(entry_id.clone(), RequiredSingleManifestEntry::Some(val));
                                        },
                                        Err(e) => {
                                          err = Some(e);
                                        }
                                      }
                                    },
                                    Err(e) => {
                                      err = Some(SimpleError::new(format!("ctx: {:?}\nerr: {}", resolution_ctx.clone(), e)));
                                    }
                                  }
                                },
                              }
                            },
                            Err(e) => {
                              err = Some(e);
                            }
                          }
                        },
                      }
                    }

                    if err != None {
                      RequiredStrTHashMap(HashMap::new())
                    } else {
                      RequiredStrTHashMap(entries)
                    }
                  },
                  Err(e) => {
                    err = Some(SimpleError::new(format!("ctx: {:?}\nerr: {}", resolution_ctx.clone().base, e)));
                    RequiredStrTHashMap(HashMap::new())
                  }
                }
              },
              Resolution::NoImport(raw_val) => {
                match serde_jsonc::from_str(&raw_val) {
                  Ok(val) => {
                    RequiredStrTHashMap(val)
                  },
                  Err(e) => {
                    err = Some(SimpleError::new(format!("ctx: {:?}\nerr: {}", resolution_ctx.clone().base, e)));
                    RequiredStrTHashMap(HashMap::new())
                  }
                }
              }
            }
          },
          Err(e) => {
            err = Some(e);
            RequiredStrTHashMap(HashMap::new())
          }
        }
      },
      RequiredListManifestSpecEntry::Some(spec_list) => {
        let mut entries = HashMap::new();

        for (entry_id, raw_entry) in spec_list {
          match raw_entry {
            RequiredSingleManifestEntry::Some(raw_entry_obj) => {
              match K::compile(raw_entry_obj, resolution_ctx.clone()).await {
                Ok(val) => {
                  entries.insert(entry_id.clone(), RequiredSingleManifestEntry::Some(val));
                },
                Err(e) => {
                  err = Some(e);
                }
              }
            },
            RequiredSingleManifestEntry::ImportString(raw_entry_str) => {
              match resolve_entry_value(raw_entry_str, resolution_ctx.clone()).await {
                Ok(resolution) => {
                  match resolution {
                    Resolution::ImportedSingle((here, val_str)) => {
                      match serde_jsonc::from_str(&val_str) {
                        Ok(raw_val) => {
                          match K::compile(raw_val, ResolutionCtx { base: resolution_ctx.clone().base, here }).await {
                            Ok(val) => {
                              entries.insert(entry_id.clone(), RequiredSingleManifestEntry::Some(val));
                            },
                            Err(e) => {
                              err = Some(e);
                            }
                          }
                        },
                        Err(e) => {
                          err = Some(SimpleError::new(format!("ctx: {:?}\nerr: {}", ResolutionCtx { base: resolution_ctx.clone().base, here }, e)));
                        }
                      }
                    },
                    Resolution::ImportedMultiple((here, hash_map)) => {
                      for (val_id, val_str) in hash_map {
                        match serde_jsonc::from_str(&val_str) {
                          Ok(raw_val) => {
                            match K::compile(raw_val, ResolutionCtx { base: resolution_ctx.clone().base, here: here.clone() }).await {
                              Ok(val) => {
                                entries.insert(val_id, RequiredSingleManifestEntry::Some(val));
                              },
                              Err(e) => {
                                err = Some(e);
                              }
                            }
                          },
                          Err(e) => {
                            err = Some(SimpleError::new(format!("ctx: {:?}\nerr: {}", ResolutionCtx { base: resolution_ctx.clone().base, here: here.clone() }, e)));
                          }
                        }
                      }
                    },
                    Resolution::NoImport(val_str) => {
                      match serde_jsonc::from_str(&val_str) {
                        Ok(raw_val) => {
                          match K::compile(raw_val, resolution_ctx.clone()).await {
                            Ok(val) => {
                              entries.insert(entry_id.clone(), RequiredSingleManifestEntry::Some(val));
                            },
                            Err(e) => {
                              err = Some(e);
                            }
                          }
                        },
                        Err(e) => {
                          err = Some(SimpleError::new(format!("ctx: {:?}\nerr: {}", resolution_ctx.clone(), e)));
                        }
                      }
                    },
                  }
                },
                Err(e) => {
                  err = Some(e);
                }
              }
            },
          }
        }

        if err != None {
          RequiredStrTHashMap(HashMap::new())
        } else {
          RequiredStrTHashMap(entries)
        }
      }
    };

    match err {
      Some(e) => { Err(e) },
      None => { Ok(res) }
    }
  }
}

impl ManifestCompilationFrom<OptionalSingleManifestSpecEntry<bool>> for OptionalBoolean {
  async fn compile(spec: OptionalSingleManifestSpecEntry<bool>, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized, Option<bool>: for<'a> Deserialize<'a> {
    let mut err = None;

    let ret = match spec {
      OptionalSingleManifestSpecEntry::Some(val) => OptionalBoolean::Some(val),
      OptionalSingleManifestSpecEntry::ImportString(raw_spec) => {
        match resolve_entry_value(raw_spec, resolution_ctx.clone()).await {
          Ok(resolution) => {
            match resolution {
              Resolution::ImportedMultiple(_) => {
                err = Some(SimpleError::new("Glob imports are not supported at this level"));
                OptionalBoolean::None
              },
              Resolution::ImportedSingle((here, val_str)) => {
                match serde_jsonc::from_str(&val_str) {
                  Ok(val) => {
                    OptionalBoolean::Some(val)
                  },
                  Err(e) => {
                    err = Some(SimpleError::new(format!("ctx: {:?}\nerr: {}", ResolutionCtx { base: resolution_ctx.clone().base, here }, e)));
                    OptionalBoolean::None
                  }
                }
              },
              Resolution::NoImport(val_str) => {
                match serde_jsonc::from_str(&val_str) {
                  Ok(val) => {
                    OptionalBoolean::Some(val)
                  },
                  Err(e) => {
                    err = Some(SimpleError::new(format!("ctx: {:?}\nerr: {}", resolution_ctx.clone(), e)));
                    OptionalBoolean::None
                  }
                }
              }
            }
          },
          Err(e) => {
            err = Some(e);
            OptionalBoolean::None
          }
        }
      },
      OptionalSingleManifestSpecEntry::None => OptionalBoolean::None
    };

    match err {
      Some(e) => Err(e),
      None => Ok(ret)
    }
  }
}

//* ----------------------------

impl Manifest {
  pub async fn compile(spec: ManifestSpec, spec_path: OsPath) -> Result<Manifest, SimpleError> {
    let mut err = None;

    debug!("Resolving manifest.base");
    let base = match OptionalString::compile(spec.base.clone(), ResolutionCtx { base: None, here: spec_path.clone() }).await {
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
    
    debug!("Resolving manifest.name");
    let name = match OptionalString::compile(spec.name.clone(), resolution_ctx.clone()).await {
      Ok(val) => { val },
      Err(e) => {
        err = Some(e);
        OptionalString::None
      }
    };

    debug!("Resolving manifest.version");
    let version = match RequiredString::compile(spec.version.clone(), resolution_ctx.clone()).await {
      Ok(val) => { val },
      Err(e) => {
        err = Some(e);
        Default::default()
      }
    };

    debug!("Resolving manifest.modules");
    let modules = match OptionalStrTHashMap::compile(spec.modules.clone(), resolution_ctx.clone()).await {
      Ok(val) => {
        val
      },
      Err(e) => {
        err = Some(e);
        OptionalStrTHashMap::None
      }
    };

    debug!("Resolving manifest.applications");
    let applications = match OptionalStrTHashMap::compile(spec.applications.clone(), resolution_ctx.clone()).await {
      Ok(val) => {
        val
      },
      Err(e) => {
        err = Some(e);
        OptionalStrTHashMap::None
      }
    };

    debug!("Resolving manifest.expression-packs");
    #[cfg(feature = "core")]
    let expression_packs = match OptionalStrTHashMap::compile(spec.expression_packs.clone(), resolution_ctx.clone()).await {
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
  async fn compile(spec: RawApplicationSpec, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized {
    let mut err = None;

    debug!("Resolving application.name");
    let name = match RequiredString::compile(spec.name.clone(), resolution_ctx.clone()).await {
      Ok(val) => { val },
      Err(e) => {
        err = Some(e);
        Default::default()
      }
    };

    debug!("Resolving application.version");
    let version = match RequiredString::compile(spec.version.clone(), resolution_ctx.clone()).await {
      Ok(val) => { val },
      Err(e) => {
        err = Some(e);
        Default::default()
      }
    };

    debug!("Resolving application.intents");
    let intents = match OptionalStrStrHashMap::compile(spec.intents.clone(), resolution_ctx.clone()).await {
      Ok(val) => {
        val
      },
      Err(e) => {
        err = Some(e);
        OptionalStrStrHashMap::None
      }
    };

    debug!("Resolving application.containers");
    let containers  = match OptionalStrTHashMap::compile(spec.containers.clone(), resolution_ctx.clone()).await {
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

impl ManifestCompilationFrom<RawContainerSpec> for ContainerSpec {
  async fn compile(spec: RawContainerSpec, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized, RawContainerSpec: for<'a> Deserialize<'a> {
    let mut err = None;

    debug!("Resolving container.interface");
    let interface = match OptionalBoolean::compile(spec.interface.clone(), resolution_ctx.clone()).await {
      Ok(val) => {
        val
      },
      Err(e) => {
        err = Some(e);
        OptionalBoolean::None
      }
    };

    debug!("Resolving container.build-config");
    let build = match Optional::compile(spec.build.clone(), resolution_ctx.clone()).await {
      Ok(val) => {
        val
      },
      Err(e) => {
        err = Some(e);
        Optional::None
      }
    };

    match err {
      Some(e) => { Err(e) },
      None => {
        Ok(ContainerSpec {
          interface,
          build
        })
      }
    }
  }
}

impl ManifestCompilationFrom<RawBuildConfig> for BuildConfig {
  async fn compile(spec: RawBuildConfig, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized, RawBuildConfig: for<'a> Deserialize<'a> {
    let mut err = None;

    debug!("Resolving build-config.url");
    let url_field = match RequiredString::compile(spec.url.clone(), resolution_ctx.clone()).await {
      Ok(val) => val,
      Err(e) => {
        err = Some(e);
        Default::default()
      }
    };

    debug!("Resolving build-config.creds");
    let creds = match Optional::compile(spec.creds.clone(), resolution_ctx.clone()).await {
      Ok(val) => val,
      Err(e) => {
        err = Some(e);
        Optional::None
      }
    };

    match err {
      Some(e) => Err(e),
      None => Ok(BuildConfig {
        url: url_field,
        creds
      })
    }
  }
}

impl ManifestCompilationFrom<RawRepoCreds> for RepoCreds {
  async fn compile(spec: RawRepoCreds, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized, RawRepoCreds: for<'a> Deserialize<'a> {
    let mut err = None;

    debug!("Resolving repo-creds.username");
    let username = match OptionalString::compile(spec.username.clone(), resolution_ctx.clone()).await {
      Ok(val) => val,
      Err(e) => {
        err = Some(e);
        OptionalString::None
      }
    };

    debug!("Resolving repo-creds.key");
    let key = match RequiredString::compile(spec.key.clone(), resolution_ctx.clone()).await {
      Ok(val) => val,
      Err(e) => {
        err = Some(e);
        Default::default()
      }
    };

    match err {
      Some(e) => Err(e),
      None => Ok(RepoCreds {
        username,
        key
      })
    }
  }
}

// TODO: move to clover::server::modman::models::Module
impl ManifestCompilationFrom<RawModuleSpec> for ModuleSpec {
  async fn compile(spec: RawModuleSpec, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized {
    let mut err = None;

    debug!("Resolving module.name");
    let name = match OptionalString::compile(spec.name.clone(), resolution_ctx.clone()).await {
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

impl ManifestCompilationFrom<RawExpressionPackSpec> for ExpressionPackSpec {
  async fn compile(spec: RawExpressionPackSpec, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized {
    let mut err = None;

    debug!("Resolving expression-pack.name");
    let name = match OptionalString::compile(spec.name.clone(), resolution_ctx.clone()).await {
      Ok(val) => {
        val
      },
      Err(e) => {
        err = Some(e);
        OptionalString::None
      }
    };

    debug!("Resolving expression-pack.expressions");
    let expressions = match OptionalStrTHashMap::compile(spec.expressions.clone(), resolution_ctx.clone()).await {
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
        Ok(ExpressionPackSpec {
          name,
          expressions
        })
      }
    }
  }
}

impl ManifestCompilationFrom<RawExpressionSpec> for ExpressionSpec {
  async fn compile(spec: RawExpressionSpec, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized, RawExpressionSpec: for<'a> Deserialize<'a> {
    let mut err = None;

    debug!("Resolving raw-expression-pack");
    let ret = match spec {
      RawExpressionSpec::RawStaticExpressionSpec(raw_expression_spec) => {
        match StaticExpressionSpec::compile(raw_expression_spec, resolution_ctx.clone()).await {
          Ok(val) => { let static_expression_spec = Self::StaticExpressionSpec(val); static_expression_spec },
          Err(e) => {
            err = Some(e);
            Self::StaticExpressionSpec(StaticExpressionSpec { static_url: RequiredString(String::from("bleh")) })
          }
        }
      },
    };

    match err {
      Some(e) => Err(e),
      None => Ok(ret)
    }
  }
}

impl ManifestCompilationFrom<RawStaticExpressionSpec> for StaticExpressionSpec {
  async fn compile(spec: RawStaticExpressionSpec, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized, RawStaticExpressionSpec: for<'a> Deserialize<'a> {
    let mut err = None;

    debug!("Resolving static-expression-pack.static-url");
    let static_url = match RequiredString::compile(spec.static_url.clone(), resolution_ctx.clone()).await {
      Ok(val) => val,
      Err(e) => {
        err = Some(e);
        Default::default()
      }
    };

    match err {
      Some(e) => Err(e),
      None => Ok(Self {
        static_url
      })
    }
  }
}
