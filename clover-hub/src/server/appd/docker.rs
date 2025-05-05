use crate::server::appd::models::Application;
use bollard::{
  image::BuildImageOptions,
  Docker,
};
use futures::TryStreamExt;
use log::{
  debug,
  error,
  info,
};
use std::sync::Arc;

pub enum Error {
  ContainerBuildFailed { container_id: String },
  ContainerCreationFailed { container_id: String },
  ContainerStartFailed { container_id: String },
  ContainerStopFailed { container_id: String },
}

pub async fn init_app(
  docker: Arc<Docker>,
  app_id: &String,
  app_spec: &mut Application,
) -> Result<(), Error> {
  let app_str = format!("{} ({})", app_spec.name.clone(), app_id.clone());

  info!("Initializing {}...", app_str.clone());

  let mut app_init_errored = None;

  for (container_id, container_config) in app_spec.containers.clone() {
    let container_str = format!("{}, Container: {}", app_str.clone(), container_id.clone());
    let mut build_errored = None;

    match container_config.build {
      Option::None => {}
      Some(build_config) => {
        let image_tag = format!(
          "{}/{}:{}",
          app_id.clone(),
          container_id.clone(),
          app_spec.version.clone()
        );
        info!(
          "{}, Building image: {}...",
          container_str.clone(),
          image_tag.clone()
        );

        let mut build_progress = docker.build_image(
          BuildImageOptions {
            dockerfile: build_config.url.0.clone(),
            t: image_tag.clone(),
            ..Default::default()
          },
          None,
          None,
        );

        while let Ok(Some(response)) = build_progress.try_next().await {
          debug!("{}: {:?}", image_tag.clone(), response.clone());

          match response.error {
            Option::None => match response.progress_detail {
              Some(progress) => {
                info!(
                  "{}, Building: {}, {:?}/{:?}",
                  container_str.clone(),
                  image_tag.clone(),
                  progress.current,
                  progress.total
                );
              }
              Option::None => {}
            },
            Some(e) => {
              error!(
                "{}, Building: {}, Build failed! Stopping because of:\n{}",
                container_str.clone(),
                image_tag.clone(),
                e
              );
              build_errored = Some(Error::ContainerBuildFailed {
                container_id: container_config.options.name.clone(),
              });
            }
          }
        }

        match build_errored {
          Option::None => {
            info!(
              "{}: Finished building: {}!",
              container_str.clone(),
              image_tag.clone()
            );
          }
          Some(_) => {}
        }
      }
    }

    match build_errored {
      Some(e) => {
        app_init_errored = Some(e);
        break;
      }
      Option::None => {
        debug!("{}, Creating...", container_str.clone());
        match docker
          .create_container(
            Some(container_config.options.clone()),
            container_config.config.clone(),
          )
          .await
        {
          Ok(_) => {
            info!("{}, Starting...", container_str.clone());
            match docker
              .start_container::<String>(&container_config.options.name.clone(), None)
              .await
            {
              Ok(_) => {
                info!("{}, Started!", container_str.clone());
              }
              Err(e) => {
                // TODO: Better error formatting.
                error!(
                  "{}, Failed to start container!\n{:?}",
                  container_str.clone(),
                  e
                );
                app_init_errored = Some(Error::ContainerStartFailed {
                  container_id: container_id.clone(),
                });
                break;
              }
            }
          }
          Err(e) => {
            // TODO: Better error formatting.
            error!(
              "{}, Failed to create container!\n{:?}",
              container_str.clone(),
              e
            );
            app_init_errored = Some(Error::ContainerCreationFailed {
              container_id: container_id.clone(),
            });
            break;
          }
        }
      }
    }
  }

  match app_init_errored {
    Option::None => {
      app_spec.initialized = true;
      Ok(())
    }
    Some(e) => {
      // match e.clone() {
      //   Error::ContainerBuildFailed { container_id } => {},
      //   Error::ContainerCreationFailed { container_id } => {
      //     // TODO: Add config option to choose if containers that failed to be created should be deleted.
      //   },
      //   Error::ContainerStartFailed { container_id } => {
      //     // TODO: Add config option to choose if containers that failed to be created should be deleted.
      //   }
      // }

      Err(e)
    }
  }
}

pub async fn remove_app(
  docker: Arc<Docker>,
  app_id: &String,
  app_spec: &mut Application,
) -> Result<(), Error> {
  let app_str = format!("{} ({})", app_spec.name.clone(), app_id.clone());

  info!("Removing {}...", app_str.clone());

  let mut app_removal_errored = None;

  for (container_id, container_config) in app_spec.containers.clone() {
    let container_str = format!("{}, Container: {}", app_str.clone(), container_id.clone());

    info!("{}, Stopping...", container_str.clone());
    match docker
      .stop_container(&container_config.options.name.clone(), None)
      .await
    {
      Ok(_) => {
        info!("{}, Stopped!", container_str.clone());

        // TODO: Add config to remove containers when the app is removed.
      }
      Err(e) => {
        // TODO: Better error formatting.
        error!(
          "{}, Failed to stop container!\n{:?}",
          container_str.clone(),
          e
        );
        app_removal_errored = Some(Error::ContainerStopFailed {
          container_id: container_id.clone(),
        });
        break;
      }
    }
  }

  match app_removal_errored {
    Option::None => Ok(()),
    Some(e) => Err(e),
  }
}
