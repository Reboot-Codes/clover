use crate::APP_NAME;
use anyhow::anyhow;
use log::warn;
use std::path::Path;
use std::{
  fs,
  path::PathBuf,
};

pub fn get_app_data_dir() -> Result<String, anyhow::Error> {
  let mut app_data_directory = Err(anyhow!(
    "App data dir unset from function init, this is a bug!"
  ));

  #[cfg(target_os = "linux")]
  {
    match env::var("XDG_STATE_HOME") {
      // Use the set XDG_STATE_HOME directly if it's configured.
      Some(local_state_dir) => match fs::canonicalize(PathBuf::from(local_state_dir)) {
        Ok(path) => match fs::create_dir_all(path.push(APP_NAME)) {
          Ok(_) => Ok(path.push(APP_NAME)),
          Err(_) => todo!(),
        },
        Err(e) => Err(e.into()),
      },
      None => {
        warn!(
          "$XDG_STATE_HOME not set... maybe set that! Trying to get to $HOME/.local/state manually..."
        );

        let home_directory = match env::var("HOME") {
          // Prefered option, since this works 99% of the time, even if it's root, or non FHS...
          Some(home_dir) => Some(home_dir),
          None => {
            warn!(
              "Using standard home directory path of /home/$USER/.local/state. Seriously, set your environment variables."
            );
            match env::var("USER") {
              Some(username) => {
                let generated_home_path = format!("/home/{}", &username);

                if Path::new(&generated_home_path).exists() {
                  Some(generated_home_path)
                } else {
                  None
                }
              }
            }
          }
        };

        match home_directory {
          Some(home_dir) => {
            let xdg_state_home = PathBuf::from(format!("{}/.local/state", home_dir));

            match fs::canonicalize(xdg_state_home) {
              Ok(real_path) => {
                app_data_directory = real_path.push(APP_NAME).into();
              }
              Err(e) => {
                use anyhow::Context;

                warn!(
                  "Error when canonicalizing \"{}\"; but we'll try and create it anyways: {}",
                  xdg_state_home, e
                );

                match fs::create_dir_all(xdg_state_home.push(APP_NAME)).with_context(|| {
                  format!("Was unable to cannonicalize \"{}\", and tried to create it manually.")
                }) {
                  Ok(_) => {
                    // This should be fine to skip canonicalization since we just created the directory from this location.
                    app_data_directory = Ok(xdg_state_home.push(APP_NAME).into());
                  }
                  Err(e) => {
                    app_data_directory = Err(e.into());
                  }
                }
              }
            }
          }
          None => {
            use anyhow::Context;

            error!(
              "Unable to get the user's home directory! Using \"/tmp\" instead, this is really bad, configure your system please."
            );

            let tmp_dir = PathBuf::from("/tmp");

            match fs::canonicalize(tmp_dir) {
              Ok(real_path) => {
                match fs::create_dir_all(tmp_dir.push(APP_NAME)).with_context(|| {
                  format!("Attempted to create \"/tmp/{}\", to use instead of XDG_STATE_HOME.",)
                }) {
                  Ok(_) => {
                    // This should be fine to skip canonicalization since we just created the directory from this location.
                    app_data_directory = Ok(tmp_dir.push(APP_NAME).into());
                  }
                  Err(e) => {
                    app_data_directory = Err(e.into());
                  }
                }
              }
              Err(e) => {
                app_data_directory = Err(e.into());
              }
            }
          }
        }
      }
    }
  }
}
