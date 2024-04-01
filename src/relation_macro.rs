use crate::utils::parser::{parse_attributes, ParsedAttrs};
use crate::utils::validation::{
    validate_many_to_many_attrs, validate_one_to_many_or_many_to_one_attrs,
    validate_one_to_one_attrs,
};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Error, ItemStruct};

/// Generates the implementation of the `diesel_linker` macro.
///
/// # Arguments
///
/// * `attr` - The attributes passed to the macro.
/// * `item` - The struct definition that is being extended.
///
/// # Returns
///
/// A `TokenStream` containing the generated code.
///
pub fn diesel_linker_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as syn::AttributeArgs);
    let parsed_attrs = match parse_attributes(attrs) {
        Ok(attrs) => attrs,
        Err(e) => return e.to_compile_error().into(),
    };

    // Apply validations based on the relation type
    let validation_result = match parsed_attrs.relation_type.as_str() {
        "many_to_many" => validate_many_to_many_attrs(&parsed_attrs),
        "one_to_many" | "many_to_one" => validate_one_to_many_or_many_to_one_attrs(&parsed_attrs),
        "one_to_one" => validate_one_to_one_attrs(&parsed_attrs),
        _ => Err("Unknown relation type or missing information.".into()),
    };

    if let Err(validation_error) = validation_result {
        return Error::new(Span::call_site(), validation_error)
            .to_compile_error()
            .into();
    }

    // Convert `item` to an `ItemStruct` to be used in your generator functions.
    let item_struct = parse_macro_input!(item as syn::ItemStruct);

    // Determine the code to generate based on the relation type
    let relation_code = match parsed_attrs.relation_type.as_str() {
        "many_to_many" => generate_many_to_many_code(&parsed_attrs, &item_struct),
        "many_to_one" => generate_many_to_one_code(&parsed_attrs, &item_struct),
        "one_to_many" => generate_one_to_many_code(&parsed_attrs, &item_struct),
        "one_to_one" => generate_one_to_one_code(&parsed_attrs, &item_struct),
        _ => quote! {}.into(),
    };

    // Gestion d'erreur avec panic
    /*if let Err(validation_error) = validation_result {
        panic!("La validation a échoué pour les attributs de relation donnés: {}", validation_error);
    }*/

    // Return the code generated specific to the relation
    relation_code.into()
}

// Generate code for a :
// - many-to-many relationship between two Diesel models.
// - one-to-many relationship between two Diesel models.
// - many-to-one relationship between two Diesel models.
// - one-to-one relationship between two Diesel models.
// Generate code for a many-to-many relationship between two Diesel models.
fn generate_many_to_many_code(
    attrs: &ParsedAttrs,
    item: &syn::ItemStruct,
) -> proc_macro::TokenStream {
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
            /// Adds a new relation between two records in the many-to-many relationship.
            ///
            /// # Arguments
            ///
            /// * `conn` - A reference to the database connection.
            /// * `from_id` - The ID of the record in the "one" table.
            /// * `to_id` - The ID of the record in the "many" table.
            ///
            /// # Returns
            ///
            /// A `diesel::QueryResult` containing the new relation record.
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

            /// Removes a relation between two records in the many-to-many relationship.
            ///
            /// # Arguments
            ///
            /// * `conn` - A reference to the database connection.
            /// * `from_id` - The ID of the record in the "one" table.
            /// * `to_id` - The ID of the record in the "many" table.
            ///
            /// # Returns
            ///
            /// A `diesel::QueryResult` containing the number of rows deleted.
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
// Generate code for a many-to-one relationship between two Diesel model.
fn generate_many_to_one_code(attrs: &ParsedAttrs, item: &ItemStruct) -> TokenStream {
    if let (Some(table_from), Some(column_from), _) =
        (&attrs.table_from, &attrs.column_from, &attrs.column_to)
    {
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
        Error::new(
            Span::call_site(),
            "Informations incomplètes pour la relation 'many_to_one'",
        )
        .to_compile_error()
        .into()
    }
}
fn generate_one_to_many_code(attrs: &ParsedAttrs, item: &ItemStruct) -> TokenStream {
    if let (Some(table_to), Some(column_from), Some(column_to)) =
        (&attrs.table_to, &attrs.column_from, &attrs.column_to)
    {
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
        }
        .into()
    } else {
        Error::new(
            Span::call_site(),
            "Informations incomplètes pour la relation 'one_to_many'",
        )
        .to_compile_error()
        .into()
    }
}
// Relation one_to_one
fn generate_one_to_one_code(attrs: &ParsedAttrs, item: &ItemStruct) -> TokenStream {
    if let (Some(table_from), Some(column_from), Some(column_to)) =
        (&attrs.table_from, &attrs.column_from, &attrs.column_to)
    {
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
        Error::new(
            Span::call_site(),
            "Informations incomplètes pour la relation 'one_to_one'",
        )
        .to_compile_error()
        .into()
    }
}
