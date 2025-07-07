use anyhow::anyhow;

pub fn spawn_editor(_path: &String, _use_servo: bool) -> Result<(), anyhow::Error> {
  let ret = Err(anyhow!("Did not attempt to open link!"));

  #[cfg(target_os = "linux")]
  #[cfg(target_os = "windows")]
  #[cfg(target_os = "macos")]
  todo!();

  ret
}
