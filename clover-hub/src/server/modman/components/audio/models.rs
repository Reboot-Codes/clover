use crate::server::modman::components::models::CloverComponentTrait;

#[derive(Debug, Clone)]
pub struct AudioOutputComponent {}

impl CloverComponentTrait for AudioOutputComponent {}

#[derive(Debug, Clone)]
pub struct AudioInputComponent {}

impl CloverComponentTrait for AudioInputComponent {}
