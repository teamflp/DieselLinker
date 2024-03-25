// Path: src/utils/parser.rs

use syn::{AttributeArgs, Error, Lit, MetaNameValue, NestedMeta, Result};

#[derive(Debug)]
pub struct ParsedAttrs {
    pub extra_field_name: String,
}

pub fn parse_attributes(attrs: AttributeArgs) -> Result<ParsedAttrs> {
    let extra_field_name = attrs
        .iter()
        .find_map(|attr| {
            if let NestedMeta::Meta(syn::Meta::NameValue(MetaNameValue {
                path,
                lit: Lit::Str(lit_str),
                ..
            })) = attr
            {
                if path.is_ident("extra_field_name") {
                    return Some(lit_str.value());
                }
            }
            None
        })
        .ok_or_else(|| Error::new_spanned(&attrs[0], "Attribut 'extra_field_name' attendu"))?;

    Ok(ParsedAttrs { extra_field_name })
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Meta};

    #[test]
    fn test_parse_attributes() {
        let attrs1 = vec![NestedMeta::Meta(Meta::NameValue(MetaNameValue {
            path: parse_quote! { extra_field_name },
            eq_token: Default::default(),
            lit: syn::Lit::Str(syn::LitStr::new(
                "test_field",
                proc_macro2::Span::call_site(),
            )),
        }))];

        let result1 = parse_attributes(attrs1);
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap().extra_field_name, "test_field");

        let attrs2 = vec![NestedMeta::Meta(parse_quote! { some_attribute })];

        let result2 = parse_attributes(attrs2);
        assert!(result2.is_err());
        assert_eq!(
            result2.unwrap_err().to_string(),
            "Attribut 'extra_field_name' attendu"
        );
    }
}
