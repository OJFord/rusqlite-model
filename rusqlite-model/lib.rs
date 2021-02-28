pub use rusqlite_model_derive::Model;

pub trait Model<'a>: std::convert::TryFrom<&'a rusqlite::Row<'a>> {
    fn create_table(_: &rusqlite::Connection) -> rusqlite::Result<usize>;
    fn drop_table(_: &rusqlite::Connection) -> rusqlite::Result<usize>;
    fn insert(self, _: &rusqlite::Connection) -> rusqlite::Result<usize>;
    fn into_params(self) -> std::vec::IntoIter<Box<dyn rusqlite::ToSql>>;
}
