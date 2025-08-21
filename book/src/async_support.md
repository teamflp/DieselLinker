# Async Support

`DieselLinker` supports generating `async` methods for use with `diesel-async`. To enable this, simply add `async = true` to the `#[relation]` attribute.

The generated methods will be `async fn` and must be called with `.await`.

## Dependencies for Async

To use the async functionality, you need to add `diesel-async` and an async runtime like `tokio` to your `Cargo.toml`.

```toml
[dependencies]
diesel = { version = "2.1.0", features = ["sqlite"] }
diesel-async = { version = "0.5.0", features = ["sqlite", "tokio", "sync-connection-wrapper"] }
diesel_linker = "1.3.0"
tokio = { version = "1", features = ["full"] }
```

Note that for SQLite, `diesel-async` uses a `SyncConnectionWrapper` because there is no native async driver for SQLite. For PostgreSQL and MySQL, you would enable the `postgres` or `mysql` features respectively.

## Example

Here is a complete example of how to use the async functionality with `tokio` and an in-memory SQLite database.

```rust
use diesel::prelude::*;
use diesel_async::{RunQueryDsl, sync_connection_wrapper::SyncConnectionWrapper};
use diesel_linker::relation;
use tokio;

mod schema {
    diesel::table! {
        users (id) {
            id -> Integer,
            name -> Text,
        }
    }

    diesel::table! {
        posts (id) {
            id -> Integer,
            user_id -> Integer,
            title -> Text,
        }
    }

    diesel::joinable!(posts -> users (user_id));
    diesel::allow_tables_to_appear_in_same_query!(users, posts);
}

use schema::{users, posts};

#[derive(Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(table_name = users)]
#[relation(model = "Post", relation_type = "one_to_many", backend = "sqlite", async = true)]
pub struct User {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Identifiable, Insertable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User), table_name = posts)]
#[relation(model = "User", fk = "user_id", relation_type = "many_to_one", backend = "sqlite", async = true)]
pub struct Post {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
}

#[tokio::main]
async fn main() {
    let mut conn = SyncConnectionWrapper::new(diesel::sqlite::SqliteConnection::establish(":memory:").unwrap());

    diesel::sql_query("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)").execute(&mut conn).await.unwrap();
    diesel::sql_query("CREATE TABLE posts (id INTEGER PRIMARY KEY, user_id INTEGER NOT NULL, title TEXT NOT NULL)").execute(&mut conn).await.unwrap();

    let new_user = User { id: 1, name: "Grace Hopper".to_string() };
    diesel::insert_into(users::table).values(&new_user).execute(&mut conn).await.unwrap();

    let user_posts = vec![
        Post { id: 1, user_id: 1, title: "The first compiler".to_string() },
        Post { id: 2, user_id: 1, title: "Nanoseconds".to_string() },
    ];
    diesel::insert_into(posts::table).values(&user_posts).execute(&mut conn).await.unwrap();

    println!("--- DieselLinker Async Demonstration ---");

    let user = users::table.find(1).first::<User>(&mut conn).await.unwrap();
    println!("\nFound user: {}", user.name);

    // Use the async `get_posts()` method
    let posts = user.get_posts(&mut conn).await.unwrap();

    println!("'{}' has written {} post(s):", user.name, posts.len());
    for post in posts {
        println!("- {}", post.title);
    }
}
```
