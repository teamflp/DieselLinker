# Guide d'utilisation de la Macro `DieselLinker`

Ce guide vous montre comment utiliser la macro `DieselLinker` pour faciliter la définition de relations entre les tables dans une application Rust utilisant Diesel.

## Prérequis

- Assurez-vous que `Diesel` est ajouté à vos dépendances dans `Cargo.toml`.
- La macro `DieselLinker` doit être ajoutée comme dépendance.
    
```toml
[dependencies]
diesel = { version = "1.4", features = ["postgres"] }
diesel_linker = "0.1"
```

## Étape 1 : Définir vos Modèles

Commencez par définir vos modèles Rust qui correspondent à vos tables dans la base de données.

```rust
#[derive(Queryable)]
struct User {
    pub id: i32,
    pub name: String,
    // autres champs...
}

#[derive(Queryable)]
struct Post {
    pub id: i32,
    pub user_id: i32, // Clé étrangère vers la table User
    pub title: String,
    // autres champs...
}
```

## Étape 2 : Utiliser la Macro DieselLinker
Appliquez la macro `DieselLinker` à votre modèle pour définir la relation entre les tables.

### Pour une Relation `One-to-Many`
Si un utilisateur peut avoir plusieurs posts, vous pouvez définir cette relation comme suit :

```rust
use diesel_linker::DieselLinker;

#[derive(DieselLinker)]
#[relation(type = "one-to-many", table1 = "users", table2 = "posts", column1 = "id", column2 = "user_id")]
struct User {
    // Définitions de champs...
}

#[derive(DieselLinker)]
#[relation(type = "many-to-one", table1 = "users", table2 = "posts", column1 = "id", column2 = "user_id")]
struct Post {
    // Définitions de champs...
}
```

Cette syntaxe indique que la table `users` a une relation "`one-to-many`" avec la table `posts`, où `column1 (id dans users)` est la clé primaire et c`olumn2 (user_id dans posts)` est la clé étrangère.

## Étape 3 : Compiler et Tester
Après avoir appliqué la macro à vos structures, compilez votre projet pour vous assurer que la macro fonctionne comme attendu.

```bash
cargo build
```
Effectuez également des tests pour confirmer que les relations sont correctement gérées et que vous pouvez effectuer des opérations liées à la base de données selon vos besoins.

## Conclusion
La macro DieselLinker vise à simplifier la définition des relations entre les tables dans les applications Diesel. Assurez-vous de tester soigneusement dans divers scénarios pour confirmer que tout fonctionne comme prévu.
