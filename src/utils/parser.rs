// Path: src/utils/parser.rs

use proc_macro2::Span;
use syn::{spanned::Spanned, AttributeArgs, Error, Lit, Meta, NestedMeta, Result};

#[derive(Debug)]
pub struct Attr<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Attr<T> {
    fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

#[derive(Debug, Default)]
pub struct ParsedAttrs {
    pub relation_type: Option<Attr<String>>,
    pub model: Option<Attr<String>>,
    pub fk: Option<Attr<String>>,         // Foreign key for the relation. Required for `many_to_one`.
    pub parent_primary_key: Option<Attr<String>>, // Primary key of the parent model for `many_to_one` eager loading.
    pub join_table: Option<Attr<String>>, // Join table for `many_to_many` relations.
    pub fk_parent: Option<Attr<String>>,  // Foreign key in the join table pointing to the parent model.
    pub fk_child: Option<Attr<String>>,   // Foreign key in the join table pointing to the child model.
    pub method_name: Option<Attr<String>>,
    pub backend: Option<Attr<String>>,
    pub primary_key: Option<Attr<String>>,
    pub child_primary_key: Option<Attr<String>>,
    pub eager_loading: Option<Attr<bool>>,
}

// Parses the attributes passed to the `relation` macro.
pub fn parse_attributes(attrs: AttributeArgs) -> Result<ParsedAttrs> {
    let mut parsed_attrs = ParsedAttrs::default();

    for attr in attrs {
        if let NestedMeta::Meta(Meta::NameValue(nv)) = attr {
            let ident = nv
                .path
                .get_ident()
                .ok_or_else(|| Error::new(nv.path.span(), "Expected a single identifier"))?
                .to_string();
            let span = nv.span();
            match ident.as_str() {
                "relation_type" => {
                    if let Lit::Str(s) = &nv.lit {
                        parsed_attrs.relation_type = Some(Attr::new(s.value(), span));
                    }
                }
                "model" => {
                    if let Lit::Str(s) = &nv.lit {
                        parsed_attrs.model = Some(Attr::new(s.value(), span));
                    }
                }
                "fk" => {
                    if let Lit::Str(s) = &nv.lit {
                        parsed_attrs.fk = Some(Attr::new(s.value(), span));
                    }
                }
                "parent_primary_key" => {
                    if let Lit::Str(s) = &nv.lit {
                        parsed_attrs.parent_primary_key = Some(Attr::new(s.value(), span));
                    }
                }
                "join_table" => {
                    if let Lit::Str(s) = &nv.lit {
                        parsed_attrs.join_table = Some(Attr::new(s.value(), span));
                    }
                }
                "fk_parent" => {
                    if let Lit::Str(s) = &nv.lit {
                        parsed_attrs.fk_parent = Some(Attr::new(s.value(), span));
                    }
                }
                "fk_child" => {
                    if let Lit::Str(s) = &nv.lit {
                        parsed_attrs.fk_child = Some(Attr::new(s.value(), span));
                    }
                }
                "method_name" => {
                    if let Lit::Str(s) = &nv.lit {
                        parsed_attrs.method_name = Some(Attr::new(s.value(), span));
                    }
                }
                "backend" => {
                    if let Lit::Str(s) = &nv.lit {
                        parsed_attrs.backend = Some(Attr::new(s.value(), span));
                    }
                }
                "primary_key" => {
                    if let Lit::Str(s) = &nv.lit {
                        parsed_attrs.primary_key = Some(Attr::new(s.value(), span));
                    }
                }
                "child_primary_key" => {
                    if let Lit::Str(s) = &nv.lit {
                        parsed_attrs.child_primary_key = Some(Attr::new(s.value(), span));
                    }
                }
                "eager_loading" => {
                    if let Lit::Bool(b) = &nv.lit {
                        parsed_attrs.eager_loading = Some(Attr::new(b.value(), span));
                    }
                }
                _ => {
                    return Err(Error::new(
                        nv.path.span(),
                        "Unknown attribute, expected one of: `relation_type`, `model`, `fk`, `parent_primary_key`, `join_table`, `fk_parent`, `fk_child`, `method_name`, `backend`, `primary_key`, `child_primary_key`, `eager_loading`",
                    ))
                }
            }
        } else {
            return Err(Error::new(
                attr.span(),
                "Unexpected attribute format, expected `name = \"value\"`",
            ));
        }
    }

    // --- Validation ---

    let relation_type = if let Some(rt) = &parsed_attrs.relation_type {
        rt
    } else {
        return Err(Error::new(
            Span::call_site(),
            "The required attribute `relation_type` is missing.",
        ));
    };

    if parsed_attrs.backend.is_none() {
        return Err(Error::new(
            Span::call_site(),
            "The required attribute `backend` is missing.",
        ));
    }

    match relation_type.value.as_str() {
        "one_to_many" | "one_to_one" => {
            if parsed_attrs.model.is_none() {
                return Err(Error::new(
                    relation_type.span,
                    "The `model` attribute is required for this relation type.",
                ));
            }
            if let Some(attr) = &parsed_attrs.fk {
                return Err(Error::new(attr.span, "`fk` is not used for this relation type. The foreign key is defined on the child model with `#[diesel(belongs_to(...))]`."));
            }
            if let Some(attr) = &parsed_attrs.join_table {
                return Err(Error::new(attr.span, "`join_table` is only used for `many_to_many` relations."));
            }
        }
        "many_to_one" => {
            if parsed_attrs.model.is_none() {
                return Err(Error::new(relation_type.span, "The `model` attribute is required for a `many_to_one` relationship."));
            }
            if parsed_attrs.fk.is_none() {
                return Err(Error::new(relation_type.span, "The `fk` attribute (foreign key) is required for a `many_to_one` relationship."));
            }
            if let Some(attr) = &parsed_attrs.join_table {
                return Err(Error::new(attr.span, "`join_table` is only used for `many_to_many` relations."));
            }
            if let Some(attr) = &parsed_attrs.parent_primary_key {
                if !parsed_attrs.eager_loading.as_ref().map_or(false, |a| a.value) {
                    return Err(Error::new(attr.span, "`parent_primary_key` is only used for eager loading."));
                }
            }
        }
        "many_to_many" => {
            if parsed_attrs.model.is_none() {
                return Err(Error::new(relation_type.span, "The `model` attribute is required for a `many_to_many` relationship."));
            }
            if parsed_attrs.join_table.is_none() {
                return Err(Error::new(relation_type.span, "The `join_table` attribute is required for a `many_to_many` relationship."));
            }
            if parsed_attrs.fk_parent.is_none() {
                return Err(Error::new(relation_type.span, "The `fk_parent` attribute is required for a `many_to_many` relationship."));
            }
            if parsed_attrs.fk_child.is_none() {
                return Err(Error::new(relation_type.span, "The `fk_child` attribute is required for a `many_to_many` relationship."));
            }
            if let Some(attr) = &parsed_attrs.fk {
                return Err(Error::new(attr.span, "`fk` is not used for `many_to_many` relations. Use `fk_parent` and `fk_child` instead."));
            }
        }
        _ => {
            return Err(Error::new(
                relation_type.span,
                "Unsupported relation type. Supported types are: `one_to_many`, `many_to_one`, `one_to_one`, `many_to_many`.",
            ))
        }
    }

    Ok(parsed_attrs)
}

// The test module is only compiled when running `cargo test`.
#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, NestedMeta};

    #[test]
    fn test_valid_many_to_one() {
        let attrs = vec![
            NestedMeta::Meta(parse_quote! { relation_type = "many_to_one" }),
            NestedMeta::Meta(parse_quote! { model = "User" }),
            NestedMeta::Meta(parse_quote! { fk = "user_id" }),
            NestedMeta::Meta(parse_quote! { backend = "sqlite" }),
        ];
        let result = parse_attributes(attrs);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.relation_type.unwrap().value, "many_to_one");
        assert_eq!(parsed.model.unwrap().value, "User");
        assert_eq!(parsed.fk.unwrap().value, "user_id");
    }

    #[test]
    fn test_valid_many_to_many() {
        let attrs = vec![
            NestedMeta::Meta(parse_quote! { relation_type = "many_to_many" }),
            NestedMeta::Meta(parse_quote! { model = "Tag" }),
            NestedMeta::Meta(parse_quote! { join_table = "post_tags" }),
            NestedMeta::Meta(parse_quote! { fk_parent = "post_id" }),
            NestedMeta::Meta(parse_quote! { fk_child = "tag_id" }),
            NestedMeta::Meta(parse_quote! { backend = "sqlite" }),
        ];
        let result = parse_attributes(attrs);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.relation_type.unwrap().value, "many_to_many");
        assert_eq!(parsed.model.unwrap().value, "Tag");
        assert!(parsed.fk.is_none());
        assert_eq!(parsed.join_table.unwrap().value, "post_tags");
        assert_eq!(parsed.fk_parent.unwrap().value, "post_id");
        assert_eq!(parsed.fk_child.unwrap().value, "tag_id");
    }

    #[test]
    fn test_valid_one_to_one() {
        let attrs = vec![
            NestedMeta::Meta(parse_quote! { relation_type = "one_to_one" }),
            NestedMeta::Meta(parse_quote! { model = "Profile" }),
            NestedMeta::Meta(parse_quote! { backend = "sqlite" }),
        ];
        let result = parse_attributes(attrs);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.relation_type.unwrap().value, "one_to_one");
        assert_eq!(parsed.model.unwrap().value, "Profile");
        assert!(parsed.fk.is_none());
    }

    #[test]
    fn test_superfluous_fk_is_err() {
        let attrs = vec![
            NestedMeta::Meta(parse_quote! { relation_type = "one_to_one" }),
            NestedMeta::Meta(parse_quote! { model = "users" }),
            NestedMeta::Meta(parse_quote! { fk = "user_id" }), // `fk` is not used for one_to_one
            NestedMeta::Meta(parse_quote! { backend = "sqlite" }),
        ];
        let result = parse_attributes(attrs);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "`fk` is not used for this relation type. The foreign key is defined on the child model with `#[diesel(belongs_to(...))]`.");
    }

    #[test]
    fn test_superfluous_join_table_is_err() {
        let attrs = vec![
            NestedMeta::Meta(parse_quote! { relation_type = "many_to_one" }),
            NestedMeta::Meta(parse_quote! { model = "User" }),
            NestedMeta::Meta(parse_quote! { fk = "user_id" }),
            NestedMeta::Meta(parse_quote! { join_table = "user_posts" }), // `join_table` is not used for many_to_one
            NestedMeta::Meta(parse_quote! { backend = "sqlite" }),
        ];
        let result = parse_attributes(attrs);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "`join_table` is only used for `many_to_many` relations.");
    }

    #[test]
    fn test_missing_fk_for_many_to_one_is_err() {
        let attrs = vec![
            NestedMeta::Meta(parse_quote! { relation_type = "many_to_one" }),
            NestedMeta::Meta(parse_quote! { model = "users" }),
            // `fk` is missing
            NestedMeta::Meta(parse_quote! { backend = "sqlite" }),
        ];
        let result = parse_attributes(attrs);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "The `fk` attribute (foreign key) is required for a `many_to_one` relationship.");
    }
}