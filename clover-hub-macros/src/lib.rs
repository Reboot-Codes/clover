use proc_macro::TokenStream;
use quote::quote;
use syn::{
  Data,
  DataEnum,
  DataStruct,
  DeriveInput,
  Fields,
  FieldsUnnamed,
  parse_macro_input,
};

#[proc_macro_derive(ManifestCompile, attributes(manifest_compile))]
pub fn derive_manifest_compile(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let name = &input.ident;

  // Extract the "Raw" prefix version for the spec type
  let spec_name = syn::Ident::new(&format!("Raw{}", name), name.span());

  let expanded = match &input.data {
    Data::Struct(data) => generate_struct_impl(name, &spec_name, data),
    Data::Enum(data) => generate_enum_impl(name, &spec_name, data),
    _ => panic!("ManifestCompile only supports structs and enums"),
  };

  TokenStream::from(expanded)
}

fn generate_struct_impl(
  name: &syn::Ident,
  spec_name: &syn::Ident,
  data: &DataStruct,
) -> proc_macro2::TokenStream {
  let fields = match &data.fields {
    Fields::Named(fields) => &fields.named,
    _ => panic!("ManifestCompile only supports named fields for structs"),
  };

  let field_compilations = fields.iter().map(|f| {
    let field_name = &f.ident;
    let field_name_str = field_name.as_ref().unwrap().to_string();
    let field_type = &f.ty;

    quote! {
      debug!(concat!("Resolving ", stringify!(#spec_name), ".", #field_name_str));

      let #field_name = match <#field_type as ManifestCompilationFrom<_>>::compile(
        spec.#field_name.clone(),
        resolution_ctx.clone(),
        repo_dir_path.clone(),
      ).await {
        Ok(val) => {
          debug!(concat!("Resolved ", stringify!(#spec_name), ".", #field_name_str));
          val
        }
        Err(e) => {
          err = Some(e);
          Default::default()
        }
      };
    }
  });

  let field_names: Vec<_> = fields.iter().map(|f| &f.ident).collect();

  quote! {
    impl ManifestCompilationFrom<#spec_name> for #name {
      async fn compile(
        spec: #spec_name,
        resolution_ctx: ResolutionCtx,
        repo_dir_path: OsPath,
      ) -> Result<Self, SimpleError>
      where
        Self: Sized,
        #spec_name: for<'a> Deserialize<'a>,
      {
        use log::debug;
        let mut err = None;

        #(#field_compilations)*

        match err {
          Some(e) => Err(e),
          None => Ok(Self {
            #(#field_names),*
          }),
        }
      }
    }
  }
}

fn generate_enum_impl(
  name: &syn::Ident,
  spec_name: &syn::Ident,
  data: &DataEnum,
) -> proc_macro2::TokenStream {
  let variant_matches = data.variants.iter().map(|variant| {
    let compiled_variant_name = &variant.ident;

    // For Raw variants, add "Raw" prefix to match the spec enum
    let raw_variant_name = syn::Ident::new(
      &format!("Raw{}", compiled_variant_name),
      compiled_variant_name.span(),
    );

    // Get the inner type for single-field tuple variants
    let inner_type = match &variant.fields {
      Fields::Unnamed(FieldsUnnamed { unnamed, .. }) if unnamed.len() == 1 => {
        let field = unnamed.first().unwrap();
        &field.ty
      }
      _ => panic!("ManifestCompile enum variants must have exactly one unnamed field"),
    };

    let debug_msg = format!("raw-{}", convert_to_kebab_case(&name.to_string()));

    quote! {
      #spec_name::#raw_variant_name(raw_spec) => {
        match <#inner_type as ManifestCompilationFrom<_>>::compile(
            raw_spec,
            resolution_ctx.clone(),
            repo_dir_path.clone(),
        ).await {
          Ok(val) => {
            debug!(concat!("Resolved ", #debug_msg));
            Self::#compiled_variant_name(val)
          }
          Err(e) => {
            err = Some(e);
            Self::#compiled_variant_name(Default::default())
          }
        }
      }
    }
  });

  quote! {
    impl ManifestCompilationFrom<#spec_name> for #name {
        async fn compile(
          spec: #spec_name,
          resolution_ctx: ResolutionCtx,
          repo_dir_path: OsPath,
        ) -> Result<Self, SimpleError>
        where
          Self: Sized,
          #spec_name: for<'a> Deserialize<'a>,
        {
          use log::debug;
          let mut err = None;

          let ret = match spec {
            #(#variant_matches)*
          };

          match err {
            Some(e) => Err(e),
            None => Ok(ret),
          }
      }
    }
  }
}

fn convert_to_kebab_case(s: &str) -> String {
  let mut result = String::new();
  for (i, ch) in s.chars().enumerate() {
    if ch.is_uppercase() && i > 0 {
      result.push('-');
    }
    result.push(ch.to_ascii_lowercase());
  }
  result
}
