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
- `eager_loading`: **(Optional)** A boolean (`true` or `false`) that, when enabled, generates an additional static method for eager loading the relationship. Defaults to `false`.

#### Attributes for `many_to_one`

- `fk`: **(Required)** The name of the foreign key column on the current model's table (e.g., `"user_id"`).

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

## Conclusion

The `diesel_linker` macro simplifies the definition of relationships between tables in a Rust application using Diesel. By following the steps outlined in this guide, you can easily define and manage relationships between your models, and efficiently load them to prevent performance bottlenecks.