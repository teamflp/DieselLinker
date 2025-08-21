# Custom Error Types

For better integration into applications with a dedicated error handling strategy, `DieselLinker` allows you to specify a custom error type for the generated methods.

To use this feature, you need to:
1.  Define your custom error type.
2.  Implement the `From<diesel::result::Error>` trait for your custom error type.
3.  Specify your custom error type in the `#[relation]` attribute using `error_type = "MyError"`.

## Example

Here is an example of how to define and use a custom error type.

```rust
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_linker::relation;

mod schema {
    diesel::table! {
        users (id) {
            id -> Integer,
            name -> Text,
        }
    }

    diesel::table! {
        papers (id) {
            id -> Integer,
            scientist_id -> Integer,
            title -> Text,
        }
    }

    diesel::joinable!(papers -> users (scientist_id));
    diesel::allow_tables_to_appear_in_same_query!(users, papers);
}

use schema::{users, papers};

// 1. Define a custom error type
#[derive(Debug)]
pub enum MyError {
    DieselError(diesel::result::Error),
    CustomError(String),
}

// 2. Implement From<diesel::result::Error>
impl From<diesel::result::Error> for MyError {
    fn from(err: diesel::result::Error) -> MyError {
        MyError::DieselError(err)
    }
}

#[derive(Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(table_name = users)]
// 3. Use the `error_type` attribute
#[relation(model = "Paper", relation_type = "one_to_many", backend = "sqlite", error_type = "MyError")]
pub struct Scientist {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Identifiable, Insertable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(Scientist), table_name = papers)]
pub struct Paper {
    pub id: i32,
    pub scientist_id: i32,
    pub title: String,
}

fn main() {
    let mut conn = SqliteConnection::establish(":memory:").unwrap();

    diesel::sql_query("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE papers (id INTEGER PRIMARY KEY, scientist_id INTEGER NOT NULL, title TEXT NOT NULL)").execute(&mut conn).unwrap();

    let new_scientist = Scientist { id: 1, name: "Marie Curie".to_string() };
    diesel::insert_into(users::table).values(&new_scientist).execute(&mut conn).unwrap();

    let new_paper = Paper { id: 1, scientist_id: 1, title: "Recherches sur les substances radioactives".to_string() };
    diesel::insert_into(papers::table).values(&new_paper).execute(&mut conn).unwrap();

    let scientist = users::table.find(1).first::<Scientist>(&mut conn).unwrap();

    // The generated method now returns Result<Vec<Paper>, MyError>
    let result: Result<Vec<Paper>, MyError> = scientist.get_papers(&mut conn);

    match result {
        Ok(papers) => {
            println!("Successfully fetched {} paper(s) for {}.", papers.len(), scientist.name);
            assert_eq!(papers.len(), 1);
        }
        Err(e) => {
            println!("An error occurred: {:?}", e);
            panic!("Test failed");
        }
    }
}
```
