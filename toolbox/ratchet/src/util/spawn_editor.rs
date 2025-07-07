use std::path::Path;

use anyhow::anyhow;

pub fn spawn_editor(_path: &Path) -> Result<(), anyhow::Error> {
  let ret = Err(anyhow!("Did not attempt to spawn editor!"));

  #[cfg(target_os = "linux")]
  #[cfg(target_os = "windows")]
  #[cfg(target_os = "macos")]
  todo!();

  ret
}
