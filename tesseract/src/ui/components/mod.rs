mod container;

pub use container::*;

#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
extern crate proc_macro;
use proc_macro::TokenStream;
use syn::DeriveInput;

pub struct Context {}

pub trait Component<'a> {
  fn get_context(&'a mut self) -> &'a mut Context;
}

pub trait Children<'a>: Component<'a> {
  fn add_children(&'a mut self, get_children: impl Fn(&mut Context) -> Vec<Box<dyn Component>>) {
    self.get_children().append(&mut get_children(self.get_context()));
  }

  fn get_children(&mut self) -> &'a mut Vec<Box<dyn Component<'a>>>;
}

#[proc_macro_derive(Component)]
pub fn derive_get_context(item: TokenStream) -> TokenStream {
  // Parse the input tokens into a syntax tree.
  let input = parse_macro_input!(input as DeriveInput);

  // Used in the quasi-quotation below as `#name`.
  let name = input.ident;

  // Add a bound `T: HeapSize` to every type parameter T.
  let generics = add_trait_bounds(input.generics);
  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

  let expanded = quote! {
    impl #impl_generics tesseract::ui::components::Component<'a> for #name #ty_generics #where_clause {
      fn get_context(&'a mut self) -> &'a mut Context {
        &mut self.context
      }
    }
  };

  // Hand the output tokens back to the compiler.
  proc_macro::TokenStream::from(expanded)
}
