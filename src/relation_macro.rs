use crate::utils::parser::parse_attributes;
use crate::utils::parser::ParsedAttrs;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::ItemStruct;
use syn::{self, parse_macro_input, AttributeArgs, Ident};
use inflector::Inflector;

#[derive(Debug)]
pub struct RelationAttributes {
    pub model: String,
    pub fk: String,
    pub relation_type: String,
    pub join_table: Option<String>,
    pub fk_parent: Option<String>,
    pub fk_child: Option<String>,
    pub method_name: Option<String>,
}

// Extracts the relation attributes from the attributes passed to the macro.
fn extract_relation_attrs(parsed_attrs: &ParsedAttrs) -> Result<RelationAttributes, syn::Error> {
    // Supposons que parsed_attrs contient déjà toutes les informations nécessaires
    Ok(RelationAttributes {
        model: parsed_attrs
            .model
            .clone()
            .ok_or_else(|| syn::Error::new(Span::call_site(), "model is missing"))?,
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
        method_name: parsed_attrs.method_name.clone(),
    })
}
pub fn diesel_linker_impl(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(item as ItemStruct);
    let attrs = parse_macro_input!(attrs as AttributeArgs);

    let parsed_attrs = match parse_attributes(attrs) {
        Ok(attrs) => attrs,
        Err(e) => return e.to_compile_error().into(),
    };

    let relation_attrs = match extract_relation_attrs(&parsed_attrs) {
        Ok(attrs) => attrs,
        Err(e) => return e.to_compile_error().into(),
    };

    let struct_name = &item_struct.ident;
    let gen_code = generate_relation_code(
        struct_name,
        &relation_attrs.model,
        &relation_attrs.fk,
        &relation_attrs.relation_type,
        relation_attrs.join_table,
        relation_attrs.fk_parent,
        relation_attrs.fk_child,
        &relation_attrs.method_name,
    );

    TokenStream::from(quote! {
        #item_struct
        #gen_code
    })
}

fn generate_relation_code(
    struct_name: &Ident,
    model: &str,
    fk: &String,
    relation_type: &str,
    join_table: Option<String>,
    fk_parent: Option<String>,
    fk_child: Option<String>,
    method_name: &Option<String>,
) -> proc_macro2::TokenStream {
    let model_ident = Ident::new(model, proc_macro2::Span::call_site());

    match relation_type {
        "one_to_many" => {
            let get_method_name = method_name.as_ref().map(|s| Ident::new(s, proc_macro2::Span::call_site())).unwrap_or_else(|| Ident::new(&format!("get_{}", model.to_lowercase().to_plural()), proc_macro2::Span::call_site()));
            let add_method_name = Ident::new(&format!("add_{}", model.to_lowercase().to_singular()), proc_macro2::Span::call_site());
            let remove_method_name = Ident::new(&format!("remove_{}", model.to_lowercase().to_singular()), proc_macro2::Span::call_site());
            let fk_ident = Ident::new(fk, proc_macro2::Span::call_site());

            quote! {
                impl #struct_name {
                    pub fn #get_method_name<C>(&self, conn: &mut C) -> diesel::QueryResult<Vec<#model_ident>>
                    where C: diesel::Connection
                    {
                        use diesel::prelude::*;
                        #model_ident::belonging_to(self).load::<#model_ident>(conn)
                    }

                    pub fn #add_method_name<C>(&self, conn: &mut C, child: &#model_ident) -> diesel::QueryResult<usize>
                    where C: diesel::Connection
                    {
                        use diesel::prelude::*;
                        diesel::update(child).set(crate::schema::#fk_ident.eq(self.id)).execute(conn)
                    }

                    pub fn #remove_method_name<C>(&self, conn: &mut C, child: &#model_ident) -> diesel::QueryResult<usize>
                    where C: diesel::Connection
                    {
                        use diesel::prelude::*;
                        diesel::update(child).set(crate::schema::#fk_ident.eq(None::<i32>)).execute(conn)
                    }
                }
            }
        }
        "many_to_one" => {
            let method_name = method_name.as_ref().map(|s| Ident::new(s, proc_macro2::Span::call_site())).unwrap_or_else(|| Ident::new(&format!("get_{}", model.to_lowercase()), proc_macro2::Span::call_site()));
            let fk_ident = Ident::new(fk, proc_macro2::Span::call_site());
            quote! {
                impl #struct_name {
                    pub fn #method_name<C>(&self, conn: &mut C) -> diesel::QueryResult<#model_ident>
                    where C: diesel::Connection,
                    {
                        use diesel::prelude::*;
                        crate::schema::#model_ident::table.find(self.#fk_ident).get_result::<#model_ident>(conn)
                    }
                }
            }
        }
        "one_to_one" => {
            let method_name = method_name.as_ref().map(|s| Ident::new(s, proc_macro2::Span::call_site())).unwrap_or_else(|| Ident::new(&format!("get_{}", model.to_lowercase()), proc_macro2::Span::call_site()));
            quote! {
                impl #struct_name {
                    pub fn #method_name<C>(&self, conn: &mut C) -> diesel::QueryResult<#model_ident>
                    where C: diesel::Connection
                    {
                        use diesel::prelude::*;
                        #model_ident::belonging_to(self).first::<#model_ident>(conn)
                    }
                }
            }
        }
        "many_to_many" => {
            if let (Some(join_table), Some(fk_parent), Some(fk_child)) = (join_table, fk_parent, fk_child) {
                let join_table_ident = Ident::new(&join_table, proc_macro2::Span::call_site());
                let parent_fk_ident = Ident::new(&fk_parent, proc_macro2::Span::call_site());
                let child_fk_ident = Ident::new(&fk_child, proc_macro2::Span::call_site());
                let get_method_name = method_name.as_ref().map(|s| Ident::new(s, proc_macro2::Span::call_site())).unwrap_or_else(|| Ident::new(&format!("get_{}", model.to_lowercase().to_plural()), proc_macro2::Span::call_site()));
                let add_method_name = Ident::new(&format!("add_{}", model.to_lowercase().to_singular()), proc_macro2::Span::call_site());
                let remove_method_name = Ident::new(&format!("remove_{}", model.to_lowercase().to_singular()), proc_macro2::Span::call_site());

                quote! {
                    impl #struct_name {
                        pub fn #get_method_name<C>(&self, conn: &mut C) -> diesel::QueryResult<Vec<#model_ident>>
                        where
                            C: diesel::Connection,
                        {
                            use diesel::prelude::*;
                            let related_ids = crate::schema::#join_table_ident::table
                                .filter(crate::schema::#parent_fk_ident.eq(self.id))
                                .select(crate::schema::#child_fk_ident)
                                .load::<i32>(conn)?;
                            crate::schema::#model_ident::table.filter(crate::schema::id.eq_any(related_ids)).load::<#model_ident>(conn)
                        }

                        pub fn #add_method_name<C>(&self, conn: &mut C, child: &#model_ident) -> diesel::QueryResult<usize>
                        where
                            C: diesel::Connection,
                        {
                            use diesel::prelude::*;
                            diesel::insert_into(crate::schema::#join_table_ident::table)
                                .values((crate::schema::#parent_fk_ident.eq(self.id), crate::schema::#child_fk_ident.eq(child.id)))
                                .execute(conn)
                        }

                        pub fn #remove_method_name<C>(&self, conn: &mut C, child: &#model_ident) -> diesel::QueryResult<usize>
                        where
                            C: diesel::Connection,
                        {
                            use diesel::prelude::*;
                            diesel::delete(crate::schema::#join_table_ident::table
                                .filter(crate::schema::#parent_fk_ident.eq(self.id))
                                .filter(crate::schema::#child_fk_ident.eq(child.id)))
                                .execute(conn)
                        }
                    }
                }
            } else {
                quote! {
                    compile_error!("'join_table', 'fk_parent', and 'fk_child' attributes are required for 'many_to_many' relations");
                }
            }
        }
        _ => quote! {
            compile_error!("Unsupported relation type");
        },
    }
}
