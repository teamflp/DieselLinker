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
/// This macro is used to define relationships between tables in a Diesel schema.```
///
/// # Attributes
///
/// The `relation` attribute is used to define the relationship between tables.
///
/// The following attributes are supported:
///
/// - `type`: The type of relationship. The possible values are `one_to_one`, `one_to_many`, `many_to_one`, and `many_to_many`.
/// - `table_from`: The name of the table from which the relationship originates.
/// - `table_to`: The name of the table to which the relationship points.
/// - `column_from`: The name of the column in the `table_from` table.
/// - `column_to`: The name of the column in the `table_to` table.
///
/// # Example usage
///
/// ```rust
/// // #[macro_use]
/// // extern crate diesel;
///
/// // Import the schema module, which contains the tables. You can import it as mod schema;
/// // if you have a schema.rs file in your project.
/// // mod schema;
/// // use crate::schema::*; // Import the schema module.
/// // use chrono::NaiveDateTime;
/// // use diesel::prelude::*;
/// // use diesel_linker::{relation, DieselLinker};
/// // use std::env; // Import the `env` module from the standard library if you want to use environment variables.
/// // use diesel_linker::DieselLinker; // Import the `DieselLinker` trait.
/// // use diesel::{Queryable, Identifiable, Associations}; // Import the necessary Diesel traits.
///
/// // User Struct
///
/// // #[derive(DieselLinker)]
/// // #[derive(Queryable, Identifiable, Associations)]
/// // #[diesel(belongs_to(User), table_name = "posts")]
///
/// struct User {
///     // Definition of fields in the User struct...
/// }
///
/// // Post Struct
///
/// // #[derive(DieselLinker)]
/// // #[derive(Queryable, Identifiable, Associations)] // Definition of Diesel derivations
/// // #[diesel(table_name = "posts")] // Table name
///
/// struct Post {
///    // Definition of fields in the Post struct...
/// }
///
/// // Relation Definition
///
/// // #[derive(DieselLinker)]
/// // #[derive(Queryable, Identifiable)]
/// // #[relation(type = "one_to_many", table_from = "users", table_to = "posts", column_from = "id", column_to = "user_id")]
/// struct UserPostRelation; // Struct to manage the many-to-many relation between `users` and `posts`
/// ```
///
/// # Notes
///
/// The `relation` attribute is used to define the relationship between tables in a Diesel schema.
///

#[proc_macro_attribute]
pub fn relation(attr: TokenStream, item: TokenStream) -> TokenStream {
    diesel_linker_impl(attr, item)
}
