use super::{
  models::*,
  replace_simple_directives,
  resolve_entry_value,
  resolve_list_entry,
};
use crate::server::{
  appd::models::{
    BuildConfig,
    RepoCreds,
  },
  warehouse::repos::builtin_rfqdn,
};
use log::debug;
use os_path::OsPath;
use serde::Deserialize;
use simple_error::SimpleError;
use std::collections::HashMap;

impl ManifestCompilationFrom<Option<String>> for OptionalString {
  async fn compile(
    spec: Option<String>,
    resolution_ctx: ResolutionCtx,
    repo_dir_path: OsPath,
  ) -> Result<Self, SimpleError>
  where
    Self: Sized,
  {
    let mut err = None;

    let res = match spec.clone() {
      Some(raw_str) => {
        match resolve_entry_value(raw_str, resolution_ctx.clone(), repo_dir_path.clone()).await {
          Ok(resolution) => match resolution {
            Resolution::ImportedMultiple(_) => {
              err = Some(SimpleError::new("Glob import not supported at this level!"));
              OptionalString::None
            }
            Resolution::ImportedSingle((here, imported)) => {
              match serde_json_lenient::from_str::<String>(&imported) {
                Ok(val) => OptionalString::Some(val),
                Err(e) => {
                  err = Some(SimpleError::new(format!(
                    "OptionalString, ctx: {:#?}\nerr: {}",
                    ResolutionCtx {
                      base: resolution_ctx.clone().base,
                      builtin: resolution_ctx.clone().builtin,
                      here
                    },
                    e
                  )));
                  OptionalString::None
                }
              }
            }
            Resolution::NoImport(val) => OptionalString::Some(val),
          },
          Err(e) => {
            err = Some(e);
            OptionalString::None
          }
        }
      }
      None => OptionalString::None,
    };

    match err {
      Some(e) => Err(e),
      None => Ok(res),
    }
  }
}

impl<T: Clone + for<'a> Deserialize<'a>, K: ManifestCompilationFrom<T>>
  ManifestCompilationFrom<OptionalSingleManifestSpecEntry<T>> for Optional<K>
{
  async fn compile(
    spec: OptionalSingleManifestSpecEntry<T>,
    resolution_ctx: ResolutionCtx,
    repo_dir_path: OsPath,
  ) -> Result<Self, SimpleError>
  where
    Self: Sized,
  {
    let mut err = None;

    let res = match spec.clone() {
      OptionalSingleManifestSpecEntry::Some(raw_val) => {
        match K::compile(raw_val, resolution_ctx.clone(), repo_dir_path.clone()).await {
          Ok(val) => Optional::Some(val),
          Err(e) => {
            err = Some(e);
            Optional::None
          }
        }
      }
      OptionalSingleManifestSpecEntry::ImportString(raw_str) => {
        match resolve_entry_value(raw_str, resolution_ctx.clone(), repo_dir_path.clone()).await {
          Ok(resolution) => match resolution {
            Resolution::ImportedMultiple(_) => {
              err = Some(SimpleError::new("Glob import not supported at this level!"));
              Optional::None
            }
            Resolution::ImportedSingle((here, imported)) => {
              match serde_json_lenient::from_str(&imported) {
                Ok(raw_val) => {
                  match K::compile(
                    raw_val,
                    ResolutionCtx {
                      base: resolution_ctx.clone().base,
                      builtin: resolution_ctx.clone().builtin,
                      here,
                    },
                    repo_dir_path.clone(),
                  )
                  .await
                  {
                    Ok(val) => Optional::Some(val),
                    Err(e) => {
                      err = Some(e);
                      Optional::None
                    }
                  }
                }
                Err(e) => {
                  err = Some(SimpleError::new(format!(
                    "Optional, ctx: {:#?}\nerr: {}",
                    ResolutionCtx {
                      base: resolution_ctx.clone().base,
                      builtin: resolution_ctx.clone().builtin,
                      here
                    },
                    e
                  )));
                  Optional::None
                }
              }
            }
            Resolution::NoImport(val_str) => match serde_json_lenient::from_str(&val_str) {
              Ok(raw_val) => {
                match K::compile(raw_val, resolution_ctx.clone(), repo_dir_path.clone()).await {
                  Ok(val) => Optional::Some(val),
                  Err(e) => {
                    err = Some(e);
                    Optional::None
                  }
                }
              }
              Err(e) => {
                err = Some(SimpleError::new(format!(
                  "Optional, ctx: {:#?}\nerr: {}",
                  resolution_ctx.clone(),
                  e
                )));
                Optional::None
              }
            },
          },
          Err(e) => {
            err = Some(e);
            Optional::None
          }
        }
      }
      OptionalSingleManifestSpecEntry::None => Optional::None,
    };

    match err {
      Some(e) => Err(e),
      None => Ok(res),
    }
  }
}

impl ManifestCompilationFrom<String> for RequiredString {
  async fn compile(
    spec: String,
    resolution_ctx: ResolutionCtx,
    repo_dir_path: OsPath,
  ) -> Result<Self, SimpleError>
  where
    Self: Sized,
  {
    let mut err = None;

    let res = match resolve_entry_value(spec.clone(), resolution_ctx.clone(), repo_dir_path.clone())
      .await
    {
      Ok(resolution) => match resolution {
        Resolution::ImportedMultiple(_) => {
          err = Some(SimpleError::new("Glob import not supported at this level!"));
          Default::default()
        }
        Resolution::ImportedSingle((here, imported)) => {
          match serde_json_lenient::from_str::<String>(&imported) {
            Ok(val) => RequiredString(val),
            Err(e) => {
              err = Some(SimpleError::new(format!(
                "RequiredString, ctx: {:#?}\nerr: {}",
                ResolutionCtx {
                  base: resolution_ctx.clone().base,
                  builtin: resolution_ctx.clone().builtin,
                  here
                },
                e
              )));
              Default::default()
            }
          }
        }
        Resolution::NoImport(val) => RequiredString(val),
      },
      Err(e) => {
        err = Some(e);
        Default::default()
      }
    };

    match err {
      Some(e) => Err(e),
      None => Ok(res),
    }
  }
}

impl ManifestCompilationFrom<OptionalStringListManifestSpecEntry> for OptionalStrStrHashMap {
  async fn compile(
    spec: OptionalStringListManifestSpecEntry,
    resolution_ctx: ResolutionCtx,
    repo_dir_path: OsPath,
  ) -> Result<Self, SimpleError>
  where
    Self: Sized,
  {
    let mut err = None;

    let ret: OptionalStrStrHashMap = match spec.clone() {
      OptionalStringListManifestSpecEntry::Some(raw_intents) => {
        let mut entries = HashMap::new();

        for (raw_intent_id, raw_intent) in raw_intents {
          let intent_id = replace_simple_directives(raw_intent_id, resolution_ctx.clone());

          match resolve_entry_value(
            raw_intent.try_into().unwrap(),
            resolution_ctx.clone(),
            repo_dir_path.clone(),
          )
          .await
          {
            Ok(resolution) => match resolution {
              Resolution::ImportedMultiple(_) => {
                err = Some(SimpleError::new("Glob import not supported at this level!"));
                break;
              }
              Resolution::ImportedSingle((here, imported)) => {
                match serde_json_lenient::from_str::<String>(&imported) {
                  Ok(val) => {
                    entries.insert(intent_id, val);
                  }
                  Err(e) => {
                    err = Some(SimpleError::new(format!(
                      "OptionalStrStrHashMap, ctx: {:#?}\nerr: {}",
                      ResolutionCtx {
                        base: resolution_ctx.clone().base,
                        builtin: resolution_ctx.clone().builtin,
                        here
                      },
                      e
                    )));
                    break;
                  }
                }
              }
              Resolution::NoImport(val) => {
                entries.insert(intent_id, val);
              }
            },
            Err(e) => {
              err = Some(e);
              break;
            }
          }
        }

        match err {
          Some(_) => OptionalStrStrHashMap::None,
          None => OptionalStrStrHashMap::Some(entries),
        }
      }
      OptionalStringListManifestSpecEntry::ImportString(import_str) => {
        match resolve_entry_value(import_str, resolution_ctx.clone(), repo_dir_path.clone()).await {
          Ok(resolution) => match resolution {
            Resolution::ImportedSingle((here, raw_val)) => {
              match serde_json_lenient::from_str::<HashMap<String, String>>(&raw_val) {
                Ok(val) => OptionalStrStrHashMap::Some(val),
                Err(e) => {
                  err = Some(SimpleError::new(format!(
                    "OptionalStrStrHashMap, ctx: {:#?}\nerr: {}",
                    ResolutionCtx {
                      base: resolution_ctx.clone().base,
                      builtin: resolution_ctx.clone().builtin,
                      here
                    },
                    e
                  )));
                  OptionalStrStrHashMap::None
                }
              }
            }
            Resolution::ImportedMultiple((here, raw_vals)) => {
              let mut entries = HashMap::new();

              for (val_key, raw_val) in raw_vals {
                match serde_json_lenient::from_str::<String>(&raw_val) {
                  Ok(val) => {
                    entries.insert(val_key, val);
                  }
                  Err(e) => {
                    err = Some(SimpleError::new(format!(
                      "OptionalStrStrHashMap, ctx: {:#?}\nerr: {}",
                      ResolutionCtx {
                        base: resolution_ctx.clone().base,
                        builtin: resolution_ctx.clone().builtin,
                        here
                      },
                      e
                    )));
                    break;
                  }
                }
              }

              match err {
                Some(_) => OptionalStrStrHashMap::None,
                None => OptionalStrStrHashMap::Some(entries),
              }
            }
            Resolution::NoImport(raw_val) => {
              match serde_json_lenient::from_str::<HashMap<String, String>>(&raw_val) {
                Ok(val) => OptionalStrStrHashMap::Some(val),
                Err(e) => {
                  err = Some(SimpleError::new(format!(
                    "OptionalStrStrHashMap, ctx: {:#?}\nerr: {}",
                    resolution_ctx.clone(),
                    e
                  )));
                  OptionalStrStrHashMap::None
                }
              }
            }
          },
          Err(e) => {
            err = Some(e);
            OptionalStrStrHashMap::None
          }
        }
      }
      OptionalStringListManifestSpecEntry::None => OptionalStrStrHashMap::None,
    };

    match err {
      Some(e) => Err(e),
      None => Ok(ret),
    }
  }
}

impl<T: Clone + std::fmt::Debug, K> ManifestCompilationFrom<OptionalListManifestSpecEntry<T>>
  for OptionalStrTHashMap<K>
where
  K: ManifestCompilationFrom<T> + std::fmt::Debug,
  T: for<'a> Deserialize<'a>,
{
  async fn compile(
    spec: OptionalListManifestSpecEntry<T>,
    resolution_ctx: ResolutionCtx,
    repo_dir_path: OsPath,
  ) -> Result<Self, SimpleError>
  where
    Self: Sized,
  {
    let mut err = None;
    let mut entries = OptionalStrTHashMap::None;

    debug!("OptionalStrTHashMap (initial val): {:#?}", spec.clone());

    match spec {
      OptionalListManifestSpecEntry::Some(hash_map) => {
        debug!(
          "OptionalListManifestSpecEntry::Some: {:#?}",
          hash_map.clone()
        );

        match resolve_list_entry(hash_map, resolution_ctx.clone(), repo_dir_path.clone()).await {
          Ok(list) => {
            debug!("serde_jsonc::from_str(): {:#?}", (&list));

            entries = OptionalStrTHashMap::Some(list);
          }
          Err(e) => {
            err = Some(e);
          }
        }
      }
      OptionalListManifestSpecEntry::ImportString(raw_str) => {
        debug!(
          "OptionalListManifestSpecEntry::ImportString: {}",
          raw_str.clone()
        );

        match resolve_entry_value(
          raw_str.clone(),
          resolution_ctx.clone(),
          repo_dir_path.clone(),
        )
        .await
        {
          Ok(resolution) => match resolution {
            Resolution::ImportedSingle((here, res_str)) => {
              match serde_json_lenient::from_str::<HashMap<String, RequiredSingleManifestEntry<T>>>(
                &res_str,
              ) {
                Ok(hash_map) => {
                  debug!("serde_jsonc::from_str(): {:#?}", hash_map.clone());

                  match resolve_list_entry(
                    hash_map,
                    ResolutionCtx {
                      base: resolution_ctx.clone().base,
                      builtin: resolution_ctx.clone().builtin,
                      here,
                    },
                    repo_dir_path.clone(),
                  )
                  .await
                  {
                    Ok(list) => {
                      entries = OptionalStrTHashMap::Some(list);
                    }
                    Err(e) => {
                      err = Some(e);
                    }
                  }
                }
                Err(e) => {
                  err = Some(SimpleError::new(format!(
                    "OptionalStrTHashMap, ctx: {:#?}\nerr: {}",
                    resolution_ctx.clone(),
                    e
                  )));
                }
              }
            }
            Resolution::ImportedMultiple(_) => {
              err = Some(SimpleError::new(
                "Glob imports are not supported at this level.",
              ))
            }
            Resolution::NoImport(_) => {
              err = Some(SimpleError::new(
                "A string is not a valid value for this field unless it's an import.",
              ));
            }
          },
          Err(e) => {
            debug!(
              "OptionalListManifestSpecEntry::ImportString => resolve_list_entry(), failed: {}",
              e
            );
            err = Some(e);
          }
        }
      }
      OptionalListManifestSpecEntry::None => {
        debug!("{:#?} => Empty manifest entry!", spec.clone());
      } // No-op.
    }

    match err {
      Some(e) => Err(e),
      None => Ok(entries),
    }
  }
}

impl<T: Clone + for<'a> Deserialize<'a>, K: ManifestCompilationFrom<T> + for<'a> Deserialize<'a>>
  ManifestCompilationFrom<RequiredListManifestSpecEntry<T>> for RequiredStrTHashMap<K>
{
  async fn compile(
    spec: RequiredListManifestSpecEntry<T>,
    resolution_ctx: ResolutionCtx,
    repo_dir_path: OsPath,
  ) -> Result<Self, SimpleError>
  where
    Self: Sized,
  {
    let mut err = None;

    let res = match spec.clone() {
      RequiredListManifestSpecEntry::ImportString(spec_str) => {
        match resolve_entry_value(
          spec_str.clone(),
          resolution_ctx.clone(),
          repo_dir_path.clone(),
        )
        .await
        {
          Ok(resolution) => match resolution {
            Resolution::ImportedMultiple(_) => {
              err = Some(SimpleError::new("Glob import not supported at this level!"));
              RequiredStrTHashMap(HashMap::new())
            }
            Resolution::ImportedSingle((here, imported)) => {
              match serde_json_lenient::from_str::<HashMap<String, RequiredSingleManifestEntry<T>>>(
                &imported,
              ) {
                Ok(spec_list) => {
                  let mut entries = HashMap::new();

                  for (entry_id, raw_entry) in spec_list {
                    match raw_entry {
                      RequiredSingleManifestEntry::Some(raw_entry_obj) => {
                        match K::compile(
                          raw_entry_obj,
                          ResolutionCtx {
                            base: resolution_ctx.clone().base,
                            builtin: resolution_ctx.clone().builtin,
                            here: here.clone(),
                          },
                          repo_dir_path.clone(),
                        )
                        .await
                        {
                          Ok(val) => {
                            entries
                              .insert(entry_id.clone(), RequiredSingleManifestEntry::Some(val));
                          }
                          Err(e) => {
                            err = Some(e);
                          }
                        }
                      }
                      RequiredSingleManifestEntry::ImportString(raw_entry_str) => {
                        match resolve_entry_value(
                          raw_entry_str,
                          ResolutionCtx {
                            base: resolution_ctx.clone().base,
                            builtin: resolution_ctx.clone().builtin,
                            here: here.clone(),
                          },
                          repo_dir_path.clone(),
                        )
                        .await
                        {
                          Ok(resolution) => match resolution {
                            Resolution::ImportedSingle((here, val_str)) => {
                              match serde_json_lenient::from_str(&val_str) {
                                Ok(raw_val) => {
                                  match K::compile(
                                    raw_val,
                                    ResolutionCtx {
                                      base: resolution_ctx.clone().base,
                                      builtin: resolution_ctx.clone().builtin,
                                      here: here.clone(),
                                    },
                                    repo_dir_path.clone(),
                                  )
                                  .await
                                  {
                                    Ok(val) => {
                                      entries.insert(
                                        entry_id.clone(),
                                        RequiredSingleManifestEntry::Some(val),
                                      );
                                    }
                                    Err(e) => {
                                      err = Some(e);
                                    }
                                  }
                                }
                                Err(e) => {
                                  err = Some(SimpleError::new(format!(
                                    "RequiredStrTHashMap, ctx: {:#?}\nerr: {}",
                                    ResolutionCtx {
                                      base: resolution_ctx.clone().base,
                                      builtin: resolution_ctx.clone().builtin,
                                      here: here.clone()
                                    },
                                    e
                                  )));
                                }
                              }
                            }
                            Resolution::ImportedMultiple((here, hash_map)) => {
                              for (val_id, val_str) in hash_map {
                                match serde_json_lenient::from_str(&val_str) {
                                  Ok(raw_val) => {
                                    match K::compile(
                                      raw_val,
                                      ResolutionCtx {
                                        base: resolution_ctx.clone().base,
                                        builtin: resolution_ctx.clone().builtin,
                                        here: here.clone(),
                                      },
                                      repo_dir_path.clone(),
                                    )
                                    .await
                                    {
                                      Ok(val) => {
                                        entries
                                          .insert(val_id, RequiredSingleManifestEntry::Some(val));
                                      }
                                      Err(e) => {
                                        err = Some(e);
                                      }
                                    }
                                  }
                                  Err(e) => {
                                    err = Some(SimpleError::new(format!(
                                      "RequiredStrTHashMap, ctx: {:#?}\nerr: {}",
                                      ResolutionCtx {
                                        base: resolution_ctx.clone().base,
                                        builtin: resolution_ctx.clone().builtin,
                                        here: here.clone()
                                      },
                                      e
                                    )));
                                  }
                                }
                              }
                            }
                            Resolution::NoImport(val_str) => {
                              match serde_json_lenient::from_str(&val_str) {
                                Ok(raw_val) => {
                                  match K::compile(
                                    raw_val,
                                    resolution_ctx.clone(),
                                    repo_dir_path.clone(),
                                  )
                                  .await
                                  {
                                    Ok(val) => {
                                      entries.insert(
                                        entry_id.clone(),
                                        RequiredSingleManifestEntry::Some(val),
                                      );
                                    }
                                    Err(e) => {
                                      err = Some(e);
                                    }
                                  }
                                }
                                Err(e) => {
                                  err = Some(SimpleError::new(format!(
                                    "RequiredStrTHashMap, ctx: {:#?}\nerr: {}",
                                    resolution_ctx.clone(),
                                    e
                                  )));
                                }
                              }
                            }
                          },
                          Err(e) => {
                            err = Some(e);
                          }
                        }
                      }
                    }
                  }

                  if err != None {
                    RequiredStrTHashMap(HashMap::new())
                  } else {
                    RequiredStrTHashMap(entries)
                  }
                }
                Err(e) => {
                  err = Some(SimpleError::new(format!(
                    "RequiredStrTHashMap, ctx: {:#?}\nerr: {}",
                    resolution_ctx.clone().base,
                    e
                  )));
                  RequiredStrTHashMap(HashMap::new())
                }
              }
            }
            Resolution::NoImport(raw_val) => match serde_json_lenient::from_str(&raw_val) {
              Ok(val) => RequiredStrTHashMap(val),
              Err(e) => {
                err = Some(SimpleError::new(format!(
                  "RequiredStrTHashMap, ctx: {:#?}\nerr: {}",
                  resolution_ctx.clone().base,
                  e
                )));
                RequiredStrTHashMap(HashMap::new())
              }
            },
          },
          Err(e) => {
            err = Some(e);
            RequiredStrTHashMap(HashMap::new())
          }
        }
      }
      RequiredListManifestSpecEntry::Some(spec_list) => {
        let mut entries = HashMap::new();

        for (entry_id, raw_entry) in spec_list {
          match raw_entry {
            RequiredSingleManifestEntry::Some(raw_entry_obj) => {
              match K::compile(raw_entry_obj, resolution_ctx.clone(), repo_dir_path.clone()).await {
                Ok(val) => {
                  entries.insert(entry_id.clone(), RequiredSingleManifestEntry::Some(val));
                }
                Err(e) => {
                  err = Some(e);
                }
              }
            }
            RequiredSingleManifestEntry::ImportString(raw_entry_str) => {
              match resolve_entry_value(
                raw_entry_str,
                resolution_ctx.clone(),
                repo_dir_path.clone(),
              )
              .await
              {
                Ok(resolution) => match resolution {
                  Resolution::ImportedSingle((here, val_str)) => {
                    match serde_json_lenient::from_str(&val_str) {
                      Ok(raw_val) => {
                        match K::compile(
                          raw_val,
                          ResolutionCtx {
                            base: resolution_ctx.clone().base,
                            builtin: resolution_ctx.clone().builtin,
                            here,
                          },
                          repo_dir_path.clone(),
                        )
                        .await
                        {
                          Ok(val) => {
                            entries
                              .insert(entry_id.clone(), RequiredSingleManifestEntry::Some(val));
                          }
                          Err(e) => {
                            err = Some(e);
                          }
                        }
                      }
                      Err(e) => {
                        err = Some(SimpleError::new(format!(
                          "RequiredStrTHashMap.RequiredSingleManifestEntry, ctx: {:#?}\nerr: {}",
                          ResolutionCtx {
                            base: resolution_ctx.clone().base,
                            builtin: resolution_ctx.clone().builtin,
                            here
                          },
                          e
                        )));
                      }
                    }
                  }
                  Resolution::ImportedMultiple((here, hash_map)) => {
                    for (val_id, val_str) in hash_map {
                      match serde_json_lenient::from_str(&val_str) {
                        Ok(raw_val) => {
                          match K::compile(
                            raw_val,
                            ResolutionCtx {
                              base: resolution_ctx.clone().base,
                              builtin: resolution_ctx.clone().builtin,
                              here: here.clone(),
                            },
                            repo_dir_path.clone(),
                          )
                          .await
                          {
                            Ok(val) => {
                              entries.insert(val_id, RequiredSingleManifestEntry::Some(val));
                            }
                            Err(e) => {
                              err = Some(e);
                            }
                          }
                        }
                        Err(e) => {
                          err = Some(SimpleError::new(format!(
                            "RequiredStrTHashMap.RequiredSingleManifestEntry, ctx: {:#?}\nerr: {}",
                            ResolutionCtx {
                              base: resolution_ctx.clone().base,
                              builtin: resolution_ctx.clone().builtin,
                              here: here.clone()
                            },
                            e
                          )));
                        }
                      }
                    }
                  }
                  Resolution::NoImport(val_str) => match serde_json_lenient::from_str(&val_str) {
                    Ok(raw_val) => {
                      match K::compile(raw_val, resolution_ctx.clone(), repo_dir_path.clone()).await
                      {
                        Ok(val) => {
                          entries.insert(entry_id.clone(), RequiredSingleManifestEntry::Some(val));
                        }
                        Err(e) => {
                          err = Some(e);
                        }
                      }
                    }
                    Err(e) => {
                      err = Some(SimpleError::new(format!(
                        "RequiredStrTHashMap.RequiredSingleManifestEntry, ctx: {:#?}\nerr: {}",
                        resolution_ctx.clone(),
                        e
                      )));
                    }
                  },
                },
                Err(e) => {
                  err = Some(e);
                }
              }
            }
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
      Some(e) => Err(e),
      None => Ok(res),
    }
  }
}

impl ManifestCompilationFrom<OptionalSingleManifestSpecEntry<bool>> for OptionalBoolean {
  async fn compile(
    spec: OptionalSingleManifestSpecEntry<bool>,
    resolution_ctx: ResolutionCtx,
    repo_dir_path: OsPath,
  ) -> Result<Self, SimpleError>
  where
    Self: Sized,
    Option<bool>: for<'a> Deserialize<'a>,
  {
    let mut err = None;

    let ret = match spec {
      OptionalSingleManifestSpecEntry::Some(val) => OptionalBoolean::Some(val),
      OptionalSingleManifestSpecEntry::ImportString(raw_spec) => {
        match resolve_entry_value(raw_spec, resolution_ctx.clone(), repo_dir_path.clone()).await {
          Ok(resolution) => match resolution {
            Resolution::ImportedMultiple(_) => {
              err = Some(SimpleError::new(
                "Glob imports are not supported at this level",
              ));
              OptionalBoolean::None
            }
            Resolution::ImportedSingle((here, val_str)) => {
              match serde_json_lenient::from_str(&val_str) {
                Ok(val) => OptionalBoolean::Some(val),
                Err(e) => {
                  err = Some(SimpleError::new(format!(
                    "OptionalBoolean, ctx: {:#?}\nerr: {}",
                    ResolutionCtx {
                      base: resolution_ctx.clone().base,
                      builtin: resolution_ctx.clone().builtin,
                      here
                    },
                    e
                  )));
                  OptionalBoolean::None
                }
              }
            }
            Resolution::NoImport(val_str) => match serde_json_lenient::from_str(&val_str) {
              Ok(val) => OptionalBoolean::Some(val),
              Err(e) => {
                err = Some(SimpleError::new(format!(
                  "OptionalBoolean, ctx: {:#?}\nerr: {}",
                  resolution_ctx.clone(),
                  e
                )));
                OptionalBoolean::None
              }
            },
          },
          Err(e) => {
            err = Some(e);
            OptionalBoolean::None
          }
        }
      }
      OptionalSingleManifestSpecEntry::None => OptionalBoolean::None,
    };

    match err {
      Some(e) => Err(e),
      None => Ok(ret),
    }
  }
}

//* ----------------------------

impl Manifest {
  pub async fn compile(
    spec: ManifestSpec,
    spec_path: OsPath,
    repo_dir_path: OsPath,
  ) -> Result<Manifest, SimpleError> {
    let mut err = None;

    debug!("Resolving manifest.base");
    let base = match OptionalString::compile(
      spec.base.clone(),
      ResolutionCtx {
        base: None,
        builtin: builtin_rfqdn(false),
        here: spec_path.clone(),
      },
      repo_dir_path.clone(),
    )
    .await
    {
      Ok(val) => {
        debug!("Resolved manifest.base");
        val
      }
      Err(e) => {
        err = Some(e);
        OptionalString::None
      }
    };

    let resolution_ctx = ResolutionCtx {
      base: match base.clone() {
        OptionalString::Some(val) => Some(val),
        OptionalString::None => None,
      },
      builtin: builtin_rfqdn(false),
      here: spec_path.clone(),
    };

    debug!("Resolving manifest.name");
    let name = match OptionalString::compile(
      spec.name.clone(),
      resolution_ctx.clone(),
      repo_dir_path.clone(),
    )
    .await
    {
      Ok(val) => {
        debug!("Resolved manifest.name");
        val
      }
      Err(e) => {
        err = Some(e);
        OptionalString::None
      }
    };

    debug!("Resolving manifest.version");
    let version = match RequiredString::compile(
      spec.version.clone(),
      resolution_ctx.clone(),
      repo_dir_path.clone(),
    )
    .await
    {
      Ok(val) => {
        debug!("Resolved manifest.version");
        val
      }
      Err(e) => {
        err = Some(e);
        Default::default()
      }
    };

    let directory = match Optional::compile(
      spec.directory.clone(),
      resolution_ctx.clone(),
      repo_dir_path.clone(),
    )
    .await
    {
      Ok(val) => {
        debug!("Resolved manifest.directory");
        val
      }
      Err(e) => {
        err = Some(e);
        Default::default()
      }
    };

    match err {
      Some(e) => Err(e),
      None => Ok(Manifest {
        name,
        version,
        base,
        directory,
      }),
    }
  }
}

impl ManifestCompilationFrom<RawDirectorySpec> for DirectorySpec {
  async fn compile(
    spec: RawDirectorySpec,
    resolution_ctx: ResolutionCtx,
    repo_dir_path: OsPath,
  ) -> Result<Self, SimpleError>
  where
    Self: Sized,
    RawDirectorySpec: for<'a> Deserialize<'a>,
  {
    let mut err = None;

    debug!("Resolving manifest.modules");
    let modules = match OptionalStrTHashMap::compile(
      spec.modules.clone(),
      resolution_ctx.clone(),
      repo_dir_path.clone(),
    )
    .await
    {
      Ok(val) => {
        debug!("Resolved manifest.modules");
        val
      }
      Err(e) => {
        err = Some(e);
        OptionalStrTHashMap::None
      }
    };

    debug!("Resolving manifest.applications");
    let applications = match OptionalStrTHashMap::compile(
      spec.applications.clone(),
      resolution_ctx.clone(),
      repo_dir_path.clone(),
    )
    .await
    {
      Ok(val) => {
        debug!("Resolved manifest.applications");
        val
      }
      Err(e) => {
        err = Some(e);
        OptionalStrTHashMap::None
      }
    };

    debug!("Resolving manifest.gesture-packs");
    #[cfg(feature = "core")]
    let gesture_packs = match OptionalStrTHashMap::compile(
      spec.gesture_packs.clone(),
      ResolutionCtx {
        base: resolution_ctx.clone().base,
        builtin: builtin_rfqdn(true),
        here: resolution_ctx.clone().here,
      },
      repo_dir_path.clone(),
    )
    .await
    {
      Ok(val) => {
        debug!("Resolved manifest.gesture-packs");
        val
      }
      Err(e) => {
        err = Some(e);
        OptionalStrTHashMap::None
      }
    };

    match err {
      Some(e) => Err(e),
      None => Ok(DirectorySpec {
        modules,
        applications,
        #[cfg(feature = "core")]
        gesture_packs,
      }),
    }
  }
}

impl ManifestCompilationFrom<RawApplicationSpec> for ApplicationSpec {
  async fn compile(
    spec: RawApplicationSpec,
    resolution_ctx: ResolutionCtx,
    repo_dir_path: OsPath,
  ) -> Result<Self, SimpleError>
  where
    Self: Sized,
  {
    let mut err = None;

    debug!("Resolving application.name");
    let name = match RequiredString::compile(
      spec.name.clone(),
      resolution_ctx.clone(),
      repo_dir_path.clone(),
    )
    .await
    {
      Ok(val) => {
        debug!("Resolved application.name");
        val
      }
      Err(e) => {
        err = Some(e);
        Default::default()
      }
    };

    debug!("Resolving application.version");
    let version = match RequiredString::compile(
      spec.version.clone(),
      resolution_ctx.clone(),
      repo_dir_path.clone(),
    )
    .await
    {
      Ok(val) => {
        debug!("Resolved application.version");
        val
      }
      Err(e) => {
        err = Some(e);
        Default::default()
      }
    };

    debug!("Resolving application.intents");
    let intents = match OptionalStrStrHashMap::compile(
      spec.intents.clone(),
      resolution_ctx.clone(),
      repo_dir_path.clone(),
    )
    .await
    {
      Ok(val) => {
        debug!("Resolved application.intents");
        val
      }
      Err(e) => {
        err = Some(e);
        OptionalStrStrHashMap::None
      }
    };

    debug!("Resolving application.containers");
    let containers = match OptionalStrTHashMap::compile(
      spec.containers.clone(),
      resolution_ctx.clone(),
      repo_dir_path.clone(),
    )
    .await
    {
      Ok(val) => {
        debug!("Resolved application.containers");
        val
      }
      Err(e) => {
        err = Some(e);
        OptionalStrTHashMap::None
      }
    };

    match err {
      Some(e) => Err(e),
      None => Ok(Self {
        name,
        version,
        intents,
        containers,
      }),
    }
  }
}

impl ManifestCompilationFrom<RawContainerSpec> for ContainerSpec {
  async fn compile(
    spec: RawContainerSpec,
    resolution_ctx: ResolutionCtx,
    repo_dir_path: OsPath,
  ) -> Result<Self, SimpleError>
  where
    Self: Sized,
    RawContainerSpec: for<'a> Deserialize<'a>,
  {
    let mut err = None;

    debug!("Resolving container.interface");
    let interface = match OptionalBoolean::compile(
      spec.interface.clone(),
      resolution_ctx.clone(),
      repo_dir_path.clone(),
    )
    .await
    {
      Ok(val) => {
        debug!("Resolved container.interface");
        val
      }
      Err(e) => {
        err = Some(e);
        OptionalBoolean::None
      }
    };

    debug!("Resolving container.build-config");
    let build = match Optional::compile(
      spec.build.clone(),
      resolution_ctx.clone(),
      repo_dir_path.clone(),
    )
    .await
    {
      Ok(val) => {
        debug!("Resolved container.build-config");
        val
      }
      Err(e) => {
        err = Some(e);
        Optional::None
      }
    };

    match err {
      Some(e) => Err(e),
      None => Ok(ContainerSpec { interface, build }),
    }
  }
}

impl ManifestCompilationFrom<RawBuildConfig> for BuildConfig {
  async fn compile(
    spec: RawBuildConfig,
    resolution_ctx: ResolutionCtx,
    repo_dir_path: OsPath,
  ) -> Result<Self, SimpleError>
  where
    Self: Sized,
    RawBuildConfig: for<'a> Deserialize<'a>,
  {
    let mut err = None;

    debug!("Resolving build-config.url");
    let url_field = match RequiredString::compile(
      spec.url.clone(),
      resolution_ctx.clone(),
      repo_dir_path.clone(),
    )
    .await
    {
      Ok(val) => {
        debug!("Resolved build-config.url");
        val
      }
      Err(e) => {
        err = Some(e);
        Default::default()
      }
    };

    debug!("Resolving build-config.creds");
    let creds = match Optional::compile(
      spec.creds.clone(),
      resolution_ctx.clone(),
      repo_dir_path.clone(),
    )
    .await
    {
      Ok(val) => {
        debug!("Resolved build-config.creds");
        val
      }
      Err(e) => {
        err = Some(e);
        Optional::None
      }
    };

    match err {
      Some(e) => Err(e),
      None => Ok(BuildConfig {
        url: url_field,
        creds,
      }),
    }
  }
}

impl ManifestCompilationFrom<RawRepoCreds> for RepoCreds {
  async fn compile(
    spec: RawRepoCreds,
    resolution_ctx: ResolutionCtx,
    repo_dir_path: OsPath,
  ) -> Result<Self, SimpleError>
  where
    Self: Sized,
    RawRepoCreds: for<'a> Deserialize<'a>,
  {
    let mut err = None;

    debug!("Resolving repo-creds.username");
    let username = match OptionalString::compile(
      spec.username.clone(),
      resolution_ctx.clone(),
      repo_dir_path.clone(),
    )
    .await
    {
      Ok(val) => {
        debug!("Resolved repo-creds.username");
        val
      }
      Err(e) => {
        err = Some(e);
        OptionalString::None
      }
    };

    debug!("Resolving repo-creds.key");
    let key = match RequiredString::compile(
      spec.key.clone(),
      resolution_ctx.clone(),
      repo_dir_path.clone(),
    )
    .await
    {
      Ok(val) => {
        debug!("Resolved repo-creds.key");
        val
      }
      Err(e) => {
        err = Some(e);
        Default::default()
      }
    };

    match err {
      Some(e) => Err(e),
      None => Ok(RepoCreds { username, key }),
    }
  }
}

// TODO: move to clover::server::modman::models::Module
impl ManifestCompilationFrom<RawModuleSpec> for ModuleSpec {
  async fn compile(
    spec: RawModuleSpec,
    resolution_ctx: ResolutionCtx,
    repo_dir_path: OsPath,
  ) -> Result<Self, SimpleError>
  where
    Self: Sized,
  {
    let mut err = None;

    debug!("Resolving module.name");
    let name = match OptionalString::compile(
      spec.name.clone(),
      resolution_ctx.clone(),
      repo_dir_path.clone(),
    )
    .await
    {
      Ok(val) => {
        debug!("Resolved module.name");
        val
      }
      Err(e) => {
        err = Some(e);
        OptionalString::None
      }
    };

    match err {
      Some(e) => Err(e),
      None => Ok(Self { name }),
    }
  }
}

impl ManifestCompilationFrom<RawGesturePackSpec> for GesturePackSpec {
  async fn compile(
    spec: RawGesturePackSpec,
    resolution_ctx: ResolutionCtx,
    repo_dir_path: OsPath,
  ) -> Result<Self, SimpleError>
  where
    Self: Sized,
  {
    let mut err = None;

    debug!("Resolving gesture-pack.name");
    let name = match OptionalString::compile(
      spec.name.clone(),
      resolution_ctx.clone(),
      repo_dir_path.clone(),
    )
    .await
    {
      Ok(val) => {
        debug!("Resolved gesture-pack.name");
        val
      }
      Err(e) => {
        err = Some(e);
        OptionalString::None
      }
    };

    debug!("Resolving gesture-pack.gestures");
    let gestures = match OptionalStrTHashMap::compile(
      spec.gestures.clone(),
      resolution_ctx.clone(),
      repo_dir_path.clone(),
    )
    .await
    {
      Ok(val) => {
        debug!("Resolved gesture-pack.gestures");
        val
      }
      Err(e) => {
        err = Some(e);
        OptionalStrTHashMap::None
      }
    };

    match err {
      Some(e) => Err(e),
      None => Ok(GesturePackSpec { name, gestures }),
    }
  }
}

impl ManifestCompilationFrom<RawGestureSpec> for GestureSpec {
  async fn compile(
    spec: RawGestureSpec,
    resolution_ctx: ResolutionCtx,
    repo_dir_path: OsPath,
  ) -> Result<Self, SimpleError>
  where
    Self: Sized,
    RawGestureSpec: for<'a> Deserialize<'a>,
  {
    let mut err = None;

    debug!("Resolving raw-gesture-pack");
    let ret = match spec {
      RawGestureSpec::RawStaticGestureSpec(raw_gesture_spec) => match StaticGestureSpec::compile(
        raw_gesture_spec,
        resolution_ctx.clone(),
        repo_dir_path.clone(),
      )
      .await
      {
        Ok(val) => {
          debug!("Resolved raw-gesture-pack");
          Self::StaticGestureSpec(val)
        }
        Err(e) => {
          err = Some(e);
          Self::StaticGestureSpec(StaticGestureSpec {
            static_url: RequiredString(String::from("bleh")),
          })
        }
      },
    };

    match err {
      Some(e) => Err(e),
      None => Ok(ret),
    }
  }
}

impl ManifestCompilationFrom<RawStaticGestureSpec> for StaticGestureSpec {
  async fn compile(
    spec: RawStaticGestureSpec,
    resolution_ctx: ResolutionCtx,
    repo_dir_path: OsPath,
  ) -> Result<Self, SimpleError>
  where
    Self: Sized,
    RawStaticGestureSpec: for<'a> Deserialize<'a>,
  {
    let mut err = None;

    debug!("Resolving static-gesture-pack.static-url");
    let static_url = match RequiredString::compile(
      spec.static_url.clone(),
      resolution_ctx.clone(),
      repo_dir_path.clone(),
    )
    .await
    {
      Ok(val) => {
        debug!("Resolved static-gesture-pack.static-url");
        val
      }
      Err(e) => {
        err = Some(e);
        Default::default()
      }
    };

    match err {
      Some(e) => Err(e),
      None => Ok(Self { static_url }),
    }
  }
}
