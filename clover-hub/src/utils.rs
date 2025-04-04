use chrono::prelude::{
  DateTime,
  Utc,
};
use os_path::OsPath;
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
