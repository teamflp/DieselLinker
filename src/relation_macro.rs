use crate::utils::parser::{parse_attributes, ParsedAttrs};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemStruct};

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
/// ```ignore
/// use diesel_linker::DieselLinker;
///
/// #[derive(DieselLinker)]
/// #[relation(type = "one-to-many", table1 = "users", table2 = "posts", column1 = "id", column2 = "user_id")]
/// struct User {
///     // Définitions des champs de la structure `User`...
/// }
/// ```
///
/// # Arguments
///
/// * `attr` - Les attributs passés à la macro.
/// * `item` - La définition de la structure à étendre.
///
/// # Retourne
///
/// Un `TokenStream` contenant le code généré.
pub fn diesel_linker_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
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

    let expanded = quote! {
        struct #base_struct_name {
            #( #field_tokens, )*
            // Ajoute un champ supplémentaire avec le nom généré dynamiquement
            #extra_field_name: String,
        }
    };

    expanded.into()
}
