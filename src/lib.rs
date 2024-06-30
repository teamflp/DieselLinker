// lib.rs
extern crate proc_macro;
mod relation_macro;
mod utils;

use proc_macro::TokenStream;
use relation_macro::diesel_linker_impl;

#[proc_macro_derive(DieselLinker, attributes(relation))]
pub fn diesel_linker_derive(input: TokenStream) -> TokenStream {
    diesel_linker_impl(input, TokenStream::new())
}

/// Implements the `diesel_linker` macro.
///
/// The diesel_linker macro simplifies defining and managing relationships between database tables in Rust applications using Diesel ORM. It automates generating ORM layer code necessary for handling relationships such as one_to_one,one-to-many, many-to-one, and many-to-many.
///
/// # Attributes
///
/// The `relation` attribute is used to define the relationship between tables.
///
/// The following attributes are supported:
///
/// `attrs`: The attributes provided to the macro, which describe the relationship:
/// - `child`: The name of the table from which the relationship originates.
/// - `fk`: Indicates the foreign key in the parent table linking to the child.
/// - `join_table`: The name of the join table for many-to-many relationships.
/// - `fk_parent`: The foreign key in the join table linking to the parent table.
/// - `relation_type`: The type of relationship (one_to_one, one_to_many, many_to_one, many_to_many).
///
/// Returns :
///
/// `Result<(String, String, String), String>`: On successful execution, returns a tuple containing:
/// - Child model name.
/// - Foreign key used in the parent table.
/// - Relation type (one_to_one, one_to_many, many_to_one, many_to_many).
///
/// On failure, it returns an error message explaining the execution failure.
///
/// # Example usage
///
// ```rust
// Apply the `diesel_linker` macro to the User struct to define a one-to-many relationship with Post.
/// use diesel_linker::relation;
///
/// #[derive(DieselLinker)]
/// #[derive(Queryable, Identifiable, Debug)]
/// #[table_name = "users"]
/// #[relation(child = "Post", fk = "user_id", relation_type = "one_to_many")]
/// pub struct User {
///     pub id: i32,
///     pub name: String,
///     pub email: String,
/// }
///
/// Apply the `diesel_linker` macro to the Post struct to define a many-to-one relationship with User.
///
/// #[derive(DieselLinker)]
/// #[derive(Queryable, Identifiable, Debug, Associations)]
/// #[diesel(belongs_to = "User"), table_name = "posts"]
/// #[relation(child = "Post", fk = "user_id", relation_type = "many_to_one")]
/// pub struct Post {
///     pub id: i32,
///     pub user_id: i32,
///     pub title: String,
///     pub body: String,
/// }
// ```
///
// # Generated methods for the relation
///
/// Based on the specified relation_type, the macro generates appropriate Rust methods to manage the relationships:
///
/// Example for a one_to_many relationship:
///
/// For the struct User related to Post through a one_to_many relationship:
///
/// Retrieves all posts related to this user instance from the database.
///
/// ```rust
//// impl User {
///    pub fn posts(&self, conn: &PgConnection) -> diesel::QueryResult<Vec<Post>> {
///         use crate::schema::posts::dsl::*;
///         posts.filter(user_id.eq(self.id)).load::<Post>(conn)
///     }
/// }
/// ```
///
/// # Usage in your code:
///
// How to use the generated methods in a Rust application:
/// ```rust
//// Fetches all posts for a specific user from the database.
/// fn get_user_posts(conn: &PgConnection, user_id: i32) -> QueryResult<Vec<Post>> {
///    let user = users::table.find(user_id).first::<User>(conn)?;
///    user.posts(conn)
// }
/// ```
///

#[proc_macro_attribute]
pub fn relation(attr: TokenStream, item: TokenStream) -> TokenStream {
    diesel_linker_impl(attr, item)
}
