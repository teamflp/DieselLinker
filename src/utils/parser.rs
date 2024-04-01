// Path: src/utils/parser.rs

use crate::utils::validation::{
    validate_many_to_many_attrs, validate_one_to_many_or_many_to_one_attrs,
    validate_one_to_one_attrs,
};
use proc_macro2::Span;
use syn::{AttributeArgs, Error, Lit, Meta, NestedMeta, Result};

/// Structure définissant les attributs parsés.
///
/// Cette structure contient tous les attributs nécessaires
/// pour une relation spécifique définie par la macro `DieselLinker`.
///
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
    pub relation_type: String,            // relation type
    pub join_table: Option<String>,       // For many-to-many relations
    pub table_from: Option<String>,       // Table source
    pub table_to: Option<String>,         // target table
    pub column_from: Option<String>,      // Column source table
    pub column_to: Option<String>,        // Column target table
    pub extra_field_name: Option<String>, // Relationship extra field name
}

/// Parse the attributes given to the `DieselLinker` macro.
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
            NestedMeta::Meta(Meta::NameValue(nv)) => match nv.path.get_ident() {
                Some(ident) => {
                    let ident_str = ident.to_string();
                    match ident_str.as_str() {
                        "type" => {
                            if let Lit::Str(lit_str) = &nv.lit {
                                parsed_attrs.relation_type = lit_str.value();
                            }
                        }
                        "join_table" => {
                            if let Lit::Str(lit_str) = &nv.lit {
                                parsed_attrs.join_table = Some(lit_str.value());
                            }
                        }
                        "table_from" => {
                            if let Lit::Str(lit_str) = &nv.lit {
                                parsed_attrs.table_from = Some(lit_str.value());
                            }
                        }
                        "table_to" => {
                            if let Lit::Str(lit_str) = &nv.lit {
                                parsed_attrs.table_to = Some(lit_str.value());
                            }
                        }
                        "column_from" => {
                            if let Lit::Str(lit_str) = &nv.lit {
                                parsed_attrs.column_from = Some(lit_str.value());
                            }
                        }
                        "column_to" => {
                            if let Lit::Str(lit_str) = &nv.lit {
                                parsed_attrs.column_to = Some(lit_str.value());
                            }
                        }
                        "extra_field_name" => {
                            if let Lit::Str(lit_str) = &nv.lit {
                                parsed_attrs.extra_field_name = Some(lit_str.value());
                            }
                        }
                        _ => {}
                    }
                }
                None => {}
            },
            _ => {}
        }
    }

    if parsed_attrs.relation_type.is_empty() {
        return Err(Error::new(
            Span::call_site(),
            "Attribut 'type' required to specify the relation type.",
        ));
    }

    match parsed_attrs.relation_type.as_str() {
        "many_to_many" => validate_many_to_many_attrs(&parsed_attrs)
            .map_err(|e| Error::new(Span::call_site(), e))?,
        "many_to_one" | "one_to_many" => validate_one_to_many_or_many_to_one_attrs(&parsed_attrs)
            .map_err(|e| Error::new(Span::call_site(), e))?,
        "one_to_one" => validate_one_to_one_attrs(&parsed_attrs)
            .map_err(|e| Error::new(Span::call_site(), e))?,
        _ => {
            return Err(Error::new(
                Span::call_site(),
                "Relation type not supported.",
            ))
        }
    }

    Ok(parsed_attrs)
}

// Tests unitaires
#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, NestedMeta};

    // Test for a one-to-one relationship
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
        assert!(
            result.is_ok(),
            "Should succeed for a valid one-to-one relationship."
        );

        let parsed = result.unwrap();
        assert_eq!(parsed.relation_type, "one_to_one");
        assert_eq!(parsed.table_from.unwrap(), "users");
        assert_eq!(parsed.table_to.unwrap(), "profiles");
        assert_eq!(parsed.column_from.unwrap(), "id");
        assert_eq!(parsed.column_to.unwrap(), "user_id");
    }

    // Test for a one-to-many relationship
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
        assert!(
            result.is_ok(),
            "Should succeed for a valid one-to-many relationship."
        );

        let parsed = result.unwrap();
        assert_eq!(parsed.relation_type, "one_to_many");
        assert_eq!(parsed.table_from.unwrap(), "users");
        assert_eq!(parsed.table_to.unwrap(), "posts");
        assert_eq!(parsed.column_from.unwrap(), "id");
        assert_eq!(parsed.column_to.unwrap(), "user_id");
    }

    // Test for a many-to-one relationship
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
        assert!(
            result.is_ok(),
            "Should succeed for a valid many-to-one relationship."
        );

        let parsed = result.unwrap();
        assert_eq!(parsed.relation_type, "many_to_one");
        assert_eq!(parsed.table_from.unwrap(), "posts");
        assert_eq!(parsed.table_to.unwrap(), "users");
        assert_eq!(parsed.column_from.unwrap(), "user_id");
        assert_eq!(parsed.column_to.unwrap(), "id");
    }

    // Test for a many-to-many relationship
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
        assert!(
            result.is_ok(),
            "Should succeed for a valid many-to-many relationship."
        );

        let parsed = result.unwrap();
        assert_eq!(parsed.relation_type, "many_to_many");
        assert_eq!(parsed.join_table.unwrap(), "users_posts");
        assert_eq!(parsed.table_from.unwrap(), "users");
        assert_eq!(parsed.table_to.unwrap(), "posts");
        assert_eq!(parsed.column_from.unwrap(), "user_id");
        assert_eq!(parsed.column_to.unwrap(), "post_id");
    }

    // Test for missing relation type
    #[test]
    fn test_missing_relation_type() {
        let attrs = vec![
            NestedMeta::Meta(parse_quote! { table_from = "users" }),
            NestedMeta::Meta(parse_quote! { table_to = "profiles" }),
            NestedMeta::Meta(parse_quote! { column_from = "id" }),
            NestedMeta::Meta(parse_quote! { column_to = "user_id" }),
        ];

        let result = parse_attributes(attrs);
        assert!(
            result.is_err(),
            "Should fail for a missing relationship type.."
        );
    }

    // Test for invalid relation type
    #[test]
    fn test_invalid_relation_type() {
        let attrs = vec![
            NestedMeta::Meta(parse_quote! { type = "invalid" }),
            NestedMeta::Meta(parse_quote! { table_from = "users" }),
            NestedMeta::Meta(parse_quote! { table_to = "profiles" }),
            NestedMeta::Meta(parse_quote! { column_from = "id" }),
            NestedMeta::Meta(parse_quote! { column_to = "user_id" }),
        ];

        let result = parse_attributes(attrs);
        assert!(
            result.is_err(),
            "Should fail for an invalid relationship type."
        );
    }

    // Test for missing table_from
    #[test]
    fn test_validate_one_to_one_attrs() {
        let attrs = ParsedAttrs {
            relation_type: "one_to_one".to_string(),
            join_table: None,
            table_from: Some("users".to_string()),
            table_to: Some("profiles".to_string()),
            column_from: Some("id".to_string()),
            column_to: Some("user_id".to_string()),
            extra_field_name: None,
        };

        let result = validate_one_to_one_attrs(&attrs);
        assert!(
            result.is_ok(),
            "Should succeed for a valid one-to-one relationship."
        );
    }

    // Test for missing table_from
    #[test]
    fn test_validate_one_to_many_or_many_to_one_attrs() {
        let attrs = ParsedAttrs {
            relation_type: "one_to_many".to_string(),
            join_table: None,
            table_from: Some("users".to_string()),
            table_to: Some("posts".to_string()),
            column_from: Some("id".to_string()),
            column_to: Some("user_id".to_string()),
            extra_field_name: None,
        };

        let result = validate_one_to_many_or_many_to_one_attrs(&attrs);
        assert!(
            result.is_ok(),
            "Should succeed for a valid one-to-many or many-to-one relationship."
        );
    }

    // Test for missing table_from
    #[test]
    fn test_validate_many_to_many_attrs() {
        let attrs = ParsedAttrs {
            relation_type: "many_to_many".to_string(),
            join_table: Some("users_posts".to_string()),
            table_from: Some("users".to_string()),
            table_to: Some("posts".to_string()),
            column_from: Some("user_id".to_string()),
            column_to: Some("post_id".to_string()),
            extra_field_name: None,
        };

        let result = validate_many_to_many_attrs(&attrs);
        assert!(
            result.is_ok(),
            "Should succeed for a valid many-to-many relationship."
        );
    }
}
