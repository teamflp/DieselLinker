use crate::utils::parser::{parse_attributes, ParsedAttrs};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemStruct};

/// Crée une nouvelle instance de la structure donnée, avec un champ supplémentaire pour stocker des données supplémentaires.
///
/// # Arguments
///
/// * attr - Les attributs passés à la macro.
/// * item - La définition de la structure à étendre.
///
/// # Retourne
///
/// Un TokenStream contenant le code généré.

pub fn relation_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as syn::AttributeArgs);
    let parsed_attrs: ParsedAttrs = match parse_attributes(attrs) {
        Ok(attrs) => attrs,
        Err(e) => return e.to_compile_error().into(),
    };

    let item = parse_macro_input!(item as ItemStruct);
    let struct_name = &item.ident;
    let base_struct_name = format_ident!("{}_base", struct_name);
    let extra_field_name = format_ident!("{}_extra", &parsed_attrs.extra_field_name);

    let field_tokens = item.fields.iter().map(|f| {
        let name = &<Option<syn::Ident> as Clone>::clone(&f.ident).expect("Champ nommé attendu");
        let ty = &f.ty;
        quote! { #name: #ty }
    });

    // Utilisation correcte des tokens générés dans `quote!`
    let expanded = quote! {
        struct #base_struct_name {
            #( #field_tokens, )*
            // Ajoute un champ supplémentaire avec le nom généré dynamiquement
            #extra_field_name: String,
        }
    };

    expanded.into()
}
