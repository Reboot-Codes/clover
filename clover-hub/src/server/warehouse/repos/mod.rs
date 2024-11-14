pub mod models;

use std::{collections::HashMap, sync::Arc};
use git2::{BranchType, FileFavor, MergeOptions, Repository};
use log::{info, warn};
use os_path::OsPath;
use simple_error::SimpleError;

use crate::server::evtbuzz::models::Store;
use super::config::models::RepoSpec;

pub struct Error(SimpleError);

enum RepoDirTreeEntry {
  String,
  HashMap(String, Box<RepoDirTreeEntry>)
}

pub async fn update_repo_dir_structure(repos: HashMap<String, RepoSpec>) -> Result<(), Error> {
  let mut err = None;
  let mut repo_dir_tree: HashMap<String, RepoDirTreeEntry> = HashMap::new();

  // Build the tree where strings are the source url (and therefore during dir creation, create a directory called `@repo` under it), and hashmaps are more directories to create
  for (repo_id, repo_spec) in repos {
    for id_segment in repo_id.split(".") {

    }
  }

  // Recursively create directories following the tree structure

  match err {
    Some(e) => { Err(e) },
    None => { Ok(()) }
  }
}

impl From<git2::Error> for Error {
  fn from(value: git2::Error) -> Self {
    Error(SimpleError::from(value))
  }
}

pub async fn download_repo_updates(repos: HashMap<String, RepoSpec>, store: Arc<Store>, repo_dir_path: OsPath) -> Result<(), Error> {
  let mut err = None;
  let mut repos_updated = 0;

  for (repo_id, repo_spec) in repos {
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

    if repo_dir_path.join("/.git/").is_dir() {
      match Repository::open(repo_dir_path.to_string()) {
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
                                info!("Updated {} to {} ({})!", repo_str, comm.message().unwrap_or("*no message*"), comm.id());
                              },
                              Err(e) => {

                              }
                            }
                          },
                          Err(e) => {
                            repo.cleanup_state();
                          }
                        }
                      }
                    },
                    Err(e) => {

                    }
                  }
                },
                None => {
                  warn!("No remote source for {}!", repo_str.clone());
                }
              }
            },
            Err(e) => {
              match Repository::clone_recurse(&repo_spec.src.clone(), repo_path.clone()) {
                Ok(repo) => {

                },
                Err(e) => {

                }
              }
            }
          }
        },
        Err(e) => {

        }
      }
    }
  }

  match err {
    Some(e) => { Err(e) },
    None => { 
      if repos_updated > 0 { info!("Updated {} repo(s)!", repos_updated); }
      Ok(()) 
    }
  }
}
