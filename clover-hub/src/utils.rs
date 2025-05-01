use anyhow::anyhow;
use chrono::prelude::{
  DateTime,
  Utc,
};
use log::{
  debug,
  error,
};
use os_path::OsPath;
use serde::Deserialize;
use simple_error::SimpleError;
use std::hash::{
  DefaultHasher,
  Hash,
  Hasher,
};
use tokio::{
  fs,
  io::AsyncReadExt,
};

pub struct RecvSync<T>(pub std::sync::mpsc::Receiver<T>);

unsafe impl<T> Sync for RecvSync<T> {}

/// formats like "2001-07-08T00:34:60.026490+09:30"
pub fn iso8601(st: &std::time::SystemTime) -> String {
  let dt: DateTime<Utc> = st.clone().into();
  format!("{}", dt.format("%+"))
}

#[allow(dead_code)]
pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
  let mut s = DefaultHasher::new();
  t.hash(&mut s);
  s.finish()
}

pub async fn read_file(path: OsPath) -> Result<String, SimpleError> {
  let mut err = None;
  let mut ret = None;
  let mut contents = String::new();

  if path.exists() {
    match fs::File::open(path.to_path()).await {
      Ok(mut file) => match file.read_to_string(&mut contents).await {
        Ok(_) => {
          ret = Some(contents);
        }
        Err(e) => {
          err = Some(SimpleError::from(e));
        }
      },
      Err(e) => err = Some(SimpleError::from(e)),
    }
  } else {
    err = Some(SimpleError::new("Path does not exist!"));
  }

  match err {
    Some(e) => Err(e),
    None => match ret {
      Some(val) => Ok(val),
      None => Err(SimpleError::new(
        "Impossible state, no error reported but return value is missing!",
      )),
    },
  }
}

pub fn deserialize_base64<T>(slice: &[u8]) -> Result<T, anyhow::Error>
where
  T: for<'a> Deserialize<'a>,
{
  let mut ret = None;
  let mut err: Option<anyhow::Error> = None;

  debug!("Input: {}", std::str::from_utf8(slice).unwrap());

  match base64::Engine::decode(&base64::prelude::BASE64_STANDARD, slice) {
    Ok(msg_vec) => match std::str::from_utf8(&msg_vec) {
      Ok(msg_str) => match serde_json_lenient::from_str(msg_str) {
        Ok(msg_obj) => {
          ret = Some(msg_obj);
        }
        Err(e) => {
          error!("Failed to deserialize data...");
          err = Some(e.into());
        }
      },
      Err(e) => {
        error!("Failed to turn base64 into a UTF-8 String...");
        err = Some(e.into());
      }
    },
    Err(e) => {
      error!("Failed to turn data into base64 bytes...");
      err = Some(e.into());
    }
  }

  match ret {
    Some(obj) => return Ok(obj),
    None => match err {
      Some(e) => Err(e.into()),
      None => Err(anyhow!("Damn... no error but couldn't return an object?")),
    },
  }
}
