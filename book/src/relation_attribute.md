# The `#[relation]` attribute

The `#[relation]` attribute is the core of `DieselLinker`. It generates methods on your model structs to fetch related models.

You can apply multiple `#[relation]` attributes to a single struct to define multiple relationships.

## Common Attributes

These attributes are common to all relationship types:

-   `model`: **(Required)** The name of the related model as a string (e.g., `"Post"`).
-   `relation_type`: **(Required)** The type of relationship. Can be `"one_to_many"`, `"many_to_one"`, `"one_to_one"`, or `"many_to_many"`.
-   `backend`: **(Required)** The database backend you are using. Supported values are `"postgres"`, `"sqlite"`, and `"mysql"`.
-   `method_name`: **(Optional)** A string that specifies a custom name for the generated getter method. If not provided, a name is inferred from the model name (e.g., `get_posts` for a `Post` model).
-   `eager_loading`: **(Optional)** A boolean (`true` or `false`) that, when enabled, generates an additional static method for eager loading the relationship. Defaults to `false`.
-   `async`: **(Optional)** A boolean (`true` or `false`) that, when enabled, generates `async` methods for use with `diesel-async`. Defaults to `false`.
-   `error_type`: **(Optional)** A string representing a custom error type to be used in the return type of the generated methods. The custom error type must implement `From<diesel::result::Error>`.

## Attributes for `many_to_one`

-   `fk`: **(Required)** The name of the foreign key column on the current model's table (e.g., `"user_id"`).
-   `parent_primary_key`: **(Optional)** The name of the primary key on the parent model. Defaults to `"id"`. This is only used when `eager_loading` is set to `true`.

## Attributes for `many_to_many`

-   `join_table`: **(Required)** The name of the join table as a string (e.g., `"post_tags"`).
-   `fk_parent`: **(Required)** The foreign key in the join table that points to the current model (e.g., `"post_id"`).
-   `fk_child`: **(Required)** The foreign key in the join table that points to the related model (e.g., `"tag_id"`).
-   `primary_key`: The name of the primary key of the current model. Defaults to `"id"`.
-   `child_primary_key`: The name of the primary key of the related model. Defaults to the value of `primary_key` if specified, otherwise `"id"`.
