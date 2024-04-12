use crate::utils::parser::parse_attributes;
use crate::utils::parser::ParsedAttrs;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::ItemStruct;
use syn::{self, parse_macro_input, AttributeArgs, Ident};

/// Generates the implementation of the `diesel_linker` macro.
///
/// # Arguments
///
/// * `input` - The input token stream of the macro.
/// * `output` - The output token stream of the macro.
///
/// # Returns
///
/// A `TokenStream` of the generated implementation.
///
/// # Example usage:
///
/// ```rust
// /// #[relation(child = "Post", fk = "user_id", relation_type = "one_to_many")]
/// struct User {
///    pub id: i32,
///    pub name: String,
/// }
/// ```
///
/// # Panics
///
/// If the relation attributes are missing, the function will panic.
///

#[derive(Debug)]
pub struct RelationAttributes {
    pub child_model: String,
    pub fk: String,
    pub relation_type: String,
    pub join_table: Option<String>,
    pub fk_parent: Option<String>,
    pub fk_child: Option<String>,
}

// Extracts the relation attributes from the attributes passed to the macro.
fn extract_relation_attrs(parsed_attrs: &ParsedAttrs) -> Result<RelationAttributes, syn::Error> {
    // Supposons que parsed_attrs contient déjà toutes les informations nécessaires
    Ok(RelationAttributes {
        child_model: parsed_attrs
            .child
            .clone()
            .ok_or_else(|| syn::Error::new(Span::call_site(), "child_model is missing"))?,
        fk: parsed_attrs
            .fk
            .clone()
            .ok_or_else(|| syn::Error::new(Span::call_site(), "fk is missing"))?,
        relation_type: parsed_attrs
            .relation_type
            .clone()
            .ok_or_else(|| syn::Error::new(Span::call_site(), "relation_type is missing"))?,
        join_table: parsed_attrs.join_table.clone(),
        fk_parent: parsed_attrs.fk_parent.clone(),
        fk_child: parsed_attrs.fk_child.clone(),
    })
}
pub fn diesel_linker_impl(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(item as ItemStruct);
    let attrs = parse_macro_input!(attrs as AttributeArgs);

    // Utilisation dela fonction parse_attributes pour obtenir un objet ParsedAttrs depuis attrs
    let parsed_attrs = parse_attributes(attrs).expect("Failed to parse attributes");

    // On construit un objet ParsedAttrs qui sera utilisé
    let relation_attrs =
        extract_relation_attrs(&parsed_attrs).expect("Failed to extract relation attributes");

    let struct_name = &item_struct.ident;
    let gen_code = generate_relation_code(
        struct_name,
        &relation_attrs.child_model,
        &relation_attrs.fk,
        &relation_attrs.relation_type,
        relation_attrs.join_table,
        relation_attrs.fk_parent,
        relation_attrs.fk_child,
    );

    TokenStream::from(quote! {
        #item_struct
        #gen_code
    })
}

fn generate_relation_code(
    struct_name: &Ident,
    child_model: &str,
    fk: &str,
    relation_type: &str,
    join_table: Option<String>,
    fk_parent: Option<String>,
    fk_child: Option<String>,
) -> proc_macro2::TokenStream {
    let child_ident = Ident::new(child_model, proc_macro2::Span::call_site());
    let fk_ident = Ident::new(fk, proc_macro2::Span::call_site());
    match relation_type {
        "one_to_many" => {
            // Générer le code pour la relation one_to_many
            quote! {
                impl #struct_name {
                    pub fn children<C>(&self, conn: &C) -> diesel::QueryResult<Vec<#child_ident>>
                    where C: diesel::Connection,{
                        use crate::schema::#child_ident::dsl::*;
                        use diesel::prelude::*;

                        #child_ident.filter(#fk_ident.eq(self.id)).load::<#child_ident>(conn)
                    }

                    pub fn add_child<C>(&self, conn: &C, new_child: &#child_ident) -> Result<usize, diesel::result::Error>
                    where C: diesel::Connection, {
                        use diesel::RunQueryDsl;
                        diesel::insert_into(#child_ident::table).values(new_child).execute(conn)
                    }

                    // Supprimer un enfant spécifique
                    pub fn remove_child<C>(&self, conn: &C, child_id: i32) -> Result<usize, diesel::result::Error>
                    where C: diesel::Connection, {
                        use diesel::RunQueryDsl;
                        diesel::delete(#child_ident.filter(id.eq(child_id).and(#fk_ident.eq(self.id)))).execute(conn)
                    }
                }
            }
        }
        "many_to_one" => {
            // Identifiant de l'entité parent et de la clé étrangère dans l'entité enfant.
            let parent_model = "ParentModel"; // Replace "ParentModel" with the actual value of parent_model
            let parent_ident = Ident::new(&parent_model, proc_macro2::Span::call_site());
            let fk_ident = Ident::new(fk, proc_macro2::Span::call_site());

            quote! {
                impl #struct_name {
                    // Récupère l'instance parente associée à cette instance enfant.
                    pub fn get_parent<C>(&self, conn: &C) -> diesel::QueryResult<#parent_ident>
                    where C: diesel::Connection, {
                        use crate::schema::#parent_ident::dsl::*;
                        use diesel::prelude::*;

                        #parent_ident.filter(id.eq(self.#fk_ident)).first::<#parent_ident>(conn)
                    }

                    // Optionnellement, si vous voulez aussi définir la relation dans l'autre sens :
                    impl #parent_ident {
                        // Récupère toutes les instances enfants liées à cette instance parent.
                        pub fn get_children<C>(&self, conn: &C) -> diesel::QueryResult<Vec<#struct_name>>
                        where C: diesel::Connection, {
                            use crate::schema::#struct_name::dsl::*;
                            use diesel::prelude::*;

                            #struct_name.filter(#fk_ident.eq(self.id)).load::<#struct_name>(conn)
                        }
                    }
                }
            }
        }
        "one_to_one" => {
            let child_ident = Ident::new(&child_model, proc_macro2::Span::call_site());
            let fk_ident = Ident::new(fk, proc_macro2::Span::call_site());

            quote! {
                impl #struct_name {
                    // Obtient l'entité liée depuis l'entité courante.
                    pub fn get_related_entity<C>(&self, conn: &C) -> diesel::QueryResult<Option<#child_ident>>
                    where C: diesel::Connection, {
                        use crate::schema::#child_ident::dsl::*;
                        use diesel::prelude::*;

                        #child_ident.filter(#fk_ident.eq(self.id)).first::<#child_ident>(conn).optional()
                    }

                    // Définit ou met à jour l'entité liée.
                    pub fn set_related_entity<C>(&self, conn: &C, entity: &#child_ident) -> diesel::QueryResult<#child_ident>
                    where C: diesel::Connection, {
                        use diesel::RunQueryDsl;
                        use crate::schema::#child_ident::dsl::*;

                        diesel::insert_into(#child_ident::table)
                            .values(entity)
                            .on_conflict(#fk_ident)
                            .do_update()
                            .set(entity)
                            .get_result::<#child_ident>(conn)
                    }
                }
            }
        }
        "many_to_many" => {
            if let (Some(join_table), Some(fk_parent), Some(fk_child)) =
                (join_table, fk_parent, fk_child)
            {
                let join_table_ident = Ident::new(&join_table, proc_macro2::Span::call_site());
                let parent_fk_ident = Ident::new(&fk_parent, proc_macro2::Span::call_site());
                let child_fk_ident = Ident::new(&fk_child, proc_macro2::Span::call_site());

                quote! {
                    impl #struct_name {
                        pub fn related_entities<C>(&self, conn: &C) -> diesel::QueryResult<Vec<#child_ident>>
                        where
                            C: diesel::Connection,
                        {
                            use diesel::prelude::*;
                            use crate::schema::#join_table_ident::dsl as join_dsl;
                            use crate::schema::#child_ident::dsl::*;

                            let related_ids = join_dsl::#join_table_ident
                                .filter(join_dsl::#parent_fk_ident.eq(self.id))
                                .select(join_dsl::#child_fk_ident)
                                .load::<i32>(conn)?;

                            #child_ident.filter(id.eq_any(related_ids)).load::<#child_ident>(conn)
                        }
                    }
                }
            } else {
                quote! {
                    compile_error!("join_table, fk_parent, and fk_child attributes are required for many_to_many relations");
                }
            }
        }
        _ => panic!("Unsupported relation type: {}", relation_type),
    }
}
