// Path: src/utils/parser.rs

use proc_macro2::Span;
use syn::{AttributeArgs, Error, Lit, Meta, NestedMeta, Result};

/// Structure définissant les attributs parsés.
///
/// Cette structure contient tous les attributs nécessaires
/// pour une relation spécifique définie par la macro `DieselLinker`.
/*#[derive(Debug)]
pub struct ParsedAttrs {
    /// Le nom du champ extra de la relation.
    /// Ceci est utilisé pour générer dynamiquement des champs supplémentaires
    /// ou des fonctions basées sur les attributs passés à la macro.
    pub extra_field_name: String,
}*/

/// Parse les attributs donnés à la macro et extrait les informations nécessaires.
///
/// Cette fonction analyse les `AttributeArgs` fournis à une macro d'attribut
/// et tente d'extraire et de valider les informations nécessaires comme le nom du champ extra.
///
/// # Arguments
///
/// * `attrs` - Les attributs passés à la macro, représentés comme `AttributeArgs`.
///
/// # Retourne
///
/// Cette fonction retourne un `Result` contenant soit `ParsedAttrs`
/// avec les informations extraites en cas de succès, soit une erreur `syn::Error`
/// si les attributs ne sont pas conformes aux attentes.

// Exemple de définir une relation one-to-many entre les tables `users` et `posts`
// #[derive(DieselLinker)]
// #[relation(type = "one_to_many", table_from = "users", table_to = "posts", column_from = "id", column_to = "user_id")]

// Exemple de définir une relation many-to-many entre les tables `users` et `posts`
// #[derive(DieselLinker)]
// #[relation(type = "many_to_many", join_table = "users_posts", table_from = "users", table_to = "posts", column_from = "user_id", column_to = "post_id")]

// Exemple de définir une relation many-to-one entre les tables `posts` et `users`
// #[derive(DieselLinker)]
// #[relation(type = "many_to_one", table_from = "posts", table_to = "users", column_from = "user_id", column_to = "id")]

// Exemple de définir une relation one-to-one entre les tables `users` et `profiles`
// #[derive(DieselLinker)]
// #[relation(type = "one_to_one", table_from = "users", table_to = "profiles", column_from = "id", column_to = "user_id")]

// Exemple de définir une relation one-to-many entre les tables `users` et `posts` avec un champ extra `created_at`
// #[derive(DieselLinker)]
// #[relation(type = "one_to_many", table_from = "users", table_to = "posts", column_from = "id", column_to = "user_id", extra_field_name = "created_at")]
#[derive(Debug)]
pub struct ParsedAttrs {
    pub relation_type: String, // Nouveau champ pour stocker le type de relation
    pub join_table: Option<String>, // Pour les relations many-to-many
    pub table_from: Option<String>, // Nom de la table source
    pub table_to: Option<String>, // Nom de la table cible
    pub column_from: Option<String>, // Colonne sur la table source
    pub column_to: Option<String>, // Colonne sur la table cible
    pub extra_field_name: Option<String>, // Champ extra potentiellement utilisé pour la relation
}

/// Parse les attributs donnés à la macro et extrait les informations nécessaires.
pub fn parse_attributes(attrs: AttributeArgs) -> Result<ParsedAttrs> {
    let mut parsed_attrs = ParsedAttrs {
        relation_type: String::new(),
        join_table: None,
        table_from: None,
        table_to: None,
        column_from: None,
        column_to: None,
        extra_field_name: None,
    };

    for attr in attrs {
        match attr {
            NestedMeta::Meta(Meta::NameValue(nv)) => {
                match nv.path.get_ident() {
                    Some(ident) => {
                        let ident_str = ident.to_string();
                        match ident_str.as_str() {
                            "type" => {
                                if let Lit::Str(lit_str) = &nv.lit {
                                    parsed_attrs.relation_type = lit_str.value();
                                }
                            },
                            "join_table" => {
                                if let Lit::Str(lit_str) = &nv.lit {
                                    parsed_attrs.join_table = Some(lit_str.value());
                                }
                            },
                            "table_from" => {
                                if let Lit::Str(lit_str) = &nv.lit {
                                    parsed_attrs.table_from = Some(lit_str.value());
                                }
                            },
                            "table_to" => {
                                if let Lit::Str(lit_str) = &nv.lit {
                                    parsed_attrs.table_to = Some(lit_str.value());
                                }
                            },
                            "column_from" => {
                                if let Lit::Str(lit_str) = &nv.lit {
                                    parsed_attrs.column_from = Some(lit_str.value());
                                }
                            },
                            "column_to" => {
                                if let Lit::Str(lit_str) = &nv.lit {
                                    parsed_attrs.column_to = Some(lit_str.value());
                                }
                            },
                            "extra_field_name" => {
                                if let Lit::Str(lit_str) = &nv.lit {
                                    parsed_attrs.extra_field_name = Some(lit_str.value());
                                }
                            },
                            _ => {}
                        }
                    },
                    None => {}
                }
            },
            _ => {}
        }
    }

    if parsed_attrs.relation_type.is_empty() {
        return Err(Error::new(Span::call_site(), "Attribut 'type' de relation attendu"));
    }

    Ok(parsed_attrs)
}


// Tests unitaires
#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote,NestedMeta};

    // Test pour une relation "one-to-one"
    #[test]
    fn test_one_to_one_relation_attributes() {
        let attrs = vec![
            NestedMeta::Meta(parse_quote! { type = "one_to_one" }),
            NestedMeta::Meta(parse_quote! { table_from = "users" }),
            NestedMeta::Meta(parse_quote! { table_to = "profiles" }),
            NestedMeta::Meta(parse_quote! { column_from = "id" }),
            NestedMeta::Meta(parse_quote! { column_to = "user_id" }),
        ];

        let result = parse_attributes(attrs);
        assert!(result.is_ok(), "Devrait sélectionner pour une relation one-to-one valide.");

        let parsed = result.unwrap();
        assert_eq!(parsed.relation_type, "one_to_one");
        assert_eq!(parsed.table_from.unwrap(), "users");
        assert_eq!(parsed.table_to.unwrap(), "profiles");
        assert_eq!(parsed.column_from.unwrap(), "id");
        assert_eq!(parsed.column_to.unwrap(), "user_id");
    }

    // Test pour une relation "one-to-many"
    #[test]
    fn test_one_to_many_relation_attributes() {
        let attrs = vec![
            NestedMeta::Meta(parse_quote! { type = "one_to_many" }),
            NestedMeta::Meta(parse_quote! { table_from = "users" }),
            NestedMeta::Meta(parse_quote! { table_to = "posts" }),
            NestedMeta::Meta(parse_quote! { column_from = "id" }),
            NestedMeta::Meta(parse_quote! { column_to = "user_id" }),
        ];

        let result = parse_attributes(attrs);
        assert!(result.is_ok(), "Devrait réussir pour une relation one-to-many valide.");

        let parsed = result.unwrap();
        assert_eq!(parsed.relation_type, "one_to_many");
        assert_eq!(parsed.table_from.unwrap(), "users");
        assert_eq!(parsed.table_to.unwrap(), "posts");
        assert_eq!(parsed.column_from.unwrap(), "id");
        assert_eq!(parsed.column_to.unwrap(), "user_id");
    }

    // Test pour une relation "many-to-one"
    #[test]
    fn test_many_to_one_relation_attributes() {
        let attrs = vec![
            NestedMeta::Meta(parse_quote! { type = "many_to_one" }),
            NestedMeta::Meta(parse_quote! { table_from = "posts" }),
            NestedMeta::Meta(parse_quote! { table_to = "users" }),
            NestedMeta::Meta(parse_quote! { column_from = "user_id" }),
            NestedMeta::Meta(parse_quote! { column_to = "id" }),
        ];

        let result = parse_attributes(attrs);
        assert!(result.is_ok(), "Devrait réussir pour une relation many-to-one valide.");

        let parsed = result.unwrap();
        assert_eq!(parsed.relation_type, "many_to_one");
        assert_eq!(parsed.table_from.unwrap(), "posts");
        assert_eq!(parsed.table_to.unwrap(), "users");
        assert_eq!(parsed.column_from.unwrap(), "user_id");
        assert_eq!(parsed.column_to.unwrap(), "id");
    }

    // Test pour une relation "many-to-many"
    #[test]
    fn test_many_to_many_relation_attributes() {
        let attrs = vec![
            NestedMeta::Meta(parse_quote! { type = "many_to_many" }),
            NestedMeta::Meta(parse_quote! { join_table = "users_posts" }),
            NestedMeta::Meta(parse_quote! { table_from = "users" }),
            NestedMeta::Meta(parse_quote! { table_to = "posts" }),
            NestedMeta::Meta(parse_quote! { column_from = "user_id" }),
            NestedMeta::Meta(parse_quote! { column_to = "post_id" }),
        ];

        let result = parse_attributes(attrs);
        assert!(result.is_ok(), "Devrait réussir pour une relation many-to-many valide.");

        let parsed = result.unwrap();
        assert_eq!(parsed.relation_type, "many_to_many");
        assert_eq!(parsed.join_table.unwrap(), "users_posts");
        assert_eq!(parsed.table_from.unwrap(), "users");
        assert_eq!(parsed.table_to.unwrap(), "posts");
        assert_eq!(parsed.column_from.unwrap(), "user_id");
        assert_eq!(parsed.column_to.unwrap(), "post_id");
    }
}