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
    pub backend: String,
    pub primary_key: Option<String>,
    pub child_primary_key: Option<String>,
    pub eager_loading: bool,
}

// Extracts the relation attributes from the attributes passed to the macro.
fn extract_relation_attrs(parsed_attrs: &ParsedAttrs) -> Result<RelationAttributes, syn::Error> {
    let relation_type = parsed_attrs
        .relation_type
        .clone()
        .ok_or_else(|| syn::Error::new(Span::call_site(), "relation_type is missing"))?;

    let fk = if relation_type == "many_to_one" {
        parsed_attrs
            .fk
            .clone()
            .ok_or_else(|| syn::Error::new(Span::call_site(), "fk is required for many_to_one relation"))?
    } else {
        parsed_attrs.fk.clone().unwrap_or_default()
    };

    Ok(RelationAttributes {
        model: parsed_attrs
            .model
            .clone()
            .ok_or_else(|| syn::Error::new(Span::call_site(), "model is missing"))?,
        fk,
        relation_type,
        join_table: parsed_attrs.join_table.clone(),
        fk_parent: parsed_attrs.fk_parent.clone(),
        fk_child: parsed_attrs.fk_child.clone(),
        method_name: parsed_attrs.method_name.clone(),
        backend: parsed_attrs.backend.clone().unwrap(),
        primary_key: parsed_attrs.primary_key.clone(),
        child_primary_key: parsed_attrs.child_primary_key.clone(),
        eager_loading: parsed_attrs.eager_loading.unwrap_or(false),
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
        &relation_attrs.backend,
        &relation_attrs.primary_key,
        &relation_attrs.child_primary_key,
        relation_attrs.eager_loading,
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
    backend: &str,
    primary_key: &Option<String>,
    child_primary_key: &Option<String>,
    eager_loading: bool,
) -> proc_macro2::TokenStream {
    let model_ident = Ident::new(model, proc_macro2::Span::call_site());
    let primary_key_ident = Ident::new(primary_key.as_deref().unwrap_or("id"), proc_macro2::Span::call_site());
    let child_primary_key_ident = Ident::new(child_primary_key.as_deref().unwrap_or(primary_key.as_deref().unwrap_or("id")), proc_macro2::Span::call_site());

    let conn_type = match backend {
        "postgres" => quote! { diesel::pg::PgConnection },
        "sqlite" => quote! { diesel::sqlite::SqliteConnection },
        "mysql" => quote! { diesel::mysql::MysqlConnection },
        _ => return quote! { compile_error!("Unsupported backend. Supported backends are 'postgres', 'sqlite', and 'mysql'."); }.into(),
    };

    match relation_type {
        "one_to_many" => {
            let get_method_name = method_name.as_ref().map(|s| Ident::new(s, proc_macro2::Span::call_site())).unwrap_or_else(|| Ident::new(&format!("get_{}", model.to_lowercase().to_plural()), proc_macro2::Span::call_site()));

            let lazy_load_code = quote! {
                impl #struct_name {
                    pub fn #get_method_name(&self, conn: &mut #conn_type) -> diesel::QueryResult<Vec<#model_ident>>
                    {
                        use diesel::prelude::*;
                        #model_ident::belonging_to(self).load::<#model_ident>(conn)
                    }
                }
            };

            let eager_load_code = if eager_loading {
                let load_method_name = Ident::new(&format!("load_with_{}", model.to_lowercase().to_plural()), proc_macro2::Span::call_site());
                quote! {
                    impl #struct_name {
                        pub fn #load_method_name(parents: Vec<#struct_name>, conn: &mut #conn_type) -> diesel::QueryResult<Vec<(#struct_name, Vec<#model_ident>)>> {
                            use diesel::prelude::*;
                            let children = #model_ident::belonging_to(&parents).load::<#model_ident>(conn)?;
                            let grouped_children = children.grouped_by(&parents);
                            let result = parents.into_iter().zip(grouped_children).collect::<Vec<_>>();
                            Ok(result)
                        }
                    }
                }
            } else {
                quote! {}
            };

            quote! {
                #lazy_load_code
                #eager_load_code
            }
        }
        "many_to_one" => {
            let method_name = method_name.as_ref().map(|s| Ident::new(s, proc_macro2::Span::call_site())).unwrap_or_else(|| Ident::new(&format!("get_{}", model.to_lowercase()), proc_macro2::Span::call_site()));
            let fk_ident = Ident::new(fk, proc_macro2::Span::call_site());
            let table_name = Ident::new(&model.to_plural().to_snake_case(), proc_macro2::Span::call_site());

            let lazy_load_code = quote! {
                impl #struct_name {
                    pub fn #method_name(&self, conn: &mut #conn_type) -> diesel::QueryResult<#model_ident>
                    {
                        use diesel::prelude::*;
                        crate::schema::#table_name::table.find(self.#fk_ident).get_result::<#model_ident>(conn)
                    }
                }
            };

            let eager_load_code = if eager_loading {
                let load_method_name = Ident::new(&format!("load_with_{}", model.to_lowercase()), proc_macro2::Span::call_site());
                let parent_primary_key_ident = Ident::new("id", proc_macro2::Span::call_site());

                quote! {
                    impl #struct_name {
                        /// Eager loads the parent model. The parent model must derive `Clone` and have an `id: i32` field.
                        pub fn #load_method_name(children: Vec<#struct_name>, conn: &mut #conn_type) -> diesel::QueryResult<Vec<(#struct_name, #model_ident)>> {
                            use diesel::prelude::*;
                            use std::collections::{HashMap, HashSet};

                            let parent_ids: HashSet<_> = children.iter().map(|c| c.#fk_ident).collect();
                            let parents = crate::schema::#table_name::table
                                .filter(crate::schema::#table_name::#parent_primary_key_ident.eq_any(parent_ids.into_iter().collect::<Vec<_>>()))
                                .load::<#model_ident>(conn)?;

                            let parent_map: HashMap<_, _> = parents.into_iter().map(|p| (p.id, p)).collect();

                            let result = children.into_iter().filter_map(|c| {
                                parent_map.get(&c.#fk_ident).map(|p| (c, p.clone()))
                            }).collect();

                            Ok(result)
                        }
                    }
                }
            } else {
                quote!{}
            };

            quote! {
                #lazy_load_code
                #eager_load_code
            }
        }
        "one_to_one" => {
            let method_name = method_name.as_ref().map(|s| Ident::new(s, proc_macro2::Span::call_site())).unwrap_or_else(|| Ident::new(&format!("get_{}", model.to_lowercase()), proc_macro2::Span::call_site()));

            let lazy_load_code = quote! {
                impl #struct_name {
                    pub fn #method_name(&self, conn: &mut #conn_type) -> diesel::QueryResult<#model_ident>
                    {
                        use diesel::prelude::*;
                        #model_ident::belonging_to(self).first::<#model_ident>(conn)
                    }
                }
            };

            let eager_load_code = if eager_loading {
                let load_method_name = Ident::new(&format!("load_with_{}", model.to_lowercase()), proc_macro2::Span::call_site());
                quote! {
                    impl #struct_name {
                        pub fn #load_method_name(parents: Vec<#struct_name>, conn: &mut #conn_type) -> diesel::QueryResult<Vec<(#struct_name, Vec<#model_ident>)>> {
                            use diesel::prelude::*;
                            let children = #model_ident::belonging_to(&parents).load::<#model_ident>(conn)?;
                            let grouped_children = children.grouped_by(&parents);
                            let result = parents.into_iter().zip(grouped_children).collect::<Vec<_>>();
                            Ok(result)
                        }
                    }
                }
            } else {
                quote! {}
            };

            quote! {
                #lazy_load_code
                #eager_load_code
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
                let model_table_name = Ident::new(&model.to_plural().to_snake_case(), proc_macro2::Span::call_site());

                let lazy_load_code = quote! {
                    impl #struct_name {
                        pub fn #get_method_name(&self, conn: &mut #conn_type) -> diesel::QueryResult<Vec<#model_ident>>
                        {
                            use diesel::prelude::*;
                            let related_ids = crate::schema::#join_table_ident::table
                                .filter(crate::schema::#join_table_ident::#parent_fk_ident.eq(self.#primary_key_ident))
                                .select(crate::schema::#join_table_ident::#child_fk_ident)
                                .load::<i32>(conn)?;
                            crate::schema::#model_table_name::table.filter(crate::schema::#model_table_name::#child_primary_key_ident.eq_any(related_ids)).load::<#model_ident>(conn)
                        }

                        pub fn #add_method_name(&self, conn: &mut #conn_type, child: &#model_ident) -> diesel::QueryResult<usize>
                        {
                            use diesel::prelude::*;
                            diesel::insert_into(crate::schema::#join_table_ident::table)
                                .values((crate::schema::#join_table_ident::#parent_fk_ident.eq(self.#primary_key_ident), crate::schema::#join_table_ident::#child_fk_ident.eq(child.#child_primary_key_ident)))
                                .execute(conn)
                        }

                        pub fn #remove_method_name(&self, conn: &mut #conn_type, child: &#model_ident) -> diesel::QueryResult<usize>
                        {
                            use diesel::prelude::*;
                            diesel::delete(crate::schema::#join_table_ident::table
                                .filter(crate::schema::#join_table_ident::#parent_fk_ident.eq(self.#primary_key_ident))
                                .filter(crate::schema::#join_table_ident::#child_fk_ident.eq(child.#child_primary_key_ident)))
                                .execute(conn)
                        }
                    }
                };

                let eager_load_code = if eager_loading {
                    let load_method_name = Ident::new(&format!("load_with_{}", model.to_lowercase().to_plural()), proc_macro2::Span::call_site());
                    quote! {
                        impl #struct_name {
                            pub fn #load_method_name(parents: Vec<#struct_name>, conn: &mut #conn_type) -> diesel::QueryResult<Vec<(#struct_name, Vec<#model_ident>)>> {
                                use diesel::prelude::*;
                                use std::collections::HashMap;

                                let parent_ids: Vec<_> = parents.iter().map(|p| p.#primary_key_ident).collect();

                                let children_with_fk = crate::schema::#model_table_name::table
                                    .inner_join(crate::schema::#join_table_ident::table.on(crate::schema::#model_table_name::#child_primary_key_ident.eq(crate::schema::#join_table_ident::#child_fk_ident)))
                                    .filter(crate::schema::#join_table_ident::#parent_fk_ident.eq_any(parent_ids))
                                    .select((crate::schema::#model_table_name::all_columns, crate::schema::#join_table_ident::#parent_fk_ident))
                                    .load::<(#model_ident, i32)>(conn)?;

                                let mut grouped_children: HashMap<i32, Vec<#model_ident>> = HashMap::new();
                                for (child, parent_id) in children_with_fk {
                                    grouped_children.entry(parent_id).or_default().push(child);
                                }

                                let result = parents.into_iter().map(|p| {
                                    let children = grouped_children.remove(&p.#primary_key_ident).unwrap_or_default();
                                    (p, children)
                                }).collect();

                                Ok(result)
                            }
                        }
                    }
                } else {
                    quote!{}
                };

                quote! {
                    #lazy_load_code
                    #eager_load_code
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