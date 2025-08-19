// Path: src/utils/parser.rs

use proc_macro2::Span;
use syn::{AttributeArgs, Error, Lit, Meta, NestedMeta, Result};

#[derive(Debug, Default)]
pub struct ParsedAttrs {
    pub relation_type: Option<String>,
    pub model: Option<String>,
    pub fk: Option<String>,         // Used for one_to_many et one_to_one
    pub join_table: Option<String>, // Used for many_to_many
    pub fk_parent: Option<String>,  // Foreign key for the parent in the join table for many_to_many
    pub fk_child: Option<String>,   // Foreign key for the child in the join table for many_to_many
    pub method_name: Option<String>,
}

// Parses the attributes passed to the `relation` attribute macro.
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

    match parsed_attrs.relation_type.as_deref() {
        Some("one_to_many") | Some("one_to_one") => {
            if parsed_attrs.model.is_none() || parsed_attrs.fk.is_none() {
                return Err(Error::new(Span::call_site(), "Attributes 'model' and 'fk' are required for 'one_to_many' and 'one_to_one' relations"));
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

// The test module is only compiled when running tests.
// The `#[cfg(test)]` attribute is used to conditionally compile the module only when running tests.
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
            NestedMeta::Meta(parse_quote! { fk = "post_id" }),
            NestedMeta::Meta(parse_quote! { join_table = "user_posts" }),
            NestedMeta::Meta(parse_quote! { fk_parent = "user_id" }),
            NestedMeta::Meta(parse_quote! { fk_child = "post_id" }),
        ];

        let result = parse_attributes(attrs);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.relation_type.unwrap(), "many_to_many");
        assert_eq!(parsed.model.unwrap(), "users");
        assert_eq!(parsed.fk.unwrap(), "post_id");
        assert_eq!(parsed.join_table.unwrap(), "user_posts");
        assert_eq!(parsed.fk_parent.unwrap(), "user_id");
        assert_eq!(parsed.fk_child.unwrap(), "post_id");
    }

    #[test]
    fn test_custom_method_name() {
        let attrs = vec![
            NestedMeta::Meta(parse_quote! { relation_type = "one_to_many" }),
            NestedMeta::Meta(parse_quote! { model = "posts" }),
            NestedMeta::Meta(parse_quote! { fk = "user_id" }),
            NestedMeta::Meta(parse_quote! { method_name = "get_all_posts" }),
        ];

        let result = parse_attributes(attrs);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.method_name.unwrap(), "get_all_posts");
    }
}