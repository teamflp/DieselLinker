// Path: src/utils/parser.rs

use proc_macro2::Span;
use syn::{AttributeArgs, Error, Lit, Meta, NestedMeta, Result};

#[derive(Debug, Default)]
pub struct ParsedAttrs {
    pub relation_type: Option<String>,
    pub model: Option<String>,
    pub fk: Option<String>,         // Foreign key for the relation. Required for `many_to_one`.
    pub join_table: Option<String>, // Join table for `many_to_many` relations.
    pub fk_parent: Option<String>,  // Foreign key in the join table pointing to the parent model.
    pub fk_child: Option<String>,   // Foreign key in the join table pointing to the child model.
    pub method_name: Option<String>,
    pub backend: Option<String>,
    pub primary_key: Option<String>,
    pub child_primary_key: Option<String>,
    pub eager_loading: Option<bool>,
}

// Parses the attributes passed to the `relation` macro.
pub fn parse_attributes(attrs: AttributeArgs) -> Result<ParsedAttrs> {
    let mut parsed_attrs = ParsedAttrs::default();

    for attr in attrs {
        match attr {
            NestedMeta::Meta(Meta::NameValue(nv)) => {
                let ident = nv
                    .path
                    .get_ident()
                    .ok_or_else(|| Error::new(Span::call_site(), "Expected named value"))?
                    .to_string();
                match ident.as_str() {
                    "relation_type" => {
                        if let Lit::Str(s) = &nv.lit {
                            parsed_attrs.relation_type = Some(s.value())
                        }
                    }
                    "model" => {
                        if let Lit::Str(s) = &nv.lit {
                            parsed_attrs.model = Some(s.value())
                        }
                    }
                    "fk" => {
                        if let Lit::Str(s) = &nv.lit {
                            parsed_attrs.fk = Some(s.value())
                        }
                    }
                    "join_table" => {
                        if let Lit::Str(s) = &nv.lit {
                            parsed_attrs.join_table = Some(s.value())
                        }
                    }
                    "fk_parent" => {
                        if let Lit::Str(s) = &nv.lit {
                            parsed_attrs.fk_parent = Some(s.value())
                        }
                    }
                    "fk_child" => {
                        if let Lit::Str(s) = &nv.lit {
                            parsed_attrs.fk_child = Some(s.value())
                        }
                    }
                    "method_name" => {
                        if let Lit::Str(s) = &nv.lit {
                            parsed_attrs.method_name = Some(s.value())
                        }
                    }
                    "backend" => {
                        if let Lit::Str(s) = &nv.lit {
                            parsed_attrs.backend = Some(s.value())
                        }
                    }
                    "primary_key" => {
                        if let Lit::Str(s) = &nv.lit {
                            parsed_attrs.primary_key = Some(s.value())
                        }
                    }
                    "child_primary_key" => {
                        if let Lit::Str(s) = &nv.lit {
                            parsed_attrs.child_primary_key = Some(s.value())
                        }
                    }
                    "eager_loading" => {
                        if let Lit::Bool(b) = &nv.lit {
                            parsed_attrs.eager_loading = Some(b.value())
                        }
                    }
                    _ => {
                        return Err(Error::new(
                            Span::call_site(),
                            &format!("Unknown attribute '{}'", ident),
                        ))
                    }
                }
            }
            _ => return Err(Error::new(Span::call_site(), "Unexpected attribute format")),
        }
    }

    if parsed_attrs.relation_type.is_none() {
        return Err(Error::new(
            Span::call_site(),
            "Attribute 'relation_type' is required",
        ));
    }

    if parsed_attrs.backend.is_none() {
        return Err(Error::new(
            Span::call_site(),
            "Attribute 'backend' is required",
        ));
    }

    match parsed_attrs.relation_type.as_deref() {
        Some("one_to_many") | Some("one_to_one") => {
            if parsed_attrs.model.is_none() {
                return Err(Error::new(
                    Span::call_site(),
                    "Attribute 'model' is required for 'one_to_many' and 'one_to_one' relations",
                ));
            }
        }
        Some("many_to_one") => {
            if parsed_attrs.model.is_none() || parsed_attrs.fk.is_none() {
                return Err(Error::new(
                    Span::call_site(),
                    "Attributes 'model' and 'fk' are required for 'many_to_one' relations",
                ));
            }
        }
        Some("many_to_many") => {
            if parsed_attrs.join_table.is_none()
                || parsed_attrs.fk_parent.is_none()
                || parsed_attrs.fk_child.is_none()
            {
                return Err(Error::new(Span::call_site(), "Attributes 'join_table', 'fk_parent', and 'fk_child' are required for 'many_to_many' relations"));
            }
        }
        _ => {
            return Err(Error::new(
                Span::call_site(),
                "Unsupported relation type specified",
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
    fn test_one_to_one_relation_attributes() {
        let attrs = vec![
            NestedMeta::Meta(parse_quote! { relation_type = "one_to_one" }),
            NestedMeta::Meta(parse_quote! { model = "users" }),
            NestedMeta::Meta(parse_quote! { fk = "user_id" }),
            NestedMeta::Meta(parse_quote! { backend = "sqlite" }),
        ];

        let result = parse_attributes(attrs);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.relation_type.unwrap(), "one_to_one");
        assert_eq!(parsed.model.unwrap(), "users");
        assert_eq!(parsed.fk.unwrap(), "user_id");
    }

    #[test]
    fn test_one_to_many_relation_attributes() {
        let attrs = vec![
            NestedMeta::Meta(parse_quote! { relation_type = "one_to_many" }),
            NestedMeta::Meta(parse_quote! { model = "posts" }),
            NestedMeta::Meta(parse_quote! { fk = "user_id" }),
            NestedMeta::Meta(parse_quote! { backend = "sqlite" }),
        ];

        let result = parse_attributes(attrs);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.relation_type.unwrap(), "one_to_many");
        assert_eq!(parsed.model.unwrap(), "posts");
        assert_eq!(parsed.fk.unwrap(), "user_id");
    }

    #[test]
    fn test_many_to_one_relation_attributes() {
        let attrs = vec![
            NestedMeta::Meta(parse_quote! { relation_type = "many_to_one" }),
            NestedMeta::Meta(parse_quote! { model = "users" }),
            NestedMeta::Meta(parse_quote! { fk = "user_id" }),
            NestedMeta::Meta(parse_quote! { backend = "sqlite" }),
        ];

        let result = parse_attributes(attrs);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.relation_type.unwrap(), "many_to_one");
        assert_eq!(parsed.model.unwrap(), "users");
        assert_eq!(parsed.fk.unwrap(), "user_id");
    }

    #[test]
    fn test_many_to_many_relation_attributes() {
        let attrs = vec![
            NestedMeta::Meta(parse_quote! { relation_type = "many_to_many" }),
            NestedMeta::Meta(parse_quote! { model = "users" }),
            NestedMeta::Meta(parse_quote! { join_table = "user_posts" }),
            NestedMeta::Meta(parse_quote! { fk_parent = "user_id" }),
            NestedMeta::Meta(parse_quote! { fk_child = "post_id" }),
            NestedMeta::Meta(parse_quote! { backend = "sqlite" }),
        ];

        let result = parse_attributes(attrs);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.relation_type.unwrap(), "many_to_many");
        assert_eq!(parsed.model.unwrap(), "users");
        assert!(parsed.fk.is_none());
        assert_eq!(parsed.join_table.unwrap(), "user_posts");
        assert_eq!(parsed.fk_parent.unwrap(), "user_id");
        assert_eq!(parsed.fk_child.unwrap(), "post_id");
    }

    #[test]
    fn test_one_to_one_relation_without_fk() {
        let attrs = vec![
            NestedMeta::Meta(parse_quote! { relation_type = "one_to_one" }),
            NestedMeta::Meta(parse_quote! { model = "users" }),
            NestedMeta::Meta(parse_quote! { backend = "sqlite" }),
        ];

        let result = parse_attributes(attrs);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.relation_type.unwrap(), "one_to_one");
        assert_eq!(parsed.model.unwrap(), "users");
        assert!(parsed.fk.is_none());
    }

    #[test]
    fn test_custom_method_name() {
        let attrs = vec![
            NestedMeta::Meta(parse_quote! { relation_type = "one_to_many" }),
            NestedMeta::Meta(parse_quote! { model = "posts" }),
            NestedMeta::Meta(parse_quote! { fk = "user_id" }),
            NestedMeta::Meta(parse_quote! { method_name = "get_all_posts" }),
            NestedMeta::Meta(parse_quote! { backend = "sqlite" }),
        ];

        let result = parse_attributes(attrs);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.method_name.unwrap(), "get_all_posts");
    }
}