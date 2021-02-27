use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::DeriveInput;

#[proc_macro_derive(Model)]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_model(&ast)
}

fn impl_model(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let fields = match &ast.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => fields
                .named
                .iter()
                .map(|f| f.ident.as_ref().expect("expected named fields")),
            _ => panic!("expected named fields"),
        },
        _ => panic!("expected struct"),
    };

    let gen = quote! {


      impl Model for #name {
        fn into_params(self) -> std::vec::IntoIter<Box<dyn rusqlite::ToSql>> {
          let ret: Vec<Box<dyn rusqlite::ToSql>> = vec![
            #(Box::new(self.#fields),)*
          ];
          ret.into_iter()
        }
      }
    };

    gen.into()
}
