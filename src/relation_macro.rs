use crate::utils::parser::{parse_attributes, ParsedAttrs};
use proc_macro2::Span;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, ItemStruct, parse_macro_input};

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
/// ```bash
/// use diesel_linker::DieselLinker;
///
/// #[derive(DieselLinker)]
/// #[relation(type = "one-to-many", table1 = "users", table2 = "posts", column1 = "id", column2 = "user_id")]
/// struct User {
///     // Définitions des champs de la structure `User`...
/// }
///
///  // Table post
/// #[derive(DieselLinker)]
/// #[relation(type = "many-to-one", table1 = "users", table2 = "posts", column1 = "id", column2 = "user_id")]
/// struct Post {
///    // Définitions des champs de la structure `Post`...
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
///

pub fn diesel_linker_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as syn::AttributeArgs);
    let parsed_attrs = match parse_attributes(attrs) {
        Ok(attrs) => attrs,
        Err(e) => return e.to_compile_error().into(),
    };

    // Convertissez `item` en `ItemStruct` pour être utilisé dans vos fonctions génératrices.
    let item_struct = parse_macro_input!(item as syn::ItemStruct);

    // Déterminez le code spécifique à générer basé sur le type de relation.
    let relation_code = match parsed_attrs.relation_type.as_str() {
        "many_to_many" => generate_many_to_many_code(&parsed_attrs, &item_struct),
        "many_to_one" => generate_many_to_one_code(&parsed_attrs, &item_struct),
        "one_to_many" => generate_one_to_many_code(&parsed_attrs, &item_struct),
        "one_to_one" => generate_one_to_one_code(&parsed_attrs, &item_struct),
        _ => quote! {}.into(),

    };

    // Retournez le code généré spécifique à la relation.
    relation_code.into()
}

// Relation many-to-many
// trait `Connection` pour permettre l'utilisation de n'importe quel type de connexion Diesel.

fn generate_many_to_many_code(attrs: &ParsedAttrs, item: &syn::ItemStruct) -> proc_macro::TokenStream {
    let struct_name = &item.ident;
    let join_table = format_ident!("{}", attrs.join_table.as_ref().unwrap());
    let table_from_field_name = format_ident!("{}", attrs.column_from.as_ref().unwrap());
    let table_to_field_name = format_ident!("{}", attrs.column_to.as_ref().unwrap());
    let join_struct_name = format_ident!("{}Join", struct_name);

    quote! {
        #[derive(Queryable, Insertable, Associations)]
        #[table_name = #join_table]
        pub struct #join_struct_name {
            pub #table_from_field_name: i32,
            pub #table_to_field_name: i32,
        }

        impl #struct_name {
            pub fn add_relation<Conn>(conn: &Conn, from_id: i32, to_id: i32) -> diesel::QueryResult<#join_struct_name>
            where
                Conn: diesel::Connection,
            {
                use diesel::RunQueryDsl;
                use crate::schema::#join_table::dsl::*;

                let new_relation = #join_struct_name {
                    #table_from_field_name: from_id,
                    #table_to_field_name: to_id,
                };

                diesel::insert_into(#join_table)
                    .values(&new_relation)
                    .get_result(conn)
            }

            pub fn remove_relation<Conn>(conn: &Conn, from_id: i32, to_id: i32) -> diesel::QueryResult<usize>
            where
                Conn: diesel::Connection,
            {
                use diesel::RunQueryDsl;
                use crate::schema::#join_table::dsl::*;

                diesel::delete(#join_table.filter(#table_from_field_name.eq(from_id).and(#table_to_field_name.eq(to_id))))
                    .execute(conn)
            }
        }
    }.into()
}




/// Génère du code Rust pour gérer une relation many-to-one entre deux modèles dans Diesel.
fn generate_many_to_one_code(attrs: &ParsedAttrs, item: &ItemStruct) -> TokenStream {
    if let (Some(table_from), Some(column_from), _) = (&attrs.table_from, &attrs.column_from, &attrs.column_to) {
        let struct_name = &item.ident;
        let table_from_ident = format_ident!("{}", table_from);
        let column_from_ident = format_ident!("{}", column_from);

        quote! {
            #[derive(Queryable, Associations, Identifiable, Insertable)]
            #[belongs_to(parent = #table_from_ident, foreign_key = #column_from_ident)]
            #[table_name = #table_from]
            pub struct #struct_name {
                pub id: i32,
                pub #column_from_ident: i32, // Foreign key
            }

            impl #struct_name {
                pub fn add_relation(conn: &impl Connection, new_record: &#struct_name) -> diesel::QueryResult<usize> {
                    use crate::schema::#table_from_ident::dsl::*;

                    diesel::insert_into(#table_from_ident)
                        .values(new_record)
                        .execute(conn)
                }
            }
        }.into()
    } else {
        Error::new(Span::call_site(), "Informations incomplètes pour la relation 'many_to_one'").to_compile_error().into()
    }
}


// Relation one-to-many
fn generate_one_to_many_code(attrs: &ParsedAttrs, item: &ItemStruct) -> TokenStream {
    if let (Some(table_to), Some(column_from), Some(column_to)) = (&attrs.table_to, &attrs.column_from, &attrs.column_to) {
        let struct_name = &item.ident; // Nom de la structure "one"
        let table_to_ident = format_ident!("{}", table_to); // Identifiant pour table_to
        let column_from_ident = format_ident!("{}", column_from); // Identifiant pour column_from (clé étrangère dans table "many")
        let column_to_ident = format_ident!("{}", column_to); // Identifiant pour column_to (clé primaire dans table "one")

        quote! {
            #[derive(Queryable, Associations, Identifiable)]
            #[table_name = #table_to]
            pub struct #struct_name {
                pub #column_to_ident: i32, // Clé primaire
            }

            impl #struct_name {
                // Méthode pour ajouter un enregistrement lié dans la table "many"
                pub fn add_child<C>(conn: &C, child: &NewChild) -> QueryResult<Child>
                where
                    C: Connection,
                {
                    use crate::schema::#table_to_ident::dsl::*;

                    diesel::insert_into(#table_to_ident)
                        .values(child)
                        .get_result(conn)
                }

                // Méthode pour retirer un enregistrement lié
                pub fn remove_child<C>(conn: &C, child_id: i32) -> QueryResult<usize>
                where
                    C: Connection,
                {
                    use crate::schema::#table_to_ident::dsl::*;

                    diesel::delete(#table_to_ident.filter(#column_from_ident.eq(child_id)))
                        .execute(conn)
                }
            }
        }.into()
    } else {
        Error::new(Span::call_site(), "Informations incomplètes pour la relation 'one_to_many'").to_compile_error().into()
    }
}

// Relation one_to_one
fn generate_one_to_one_code(attrs: &ParsedAttrs, item: &ItemStruct) -> TokenStream {
    if let (Some(table_from), Some(column_from), Some(column_to)) = (&attrs.table_from, &attrs.column_from, &attrs.column_to) {
        let struct_name = &item.ident;
        let table_from_ident = format_ident!("{}", table_from);
        let column_from_ident = format_ident!("{}", column_from);
        let column_to_ident = format_ident!("{}", column_to);

        quote! {
            #[derive(Queryable, Associations, Identifiable, Insertable)]
            #[table_name = #table_from]
            pub struct #struct_name {
                pub #column_to_ident: i32, // Clé primaire, utilisée ici pour simplifier.
                pub #column_from_ident: i32, // Clé étrangère avec contrainte d'unicité.
            }

            impl #struct_name {
                // Méthode pour ajouter ou mettre à jour l'entité liée dans une relation one-to-one.
                pub fn upsert_related_entity<C>(conn: &C, related_entity: &#struct_name) -> QueryResult<#struct_name>
                where
                    C: Connection,
                {
                    use crate::schema::#table_from_ident::dsl::*;

                    diesel::insert_into(#table_from_ident)
                        .values(related_entity)
                        .on_conflict(#column_from_ident)
                        .do_update()
                        .set(related_entity)
                        .get_result(conn)
                }

                // Méthode pour récupérer l'entité liée.
                pub fn get_related_entity<C>(conn: &C) -> QueryResult<#struct_name>
                where
                    C: Connection,
                {
                    #table_from_ident.find(#column_from_ident).get_result(conn)
                }

                // Méthode pour supprimer l'entité liée.
                pub fn delete_related_entity<C>(conn: &C) -> QueryResult<usize>
                where
                    C: Connection,
                {
                    diesel::delete(#table_from_ident.filter(#column_from_ident.eq(self.#column_from_ident))).execute(conn)
                }
            }
        }.into()
    } else {
        Error::new(Span::call_site(), "Informations incomplètes pour la relation 'one_to_one'").to_compile_error().into()
    }
}