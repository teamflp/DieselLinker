// lib.rs

mod relation_macro;
mod utils;

use proc_macro::TokenStream;
use relation_macro::diesel_linker_impl;

/// Macro `DieselLinker` pour faciliter la création de relations entre les modèles Diesel.
///
/// Cette macro permet de déclarer facilement des relations entre différentes tables
/// dans une base de données en utilisant Diesel. Elle supporte différents types de relations,
/// comme "one-to-many", "many-to-one", et "many-to-many".
///
/// # Exemple
///
/// Voici comment vous pourriez définir une relation "one-to-many" entre une table `users`
/// et une table `posts`, où un utilisateur peut avoir plusieurs posts.
///
/// ```bash
/// use diesel_linker::DieselLinker;
///
/// #[derive(DieselLinker)]
/// #[relation(type = "one-to-many", table1 = "users", table2 = "posts", column1 = "id", column2 = "user_id")]
/// struct User {
///     // Définitions des champs de la structure `User`...
/// }
/// ```
///
/// Dans cet exemple, `#[derive(DieselLinker)]` applique la macro `DieselLinker` à la structure `User`.
/// L'attribut `#[relation(...)]` spécifie les détails de la relation entre les tables `users` et `posts`.
/// `type` décrit le type de relation, `table1` et `table2` sont les noms des tables impliquées,
/// et `column1` et `column2` désignent les colonnes utilisées pour joindre les tables.
#[proc_macro_derive(DieselLinker, attributes(relation))]
pub fn diesel_linker_derive(input: TokenStream) -> TokenStream {
    diesel_linker_impl(input, TokenStream::new())
}

/// Définition de la macro d'attribut `relation` qui est utilisée pour spécifier les détails de la relation.
///
/// Cette macro d'attribut est destinée à être utilisée avec `DieselLinker` pour fournir des informations supplémentaires
/// sur la manière dont les modèles sont reliés entre eux.
///
/// # Arguments
///
/// * `attr` - Les attributs passés à la macro, spécifiant le type de relation et les détails des tables et colonnes.
/// * `item` - Le corps de la structure à laquelle la macro est appliquée.
///
/// # Retourne
///
/// Un `TokenStream` contenant le code généré basé sur les attributs de relation spécifiés.
#[proc_macro_attribute]
pub fn relation(attr: TokenStream, item: TokenStream) -> TokenStream {
    diesel_linker_impl(attr, item)
}
