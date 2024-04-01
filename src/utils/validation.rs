use crate::utils::parser::ParsedAttrs;

/// Validate the `join_table`, `table_from`, `table_to`, `column_from`, and `column_to` attributes for a many-to-many relationship.
///
/// # Arguments
///
/// * `attrs` - The relationship attributes.
///
/// # Returns
///
/// Returns an error if any of the conditions are not met.
pub fn validate_many_to_many_attrs(attrs: &ParsedAttrs) -> Result<(), String> {
    // Verify that the `join_table` attribute is present
    if attrs.join_table.is_none() {
        return Err(
            "The `join_table` attribute is required for a many-to-many relationship.".into(),
        );
    }
    // Verify that the `table_from` and `table_to` attributes are present
    if attrs.table_from.is_none() || attrs.table_to.is_none() {
        return Err(
            "The `table_from` and `table_to` attributes are required for a many-to-many relationship."
               .into(),
        );
    }
    // Verify that the `column_from` and `column_to` attributes are present
    if attrs.column_from.is_none() || attrs.column_to.is_none() {
        return Err(
            "The `column_from` and `column_to` attributes are required for a many-to-many relationship."
               .into(),
        );
    }

    Ok(())
}

/// Validate the `table_from`, `table_to`, `column_from`, and `column_to` attributes for a one-to-many or many-to-one relationship.
///
/// # Arguments
///
/// * `attrs` - The relationship attributes.
///
/// # Returns
///
/// Returns an error if any of the conditions are not met.
pub fn validate_one_to_many_or_many_to_one_attrs(attrs: &ParsedAttrs) -> Result<(), String> {
    // Vérifie si les attributs `table_from` et `table_to` sont présents
    if attrs.table_from.is_none() || attrs.table_to.is_none() {
        return Err("Les attributs 'table_from' et 'table_to' sont requis.".into());
    }
    // Vérifie si les attributs `column_from` et `column_to` sont présents
    if attrs.column_from.is_none() || attrs.column_to.is_none() {
        return Err("Les attributs 'column_from' et 'column_to' sont requis.".into());
    }
    Ok(())
}

/// Validate the `table_from`, `table_to`, `column_from`, and `column_to` attributes for a one-to-one relationship.
///
/// # Arguments
///
/// * `attrs` - The relationship attributes.
///
/// # Returns
///
/// Returns an error if any of the conditions are not met.
pub fn validate_one_to_one_attrs(attrs: &ParsedAttrs) -> Result<(), String> {
    // Vérifie si les attributs `table_from` et `table_to` sont présents
    if attrs.table_from.is_none() || attrs.table_to.is_none() {
        return Err(
            "Les attributs 'table_from' et 'table_to' sont requis pour une relation one-to-one."
                .into(),
        );
    }
    // Vérifie si les attributs `column_from` et `column_to` sont présents
    if attrs.column_from.is_none() || attrs.column_to.is_none() {
        return Err(
            "Les attributs 'column_from' et 'column_to' sont requis pour une relation one-to-one."
                .into(),
        );
    }

    Ok(())
}
