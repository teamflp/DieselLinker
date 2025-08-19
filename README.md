# DieselLinker

`DieselLinker` is a procedural macro that simplifies defining relationships between Diesel models. It allows you to define `one-to-many`, `many-to-one`, `one-to-one`, and `many-to-many` relationships with a simple attribute.

## Prerequisites

*   Rust and Cargo installed on your system.
*   `diesel` and `diesel_linker` added to your `Cargo.toml`.

```toml
[dependencies]
diesel = { version = "2.2.2", features = ["postgres"] } # Or any other backend
diesel_linker = "1.2.0" # Use the latest version
```

## Usage

### Step 1: Define your models and schema

First, define your Diesel models and schema as you would normally.

```rust
// In your schema.rs or similar
table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        email -> Varchar,
    }
}

table! {
    posts (id) {
        id -> Int4,
        user_id -> Int4,
        title -> Varchar,
        body -> Text,
    }
}

joinable!(posts -> users (user_id));
allow_tables_to_appear_in_same_query!(users, posts);
```

### Step 2: Add the `#[relation]` attribute to your models

Use the `#[relation]` attribute on your model structs to define the relationship.

**Example: `one-to-many` and `many-to-one`**

```rust
// In your models.rs or similar
use super::schema::{users, posts};
use diesel_linker::relation;

#[relation(model = "Post", relation_type = "one_to_many")]
#[derive(Queryable, Identifiable, Debug)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[relation(model = "User", fk= "user_id", relation_type = "many_to_one")]
#[derive(Queryable, Identifiable, Debug, Associations)]
#[diesel(belongs_to(User), table_name = posts)]
pub struct Post {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub body: String,
}
```

### Step 3: Use the generated methods

`DieselLinker` generates methods on your structs to fetch related models. The method names are generated based on the related model name (e.g., `get_<model_name>` or `get_<model_name>s`).

```rust
// For the one-to-many relationship on User
impl User {
    pub fn get_posts<C>(&self, conn: &C) -> diesel::QueryResult<Vec<Post>>
    where C: diesel::Connection,
          Post: diesel::BelongingTo<User>
    {
        use diesel::prelude::*;
        Post::belonging_to(self).load::<Post>(conn)
    }
}

// For the many-to-one relationship on Post
impl Post {
    pub fn get_user<C>(&self, conn: &C) -> diesel::QueryResult<User>
    where C: diesel::Connection,
    {
        use diesel::prelude::*;
        User::table.find(self.user_id).get_result::<User>(conn)
    }
}
```

Now you can use these methods in your application:

```rust
fn main() {
    use diesel::prelude::*;
    use diesel::pg::PgConnection;
    // assuming you have your models and schema in a `db` module
    use your_project::db::models::{User, Post};
    use your_project::db::schema::users::dsl::*;


    let mut connection = PgConnection::establish("postgres://user:password@localhost/mydb").unwrap();

    let user = users.find(1).first::<User>(&mut connection).expect("Error loading user");
    let user_posts = user.get_posts(&mut connection).expect("Error loading user posts");

    for post in user_posts {
        println!("Title: {}", post.title);
        println!("Body: {}", post.body);
    }
}
```

## Conclusion

The `diesel_linker` macro simplifies the definition of relationships between tables in a Rust application using Diesel. By following the steps outlined in this guide, you can easily define and manage relationships between your models.