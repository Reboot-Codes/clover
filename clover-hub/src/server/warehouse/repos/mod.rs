pub mod impls;
pub mod models;

// TODO: Implement YAML and TOML parsing as well.

use crate::utils::read_file;
use git2::{
  build::CheckoutBuilder,
  BranchType,
  Repository,
};
use log::{
  debug,
  error,
  info,
  warn,
};
use models::{
  Manifest,
  ManifestCompilationFrom,
  ManifestSpec,
  OptionalString,
  RequiredSingleManifestEntry,
  Resolution,
  ResolutionCtx,
};
use os_path::OsPath;
use regex::Regex;
use serde::Deserialize;
use simple_error::SimpleError;
use std::{
  collections::HashMap,
  sync::Arc,
};
use tokio::{
  fs,
  io::AsyncReadExt,
};
use tokio_stream::{
  wrappers::ReadDirStream,
  StreamExt,
};

use super::models::WarehouseStore;

#[derive(PartialEq)]
pub struct Error(pub SimpleError);

impl From<git2::Error> for Error {
  fn from(value: git2::Error) -> Self {
    Error(SimpleError::from(value))
  }
}

pub fn builtin_rfqdn(is_core: bool) -> String {
  if is_core {
    String::from("com.reboot-codes.clover.CORE")
  } else {
    String::from("com.reboot-codes.clover")
  }
}

pub fn replace_simple_directives(value: String, resolution_ctx: ResolutionCtx) -> String {
  debug!(
    "replace_simple_directives (provided): {} + {:#?}",
    value.clone(),
    resolution_ctx.clone()
  );

  let base_re = Regex::new("(?<directive>\\@base)").unwrap();
  let here_re = Regex::new("(?<directive>\\@here)").unwrap();
  let builtin_re = Regex::new("(?<directive>\\@builtin)(\\:(?<domain>core|clover))?").unwrap();

  let mut val = value.clone();
  match resolution_ctx.base {
    Some(base) => {
      val = String::from(base_re.replace_all(&value.clone(), base));
    }
    None => {}
  }

  let binding1 = val.clone();
  let here_import_temp = here_re.replace_all(&binding1, "");
  // Did the regex replace an `@here` directive? If so, append the stripped value to the "here" path.
  let val = if here_import_temp != val {
    let mut val_unsafe_path = resolution_ctx.here.join(&here_import_temp.to_string());
    val_unsafe_path.resolve();

    if val_unsafe_path
      .to_string()
      .starts_with(&resolution_ctx.here.to_string())
    {
      val_unsafe_path.to_string()
    } else {
      warn!(
        "Directory resolution for \"{}\" (resolved to: \"{}\") is not confined to repo directory, removing entry value entirely since this is a security issue.",
        binding1.clone(),
        val_unsafe_path.to_string()
      );
      "".to_string()
    }
  } else {
    val
  };

  let binding2 = val.clone();
  let builtin_import_temp = builtin_re.replace_all(&binding2, "");
  let val = if builtin_import_temp != val {
    match builtin_re.captures(&binding2).unwrap().name("domain") {
      Some(domain) => {
        if domain.as_str() == "core" {
          builtin_re
            .replace_all(&binding2, builtin_rfqdn(true))
            .to_string()
        } else if domain.as_str() == "clover" {
          builtin_re
            .replace_all(&binding2, builtin_rfqdn(false))
            .to_string()
        } else {
          error!(
            "No case for domain: \"{}\", yet it's in the directive regex, this is a bug, please report it; refusing to resolve built-in directive (value unchanged)!",
            domain.as_str()
          );
          binding2.clone()
        }
      }
      None => builtin_re
        .replace_all(&binding2, resolution_ctx.builtin.to_string())
        .to_string(),
    }
  } else {
    val
  };

  debug!("replace simple directives (completed): {}", val.clone());

  String::from(val)
}

pub async fn resolve_list_entry<T, K>(
  raw_list: HashMap<String, RequiredSingleManifestEntry<T>>,
  resolution_ctx: ResolutionCtx,
  repo_dir_path: OsPath,
) -> Result<HashMap<String, K>, SimpleError>
where
  K: ManifestCompilationFrom<T>,
  T: for<'a> Deserialize<'a>,
{
  let mut err = None;
  let mut entries = HashMap::new();
  let glob_import_key_re = Regex::new("^(?<base>[^\\*\\n\\r]+)(\\*)$").unwrap();

  for (key, raw_entry) in raw_list {
    let is_glob = glob_import_key_re.is_match(&key);
    let mut entry_err = None;

    match raw_entry {
      RequiredSingleManifestEntry::ImportString(str) => {
        match resolve_entry_value(str, resolution_ctx.clone(), repo_dir_path.clone()).await {
          Ok(resolution) => match resolution {
            Resolution::ImportedSingle((here, raw_obj)) => {
              if is_glob {
                err = Some(SimpleError::new(
                  "Resolved only one file for glob key import, import the root key instead!",
                ));
              } else {
                match serde_json_lenient::from_str::<T>(&raw_obj) {
                  Ok(obj_spec) => {
                    match K::compile(
                      obj_spec,
                      ResolutionCtx {
                        base: resolution_ctx.clone().base,
                        builtin: resolution_ctx.clone().builtin,
                        here: here.clone(),
                      },
                      repo_dir_path.clone(),
                    )
                    .await
                    {
                      Ok(obj) => {
                        entries.insert(
                          replace_simple_directives(
                            key.clone(),
                            ResolutionCtx {
                              base: resolution_ctx.clone().base,
                              builtin: resolution_ctx.clone().builtin,
                              here: here.clone(),
                            },
                          ),
                          obj,
                        );
                      }
                      Err(e) => {
                        entry_err = Some(e);
                      }
                    }
                  }
                  Err(e) => {
                    entry_err = Some(SimpleError::from(e));
                  }
                }
              }
            }
            Resolution::ImportedMultiple((here, raw_objs)) => {
              if is_glob {
                for (obj_key_seg, raw_obj) in raw_objs {
                  match serde_json_lenient::from_str::<T>(&raw_obj) {
                    Ok(obj_spec) => {
                      match K::compile(
                        obj_spec,
                        ResolutionCtx {
                          base: resolution_ctx.clone().base,
                          builtin: resolution_ctx.clone().builtin,
                          here: here.clone(),
                        },
                        repo_dir_path.clone(),
                      )
                      .await
                      {
                        Ok(obj) => {
                          let obj_key_prod = replace_simple_directives(
                            ["@base".to_string(), obj_key_seg].join("."),
                            ResolutionCtx {
                              base: resolution_ctx.clone().base,
                              builtin: resolution_ctx.clone().builtin,
                              here: here.clone(),
                            },
                          );
                          debug!("Resolution::ImportedMultiple: {}", obj_key_prod);

                          entries.insert(obj_key_prod, obj);
                        }
                        Err(e) => {
                          entry_err = Some(e);
                        }
                      }
                    }
                    Err(e) => {
                      entry_err = Some(SimpleError::from(e));
                    }
                  }
                }
              }
            }
            Resolution::NoImport(raw_obj) => match serde_json_lenient::from_str::<T>(&raw_obj) {
              Ok(obj_spec) => {
                match K::compile(obj_spec, resolution_ctx.clone(), repo_dir_path.clone()).await {
                  Ok(obj) => {
                    entries.insert(key.clone(), obj);
                  }
                  Err(e) => {
                    entry_err = Some(e);
                  }
                }
              }
              Err(e) => {
                entry_err = Some(SimpleError::from(e));
              }
            },
          },
          Err(e) => {
            err = Some(e);
          }
        }
      }
      RequiredSingleManifestEntry::Some(obj_spec) => {
        match K::compile(obj_spec, resolution_ctx.clone(), repo_dir_path.clone()).await {
          Ok(obj) => {
            entries.insert(key.clone(), obj);
          }
          Err(e) => {
            entry_err = Some(e);
          }
        }
      }
    }

    match entry_err {
      Some(e) => {
        error!(
          "Error while parsing entry \"{}\", in {}:\n{}",
          key.clone(),
          resolution_ctx.here.to_string(),
          e
        );
      }
      None => {}
    }
  }

  match err {
    Some(e) => Err(e),
    None => Ok(entries),
  }
}

pub async fn update_repo_dir_structure(
  repo_dir_path: OsPath,
  store: Arc<WarehouseStore>,
) -> Result<(), Error> {
  let mut err = None;
  let repos = store.config.lock().await.repos.clone();

  for (repo_id, _repo_spec) in repos {
    let repo_id_segments = OsPath::from(repo_dir_path.clone().to_string())
      .join(repo_id.split(".").collect::<Vec<&str>>().join("/"))
      .join("@repo");

    if !repo_id_segments.exists() {
      match fs::create_dir_all(repo_id_segments.to_string()).await {
        Ok(_) => {
          debug!("Created directory: {}!", repo_id_segments.to_string());
        }
        Err(e) => {
          err = Some(Error(SimpleError::from(e)));
          error!(
            "Failed to create repo directory: {}!",
            repo_id_segments.to_string()
          );
          break;
        }
      }
    }
  }

  match err {
    Some(e) => Err(e),
    None => Ok(()),
  }
}

/// Used to resolve repo manifest entry **values** that may have directives (`@import`, `@base`, `@here`) in them.
pub async fn resolve_entry_value(
  value: String,
  resolution_ctx: ResolutionCtx,
  repo_dir_path: OsPath,
) -> Result<Resolution, SimpleError> {
  let import_re = Regex::new("^\\@import\\(('|\"|`)(?<src>.+)('|\"|`)\\)$").unwrap();
  let mut ret: Resolution = Resolution::NoImport(value.clone());
  let mut err = None;

  debug!("resolve_entry_value: {}", value.clone());

  if import_re.is_match(&value.clone()) {
    let raw_import_path = OsPath::new().join(
      resolution_ctx
        .here
        .parent()
        .unwrap_or(OsPath::new().join("/"))
        .join(
          import_re
            .captures(&value.clone())
            .unwrap()
            .name("src")
            .unwrap()
            .as_str(),
        )
        .to_string(),
    );
    let mut import_path = OsPath::new();
    let mut segments = 0;

    for import_path_seg in raw_import_path.to_path() {
      segments += 1;

      if import_path_seg != "." {
        if (segments == 1) && raw_import_path.is_absolute() {
          import_path = OsPath::from("/".to_string() + import_path_seg.to_str().unwrap_or(""))
        } else {
          import_path.push(import_path_seg.to_str().unwrap_or(""));
        }
      }
    }

    import_path.resolve();

    if !import_path
      .to_string()
      .starts_with(&repo_dir_path.to_string())
    {
      return Err(SimpleError::new(format!(
        "Path: \"{}\", is not confined within the repository root (\"{}\"). Refusing to evaluate.",
        import_path.to_string(),
        repo_dir_path.to_string()
      )));
    }

    debug!("Attempting to import \"{}\"...", import_path.to_string());

    let glob_import_re = Regex::new("^(?<base>[^\\*\\n\\r]+)(\\*)(?<cap>[^\\*\\n\\r]*)").unwrap();
    let is_glob = glob_import_re.is_match(&import_path.clone().to_string());

    if is_glob {
      let import_path_str = import_path.clone().to_string();
      let import_captures = glob_import_re.captures(&import_path_str).unwrap();
      let here = OsPath::new().join(import_captures.name("base").unwrap().as_str());

      match fs::read_dir(&here.clone().to_path()).await {
        Ok(dir) => {
          let mut entries = HashMap::new();
          let mut failed_entries = Vec::new();
          let mut dir_stream = ReadDirStream::new(dir);

          while let Some(entry_res) = dir_stream.next().await {
            match entry_res {
              Ok(entry) => {
                let mut file_path = OsPath::from(entry.path());
                let cap = match import_captures.name("cap") {
                  Some(val) => val.as_str(),
                  None => "/manifest.clover.jsonc",
                };

                debug!(
                  "Attempting to import from glob: \"{}\"...",
                  entry.path().to_str().unwrap()
                );

                if entry.path().is_dir() {
                  file_path.push(cap);
                }

                let base = match import_captures.name("base") {
                  Some(val) => val.as_str(),
                  None => "",
                };

                if file_path.to_string().ends_with(&cap) {
                  match read_file(file_path.clone()).await {
                    Ok(contents) => {
                      debug!(
                        "resolve_entry_value {}:\n{}",
                        file_path.clone().to_string(),
                        contents.clone()
                      );
                      entries.insert(
                        replace_simple_directives(
                          file_path.to_string().replace(base, "").replace(cap, ""),
                          resolution_ctx.clone(),
                        ),
                        contents,
                      );
                    }
                    Err(e) => {
                      failed_entries.push(e);
                    }
                  }
                }
              }
              Err(e) => failed_entries.push(SimpleError::from(e)),
            }
          }

          ret = Resolution::ImportedMultiple((here, entries));
        }
        Err(e) => {
          err = Some(SimpleError::from(e));
        }
      }
    } else if import_path.exists() {
      match read_file(import_path.clone()).await {
        Ok(contents) => {
          debug!("{}", contents.clone());
          ret = Resolution::ImportedSingle((import_path.clone(), contents));
        }
        Err(e) => {
          err = Some(e);
        }
      }
    } else {
      err = Some(SimpleError::new(format!(
        "Invalid import path: \"{}\"!",
        import_path.clone().to_string()
      )));
    }
  } else {
    ret = Resolution::NoImport(replace_simple_directives(
      value.clone(),
      resolution_ctx.clone(),
    ));
  }

  match err {
    Some(e) => Err(e),
    None => Ok(ret),
  }
}

pub async fn download_repo_updates(
  store: Arc<WarehouseStore>,
  repo_dir_path: OsPath,
) -> Result<(), Error> {
  let mut err = None;
  let mut repos_updated = 0;
  let repos = store.config.lock().await.repos.clone();
  let mut repo_errors: Vec<Error> = Vec::new();

  info!("Running updates on {} repo(s)...", repos.len());

  for (repo_id, repo_spec) in repos.clone() {
    let mut repo_err = None;
    let repo_path = OsPath::new()
      .join(repo_dir_path.clone().to_string())
      .join(repo_id.split(".").collect::<Vec<&str>>().join("/"))
      .join("/@repo/");

    let repo_str;
    match repo_spec.name {
      Some(name) => {
        repo_str = format!("{} ({})", name, repo_id.clone());
      }
      None => {
        repo_str = repo_id.clone();
      }
    }

    debug!("Repo: {}, checking for updates...", repo_str.clone());

    if err == None {
      if repo_path.join("/.git/").exists() {
        match Repository::open(repo_path.to_string()) {
          Ok(repo) => {
            match repo.remotes() {
              Ok(remotes) => {
                let mut main_remote = None;
                for remote_name in remotes.into_iter() {
                  match remote_name {
                    None => {}
                    Some(remote_name_str) => {
                      match repo.find_remote(remote_name_str) {
                        Ok(remote) => {
                          // Fetch the url
                          match remote.url() {
                            Some(remote_url) => {
                              if remote_url == repo_spec.src {
                                main_remote = Some(remote);
                                break;
                              }
                            }
                            None => {}
                          }
                        }
                        Err(e) => {
                          error!(
                            "Repo: {}, failed to get specified remote, due to:\n{}",
                            repo_str.clone(),
                            e
                          );
                          repo_err = Some(Error(SimpleError::from(e)));
                        }
                      }
                    }
                  }
                }

                match main_remote {
                  Some(mut remote) => {
                    let remote_branch_name = repo_spec.branch.clone();

                    match remote.fetch(&[remote_branch_name.clone()], None, None) {
                      Ok(_) => {
                        let remote_branch = repo.find_branch(
                          &format!("{}/{}", remote.name().unwrap(), remote_branch_name.clone()),
                          BranchType::Remote,
                        )?;
                        if remote_branch.is_head()
                          && (remote_branch.get().resolve()?.target().unwrap()
                            == repo.head().unwrap().resolve()?.target().unwrap())
                        {
                        } else {
                          match repo.checkout_tree(
                            remote_branch.get().peel_to_tree().unwrap().as_object(),
                            Some(CheckoutBuilder::new().conflict_style_merge(true).force()),
                          ) {
                            Ok(_) => match repo.cleanup_state() {
                              Ok(_) => {
                                repos_updated += 1;
                                let comm = repo.head()?.peel_to_commit()?;
                                let comm_str;
                                match comm.message() {
                                  Some(message) => {
                                    comm_str = format!("{}, ({})", message, comm.id());
                                  }
                                  None => {
                                    comm_str = comm.id().to_string();
                                  }
                                }

                                info!(
                                  "Repo: {}, Updated, now using commit: {}!",
                                  repo_str, comm_str
                                );
                              }
                              Err(e) => {
                                error!(
                                  "Repo: {}, failed to merge commits due to:\n{}",
                                  repo_str, e
                                );
                                repo_err = Some(Error(SimpleError::from(e)));
                              }
                            },
                            Err(e) => {
                              error!("Repo: {}, Update failed, due to:\n{}", repo_str.clone(), e);

                              match repo.cleanup_state() {
                                Ok(_) => {
                                  debug!("Repo: {}, Cleaned up.", repo_str.clone());
                                }
                                Err(e) => {
                                  error!(
                                    "Repo: {}, failed to clean up, due to:\n{}",
                                    repo_str.clone(),
                                    e
                                  );
                                  repo_err = Some(Error(SimpleError::from(e)));
                                }
                              }
                            }
                          }
                        }
                      }
                      Err(e) => {
                        if e.class() == git2::ErrorClass::Net {
                          info!(
                            "Repo: {}, was unable to connect to remote server, skipping updates...",
                            repo_str.clone()
                          );
                          debug!("Repo: {}, network error, due to:\n{}", repo_str.clone(), e);
                        } else {
                          error!(
                            "Repo: {}, failed to fetch updates due to:\n{}",
                            repo_str.clone(),
                            e
                          );
                          repo_err = Some(Error(SimpleError::from(e)));
                        }
                      }
                    }
                  }
                  None => {
                    warn!("Repo: {}, No remote source!", repo_str.clone());
                  }
                }
              }
              Err(e) => {
                error!(
                  "Repo: {}, Failed to get remotes, due to:\n{}",
                  repo_str.clone(),
                  e
                );
                repo_err = Some(Error(SimpleError::from(e)));
              }
            }
          }
          Err(e) => {
            error!(
              "Repo: {}, failed to open git repository, due to:\n{}",
              repo_str.clone(),
              e
            );
            repo_err = Some(Error(SimpleError::from(e)));
          }
        }
      } else {
        match Repository::clone_recurse(&repo_spec.src.clone(), repo_path.clone()) {
          Ok(repo) => {
            repos_updated += 1;
            let comm = repo.head()?.peel_to_commit()?;
            let comm_str;
            match comm.message() {
              Some(message) => {
                comm_str = format!("{}, ({})", message, comm.id());
              }
              None => {
                comm_str = comm.id().to_string();
              }
            }

            info!(
              "Repo: {}, Downloaded, now using commit: {}!",
              repo_str, comm_str
            );
          }
          Err(e) => {
            error!("Repo: {}, failed to clone, due to:\n{}", repo_str, e);
            repo_err = Some(Error(SimpleError::from(e)));
          }
        }
      }
    }

    if (repo_err == None) && (err == None) {
      // Build manifest object and load it into the store.
      let manifest_path = repo_path.join("/manifest.clover.jsonc");
      if manifest_path.exists() {
        match fs::File::open(manifest_path.clone()).await {
          Ok(mut manifest_file) => {
            let mut contents = String::new();

            match manifest_file.read_to_string(&mut contents).await {
              Ok(_) => {
                debug!("{}", contents.clone());

                match serde_json_lenient::from_str::<ManifestSpec>(&contents) {
                  Ok(raw_manifest_values) => {
                    debug!("{:#?}", raw_manifest_values.clone());

                    match Manifest::compile(
                      raw_manifest_values,
                      manifest_path.clone(),
                      manifest_path.parent().unwrap().clone(),
                    )
                    .await
                    {
                      Ok(manifest) => {
                        let manifest_str;
                        match manifest.name.clone() {
                          OptionalString::Some(name) => {
                            manifest_str = format!("{} ({})", name, repo_id.clone());
                          }
                          OptionalString::None => {
                            manifest_str = repo_id.clone();
                          }
                        }

                        store
                          .repos
                          .lock()
                          .await
                          .insert(repo_id.clone(), manifest.clone());
                        info!("Loaded manifest: {}!", manifest_str);
                        debug!("Imported manifest: {:#?}", manifest.clone());
                      }
                      Err(e) => {
                        error!(
                          "Repo: {}, failed to compile manifest, due to:\n{}",
                          repo_str, e
                        );
                        repo_err = Some(Error(e));
                      }
                    }
                  }
                  Err(e) => {
                    error!(
                      "Repo: {}, failed to parse manifest file, due to:\n{}",
                      repo_str, e
                    );
                    repo_err = Some(Error(SimpleError::from(e)));
                  }
                }
              }
              Err(e) => {
                error!(
                  "Repo: {}, failed to read manifest file, due to:\n{}",
                  repo_str, e
                );
                repo_err = Some(Error(SimpleError::from(e)));
              }
            }
          }
          Err(e) => {
            error!(
              "Repo: {}, failed to open manifest file, due to:\n{}",
              repo_str, e
            );
            repo_err = Some(Error(SimpleError::from(e)));
          }
        }
      }
    }

    if repo_err != None {
      repo_errors.push(repo_err.unwrap());
    }
  }

  if (repo_errors.len() == repos.len()) && (repos.len() > 0) {
    err = Some(Error(SimpleError::new(
      "Failed to download/update all repos... this may point to a larger problem!",
    )));
  }

  match err {
    Some(e) => Err(e),
    None => {
      if repo_errors.len() > 0 {
        warn!(
          "{} repos failed to check for updates and/or to upgrade!",
          repo_errors.len()
        );
      }

      if repos_updated > 0 {
        info!("Updated {} repo(s)!", repos_updated);
      }

      info!("Finished repo update and upgrade check!");

      Ok(())
    }
  }
}
