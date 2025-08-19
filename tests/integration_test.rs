pub mod schema;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_linker::relation;
use crate::schema::{users, posts};

#[derive(Queryable, Identifiable, Insertable, Debug, PartialEq)]
#[diesel(table_name = users)]
#[relation(model = "Post", fk = "user_id", relation_type = "one_to_many")]
pub struct User {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Identifiable, Insertable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(User), table_name = posts)]
#[relation(model = "User", fk = "user_id", relation_type = "many_to_one")]
pub struct Post {
    pub id: i32,
    pub user_id: Option<i32>,
    pub title: String,
}

fn setup_db() -> SqliteConnection {
    let mut conn = SqliteConnection::establish(":memory:").unwrap();
    diesel::sql_query("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)").execute(&mut conn).unwrap();
    diesel::sql_query("CREATE TABLE posts (id INTEGER PRIMARY KEY, user_id INTEGER, title TEXT NOT NULL)").execute(&mut conn).unwrap();
    conn
}

#[test]
fn test_one_to_many_get() {
    let mut conn = setup_db();

    let new_user = User { id: 1, name: "Alice".to_string() };
    diesel::insert_into(users::table).values(&new_user).execute(&mut conn).unwrap();

    let new_post = Post { id: 1, user_id: Some(1), title: "First post".to_string() };
    diesel::insert_into(posts::table).values(&new_post).execute(&mut conn).unwrap();

    let user = users::table.find(1).first::<User>(&mut conn).unwrap();
    let posts = user.get_posts(&mut conn).unwrap();

    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].title, "First post");
}
