# Eager Loading

Eager loading is a way to solve the "N+1 query problem" that can occur with lazy loading. With eager loading, you can load all the related objects for a collection of parent objects in a constant number of queries (usually 2).

To enable eager loading, you need to set `eager_loading = true` in the `#[relation]` attribute.

## Generated Methods

When `eager_loading = true` is set, an additional static method is generated on the model struct.

-   The method is named `load_with_<relation_name>()`. For example, `load_with_posts()` or `load_with_user()`.
-   This method takes a `Vec` of parent objects and a database connection.
-   It returns a `Vec` of tuples, where each tuple contains a parent object and a `Vec` of its loaded children: `Vec<(Parent, Vec<Child>)>`.

**Important:** For `many_to_one` and `many_to_many` relationships, the related models **must** derive `Clone`.

## Example

Here is an example of how to use eager loading to fetch all users and their posts in just two queries.

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

// Note the addition of `eager_loading = true`
#[derive(Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(table_name = users)]
#[relation(model = "Post", relation_type = "one_to_many", backend = "sqlite", eager_loading = true)]
pub struct User {
    pub id: i32,
    pub name: String,
}

// The child model does not need any changes for one-to-many eager loading.
#[derive(Queryable, Identifiable, Insertable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User), table_name = posts)]
pub struct Post {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
}

fn main() {
    let mut conn = SqliteConnection::establish(":memory:").unwrap();

    diesel::sql_query("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE posts (id INTEGER PRIMARY KEY, user_id INTEGER NOT NULL, title TEXT NOT NULL)").execute(&mut conn).unwrap();

    // Insert some users and posts
    let users_to_insert = vec![
        User { id: 1, name: "User 1".to_string() },
        User { id: 2, name: "User 2".to_string() },
    ];
    diesel::insert_into(users::table).values(&users_to_insert).execute(&mut conn).unwrap();

    let posts_to_insert = vec![
        Post { id: 1, user_id: 1, title: "Post 1.1".to_string() },
        Post { id: 2, user_id: 1, title: "Post 1.2".to_string() },
        Post { id: 3, user_id: 2, title: "Post 2.1".to_string() },
    ];
    diesel::insert_into(posts::table).values(&posts_to_insert).execute(&mut conn).unwrap();

    println!("--- DieselLinker Eager Loading Demonstration ---");

    // 1. Load all users (1st query)
    let all_users = users::table.load::<User>(&mut conn).unwrap();
    println!("Loaded {} users.", all_users.len());

    // 2. Eager load all posts for all users (2nd query)
    let users_with_posts = User::load_with_posts(all_users, &mut conn).unwrap();

    for (user, posts) in users_with_posts {
        println!("\nUser '{}' has {} post(s):", user.name, posts.len());
        for post in posts {
            println!("- {}", post.title);
        }
    }
}
```
