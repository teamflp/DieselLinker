// Regression test for `join_key_type`: `many_to_many` no longer hardcodes
// `i32` for join-table lookups; the type is explicit and can be e.g. `i64`.

mod schema {
    diesel::table! {
        widgets (id) {
            id -> BigInt,
            name -> Text,
        }
    }

    diesel::table! {
        gadgets (id) {
            id -> BigInt,
            name -> Text,
        }
    }

    diesel::table! {
        widget_gadgets (id) {
            id -> BigInt,
            widget_id -> BigInt,
            gadget_id -> BigInt,
        }
    }

    diesel::joinable!(widget_gadgets -> widgets (widget_id));
    diesel::joinable!(widget_gadgets -> gadgets (gadget_id));
    diesel::allow_tables_to_appear_in_same_query!(widgets, gadgets, widget_gadgets);
}

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_linker::relation;
use schema::{gadgets, widget_gadgets, widgets};

#[derive(Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(table_name = widgets)]
#[relation(
    model = "Gadget",
    relation_type = "many_to_many",
    join_table = "widget_gadgets",
    fk_parent = "widget_id",
    fk_child = "gadget_id",
    backend = "sqlite",
    join_key_type = "i64"
)]
pub struct Widget {
    pub id: i64,
    pub name: String,
}

#[derive(Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(table_name = gadgets)]
pub struct Gadget {
    pub id: i64,
    pub name: String,
}

#[derive(Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(belongs_to(Widget), belongs_to(Gadget), table_name = widget_gadgets)]
pub struct WidgetGadget {
    pub id: i64,
    pub widget_id: i64,
    pub gadget_id: i64,
}

fn setup_db() -> SqliteConnection {
    let mut conn = SqliteConnection::establish(":memory:").unwrap();
    diesel::sql_query("CREATE TABLE widgets (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
        .execute(&mut conn)
        .unwrap();
    diesel::sql_query("CREATE TABLE gadgets (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
        .execute(&mut conn)
        .unwrap();
    diesel::sql_query("CREATE TABLE widget_gadgets (id INTEGER PRIMARY KEY, widget_id INTEGER NOT NULL, gadget_id INTEGER NOT NULL)").execute(&mut conn).unwrap();
    conn
}

#[test]
fn test_many_to_many_with_i64_keys() {
    let mut conn = setup_db();

    let widget = Widget {
        id: 1,
        name: "Widget A".to_string(),
    };
    diesel::insert_into(widgets::table)
        .values(&widget)
        .execute(&mut conn)
        .unwrap();

    let gadget = Gadget {
        id: 1,
        name: "Gadget A".to_string(),
    };
    diesel::insert_into(gadgets::table)
        .values(&gadget)
        .execute(&mut conn)
        .unwrap();

    let link = WidgetGadget {
        id: 1,
        widget_id: 1,
        gadget_id: 1,
    };
    diesel::insert_into(widget_gadgets::table)
        .values(&link)
        .execute(&mut conn)
        .unwrap();

    let widget = widgets::table
        .find(1i64)
        .first::<Widget>(&mut conn)
        .unwrap();
    let found_gadgets = widget.get_gadgets(&mut conn).unwrap();
    assert_eq!(found_gadgets.len(), 1);
    assert_eq!(found_gadgets[0].name, "Gadget A");

    let gadget2 = Gadget {
        id: 2,
        name: "Gadget B".to_string(),
    };
    diesel::insert_into(gadgets::table)
        .values(&gadget2)
        .execute(&mut conn)
        .unwrap();

    widget.add_gadget(&mut conn, &gadget2).unwrap();
    assert_eq!(widget.get_gadgets(&mut conn).unwrap().len(), 2);

    widget.remove_gadget(&mut conn, &gadget2).unwrap();
    assert_eq!(widget.get_gadgets(&mut conn).unwrap().len(), 1);
}
