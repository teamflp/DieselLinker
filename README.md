# DieselLinker

`DieselLinker` is a procedural macro that simplifies defining relationships between Diesel models. It allows you to define `one-to-many`, `many-to-one`, `one-to-one`, and `many-to-many` relationships with a simple attribute.

## Getting Started

1.  Add `diesel_linker` to your `Cargo.toml`.
2.  Define your Diesel models and schema as usual.
3.  Add the `#[relation]` attribute to your model structs to define the relationships.

### Prerequisites

*   Rust and Cargo installed on your system.
*   `diesel` and `diesel_linker` added to your `Cargo.toml`.

```toml
[dependencies]
diesel = { version = "2.2.2", features = ["postgres", "sqlite", "mysql"] } # Enable features for your database
diesel_linker = "1.2.0" # Use the latest version
```

## Usage

### The `#[relation]` attribute

The `#[relation]` attribute generates methods on your model structs to fetch related models.

#### Common Attributes

- `model`: **(Required)** The name of the related model as a string (e.g., `"Post"`).
- `relation_type`: **(Required)** The type of relationship. Can be `"one_to_many"`, `"many_to_one"`, `"one_to_one"`, or `"many_to_many"`.
- `backend`: **(Required)** The database backend you are using. Supported values are `"postgres"`, `"sqlite"`, and `"mysql"`.
- `method_name`: **(Optional)** A string that specifies a custom name for the generated getter method. If not provided, a name is inferred from the model name (e.g., `get_posts` for a `Post` model).
- `eager_loading`: **(Optional)** A boolean (`true` or `false`) that, when enabled, generates an additional static method for eager loading the relationship. Defaults to `false`.
- `async`: **(Optional)** A boolean (`true` or `false`) that, when enabled, generates `async` methods for use with `diesel-async`. Defaults to `false`.

#### Attributes for `many_to_one`

- `fk`: **(Required)** The name of the foreign key column on the current model's table (e.g., `"user_id"`).
- `parent_primary_key`: **(Optional)** The name of the primary key on the parent model. Defaults to `"id"`. This is only used when `eager_loading` is set to `true`.

#### Attributes for `many_to_many`

- `join_table`: **(Required)** The name of the join table as a string (e.g., `"post_tags"`).
- `fk_parent`: **(Required)** The foreign key in the join table that points to the current model (e.g., `"post_id"`).
- `fk_child`: **(Required)** The foreign key in the join table that points to the related model (e.g., `"tag_id"`).
- `primary_key`: The name of the primary key of the current model. Defaults to `"id"`.
- `child_primary_key`: The name of the primary key of the related model. Defaults to the value of `primary_key` if specified, otherwise `"id"`.

### Generated Methods

#### Lazy Loading

By default, the macro generates "lazy loading" methods that fetch related objects on demand.
- For `one-to-many` and `many-to-many`, it generates `get_<model_name_pluralized>()`. For example, a relation to `Post` will generate `get_posts()`.
- For `one-to-one` and `many_to_one`, it generates `get_<model_name>()`. For example, a relation to `User` will generate `get_user()`.

#### Eager Loading (with `eager_loading = true`)

When `eager_loading = true` is set, an additional static method is generated to solve the N+1 query problem. This method takes a `Vec` of parent objects and returns a `Vec` of tuples, pairing each parent with its loaded children.

- The method is named `load_with_<relation_name>()`. For example, `load_with_posts()` or `load_with_user()`.
- **Important:** For `many_to_one` and `many_to_many` relationships, the related models **must** derive `Clone`.

### Example: Lazy vs. Eager Loading

First, define your models. Note the use of `eager_loading = true` and `#[derive(Clone)]`.

```rust
// In your models.rs or similar
use super::schema::{users, posts};
use diesel_linker::relation;

// Parent model
#[derive(Queryable, Identifiable, Debug, Clone)] // Clone is needed for eager loading on Post
#[diesel(table_name = users)]
#[relation(model = "Post", relation_type = "one_to_many", backend = "mysql", eager_loading = true)]
pub struct User {
    pub id: i32,
    pub name: String,
}

// Child model
#[derive(Queryable, Identifiable, Debug, Associations)]
#[diesel(belongs_to(User), table_name = posts)]
#[relation(model = "User", fk = "user_id", relation_type = "many_to_one", backend = "mysql")]
pub struct Post {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
}
```

Now you can use these methods in your application:

```rust
fn main() {
    use diesel::prelude::*;
    use diesel::mysql::MysqlConnection;
    use your_project::db::models::{User, Post};
    use your_project::db::schema::users::dsl::*;

    let mut connection = MysqlConnection::establish("...").unwrap();
    // setup your database here...

    // --- Lazy Loading (N+1 queries) ---
    let users_lazy = users.limit(5).load::<User>(&mut connection).expect("Error loading users");
    for user in users_lazy {
        // This line executes one additional query PER user
        let user_posts = user.get_posts(&mut connection).expect("Error loading user posts");
        println!("User {} has {} posts.", user.name, user_posts.len());
    }

    // --- Eager Loading (2 queries total) ---
    let all_users = users.load::<User>(&mut connection).expect("Error loading users");
    // This line executes ONE additional query for ALL posts of ALL users
    let users_with_posts = User::load_with_posts(all_users, &mut connection).expect("Error loading posts");

    for (user, posts) in users_with_posts {
        println!("User {} has {} posts.", user.name, posts.len());
    }
}
```

### Async Support (with `async = true`)

`DieselLinker` supports generating `async` methods for use with `diesel-async`. To enable this, simply add `async = true` to the `#[relation]` attribute.

The generated methods will be `async fn` and must be called with `.await`.

Here is a complete example of how to use the async functionality with `tokio` and an in-memory SQLite database.

#### 1. Dependencies for Async

Add the following dependencies to your `Cargo.toml` file. Note the addition of `diesel-async` and `tokio`.

```toml
[dependencies]
diesel = { version = "2.1.0", features = ["sqlite"] }
diesel-async = { version = "0.5.0", features = ["sqlite", "tokio", "sync-connection-wrapper"] }
diesel_linker = "1.2.0"
tokio = { version = "1", features = ["full"] }
```

#### 2. Async Example Code

Copy and paste the following code into your `src/main.rs` and run it with `cargo run`.

```rust
use diesel::prelude::*;
use diesel_async::{RunQueryDsl, sync_connection_wrapper::SyncConnectionWrapper};
use diesel_linker::relation;
use tokio;

// Database schema definition.
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

// Model definitions.
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
    // 1. Establish a connection to an in-memory SQLite database.
    // For SQLite, we use a SyncConnectionWrapper as diesel-async does not have a native async driver.
    let mut conn = SyncConnectionWrapper::new(diesel::sqlite::SqliteConnection::establish(":memory:").unwrap());

    // 2. Create the `users` and `posts` tables.
    diesel::sql_query("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)").execute(&mut conn).await.unwrap();
    diesel::sql_query("CREATE TABLE posts (id INTEGER PRIMARY KEY, user_id INTEGER NOT NULL, title TEXT NOT NULL)").execute(&mut conn).await.unwrap();

    // 3. Insert test data.
    let new_user = User { id: 1, name: "Grace Hopper".to_string() };
    diesel::insert_into(users::table).values(&new_user).execute(&mut conn).await.unwrap();

    let user_posts = vec![
        Post { id: 1, user_id: 1, title: "The first compiler".to_string() },
        Post { id: 2, user_id: 1, title: "Nanoseconds".to_string() },
    ];
    diesel::insert_into(posts::table).values(&user_posts).execute(&mut conn).await.unwrap();

    println!("--- DieselLinker Async Demonstration ---");

    // 4. Fetch the user from the database.
    let user = users::table.find(1).first::<User>(&mut conn).await.unwrap();
    println!("\nFound user: {}", user.name);

    // 5. Use the async `get_posts()` method generated by DieselLinker.
    println!("Fetching user's posts asynchronously...");
    let posts = user.get_posts(&mut conn).await.unwrap();

    println!("'{}' has written {} post(s):", user.name, posts.len());
    for post in posts {
        println!("- {}", post.title);

        let author = post.get_user(&mut conn).await.unwrap();
        assert_eq!(author.name, user.name);
    }
}
```

## Running the Test Suite

The full test suite includes integration tests for PostgreSQL and MySQL. To run these tests, you will need to have the respective database client libraries installed and a running database instance.

For detailed instructions on how to set up your development environment, please see the [Contributing Guide](CONTRIBUTING.md).

If you do not have PostgreSQL and MySQL set up, `cargo test` will fail. However, the tests for SQLite (both sync and async) will run without any external database setup.

## Conclusion

The `diesel_linker` macro simplifies the definition of relationships between tables in a Rust application using Diesel. By following the steps outlined in this guide, you can easily define and manage relationships between your models, and efficiently load them to prevent performance bottlenecks.

## Usage Example

Here is a concrete, self-contained example that you can run to see `DieselLinker` in action. This example uses an in-memory SQLite database, so you don't need to set up an external database.

### 1. Dependencies

Add the following dependencies to your `Cargo.toml` file:

```toml
[dependencies]
diesel = { version = "2.1.0", features = ["sqlite"] }
diesel_linker = "1.2.0"
```

### 2. Example Code

Copy and paste the following code into your `src/main.rs` and run it with `cargo run`.

```rust
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_linker::relation;

// Database schema definition.
// In a real project, this would be in `src/schema.rs` and generated by `diesel print-schema`.
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

// Model definitions.
// In a real project, this would be in `src/models.rs`.

use schema::{users, posts};

#[derive(Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(table_name = users)]
// Defines a one-to-many relationship to the `Post` model.
// DieselLinker will generate a `get_posts()` method on the `User` struct.
#[relation(model = "Post", relation_type = "one_to_many", backend = "sqlite")]
pub struct User {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Identifiable, Insertable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User), table_name = posts)]
// Defines a many-to-one relationship to the `User` model.
// DieselLinker will generate a `get_user()` method on the `Post` struct.
#[relation(model = "User", fk = "user_id", relation_type = "many_to_one", backend = "sqlite")]
pub struct Post {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
}


fn main() {
    // 1. Establish a connection to an in-memory SQLite database.
    let mut conn = SqliteConnection::establish(":memory:").unwrap();

    // 2. Create the `users` and `posts` tables.
    diesel::sql_query("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE posts (id INTEGER PRIMARY KEY, user_id INTEGER NOT NULL, title TEXT NOT NULL)").execute(&mut conn).unwrap();

    // 3. Insert test data: one user and two posts.
    let new_user = User { id: 1, name: "Ada Lovelace".to_string() };
    diesel::insert_into(users::table).values(&new_user).execute(&mut conn).unwrap();

    let user_posts = vec![
        Post { id: 1, user_id: 1, title: "Notes on the Analytical Engine".to_string() },
        Post { id: 2, user_id: 1, title: "The first computer algorithm".to_string() },
    ];
    diesel::insert_into(posts::table).values(&user_posts).execute(&mut conn).unwrap();

    println!("--- DieselLinker Demonstration ---");

    // 4. Fetch the user from the database.
    let user = users::table.find(1).first::<User>(&mut conn).unwrap();
    println!("\nFound user: {}", user.name);

    // 5. Use the `get_posts()` method generated by DieselLinker.
    //    This method loads all posts associated with the user.
    println!("Fetching user's posts...");
    let posts = user.get_posts(&mut conn).unwrap();

    println!("'{}' has written {} post(s):", user.name, posts.len());
    for post in posts {
        println!("- {}", post.title);

        // We can also go back from the post to its author.
        let author = post.get_user(&mut conn).unwrap();
        assert_eq!(author.name, user.name);
    }
}
```