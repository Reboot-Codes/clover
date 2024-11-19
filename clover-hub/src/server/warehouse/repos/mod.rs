pub mod models;
pub mod impls;

use tokio_stream::{wrappers::ReadDirStream, StreamExt};
use std::{collections::HashMap, sync::Arc};
use git2::{BranchType, FileFavor, MergeOptions, Repository};
use log::{debug, error, info, warn};
use models::{Manifest, ManifestCompilationFrom, ManifestSpec, OptionalString, RequiredSingleManifestEntry, Resolution, ResolutionCtx};
use os_path::OsPath;
use regex::Regex;
use serde::Deserialize;
use simple_error::SimpleError;
use tokio::{fs, io::AsyncReadExt};
use crate::{server::evtbuzz::models::Store, utils::read_file};

#[derive(PartialEq)]
pub struct Error(pub SimpleError);

impl From<git2::Error> for Error {
  fn from(value: git2::Error) -> Self {
    Error(SimpleError::from(value))
  }
}

pub async fn resolve_list_entry<T, K>(raw_list: HashMap<String, RequiredSingleManifestEntry<T>>, resolution_ctx: ResolutionCtx) -> Result<HashMap<String, K>, SimpleError> 
  where K: ManifestCompilationFrom<T>, T: for<'a> Deserialize<'a>
{
  let mut err = None;
  let mut entries = HashMap::new();
  let glob_import_key_re = Regex::new("^(?<base>[^\\*\\n\\r]+)(\\*)$").unwrap();

  for (key, raw_entry) in raw_list {
    let is_glob = glob_import_key_re.is_match(&key);
    let mut entry_err = None;

    match raw_entry {
      RequiredSingleManifestEntry::ImportString(str) => {
        match resolve_entry_value(str, resolution_ctx.clone()).await {
          Ok(resolution) => {
            match resolution {
              Resolution::ImportedSingle(raw_obj) => {
                if is_glob {
                  err = Some(SimpleError::new("Resolved only one file for glob key import, import the root key instead!"));
                } else {
                  match serde_jsonc::from_str::<T>(&raw_obj) {
                    Ok(obj_spec) => {
                      match K::compile(obj_spec, resolution_ctx.clone()).await {
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
                        match K::compile(obj_spec, resolution_ctx.clone()).await {
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
              },
              Resolution::NoImport(raw_obj) => {
                match serde_jsonc::from_str::<T>(&raw_obj) {
                  Ok(obj_spec) => {
                    match K::compile(obj_spec, resolution_ctx.clone()).await {
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
            }
          },
          Err(e) => {
            err = Some(e);
          }
        }
      },
      RequiredSingleManifestEntry::Some(obj_spec) => {
        match K::compile(obj_spec, resolution_ctx.clone()).await {
          Ok(obj) => {
            entries.insert(key.clone(), obj);
          },
          Err(e) => {
            entry_err = Some(e);
          }
        }
      }
    }

    match entry_err {
      Some(e) => {
        error!("Error while parsing entry \"{}\", in {}:\n{}", key.clone(), resolution_ctx.here.to_string(), e);
      },
      None => {}
    }
  }

  match err {
    Some(e) => { Err(e) },
    None => { Ok(entries) }
  }
}

pub async fn update_repo_dir_structure(store: Arc<Store>) -> Result<(), Error> {
  let mut err = None;
  let repos = store.config.lock().await.repos.clone();

  for (repo_id, _repo_spec) in repos {
    let repo_id_segments = OsPath::from(repo_id.split(".").collect::<Vec<&str>>().join("/")).join("@repo");

    if !repo_id_segments.exists() {
      match fs::create_dir_all(repo_id_segments.to_string()).await {
        Ok(_) => {
          debug!("Created directory: {}!", repo_id_segments.to_string());
        },
        Err(e) => {
          err = Some(Error(SimpleError::from(e)));
          error!("Failed to create repo directory: {}!", repo_id_segments.to_string());
          break;
        }
      }
    }
  }

  match err {
    Some(e) => { Err(e) },
    None => { Ok(()) }
  }
}

/// Used to resolve repo manifest entry **values** that may have directives (`@import`, `@base`, `@here`) in them.
pub async fn resolve_entry_value(value: String, resolution_ctx: ResolutionCtx) -> Result<Resolution, SimpleError> {
  let import_re = Regex::new("^\\@import\\(('|\"|`)(?<src>.+)('|\"|`)\\)$").unwrap();
  let base_re = Regex::new("(?<directive>\\@base)").unwrap();
  let here_re = Regex::new("(?<directive>\\@here)").unwrap();
  let mut ret: Resolution = Resolution::NoImport(value.clone());
  let mut err = None;

  if import_re.is_match(&value.clone()) {
    let mut import_path = OsPath::new().join(
      resolution_ctx.here.join(import_re.captures(&value.clone()).unwrap().name("src").unwrap().as_str()).to_string()
    );

    let glob_import_re = Regex::new("^(?<base>[^\\*\\n\\r]+)(\\*)(?<cap>[^\\*\\n\\r]*)").unwrap();
    if glob_import_re.is_match(&import_path.clone().to_string()) {
      let import_path_str = import_path.clone().to_string();
      let import_captures = glob_import_re.captures(&import_path_str).unwrap();

      match fs::read_dir(&OsPath::new().join(import_captures.name("base").unwrap().as_str()).to_path()).await {
        Ok(dir) => {
          let mut entries = HashMap::new();
          let mut failed_entries = Vec::new();
          let mut dir_stream = ReadDirStream::new(dir);

          while let Some(entry_res) = dir_stream.next().await {
            match entry_res {
              Ok(entry) => {
                let mut file_path = OsPath::from(entry.path());

                if entry.path().is_dir() {
                  let cap = import_captures.name("cap").unwrap().as_str();
                  
                  if cap == "" {
                    file_path.push("/manifest.clover.jsonc");
                  } else {
                    file_path.push(cap);
                  }
                }

                match read_file(file_path.clone()).await {
                  Ok(contents) => {
                    entries.insert(file_path.name().unwrap().clone(), contents);
                  },
                  Err(e) => {
                    failed_entries.push(e);
                  }
                }
              },
              Err(e) => {
                failed_entries.push(SimpleError::from(e))
              }
            }
          }

          ret = Resolution::ImportedMultiple(entries);
        },
        Err(e) => {
          err = Some(SimpleError::from(e));
        }
      }
    }

    import_path.resolve();

    if import_path.exists() {
      match read_file(import_path.clone()).await {
        Ok(contents) => {
          ret = Resolution::ImportedSingle(contents);
        },
        Err(e) => {
          err = Some(e);
        }
      }
    } else {
      err = Some(SimpleError::new("Invalid import path!"));
    }
  } else {
    let mut val = value.clone();
    match resolution_ctx.base {
      Some(base) => {
        val = String::from(base_re.replace(&value.clone(), base));
      },
      None => {}
    }

    val = String::from(here_re.replace(&val.clone(), resolution_ctx.here.to_string()));

    ret = Resolution::NoImport(val);
  }

  match err {
    Some(e) => { Err(e) },
    None => { Ok(ret) }
  }
}

pub async fn download_repo_updates(store: Arc<Store>, repo_dir_path: OsPath) -> Result<(), Error> {
  let mut err = None;
  let mut repos_updated = 0;
  let repos = store.config.lock().await.repos.clone();
  let mut repo_errors: Vec<Error> = Vec::new();

  info!("Running updates on {} repo(s)...", repos.len());

  for (repo_id, repo_spec) in repos.clone() {
    let mut repo_err = None;

    let mut repo_path = OsPath::new().join(repo_dir_path.clone().to_string()).join("/@repo/");
    for id_segment in repo_id.split(".").collect::<Vec<&str>>() {
      repo_path.push(id_segment);
    }
    
    let repo_str;
    match repo_spec.name {
      Some(name) => {
        repo_str = format!("{} ({})", name, repo_id.clone());
      },
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
                    None => {},
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
                            },
                            None => {}
                          }
                        },
                        Err(e) => {
                          error!("Repo: {}, failed to get specified remote, due to:\n{}", repo_str.clone(), e);
                          repo_err = Some(Error(SimpleError::from(e)));
                        }
                      }
                    }
                  }
                }

                match main_remote {
                  Some(mut remote) => {
                    match remote.fetch(&[repo_spec.branch.clone()], None, None) {
                      Ok(_) => {
                        let remote_branch = repo.find_branch(&repo_spec.branch.clone(), BranchType::Remote)?;
                        if remote_branch.is_head() && (remote_branch.get().resolve()?.target().unwrap() == repo.head().unwrap().resolve()?.target().unwrap()) {

                        } else {
                          match repo.merge(
                            &[&repo.find_annotated_commit(remote_branch.into_reference().resolve()?.target().unwrap())?], 
                            Some(MergeOptions::new().file_favor(FileFavor::Theirs)), 
                            None
                          ) {
                            Ok(_) => {
                              match repo.cleanup_state() {
                                Ok(_) => {
                                  repos_updated += 1;
                                  let comm = repo.head()?.peel_to_commit()?;
                                  let comm_str;
                                  match comm.message() {
                                    Some(message) => {
                                      comm_str = format!("{}, ({})", message, comm.id());
                                    },
                                    None => {
                                      comm_str = comm.id().to_string();
                                    }
                                  }

                                  info!("Repo: {}, Updated, now using commit: {}!", repo_str, comm_str);
                                },
                                Err(e) => {
                                  repo_err = Some(Error(SimpleError::from(e)));
                                }
                              }
                            },
                            Err(e) => {
                              error!("Repo: {}, Update failed, due to:\n{}", repo_str.clone(), e);

                              match repo.cleanup_state() {
                                Ok(_) => {
                                  debug!("Repo: {}, Cleaned up.", repo_str.clone());
                                },
                                Err(e) => {
                                  error!("Repo: {}, failed to clean up, due to:\n{}", repo_str.clone(), e);
                                  repo_err = Some(Error(SimpleError::from(e)));
                                }
                              }
                            }
                          }
                        }
                      },
                      Err(e) => {
                        error!("Repo: {}, failed to fetch updates due to:\n{}", repo_str.clone(), e);
                        repo_err = Some(Error(SimpleError::from(e)));
                      }
                    }
                  },
                  None => {
                    warn!("Repo: {}, No remote source!", repo_str.clone());
                  }
                }
              },
              Err(e) => {
                error!("Repo: {}, Failed to get remotes, due to:\n{}", repo_str.clone(), e);
                repo_err = Some(Error(SimpleError::from(e)));
              }
            }
          },
          Err(e) => {
            error!("Repo: {}, failed to open git repository, due to:\n{}", repo_str.clone(),e);
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
              },
              None => {
                comm_str = comm.id().to_string();
              }
            }

            info!("Repo: {}, Downloaded, now using commit: {}!", repo_str, comm_str);
          },
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
                match serde_jsonc::from_str::<ManifestSpec>(&contents) {
                  Ok(raw_manifest_values) => {
                    match Manifest::compile(raw_manifest_values, manifest_path.clone()).await {
                      Ok(manifest) => {
                        let manifest_str;
                        match manifest.name.clone() {
                          OptionalString::Some(name) => {
                            manifest_str = format!("{} ({})", name, repo_id.clone());
                          },
                          OptionalString::None => {
                            manifest_str = repo_id.clone();
                          }
                        }

                        store.repos.lock().await.insert(repo_id.clone(), manifest.clone());
                        debug!("Loaded {}'s manifest!", manifest_str);
                      },
                      Err(e) => {
                        error!("Repo: {}, failed to compile manifest, due to:\n{}", repo_str, e);
                        repo_err = Some(Error(e));
                      }
                    }
                  },
                  Err(e) => {
                    error!("Repo: {}, failed to parse manifest file, due to:\n{}", repo_str, e);
                    repo_err = Some(Error(SimpleError::from(e)));
                  }
                }
              },
              Err(e) => {
                error!("Repo: {}, failed to read manifest file, due to:\n{}", repo_str, e);
                repo_err = Some(Error(SimpleError::from(e)));
              }
            }
          },
          Err(e) => {
            error!("Repo: {}, failed to open manifest file, due to:\n{}", repo_str, e);
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
    err = Some(Error(SimpleError::new("Failed to download/update all repos... this may point to a larger problem!")));
  }

  match err {
    Some(e) => { Err(e) },
    None => {
      if repo_errors.len() > 0 {
        warn!("{} repos failed to check for updates and/or to upgrade!", repo_errors.len());
      }

      if repos_updated > 0 { 
        info!("Updated {} repo(s)!", repos_updated);
      }

      info!("Finished repo update and upgrade check!");

      Ok(()) 
    }
  }
}
