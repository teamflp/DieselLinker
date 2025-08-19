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
diesel = { version = "2.2.2", features = ["postgres", "sqlite"] } # Enable features for your database
diesel_linker = "1.2.0" # Use the latest version
```

## Usage

### The `#[relation]` attribute

The `#[relation]` attribute generates methods on your model structs to fetch related models.

#### Common Attributes

- `model`: **(Required)** The name of the related model as a string (e.g., `"Post"`).
- `relation_type`: **(Required)** The type of relationship. Can be `"one_to_many"`, `"many_to_one"`, `"one_to_one"`, or `"many_to_many"`.
- `backend`: **(Required)** The database backend you are using. Supported values are `"postgres"` and `"sqlite"`.

#### Attributes for `many_to_one`

- `fk`: **(Required)** The name of the foreign key column on the current model's table (e.g., `"user_id"`).

#### Attributes for `many_to_many`

- `join_table`: **(Required)** The name of the join table as a string (e.g., `"post_tags"`).
- `fk_parent`: **(Required)** The foreign key in the join table that points to the current model (e.g., `"post_id"`).
- `fk_child`: **(Required)** The foreign key in the join table that points to the related model (e.g., `"tag_id"`).
- `primary_key`: The name of the primary key of the current model. Defaults to `"id"`.
- `child_primary_key`: The name of the primary key of the related model. Defaults to the value of `primary_key` if specified, otherwise `"id"`.

### Generated Methods

The macro generates methods to fetch related objects. The method names are derived from the related model's name.
- For `one-to-many` and `many-to-many`, it generates `get_<model_name_pluralized>()`. For example, a relation to `Post` will generate `get_posts()`.
- For `one-to-one` and `many-to-one`, it generates `get_<model_name>()`. For example, a relation to `User` will generate `get_user()`.

### Example: `one-to-many` and `many-to-one`

First, define your Diesel models and schema as you would normally.

```rust
// In your schema.rs or similar
table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    posts (id) {
        id -> Int4,
        user_id -> Int4,
        title -> Varchar,
    }
}

joinable!(posts -> users (user_id));
```

Then, add the `#[relation]` attribute to your models.

```rust
// In your models.rs or similar
use super::schema::{users, posts};
use diesel_linker::relation;

// Parent model
#[derive(Queryable, Identifiable, Debug)]
#[diesel(table_name = users)]
#[relation(model = "Post", relation_type = "one_to_many", backend = "sqlite")]
pub struct User {
    pub id: i32,
    pub name: String,
}

// Child model
#[derive(Queryable, Identifiable, Debug, Associations)]
#[diesel(belongs_to(User), table_name = posts)]
#[relation(model = "User", fk = "user_id", relation_type = "many_to_one", backend = "sqlite")]
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
    use diesel::sqlite::SqliteConnection;
    // assuming you have your models and schema in a `db` module
    use your_project::db::models::{User, Post};
    use your_project::db::schema::users::dsl::*;


    let mut connection = SqliteConnection::establish(":memory:").unwrap();
    // setup your database here...

    let user = users.find(1).first::<User>(&mut connection).expect("Error loading user");
    let user_posts = user.get_posts(&mut connection).expect("Error loading user posts");

    for post in user_posts {
        println!("Title: {}", post.title);
    }
}
```

## Conclusion

The `diesel_linker` macro simplifies the definition of relationships between tables in a Rust application using Diesel. By following the steps outlined in this guide, you can easily define and manage relationships between your models.