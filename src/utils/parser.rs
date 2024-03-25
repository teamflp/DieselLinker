// Path: src/utils/parser.rs

use proc_macro2::Span;
use syn::{AttributeArgs, Error, Lit, MetaNameValue, NestedMeta, Result};

/// Structure définissant les attributs parsés.
///
/// Cette structure contient tous les attributs nécessaires
/// pour une relation spécifique définie par la macro `DieselLinker`.
#[derive(Debug)]
pub struct ParsedAttrs {
    /// Le nom du champ extra de la relation.
    /// Ceci est utilisé pour générer dynamiquement des champs supplémentaires
    /// ou des fonctions basées sur les attributs passés à la macro.
    pub extra_field_name: String,
}

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
pub fn parse_attributes(attrs: AttributeArgs) -> Result<ParsedAttrs> {
    let extra_field_name = attrs
        .iter()
        .find_map(|attr| {
            if let NestedMeta::Meta(syn::Meta::NameValue(MetaNameValue { path, lit: Lit::Str(lit_str), .. })) = attr {
                if path.is_ident("extra_field_name") {
                    return Some(lit_str.value());
                }
            }
            None
        })
        .ok_or_else(|| {
            // Utilise `Span::call_site()` pour générer une erreur sans dépendre d'un élément spécifique.
            Error::new(Span::call_site(), "Attribut 'extra_field_name' attendu")
        })?;

    Ok(ParsedAttrs { extra_field_name })
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Meta};

    /// Tests pour vérifier le parsing des attributs de la macro.
    #[test]
    fn test_parse_attributes_valid() {
        let attrs = vec![NestedMeta::Meta(Meta::NameValue(MetaNameValue {
            path: parse_quote! { extra_field_name },
            eq_token: Default::default(),
            lit: syn::Lit::Str(syn::LitStr::new("valid_field", proc_macro2::Span::call_site())),
        }))];

        let result = parse_attributes(attrs);
        assert!(result.is_ok(), "Devrait réussir avec un attribut valide.");
        assert_eq!(result.unwrap().extra_field_name, "valid_field");
    }

    #[test]
    fn test_parse_attributes_invalid() {
        let attrs = vec![NestedMeta::Meta(parse_quote! { some_attribute })];

        let result = parse_attributes(attrs);
        assert!(result.is_err(), "Devrait échouer avec un attribut invalide.");
        assert_eq!(result.unwrap_err().to_string(), "Attribut 'extra_field_name' attendu");
    }

    #[test]
    fn test_no_attributes() {
        let attrs = vec![];

        let result = parse_attributes(attrs);
        assert!(result.is_err(), "Devrait échouer sans attributs.");
    }

    #[test]
    fn test_multiple_attributes_including_valid() {
        let attrs = vec![
            NestedMeta::Meta(parse_quote! { unrelated_attribute = "some_value" }),
            NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                path: parse_quote! { extra_field_name },
                eq_token: Default::default(),
                lit: syn::Lit::Str(syn::LitStr::new("valid_field", proc_macro2::Span::call_site())),
            })),
        ];

        let result = parse_attributes(attrs);
        assert!(result.is_ok(), "Devrait réussir avec plusieurs attributs, y compris un attribut valide.");
        assert_eq!(result.unwrap().extra_field_name, "valid_field");
    }

    #[test]
    fn test_malformed_attributes() {
        let attrs = vec![
            NestedMeta::Meta(parse_quote! { extra_field_name = 123 }), // Mauvais type de valeur
        ];

        let result = parse_attributes(attrs);
        assert!(result.is_err(), "Devrait échouer avec des attributs mal formés.");
    }
}
