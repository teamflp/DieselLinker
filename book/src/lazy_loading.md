# Lazy Loading

By default, `DieselLinker` generates "lazy loading" methods. This means that related objects are fetched from the database on demand, when you call the generated method.

This is a simple and intuitive way to work with relationships, but it can lead to performance issues if you are not careful, specifically the "N+1 query problem".

## Generated Methods

The names of the generated methods are inferred from the `model` attribute:

-   For `one-to-many` and `many_to_many` relationships, the method name will be `get_<model_name_pluralized>()`. For example, a relation to a `Post` model will generate a `get_posts()` method.
-   For `one-to-one` and `many_to_one` relationships, the method name will be `get_<model_name>()`. For example, a relation to a `User` model will generate a `get_user()` method.

You can override the default method name by using the `method_name` attribute: `#[relation(..., method_name = "fetch_my_posts")]`.

## Example

Here is a concrete, self-contained example that you can run to see lazy loading in action. This example uses an in-memory SQLite database.

```rust
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_linker::relation;

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
#[relation(model = "Post", relation_type = "one_to_many", backend = "sqlite")]
pub struct User {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Identifiable, Insertable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User), table_name = posts)]
#[relation(model = "User", fk = "user_id", relation_type = "many_to_one", backend = "sqlite")]
pub struct Post {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
}

fn main() {
    let mut conn = SqliteConnection::establish(":memory:").unwrap();

    diesel::sql_query("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE posts (id INTEGER PRIMARY KEY, user_id INTEGER NOT NULL, title TEXT NOT NULL)").execute(&mut conn).unwrap();

    let new_user = User { id: 1, name: "Ada Lovelace".to_string() };
    diesel::insert_into(users::table).values(&new_user).execute(&mut conn).unwrap();

    let user_posts = vec![
        Post { id: 1, user_id: 1, title: "Notes on the Analytical Engine".to_string() },
        Post { id: 2, user_id: 1, title: "The first computer algorithm".to_string() },
    ];
    diesel::insert_into(posts::table).values(&user_posts).execute(&mut conn).unwrap();

    println!("--- DieselLinker Lazy Loading Demonstration ---");

    let user = users::table.find(1).first::<User>(&mut conn).unwrap();
    println!("\nFound user: {}", user.name);

    // This call executes a query to fetch the user's posts.
    let posts = user.get_posts(&mut conn).unwrap();

    println!("'{}' has written {} post(s):", user.name, posts.len());
    for post in posts {
        println!("- {}", post.title);

        // This call executes another query to fetch the post's author.
        let author = post.get_user(&mut conn).unwrap();
        assert_eq!(author.name, user.name);
    }
}
```
