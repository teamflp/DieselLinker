# Relationships with DieselLinker

`DieselLinker` is a macro that simplifies the definition of relationships between tables in a Rust application using Diesel. 
It allows you to define `one-to-many`, `many-to-one`, `one_to_one` and `many_to_many` relationships between tables by specifying the names of the tables and columns involved.

## Prerequisites
To use the DieselLinker macro, you need to have the following:


- Rust and Cargo installed on your system.
- Ensure that  `Diesel` is added to your dependencies in `Cargo.toml`.
- The `DieselLinker`  macro must be added as a dependency.
    
```toml
[dependencies]
diesel = { version = "1.4", features = ["postgres"] }
diesel_linker = "1.1"
```
## Installation of DieselLinker in your Rust project

Visit [crates.io](https://crates.io/crates/diesel_linker) to get the latest version of DieselLinker.

To install DieselLinker in your project, you can use Cargo, the Rust package manager.
**Using Cargo** : 
- Install `DieselLinker` with the following command:

```bash
cargo install diesel_linker
```
Or, add `DieselLinker` and `diesel` to your `Cargo.toml` file :
```toml
[dependencies]
diesel = { version = "2.1.5", features = ["postgres"] }
diesel_linker = "version_number"
```
DieselLinker is compatible with Diesel version 2.1.5 and automatically detects the type of database you are using.

## Using DieselLinker to define relationships
### Step 1: Define the models that correspond to the tables in your database

```rust
use diesel_linker::relation;

#[derive(DieselLinker)]
#[derive(Queryable, Identifiable, Debug)]
#[table_name = "users"]
#[relation(child = "Post", fk = "user_id", relation_type = "one_to_many")]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[derive(DieselLinker)]
#[derive(Queryable, Identifiable, Debug, Associations)]
#[diesel(belongs_to = "User"), table_name = "posts"]
#[relation(child = "Post", fk = "user_id", relation_type = "many_to_one")]
pub struct Post {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub body: String,
}
```
In this example, we define two models `User` and `Post` with a one-to-many relationship between them.

## Methods generated for the `one-to-many` relationship :
- `DieselLinker`  automatically generates the necessary Diesel relationship methods to handle the relationships between tables.
- For exemple, for `one-to-many` relationship between the tables `User` and `Post`, the following methods are generated:
```rust
// The `posts` method is generated for the `User` model
impl User {
    pub fn posts(&self, connection: &PgConnection) -> QueryResult<Vec<Post>> {
        Post::belonging_to(self).load(connection)
    }
}

// The `user` method is generated for the `Post` model
impl Post {
    pub fn user(&self, connection: &PgConnection) -> QueryResult<User> {
        User::belonging_to(self).get_result(connection)
    }
}
```
Now, you can easily access a user's posts or a post's user using these methods.

### Step 2: Utilize the Generated Methods
After defining the models and relationships, you can use the generated methods to access the related data in your application.

```rust
fn main() {
    use diesel::prelude::*;
    use diesel::pg::PgConnection;
    use diesel_linker::schema::users::dsl::*;
    use diesel_linker::schema::posts::dsl::*;

    let connection = PgConnection::establish("postgres://user:password@localhost/mydb").unwrap();

    let user = users.find(1).first::<User>(&connection).expect("Error loading user");
    let user_posts = user.posts(&connection).expect("Error loading user posts");

    for post in user_posts {
        println!("Title: {}", post.title);
        println!("Body: {}", post.body);
    }
}
```
In this example, we load a user from the database and display the `titles` and `posts` associated with that `user`.

After applying the macro to your structures, compile your project to ensure that the macro works as expected.
```bash
cargo build
```
Finally, perform tests to confirm that the relationships are correctly managed and that you can perform database operations as needed.

## Conclusion
The DieselLinker macro simplifies the definition of one-to-one relationships between tables in a Rust application using Diesel. By following the steps outlined in this guide, you can easily define and manage one-to-one relationships between your models and their corresponding tables in the database.