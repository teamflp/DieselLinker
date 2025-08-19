// lib.rs
extern crate proc_macro;
mod relation_macro;
mod utils;

use proc_macro::TokenStream;
use relation_macro::diesel_linker_impl;

/// `diesel_linker` provides a procedural macro to simplify defining relationships between Diesel models.
///
/// This macro, `#[relation]`, generates methods on your model structs to fetch related models
/// based on the specified relationship type. It supports `one-to-many`, `many-to-one`, `one-to-one`,
/// and `many-to-many` relationships.
///
/// # Getting Started
///
/// 1.  Add `diesel_linker` to your `Cargo.toml`.
/// 2.  Define your Diesel models and schema as usual.
/// 3.  Add the `#[relation]` attribute to your model structs to define the relationships.
///
/// # Attributes
///
/// The `#[relation]` attribute accepts the following arguments:
///
/// - `model`: **(Required)** The name of the related model as a string (e.g., `"Post"`).
/// - `relation_type`: **(Required)** The type of relationship. Can be `"one_to_many"`, `"many_to_one"`, `"one_to_one"`, or `"many_to_many"`.
/// - `backend`: **(Required)** The database backend you are using. Supported values are `"postgres"`, `"sqlite"`, and `"mysql"`.
/// - `eager_loading`: **(Optional)** A boolean (`true` or `false`) that, when enabled, generates an additional static method for eager loading the relationship. Defaults to `false`.
///
/// ## For `many_to_one`
///
/// - `fk`: **(Required)** The name of the foreign key column on the current model's table (e.g., `"user_id"`).
///
/// ## For `many_to_many`
///
/// - `join_table`: **(Required)** The name of the join table as a string (e.g., `"post_tags"`).
/// - `fk_parent`: **(Required)** The foreign key in the join table that points to the current model (e.g., `"post_id"`).
/// - `fk_child`: **(Required)** The foreign key in the join table that points to the related model (e.g., `"tag_id"`).
/// - `primary_key`: The name of the primary key of the current model. Defaults to `"id"`.
/// - `child_primary_key`: The name of the primary key of the related model. Defaults to the value of `primary_key` if specified, otherwise `"id"`.
///
/// # Generated Methods
///
/// The macro generates two types of methods:
///
/// ## Lazy Loading
///
/// By default, methods are generated to fetch related objects on demand.
/// - For `one-to-many` and `many-to-many`, it generates `get_<model_name_pluralized>()`.
/// - For `one-to-one` and `many-to-one`, it generates `get_<model_name>()`.
///
/// ## Eager Loading
///
/// When `eager_loading = true` is set, an additional static method `load_with_<relation_name>()` is generated to solve the N+1 query problem. For `many_to_one` and `many_to_many` relations, the related models must derive `Clone`.
///
/// # Example: `one-to-many` and `many-to-one`
///
/// ```rust,ignore
/// # // This example is ignored because it requires a database connection and full project setup.
/// # use diesel::prelude::*;
/// # use diesel_linker::relation;
/// #
/// # table! {
/// #     users (id) {
/// #         id -> Integer,
/// #         name -> Text,
/// #     }
/// # }
/// #
/// # table! {
/// #     posts (id) {
/// #         id -> Integer,
/// #         user_id -> Integer,
/// #         title -> Text,
/// #     }
/// # }
/// #
/// # joinable!(posts -> users (user_id));
///
/// // Parent model
/// #[derive(Queryable, Identifiable, Debug)]
/// #[diesel(table_name = users)]
/// #[relation(model = "Post", relation_type = "one_to_many", backend = "sqlite")]
/// pub struct User {
///     pub id: i32,
///     pub name: String,
/// }
///
/// // Child model
/// #[derive(Queryable, Identifiable, Debug, Associations)]
/// #[diesel(belongs_to(User), table_name = posts)]
/// #[relation(model = "User", fk = "user_id", relation_type = "many_to_one", backend = "sqlite")]
/// pub struct Post {
///     pub id: i32,
///     pub user_id: i32,
///     pub title: String,
/// }
///
/// // In your application code:
/// // let user: User = ...;
/// // let posts = user.get_posts(&mut connection)?;
/// // let post: Post = ...;
/// // let user_of_post = post.get_user(&mut connection)?;
/// ```
#[proc_macro_attribute]
pub fn relation(attr: TokenStream, item: TokenStream) -> TokenStream {
    diesel_linker_impl(attr, item)
}
