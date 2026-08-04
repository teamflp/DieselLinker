// Regression test for `schema_path`: the generated code can point at a schema
// module that isn't named `crate::schema`.

mod db {
    diesel::table! {
        crew_members (id) {
            id -> Integer,
            name -> Text,
        }
    }

    diesel::table! {
        ships (id) {
            id -> Integer,
            captain_id -> Integer,
            name -> Text,
        }
    }

    diesel::joinable!(ships -> crew_members (captain_id));
}

use db::{crew_members, ships};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_linker::relation;

#[derive(Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(table_name = crew_members)]
pub struct CrewMember {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Identifiable, Insertable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(CrewMember, foreign_key = captain_id), table_name = ships)]
#[relation(
    model = "CrewMember",
    fk = "captain_id",
    relation_type = "many_to_one",
    backend = "sqlite",
    schema_path = "db"
)]
pub struct Ship {
    pub id: i32,
    pub captain_id: i32,
    pub name: String,
}

fn setup_db() -> SqliteConnection {
    let mut conn = SqliteConnection::establish(":memory:").unwrap();
    diesel::sql_query("CREATE TABLE crew_members (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
        .execute(&mut conn)
        .unwrap();
    diesel::sql_query("CREATE TABLE ships (id INTEGER PRIMARY KEY, captain_id INTEGER NOT NULL, name TEXT NOT NULL)").execute(&mut conn).unwrap();
    conn
}

#[test]
fn test_many_to_one_with_custom_schema_path() {
    let mut conn = setup_db();

    let captain = CrewMember {
        id: 1,
        name: "Kirk".to_string(),
    };
    diesel::insert_into(crew_members::table)
        .values(&captain)
        .execute(&mut conn)
        .unwrap();

    let ship = Ship {
        id: 1,
        captain_id: 1,
        name: "Enterprise".to_string(),
    };
    diesel::insert_into(ships::table)
        .values(&ship)
        .execute(&mut conn)
        .unwrap();

    let ship = ships::table.find(1).first::<Ship>(&mut conn).unwrap();
    let captain = ship.get_crewmember(&mut conn).unwrap();
    assert_eq!(captain.name, "Kirk");
}
