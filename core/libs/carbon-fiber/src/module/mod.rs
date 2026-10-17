use crate::module::component::ComponentGroup;

pub mod comms;
pub mod component;
pub mod hardware;
pub mod rand;

pub struct Module<B, G> {
  bus: B,
  components: G,
}

impl<B, G: ComponentGroup> Module<B, G> {}
