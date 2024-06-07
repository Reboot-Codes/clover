use std::hash::{DefaultHasher, Hash, Hasher};

use api_key::types::{ApiKeyResults, Default, StringGenerator};
use chrono::prelude::{DateTime, Utc};

pub fn iso8601(st: &std::time::SystemTime) -> String {
  let dt: DateTime<Utc> = st.clone().into();
  format!("{}", dt.format("%+"))
  // formats like "2001-07-08T00:34:60.026490+09:30"
}

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
  let mut s = DefaultHasher::new();
  t.hash(&mut s);
  s.finish()
}

pub fn gen_api_key() -> String {
  let options = StringGenerator {
    prefix: "CLOVER:".to_string(),
    length: 50,
    ..StringGenerator::default()
  };
  let key: ApiKeyResults = api_key::string(options);
  
  match key {
    ApiKeyResults::String(res) => res,
    ApiKeyResults::StringArray(res_vec) => res_vec.join(""),
  }
}
