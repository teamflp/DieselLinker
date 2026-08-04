use crate::utils::parser::parse_attributes;
use crate::utils::parser::ParsedAttrs;
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::ItemStruct;
use syn::Token;
use syn::{self, parse_macro_input, Ident, Meta};

#[derive(Debug)]
pub struct RelationAttributes {
    pub model: String,
    pub fk: String,
    pub relation_type: String,
    pub parent_primary_key: Option<String>,
    pub join_table: Option<String>,
    pub fk_parent: Option<String>,
    pub fk_child: Option<String>,
    pub method_name: Option<String>,
    pub backend: String,
    pub primary_key: Option<String>,
    pub child_primary_key: Option<String>,
    pub eager_loading: bool,
    pub async_: bool,
    pub error_type: Option<String>,
    pub schema_path: Option<String>,
    pub join_key_type: Option<String>,
}

// Extracts the relation attributes from the attributes passed to the macro.
fn extract_relation_attrs(parsed_attrs: &ParsedAttrs) -> Result<RelationAttributes, syn::Error> {
    // The parser should have already validated the presence of required attributes.
    // Here, we just unwrap them.
    let relation_type = parsed_attrs.relation_type.as_ref().unwrap().value.clone();

    let fk = if relation_type == "many_to_one" {
        parsed_attrs.fk.as_ref().unwrap().value.clone()
    } else {
        parsed_attrs
            .fk
            .as_ref()
            .map_or_else(String::new, |a| a.value.clone())
    };

    Ok(RelationAttributes {
        model: parsed_attrs.model.as_ref().unwrap().value.clone(),
        fk,
        relation_type,
        parent_primary_key: parsed_attrs
            .parent_primary_key
            .as_ref()
            .map(|a| a.value.clone()),
        join_table: parsed_attrs.join_table.as_ref().map(|a| a.value.clone()),
        fk_parent: parsed_attrs.fk_parent.as_ref().map(|a| a.value.clone()),
        fk_child: parsed_attrs.fk_child.as_ref().map(|a| a.value.clone()),
        method_name: parsed_attrs.method_name.as_ref().map(|a| a.value.clone()),
        backend: parsed_attrs.backend.as_ref().unwrap().value.clone(),
        primary_key: parsed_attrs.primary_key.as_ref().map(|a| a.value.clone()),
        child_primary_key: parsed_attrs
            .child_primary_key
            .as_ref()
            .map(|a| a.value.clone()),
        eager_loading: parsed_attrs.eager_loading.as_ref().is_some_and(|a| a.value),
        async_: parsed_attrs.async_.as_ref().is_some_and(|a| a.value),
        error_type: parsed_attrs.error_type.as_ref().map(|a| a.value.clone()),
        schema_path: parsed_attrs.schema_path.as_ref().map(|a| a.value.clone()),
        join_key_type: parsed_attrs.join_key_type.as_ref().map(|a| a.value.clone()),
    })
}
pub fn diesel_linker_impl(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(item as ItemStruct);
    let attrs = match Punctuated::<Meta, Token![,]>::parse_terminated.parse(attrs) {
        Ok(attrs) => attrs,
        Err(e) => return e.to_compile_error().into(),
    };

    let parsed_attrs = match parse_attributes(attrs.into_iter().collect()) {
        Ok(attrs) => attrs,
        Err(e) => return e.to_compile_error().into(),
    };

    let relation_attrs = match extract_relation_attrs(&parsed_attrs) {
        Ok(attrs) => attrs,
        Err(e) => return e.to_compile_error().into(),
    };

    let struct_name = &item_struct.ident;
    let gen_code = match generate_relation_code(struct_name, &relation_attrs) {
        Ok(code) => code,
        Err(e) => return e.to_compile_error().into(),
    };

    TokenStream::from(quote! {
        #item_struct
        #gen_code
    })
}

// Parses a user-supplied string (e.g. from `model = "..."` or `fk = "..."`) into an
// `Ident`, producing a normal compiler error instead of panicking when the string
// isn't a valid Rust identifier.
fn new_ident(s: &str) -> Result<Ident, syn::Error> {
    syn::parse_str::<Ident>(s).map_err(|_| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("`{}` is not a valid Rust identifier", s),
        )
    })
}

fn generate_relation_code(
    struct_name: &Ident,
    attrs: &RelationAttributes,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let RelationAttributes {
        model,
        fk,
        relation_type,
        parent_primary_key,
        join_table,
        fk_parent,
        fk_child,
        method_name,
        backend,
        primary_key,
        child_primary_key,
        eager_loading,
        async_,
        error_type,
        schema_path,
        join_key_type,
    } = attrs;
    let eager_loading = *eager_loading;
    let async_ = *async_;
    let model = model.as_str();
    let fk = fk.as_str();
    let relation_type = relation_type.as_str();
    let backend = backend.as_str();

    let model_ident = new_ident(model)?;
    let primary_key_ident = new_ident(primary_key.as_deref().unwrap_or("id"))?;
    let child_primary_key_ident = new_ident(
        child_primary_key
            .as_deref()
            .unwrap_or(primary_key.as_deref().unwrap_or("id")),
    )?;

    let schema_path = match schema_path {
        Some(s) => {
            let path: syn::Path = syn::parse_str(s).map_err(|_| {
                syn::Error::new(
                    proc_macro2::Span::call_site(),
                    format!("`schema_path` (\"{}\") is not a valid Rust path", s),
                )
            })?;
            quote! { #path }
        }
        None => quote! { crate::schema },
    };

    // Rust type of the join-table FK/PK columns used by `many_to_many` queries.
    // Defaults to `i32` to match the common auto-increment primary key case.
    let join_key_type = match join_key_type {
        Some(s) => {
            let ty: syn::Type = syn::parse_str(s).map_err(|_| {
                syn::Error::new(
                    proc_macro2::Span::call_site(),
                    format!("`join_key_type` (\"{}\") is not a valid Rust type", s),
                )
            })?;
            quote! { #ty }
        }
        None => quote! { i32 },
    };

    let error_type_ident = match error_type {
        Some(s) => {
            let ty: syn::Type = syn::parse_str(s).map_err(|_| {
                syn::Error::new(
                    proc_macro2::Span::call_site(),
                    format!("`error_type` (\"{}\") is not a valid Rust type", s),
                )
            })?;
            quote! { #ty }
        }
        None => quote! { diesel::result::Error },
    };

    let (conn_type, use_diesel_async) = if async_ {
        (
            match backend {
                "postgres" => quote! { diesel_async::AsyncPgConnection },
                "sqlite" => quote! { diesel_async::sync_connection_wrapper::SyncConnectionWrapper<diesel::sqlite::SqliteConnection> },
                "mysql" => quote! { diesel_async::AsyncMysqlConnection },
                _ => return Err(syn::Error::new(proc_macro2::Span::call_site(), "Unsupported backend for async. Supported backends are 'postgres', 'sqlite', and 'mysql'.")),
            },
            quote! { use diesel_async::RunQueryDsl; }
        )
    } else {
        (
            match backend {
                "postgres" => quote! { diesel::pg::PgConnection },
                "sqlite" => quote! { diesel::sqlite::SqliteConnection },
                "mysql" => quote! { diesel::mysql::MysqlConnection },
                _ => return Err(syn::Error::new(proc_macro2::Span::call_site(), "Unsupported backend. Supported backends are 'postgres', 'sqlite', and 'mysql'.")),
            },
            quote! {}
        )
    };

    let async_trait = if async_ {
        quote! { async }
    } else {
        quote! {}
    };
    let await_kw = if async_ {
        quote! { .await }
    } else {
        quote! {}
    };

    Ok(match relation_type {
        "one_to_many" => {
            let get_method_name = match method_name {
                Some(s) => new_ident(s)?,
                None => new_ident(&format!("get_{}", model.to_lowercase().to_plural()))?,
            };

            let lazy_load_code = quote! {
                impl #struct_name {
                    pub #async_trait fn #get_method_name(&self, conn: &mut #conn_type) -> Result<Vec<#model_ident>, #error_type_ident>
                    {
                        use diesel::prelude::*;
                        #use_diesel_async
                        Ok(#model_ident::belonging_to(self).load::<#model_ident>(conn)#await_kw?)
                    }
                }
            };

            let eager_load_code = if eager_loading {
                let load_method_name = new_ident(&format!("load_with_{}", model.to_lowercase().to_plural()))?;
                quote! {
                    impl #struct_name {
                        pub #async_trait fn #load_method_name(parents: Vec<#struct_name>, conn: &mut #conn_type) -> Result<Vec<(#struct_name, Vec<#model_ident>)>, #error_type_ident> {
                            use diesel::prelude::*;
                            #use_diesel_async
                            let children = #model_ident::belonging_to(&parents).load::<#model_ident>(conn)#await_kw?;
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
            let method_name = match method_name {
                Some(s) => new_ident(s)?,
                None => new_ident(&format!("get_{}", model.to_lowercase()))?,
            };
            let fk_ident = new_ident(fk)?;
            let table_name = new_ident(&model.to_plural().to_snake_case())?;

            let parent_primary_key_ident = new_ident(parent_primary_key.as_deref().unwrap_or("id"))?;
            let lazy_load_code = quote! {
                impl #struct_name {
                    pub #async_trait fn #method_name(&self, conn: &mut #conn_type) -> Result<#model_ident, #error_type_ident>
                    {
                        use diesel::prelude::*;
                        #use_diesel_async
                        Ok(#schema_path::#table_name::table
                            .filter(#schema_path::#table_name::#parent_primary_key_ident.eq(self.#fk_ident))
                            .get_result::<#model_ident>(conn)#await_kw?)
                    }
                }
            };

            let eager_load_code = if eager_loading {
                let load_method_name = new_ident(&format!("load_with_{}", model.to_lowercase()))?;
                let parent_primary_key_ident = new_ident(parent_primary_key.as_deref().unwrap_or("id"))?;

                quote! {
                    impl #struct_name {
                        /// Eager loads the parent model. The parent model must derive `Clone`.
                        pub #async_trait fn #load_method_name(children: Vec<#struct_name>, conn: &mut #conn_type) -> Result<Vec<(#struct_name, #model_ident)>, #error_type_ident> {
                            use diesel::prelude::*;
                            use std::collections::{HashMap, HashSet};
                            use std::hash::Hash;
                            #use_diesel_async

                            let parent_ids: HashSet<_> = children.iter().map(|c| c.#fk_ident).collect();
                            let parents = #schema_path::#table_name::table
                                .filter(#schema_path::#table_name::#parent_primary_key_ident.eq_any(parent_ids.into_iter().collect::<Vec<_>>()))
                                .load::<#model_ident>(conn)#await_kw?;

                            let parent_map: HashMap<_, _> = parents.into_iter().map(|p| (p.#parent_primary_key_ident, p)).collect();

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
            let method_name = match method_name {
                Some(s) => new_ident(s)?,
                None => new_ident(&format!("get_{}", model.to_lowercase()))?,
            };

            let lazy_load_code = quote! {
                impl #struct_name {
                    pub #async_trait fn #method_name(&self, conn: &mut #conn_type) -> Result<#model_ident, #error_type_ident>
                    {
                        use diesel::prelude::*;
                        #use_diesel_async
                        Ok(#model_ident::belonging_to(self).first::<#model_ident>(conn)#await_kw?)
                    }
                }
            };

            let eager_load_code = if eager_loading {
                let load_method_name = new_ident(&format!("load_with_{}", model.to_lowercase()))?;
                quote! {
                    impl #struct_name {
                        pub #async_trait fn #load_method_name(parents: Vec<#struct_name>, conn: &mut #conn_type) -> Result<Vec<(#struct_name, Vec<#model_ident>)>, #error_type_ident> {
                            use diesel::prelude::*;
                            #use_diesel_async
                            let children = #model_ident::belonging_to(&parents).load::<#model_ident>(conn)#await_kw?;
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
                let join_table_ident = new_ident(join_table)?;
                let parent_fk_ident = new_ident(fk_parent)?;
                let child_fk_ident = new_ident(fk_child)?;
                let get_method_name = match method_name {
                    Some(s) => new_ident(s)?,
                    None => new_ident(&format!("get_{}", model.to_lowercase().to_plural()))?,
                };
                let add_method_name = new_ident(&format!("add_{}", model.to_lowercase().to_singular()))?;
                let remove_method_name = new_ident(&format!("remove_{}", model.to_lowercase().to_singular()))?;
                let model_table_name = new_ident(&model.to_plural().to_snake_case())?;

                let lazy_load_code = quote! {
                    impl #struct_name {
                        pub #async_trait fn #get_method_name(&self, conn: &mut #conn_type) -> Result<Vec<#model_ident>, #error_type_ident>
                        {
                            use diesel::prelude::*;
                            #use_diesel_async
                            let related_ids = #schema_path::#join_table_ident::table
                                .filter(#schema_path::#join_table_ident::#parent_fk_ident.eq(self.#primary_key_ident))
                                .select(#schema_path::#join_table_ident::#child_fk_ident)
                                .load::<#join_key_type>(conn)#await_kw?;
                            Ok(#schema_path::#model_table_name::table.filter(#schema_path::#model_table_name::#child_primary_key_ident.eq_any(related_ids)).load::<#model_ident>(conn)#await_kw?)
                        }

                        pub #async_trait fn #add_method_name(&self, conn: &mut #conn_type, child: &#model_ident) -> Result<usize, #error_type_ident>
                        {
                            use diesel::prelude::*;
                            #use_diesel_async
                            Ok(diesel::insert_into(#schema_path::#join_table_ident::table)
                                .values((#schema_path::#join_table_ident::#parent_fk_ident.eq(self.#primary_key_ident), #schema_path::#join_table_ident::#child_fk_ident.eq(child.#child_primary_key_ident)))
                                .execute(conn)#await_kw?)
                        }

                        pub #async_trait fn #remove_method_name(&self, conn: &mut #conn_type, child: &#model_ident) -> Result<usize, #error_type_ident>
                        {
                            use diesel::prelude::*;
                            #use_diesel_async
                            Ok(diesel::delete(#schema_path::#join_table_ident::table
                                .filter(#schema_path::#join_table_ident::#parent_fk_ident.eq(self.#primary_key_ident))
                                .filter(#schema_path::#join_table_ident::#child_fk_ident.eq(child.#child_primary_key_ident)))
                                .execute(conn)#await_kw?)
                        }
                    }
                };

                let eager_load_code = if eager_loading {
                    let load_method_name = new_ident(&format!("load_with_{}", model.to_lowercase().to_plural()))?;
                    quote! {
                        impl #struct_name {
                            pub #async_trait fn #load_method_name(parents: Vec<#struct_name>, conn: &mut #conn_type) -> Result<Vec<(#struct_name, Vec<#model_ident>)>, #error_type_ident> {
                                use diesel::prelude::*;
                                use std::collections::HashMap;
                                #use_diesel_async

                                let parent_ids: Vec<_> = parents.iter().map(|p| p.#primary_key_ident).collect();

                                let children_with_fk = #schema_path::#model_table_name::table
                                    .inner_join(#schema_path::#join_table_ident::table.on(#schema_path::#model_table_name::#child_primary_key_ident.eq(#schema_path::#join_table_ident::#child_fk_ident)))
                                    .filter(#schema_path::#join_table_ident::#parent_fk_ident.eq_any(parent_ids))
                                    .select((#schema_path::#model_table_name::all_columns, #schema_path::#join_table_ident::#parent_fk_ident))
                                    .load::<(#model_ident, #join_key_type)>(conn)#await_kw?;

                                let mut grouped_children: HashMap<#join_key_type, Vec<#model_ident>> = HashMap::new();
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
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "'join_table', 'fk_parent', and 'fk_child' attributes are required for 'many_to_many' relations",
                ));
            }
        }
        _ => {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Unsupported relation type. Supported types are: `one_to_many`, `many_to_one`, `one_to_one`, `many_to_many`.",
            ))
        }
    })
}
