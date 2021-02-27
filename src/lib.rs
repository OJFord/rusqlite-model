use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::DeriveInput;

#[proc_macro_derive(Model)]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

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
    }
    .collect();

    let modelimpl = impl_model(&name, &fields);
    let tryfromrow = impl_tryfrom(&fields);

    let mut ts = TokenStream::new();
    ts.extend(modelimpl);
    ts.extend(tryfromrow);
    ts
}

fn impl_model(name: &syn::Ident, fields: &Vec<&syn::Ident>) -> TokenStream {
    let gen = quote! {
      impl Model<'_> for #name {
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

fn impl_tryfrom(fields: &Vec<&syn::Ident>) -> TokenStream {
    let gen = quote! {
      impl<'a> std::convert::TryFrom<&'a rusqlite::Row<'a>> for Transaction {
        type Error = rusqlite::Error;

        fn try_from(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
          Ok(Self {
            #(#fields: row.get(row.column_index(std::stringify!(#fields))?)?,)*
          })
        }
      }
    };

    gen.into()
}
