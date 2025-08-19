// lib.rs
extern crate proc_macro;
mod relation_macro;
mod utils;

use proc_macro::TokenStream;
use relation_macro::diesel_linker_impl;

/// The `relation` attribute macro simplifies defining relationships between Diesel models.
///
/// It generates methods to fetch related models based on the specified relationship type.
///
/// # Attributes
///
/// - `model`: The name of the related model.
/// - `relation_type`: The type of relationship. Can be `one_to_many`, `many_to_one`, `one_to_one`, or `many_to_many`.
/// - `fk`: The foreign key used for the relationship. Required for `many_to_one`.
/// - `join_table`: The name of the join table. Required for `many_to_many`.
/// - `fk_parent`: The foreign key for the parent in the join table. Required for `many_to_many`.
/// - `fk_child`: The foreign key for the child in the join table. Required for `many_to_many`.
///
/// # Example
///
/// ```rust,ignore
/// use diesel::prelude::*;
/// use diesel_linker::relation;
///
/// table! {
///     users (id) {
///         id -> Integer,
///         name -> Text,
///     }
/// }
///
/// table! {
///     posts (id) {
///         id -> Integer,
///         user_id -> Integer,
///         title -> Text,
///     }
/// }
///
/// joinable!(posts -> users (user_id));
///
/// #[derive(Queryable, Identifiable, Debug)]
/// #[diesel(table_name = users)]
/// #[relation(model = "Post", relation_type = "one_to_many")]
/// pub struct User {
///     pub id: i32,
///     pub name: String,
/// }
///
/// #[derive(Queryable, Identifiable, Debug, Associations)]
/// #[diesel(belongs_to(User), table_name = posts)]
/// #[relation(model = "User", fk = "user_id", relation_type = "many_to_one")]
/// pub struct Post {
///     pub id: i32,
///     pub user_id: i32,
///     pub title: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn relation(attr: TokenStream, item: TokenStream) -> TokenStream {
    diesel_linker_impl(attr, item)
}
