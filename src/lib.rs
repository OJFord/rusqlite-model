use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::DeriveInput;

#[proc_macro_derive(Model, attributes(sql_type))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let name = &ast.ident;
    let fields = match &ast.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => fields.named.iter(),
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

fn impl_model(name: &syn::Ident, fields: &Vec<&syn::Field>) -> TokenStream {
    let param_holders = vec![quote!(?); fields.len()];
    let mut table_name = format!("{}s", name);
    table_name.make_ascii_lowercase();

    let field_names: Vec<_> = fields
        .iter()
        .map(|f| f.ident.as_ref().expect("expected named field"))
        .collect();

    let sql_types = fields.iter().map(|f| {
        for attr in &f.attrs {
            if let Some(attrident) = attr.path.get_ident() {
                if attrident.to_string() == "sql_type" {
                    return attr.parse_args().expect("failed to read sql_type");
                }
            }
        }

        match &f.ty {
            syn::Type::Path(tp) => match tp.path.get_ident() {
                Some(ident) => match ident.to_string().as_ref() {
                    "bool" => quote!(BOOL NOT NULL),
                    "String" => quote!(TEXT NOT NULL),
                    _ => panic!("Don't know how to map to SQL type {}", ident.to_string()),
                },
                None => panic!("Unsupported type path"),
            },
            _ => panic!("Don't know how to map to SQL type {:?}", f.ty),
        }
    });

    let gen = quote! {
      impl Model<'_> for #name {

        fn create_table(conn: &rusqlite::Connection) -> rusqlite::Result<usize> {
          conn.execute(std::stringify!(
              CREATE TABLE #table_name (
                #(#field_names #sql_types),*
              )
            ),
            rusqlite::NO_PARAMS,
          )
        }

        fn drop_table(conn: &rusqlite::Connection) -> rusqlite::Result<usize> {
          conn.execute(std::stringify!(
              DROP TABLE IF EXISTS #table_name
            ),
            rusqlite::NO_PARAMS,
          )
        }

        fn insert(self, conn: &rusqlite::Connection) -> rusqlite::Result<usize> {
          let mut stmt = conn
            .prepare(std::stringify!(INSERT INTO #table_name (#(#field_names),*) VALUES (#(#param_holders),*)))
            .unwrap();

          println!("{:?}", stmt);
          stmt.execute(self.into_params())
        }

        fn into_params(self) -> std::vec::IntoIter<Box<dyn rusqlite::ToSql>> {
          let ret: Vec<Box<dyn rusqlite::ToSql>> = vec![
            #(Box::new(self.#field_names),)*
          ];
          ret.into_iter()
        }

      }
    };

    gen.into()
}

fn impl_tryfrom(fields: &Vec<&syn::Field>) -> TokenStream {
    let field_names = fields
        .iter()
        .map(|f| f.ident.as_ref().expect("expected named field"));

    let gen = quote! {
      impl<'a> std::convert::TryFrom<&'a rusqlite::Row<'a>> for Transaction {
        type Error = rusqlite::Error;

        fn try_from(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
          Ok(Self {
            #(#field_names: row.get(row.column_index(std::stringify!(#field_names))?)?,)*
          })
        }
      }
    };

    gen.into()
}
