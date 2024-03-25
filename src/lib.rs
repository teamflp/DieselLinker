mod relation_macro;
mod utils;

use proc_macro::TokenStream;

// Définition de l'attribut de macro `relation`.
#[proc_macro_attribute]
pub fn relation(attr: TokenStream, item: TokenStream) -> TokenStream {
    relation_macro::relation_impl(attr, item)
}
