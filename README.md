# rusqlite-model

[![Crates.io](https://img.shields.io/crates/v/rusqlite-model?style=flat-square)](//crates.io/crates/rusqlite-model) [![docs.rs](https://img.shields.io/docsrs/rusqlite-model?style=flat-square)](//docs.rs/rusqlite-model)

For when serialising/deserialising a struct into/from your ([rusqlite](//github.com/rusqlite/rusqlite)) database queries would be convenient, but you don't need all of [diesel](//diesel.rs).

## Usage

```toml
[dependencies]
rusqlite-model = "0.1"
```

```rust
use rusqlite_model::Model;

#[derive(Model)]
struct User {
    name: String,
    email: String,
    is_active: bool,
    #[sql_type(DATE)]
    last_login: String,
}

fn ensure_exactly_one_user(conn: &rusqlite::Connection) -> rusqlite::Result<usize> {
    User::drop_table(&conn)?;
    User::create_table(&conn)?;

    User {
        name: "OJFord".into(),
        email: "foo@example.com".into(),
        is_active: true,
        last_login: "2021/02/28".into(),
    }
    .insert(&conn)
}

fn get_active_users(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<User>> {
    let mut stmt = conn.prepare("SELECT * FROM users WHERE is_active = ?")?;
    stmt.query_map(params![true], |row| row.try_into::<User>())
}
```

## To do

- Docs & tests
- More types (String -> TEXT; bool -> BOOL currently supported, anything else requires explicit `#[sql_type(...)]`)
- Some kind of `::from_query` helper (connection & query string -> maybe model)

## Won't do

- Not intending to be a full ORM with much DSL, use `diesel` if that's wanted;
- I like `rusqlite`'s simplicity when only SQLite is needed, this is just intended to reduce boilerplate around inserting (`VALUES (?,?,?,?,`.. how many `?` again?) and parsing results into structs for better ergonomics;
- Migrations
- Pan-DBMS portability (`rusqlite`, and hence SQLite, only)
