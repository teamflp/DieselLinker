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
- `method_name`: **(Optional)** A string that specifies a custom name for the generated getter method. If not provided, the name is inferred from the model name (e.g., `get_posts` for a `Post` model).
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

## Exemple d'utilisation

Voici un exemple concret et auto-contenu que vous pouvez exécuter pour voir `DieselLinker` en action. Cet exemple utilise une base de données SQLite en mémoire, vous n'avez donc pas besoin de configurer une base de données externe.

### 1. Dépendances

Ajoutez les dépendances suivantes à votre fichier `Cargo.toml` :

```toml
[dependencies]
diesel = { version = "2.1.0", features = ["sqlite"] }
diesel_linker = "1.2.0"
```

### 2. Code de l'exemple

Copiez-collez le code suivant dans votre `src/main.rs` et exécutez-le avec `cargo run`.

```rust
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_linker::relation;

// Définition du schéma de la base de données.
// Dans un projet réel, cela serait dans `src/schema.rs` et généré par `diesel print-schema`.
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

// Définition des modèles.
// Dans un projet réel, cela serait dans `src/models.rs`.

use schema::{users, posts};

#[derive(Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(table_name = users)]
// Définit une relation "un-à-plusieurs" vers le modèle `Post`.
// DieselLinker générera une méthode `get_posts()` sur la structure `User`.
#[relation(model = "Post", relation_type = "one_to_many", backend = "sqlite")]
pub struct User {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Identifiable, Insertable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User), table_name = posts)]
// Définit une relation "plusieurs-à-un" vers le modèle `User`.
// DieselLinker générera une méthode `get_user()` sur la structure `Post`.
#[relation(model = "User", fk = "user_id", relation_type = "many_to_one", backend = "sqlite")]
pub struct Post {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
}


fn main() {
    // 1. Établir une connexion à une base de données SQLite en mémoire.
    let mut conn = SqliteConnection::establish(":memory:").unwrap();

    // 2. Créer les tables `users` et `posts`.
    diesel::sql_query("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE posts (id INTEGER PRIMARY KEY, user_id INTEGER NOT NULL, title TEXT NOT NULL)").execute(&mut conn).unwrap();

    // 3. Insérer des données de test : un utilisateur et deux articles.
    let new_user = User { id: 1, name: "Marie Curie".to_string() };
    diesel::insert_into(users::table).values(&new_user).execute(&mut conn).unwrap();

    let user_posts = vec![
        Post { id: 1, user_id: 1, title: "Recherches sur la radioactivité".to_string() },
        Post { id: 2, user_id: 1, title: "Découverte du polonium et du radium".to_string() },
    ];
    diesel::insert_into(posts::table).values(&user_posts).execute(&mut conn).unwrap();

    println!("--- Démonstration de DieselLinker ---");

    // 4. Récupérer l'utilisateur depuis la base de données.
    let user = users::table.find(1).first::<User>(&mut conn).unwrap();
    println!("\nUtilisateur trouvé: {}", user.name);

    // 5. Utiliser la méthode `get_posts()` générée par DieselLinker.
    //    Cette méthode charge tous les articles associés à l'utilisateur.
    println!("Récupération des articles de l'utilisateur...");
    let posts = user.get_posts(&mut conn).unwrap();

    println!("'{}' a écrit {} article(s):", user.name, posts.len());
    for post in posts {
        println!("- {}", post.title);

        // On peut aussi remonter à l'auteur depuis l'article.
        let author = post.get_user(&mut conn).unwrap();
        assert_eq!(author.name, user.name);
    }
}
```